use std::io::{self, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, str};

use anyhow::{bail, Result};
use cargo_metadata::camino::Utf8Component;
use cargo_metadata::{Artifact, CargoOpt, Message, Metadata, MetadataCommand};
use clap::{Arg, ArgAction, ArgMatches, Command as ClapCommand};
use rustc_cfg::Cfg;

pub use tool::Tool;

mod llvm;
mod postprocess;
mod rustc;
mod tool;

/// Search for `file` in `path` and its parent directories
fn search<'p>(path: &'p Path, file: &str) -> Option<&'p Path> {
    path.ancestors().find(|dir| dir.join(file).exists())
}

fn parse<T>(path: &Path) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    use std::fs::File;
    use std::io::Read;
    use toml::de;

    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(de::from_str(&s)?)
}

/// Execution context
// TODO this should be some sort of initialize once, read-only singleton
pub struct Context {
    cfg: Cfg,
    /// Final compilation target
    target: String,
}

impl Context {
    /* Constructors */
    /// Get a context structure from a built artifact.
    fn from_artifact(metadata: Metadata, artifact: &Artifact) -> Result<Self> {
        // Currently there is no clean way to get the target triple from cargo so we can only make
        // an approximation, we do this by extracting the target triple from the artifacts path.
        // For more info on the path structure see: https://doc.rust-lang.org/cargo/guide/build-cache.html

        // In the future it may be possible to replace this code and use a cargo feature:
        // See: https://github.com/rust-lang/cargo/issues/5579, https://github.com/rust-lang/cargo/issues/8002

        // Should always succeed.
        let target_path = artifact.filenames[0].strip_prefix(metadata.target_directory)?;
        let target_name = if let Some(Utf8Component::Normal(path)) = target_path.components().next()
        {
            // TODO: How will custom profiles impact this?
            if path == "debug" || path == "release" {
                // Looks like this artifact was built for the host.
                rustc_version::version_meta()?.host
            } else {
                // The artifact
                path.to_string()
            }
        } else {
            unreachable!();
        };

        Self::from_target_name(&target_name)
    }

    /// Get a context structure from a provided target flag, used when cargo
    /// was not used to build the binary.
    fn from_flag(metadata: Metadata, target_flag: Option<&str>) -> Result<Self> {
        let host_target_name = rustc_version::version_meta()?.host;

        // Get the "default" target override in .cargo/config.
        let mut config_target_name = None;
        let config: toml::Value;

        if let Some(path) = search(metadata.workspace_root.as_std_path(), ".cargo/config") {
            config = parse(&path.join(".cargo/config"))?;
            config_target_name = config
                .get("build")
                .and_then(|build| build.get("target"))
                .and_then(|target| target.as_str());
        }

        // Find the actual target.
        let target_name = target_flag
            .or(config_target_name)
            .unwrap_or(&host_target_name);

        Self::from_target_name(target_name)
    }

    fn from_target_name(target_name: &str) -> Result<Self> {
        let cfg = Cfg::of(target_name)?;

        Ok(Context {
            cfg,
            target: target_name.to_string(),
        })
    }
}

enum BuildType<'a> {
    Any,
    Bin(&'a str),
    Example(&'a str),
    Test(&'a str),
    Bench(&'a str),
    Lib,
}

impl BuildType<'_> {
    fn matches(&self, artifact: &Artifact) -> bool {
        match self {
            BuildType::Bin(target_name)
            | BuildType::Example(target_name)
            | BuildType::Test(target_name)
            | BuildType::Bench(target_name) => {
                artifact.target.name == *target_name && artifact.executable.is_some()
            }
            // For info about 'kind' values see:
            // https://github.com/rust-lang/cargo/blob/d47a9545db81fe6d7e6c542bc8154f09d0e6c788/src/cargo/core/manifest.rs#L166-L181
            // The only "Any" artifacts we can support are bins and examples, so let's make sure
            // no-one slips us a "custom-build" in form of a build.rs in
            BuildType::Any => artifact
                .target
                .kind
                .iter()
                .any(|s| s == "bin" || s == "example"),
            // Since LibKind can be an arbitrary string `LibKind:Other(String)` we filter by what it can't be
            BuildType::Lib => artifact.target.kind.iter().any(|s| {
                s != "bin" && s != "example" && s != "test" && s != "custom-build" && s != "bench"
            }),
        }
    }
}

fn args(tool: Tool, examples: Option<&str>) -> ArgMatches {
    let name = tool.name();
    let about = format!("Proxy for the `llvm-{name}` tool shipped with the Rust toolchain.");
    let after_help = format!(
        "\
The arguments specified *after* the `--` will be passed to the proxied tool invocation.

To see all the flags the proxied tool accepts run `cargo-{} -- --help`.{}",
        name,
        examples.unwrap_or("")
    );

    let app = ClapCommand::new(format!("cargo-{name}"))
        .about(about)
        .version(env!("CARGO_PKG_VERSION"))
        // as this is used as a Cargo subcommand the first argument will be the name of the binary
        // we ignore this argument
        .args(&[
            Arg::new("binary-name").hide(true),
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(ArgAction::Count)
                .help("Use verbose output (-vv cargo verbose or -vvv for build.rs output)"),
            Arg::new("args")
                .last(true)
                .num_args(1..)
                .help("The arguments to be proxied to the tool"),
        ])
        .after_help(after_help);

    if tool.needs_build() {
        app.args(&[
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .action(ArgAction::SetTrue)
                .help("Don't print build output from `cargo build`"),
            Arg::new("package")
                .long("package")
                .short('p')
                .value_name("SPEC")
                .help("Package to build (see `cargo help pkgid`)"),
            Arg::new("jobs")
                .long("jobs")
                .short('j')
                .value_name("N")
                .help("Number of parallel jobs, defaults to # of CPUs"),
            Arg::new("lib")
                .long("lib")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["bin", "example", "test", "bench"])
                .help("Build only this package's library"),
            Arg::new("bin")
                .long("bin")
                .value_name("NAME")
                .conflicts_with_all(["lib", "example", "test", "bench"])
                .help("Build only the specified binary"),
            Arg::new("example")
                .long("example")
                .value_name("NAME")
                .conflicts_with_all(["lib", "bin", "test", "bench"])
                .help("Build only the specified example"),
            Arg::new("test")
                .long("test")
                .value_name("NAME")
                .conflicts_with_all(["lib", "bin", "example", "bench"])
                .help("Build only the specified test target"),
            Arg::new("bench")
                .long("bench")
                .value_name("NAME")
                .conflicts_with_all(["lib", "bin", "example", "test"])
                .help("Build only the specified bench target"),
            Arg::new("release")
                .long("release")
                .action(ArgAction::SetTrue)
                .help("Build artifacts in release mode, with optimizations"),
            Arg::new("profile")
                .long("profile")
                .value_name("PROFILE-NAME")
                .help("Build artifacts with the specified profile"),
            Arg::new("manifest-path")
                .long("manifest-path")
                .help("Path to Cargo.tom"),
            Arg::new("features")
                .long("features")
                .short('F')
                .value_name("FEATURES")
                .help("Space-separated list of features to activate"),
            Arg::new("all-features")
                .long("all-features")
                .action(ArgAction::SetTrue)
                .help("Activate all available features"),
            Arg::new("no-default-features")
                .long("no-default-features")
                .action(ArgAction::SetTrue)
                .help("Do not activate the `default` feature"),
            Arg::new("target")
                .long("target")
                .value_name("TRIPLE")
                .help("Target triple for which the code is compiled"),
            Arg::new("config")
                .long("config")
                .value_name("CONFIG")
                .help("Override a configuration value"),
            Arg::new("color")
                .long("color")
                .action(ArgAction::Set)
                .value_parser(clap::builder::PossibleValuesParser::new([
                    "auto", "always", "never",
                ]))
                .help("Coloring: auto, always, never"),
            Arg::new("frozen")
                .long("frozen")
                .action(ArgAction::SetTrue)
                .help("Require Cargo.lock and cache are up to date"),
            Arg::new("locked")
                .long("locked")
                .action(ArgAction::SetTrue)
                .help("Require Cargo.lock is up to date"),
            Arg::new("offline")
                .long("offline")
                .action(ArgAction::SetTrue)
                .help("Run without accessing the network"),
            Arg::new("unstable-features")
                .short('Z')
                .action(ArgAction::Append)
                .value_name("FLAG")
                .help("Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details"),
        ])
        .get_matches()
    } else {
        app.get_matches()
    }
}

pub fn run(tool: Tool, matches: ArgMatches) -> Result<i32> {
    let mut metadata_command = MetadataCommand::new();
    if let Some(features) = matches.get_many::<String>("features") {
        metadata_command.features(CargoOpt::SomeFeatures(
            features.map(|s| s.to_owned()).collect(),
        ));
    }
    if matches.get_flag("no-default-features") {
        metadata_command.features(CargoOpt::NoDefaultFeatures);
    }
    if matches.get_flag("all-features") {
        metadata_command.features(CargoOpt::AllFeatures);
    }
    let metadata = metadata_command.exec()?;
    if metadata.workspace_members.is_empty() {
        bail!("Unable to find workspace members");
    }

    let mut tool_args = vec![];
    if let Some(args) = matches.get_many::<String>("args") {
        tool_args.extend(args.map(|s| s.as_str()));
    }

    let tool_help = tool_args.first() == Some(&"--help");

    let target_artifact = if tool.needs_build() && !tool_help {
        cargo_build(&matches, &metadata)?
    } else {
        None
    };

    let mut lltool = Command::new(format!("rust-{}", tool.name()));

    if tool == Tool::Objdump {
        let ctxt = if let Some(artifact) = &target_artifact {
            Context::from_artifact(metadata, artifact)?
        } else {
            Context::from_flag(
                metadata,
                matches.get_one::<String>("target").map(|s| s.as_str()),
            )?
        };

        let arch_name = llvm::arch_name(&ctxt.cfg, &ctxt.target);

        if arch_name == "thumb" {
            // `-arch-name=thumb` doesn't produce the right output so instead we pass
            // `-triple=$target`, which contains more information about the target
            lltool.args(["--triple", &ctxt.target]);
        } else {
            lltool.args(&[format!("--arch-name={arch_name}")]);
        }
    }

    // Extra flags
    if let Tool::Readobj = tool {
        // The default output style of `readobj` is JSON-like, which is not user friendly, so we
        // change it to the human readable GNU style
        lltool.arg("--elf-output-style=GNU");
    }

    if tool.needs_build() {
        // Artifact
        if let Some(artifact) = &target_artifact {
            let file = match &artifact.executable {
                // Example and bins have an executable
                Some(val) => val,
                // Libs have an rlib and an rmeta. We want the rlib, which always
                // comes first in the filenames array after some quick testing.
                //
                // We could instead look for files ending in .rlib, but that would
                // fail for cdylib and other fancy crate kinds.
                None => &artifact.filenames[0],
            };

            match tool {
                // Tools that don't need a build
                Tool::Ar | Tool::As | Tool::Cov | Tool::Lld | Tool::Profdata => {}
                // for some tools we change the CWD (current working directory) and
                // make the artifact path relative. This makes the path that the
                // tool will print easier to read. e.g. `libfoo.rlib` instead of
                // `/home/user/rust/project/target/$T/debug/libfoo.rlib`.
                Tool::Objdump | Tool::Nm | Tool::Readobj | Tool::Size => {
                    lltool
                        .current_dir(file.parent().unwrap())
                        .arg(file.file_name().unwrap());
                }
                Tool::Objcopy | Tool::Strip => {
                    lltool.arg(file);
                }
            }
        }
    }

    // User flags
    lltool.args(&tool_args);

    if matches.get_count("verbose") > 0 {
        eprintln!("{lltool:?}");
    }

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let output = lltool.stderr(Stdio::inherit()).output()?;

    // post process output
    let processed_output = match tool {
        Tool::Ar
        | Tool::As
        | Tool::Cov
        | Tool::Lld
        | Tool::Objcopy
        | Tool::Profdata
        | Tool::Strip => output.stdout.into(),
        Tool::Nm | Tool::Objdump | Tool::Readobj => postprocess::demangle(&output.stdout),
        Tool::Size => postprocess::size(&output.stdout),
    };

    stdout.write_all(&processed_output)?;

    if output.status.success() {
        Ok(0)
    } else {
        Ok(output.status.code().unwrap_or(1))
    }
}

fn cargo_build(matches: &ArgMatches, metadata: &Metadata) -> Result<Option<Artifact>> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cargo = Command::new(cargo);
    cargo.arg("build");

    let (build_type, verbose) = cargo_build_args(matches, &mut cargo);
    let quiet = matches.get_flag("quiet");

    cargo.arg("--message-format=json-diagnostic-rendered-ansi");
    cargo.stdout(Stdio::piped());

    if verbose > 0 {
        eprintln!("{cargo:?}");
    }

    let mut child = cargo.spawn()?;
    let stdout = BufReader::new(child.stdout.take().expect("Pipe to cargo process failed"));

    // Note: We call `collect` to ensure we don't block stdout which could prevent the process from exiting
    let messages = Message::parse_stream(stdout).collect::<Vec<_>>();

    let status = child.wait()?;
    if !status.success() {
        bail!("Failed to parse crate metadata");
    }

    let mut target_artifact: Option<Artifact> = None;
    for message in messages {
        match message? {
            Message::CompilerArtifact(artifact) => {
                if metadata.workspace_members.contains(&artifact.package_id)
                    && build_type.matches(&artifact)
                {
                    if target_artifact.is_some() {
                        bail!("Can only have one matching artifact but found several");
                    }

                    target_artifact = Some(artifact);
                }
            }
            Message::CompilerMessage(msg) => {
                if !quiet || verbose > 1 {
                    if let Some(rendered) = msg.message.rendered {
                        print!("{rendered}");
                    }
                }
            }
            _ => (),
        }
    }

    if target_artifact.is_none() {
        bail!("Could not determine the wanted artifact");
    }

    Ok(target_artifact)
}

fn cargo_build_args<'a>(matches: &'a ArgMatches, cargo: &mut Command) -> (BuildType<'a>, u64) {
    if matches.get_flag("quiet") {
        cargo.arg("--quiet");
    }

    if let Some(package) = matches.get_one::<String>("package") {
        cargo.arg("--package");
        cargo.arg(package);
    }

    if let Some(config) = matches.get_many::<String>("config") {
        for c in config {
            cargo.args(["--config", c]);
        }
    }

    if let Some(jobs) = matches.get_one::<String>("jobs") {
        cargo.arg("-j");
        cargo.arg(jobs);
    }

    let build_type = if matches.get_flag("lib") {
        cargo.args(["--lib"]);
        BuildType::Lib
    } else if let Some(bin_name) = matches.get_one::<String>("bin") {
        cargo.args(["--bin", bin_name]);
        BuildType::Bin(bin_name)
    } else if let Some(example_name) = matches.get_one::<String>("example") {
        cargo.args(["--example", example_name]);
        BuildType::Example(example_name)
    } else if let Some(test_name) = matches.get_one::<String>("test") {
        cargo.args(["--test", test_name]);
        BuildType::Test(test_name)
    } else if let Some(bench_name) = matches.get_one::<String>("bench") {
        cargo.args(["--bench", bench_name]);
        BuildType::Bench(bench_name)
    } else {
        BuildType::Any
    };

    if matches.get_flag("release") {
        cargo.arg("--release");
    }

    if let Some(profile) = matches.get_one::<String>("profile") {
        cargo.arg("--profile");
        cargo.arg(profile);
    }

    if let Some(manifest_path) = matches.get_one::<String>("manifest-path") {
        cargo.args(["--manifest-path", manifest_path]);
    }

    if let Some(features) = matches.get_many::<String>("features") {
        for feature in features {
            cargo.args(["--features", feature]);
        }
    }
    if matches.get_flag("no-default-features") {
        cargo.arg("--no-default-features");
    }
    if matches.get_flag("all-features") {
        cargo.arg("--all-features");
    }

    // NOTE we do *not* use `project.target()` here because Cargo will figure things out on
    // its own (i.e. it will search and parse .cargo/config, etc.)
    if let Some(target) = matches.get_one::<String>("target") {
        cargo.args(["--target", target]);
    }

    let verbose = matches.get_count("verbose") as u64;
    if verbose > 1 {
        cargo.arg(format!("-{}", "v".repeat((verbose - 1) as usize)));
    }

    if let Some(color) = matches.get_one::<String>("color") {
        cargo.arg("--color");
        cargo.arg(color);
    }

    if matches.get_flag("frozen") {
        cargo.arg("--frozen");
    }

    if matches.get_flag("locked") {
        cargo.arg("--locked");
    }

    if matches.get_flag("offline") {
        cargo.arg("--offline");
    }

    if let Some(unstable_features) = matches.get_many::<String>("unstable-features") {
        for unstable_feature in unstable_features {
            cargo.args(["-Z", unstable_feature]);
        }
    }

    (build_type, verbose)
}
