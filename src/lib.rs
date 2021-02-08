#![deny(warnings)]

use std::io::{self, BufReader, Write};
use std::path::{Component, Path};
use std::process::{Command, Stdio};
use std::{env, str};

use anyhow::{bail, Result};
use cargo_metadata::{Artifact, CargoOpt, Message, Metadata, MetadataCommand};
use clap::{App, AppSettings, Arg, ArgMatches};
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
        let target_name = if let Some(Component::Normal(path)) = target_path.components().next() {
            let path = path.to_string_lossy();
            // TODO: How will custom profiles impact this?
            if path == "debug" || path == "release" {
                // Looks like this artifact was built for the host.
                rustc::version_meta()?.host.as_str().into()
            } else {
                // The artifact
                path
            }
        } else {
            unreachable!();
        };

        Self::from_target_name(&target_name)
    }

    /// Get a context structure from a provided target flag, used when cargo
    /// was not used to build the binary.
    fn from_flag(metadata: Metadata, target_flag: Option<&str>) -> Result<Self> {
        let host_target_name = rustc::version_meta()?.host.as_str();

        // Get the "default" target override in .cargo/config.
        let mut config_target_name = None;
        let config: toml::Value;

        if let Some(path) = search(&metadata.workspace_root, ".cargo/config") {
            config = parse(&path.join(".cargo/config"))?;
            config_target_name = config
                .get("build")
                .and_then(|build| build.get("target"))
                .and_then(|target| target.as_str());
        }

        // Find the actual target.
        let target_name = target_flag
            .or(config_target_name)
            .unwrap_or(host_target_name);

        Self::from_target_name(target_name)
    }

    fn from_target_name(target_name: &str) -> Result<Self> {
        let cfg = Cfg::of(target_name).map_err(|e| e.compat())?;

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

impl<'a> BuildType<'a> {
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
    let about = format!(
        "Proxy for the `llvm-{}` tool shipped with the Rust toolchain.",
        name
    );
    let after_help = format!(
        "\
The arguments specified *after* the `--` will be passed to the proxied tool invocation.

To see all the flags the proxied tool accepts run `cargo-{} -- -help`.{}",
        name,
        examples.unwrap_or("")
    );

    let app = App::new(format!("cargo-{}", name))
        .about(&*about)
        .version(env!("CARGO_PKG_VERSION"))
        .settings(&[
            AppSettings::UnifiedHelpMessage,
            AppSettings::DeriveDisplayOrder,
            AppSettings::DontCollapseArgsInUsage,
        ])
        // as this is used as a Cargo subcommand the first argument will be the name of the binary
        // we ignore this argument
        .args(&[
            Arg::with_name("binary-name").hidden(true),
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .multiple(true)
                .help("Use verbose output (-vv cargo verbose or -vvv for build.rs output)"),
            Arg::with_name("args")
                .last(true)
                .multiple(true)
                .help("The arguments to be proxied to the tool"),
        ])
        .after_help(&*after_help);

    if tool.needs_build() {
        app.args(&[
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("Don't print build output from `cargo build`"),
            Arg::with_name("package")
                .long("package")
                .short("p")
                .takes_value(true)
                .value_name("SPEC")
                .help("Package to build (see `cargo help pkgid`)"),
            Arg::with_name("jobs")
                .long("jobs")
                .short("j")
                .value_name("N")
                .help("Number of parallel jobs, defaults to # of CPUs"),
            Arg::with_name("lib")
                .long("lib")
                .conflicts_with_all(&["bin", "example", "test", "bench"])
                .help("Build only this package's library"),
            Arg::with_name("bin")
                .long("bin")
                .takes_value(true)
                .value_name("NAME")
                .conflicts_with_all(&["lib", "example", "test", "bench"])
                .help("Build only the specified binary"),
            Arg::with_name("example")
                .long("example")
                .takes_value(true)
                .value_name("NAME")
                .conflicts_with_all(&["lib", "bin", "test", "bench"])
                .help("Build only the specified example"),
            Arg::with_name("test")
                .long("test")
                .takes_value(true)
                .value_name("NAME")
                .conflicts_with_all(&["lib", "bin", "example", "bench"])
                .help("Build only the specified test target"),
            Arg::with_name("bench")
                .long("bench")
                .takes_value(true)
                .value_name("NAME")
                .conflicts_with_all(&["lib", "bin", "example", "test"])
                .help("Build only the specified bench target"),
            Arg::with_name("release")
                .long("release")
                .help("Build artifacts in release mode, with optimizations"),
            Arg::with_name("profile")
                .long("profile")
                .value_name("PROFILE-NAME")
                .help("Build artifacts with the specified profile"),
            Arg::with_name("features")
                .long("features")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .value_name("FEATURES")
                .help("Space-separated list of features to activate"),
            Arg::with_name("all-features")
                .long("all-features")
                .help("Do not activate the `default` feature"),
            Arg::with_name("no-default-features")
                .long("no-default-features")
                .help("Activate all available features"),
            Arg::with_name("target")
                .long("target")
                .takes_value(true)
                .value_name("TRIPLE")
                .help("Target triple for which the code is compiled"),
            Arg::with_name("color")
                .long("color")
                .takes_value(true)
                .possible_values(&["auto", "always", "never"])
                .help("Coloring: auto, always, never"),
            Arg::with_name("frozen")
                .long("frozen")
                .help("Require Cargo.lock and cache are up to date"),
            Arg::with_name("locked")
                .long("locked")
                .help("Require Cargo.lock is up to date"),
            Arg::with_name("offline")
                .long("offline")
                .help("Run without accessing the network"),
            Arg::with_name("unstable-features")
                .short("Z")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
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
    if let Some(features) = matches.values_of("features") {
        metadata_command.features(CargoOpt::SomeFeatures(
            features.map(|s| s.to_owned()).collect(),
        ));
    }
    if matches.is_present("no-default-features") {
        metadata_command.features(CargoOpt::NoDefaultFeatures);
    }
    if matches.is_present("all-features") {
        metadata_command.features(CargoOpt::AllFeatures);
    }
    let metadata = metadata_command.exec()?;
    if metadata.workspace_members.is_empty() {
        bail!("Unable to find workspace members");
    }

    let target_artifact = if tool.needs_build() {
        cargo_build(&matches, &metadata)?
    } else {
        None
    };

    let mut tool_args = vec![];
    if let Some(args) = matches.values_of("args") {
        tool_args.extend(args);
    }

    let mut lltool = Command::new(format!("rust-{}", tool.name()));

    if tool == Tool::Objdump {
        let ctxt = if let Some(artifact) = &target_artifact {
            Context::from_artifact(metadata, artifact)?
        } else {
            Context::from_flag(metadata, matches.value_of("target"))?
        };

        let arch_name = llvm::arch_name(&ctxt.cfg, &ctxt.target);

        if arch_name == "thumb" {
            // `-arch-name=thumb` doesn't produce the right output so instead we pass
            // `-triple=$target`, which contains more information about the target
            lltool.args(&["--triple", &ctxt.target]);
        } else {
            lltool.args(&["--arch-name", arch_name]);
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
                Tool::Ar | Tool::Cov | Tool::Lld | Tool::Profdata => {}
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

    if matches.is_present("verbose") {
        eprintln!("{:?}", lltool);
    }

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let output = lltool.stderr(Stdio::inherit()).output()?;

    // post process output
    let processed_output = match tool {
        Tool::Ar | Tool::Cov | Tool::Lld | Tool::Objcopy | Tool::Profdata | Tool::Strip => {
            output.stdout.into()
        }
        Tool::Nm | Tool::Objdump | Tool::Readobj => postprocess::demangle(&output.stdout),
        Tool::Size => postprocess::size(&output.stdout),
    };

    stdout.write_all(&*processed_output)?;

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
    let quiet = matches.is_present("quiet");

    cargo.arg("--message-format=json");
    cargo.stdout(Stdio::piped());

    if verbose > 0 {
        eprintln!("{:?}", cargo);
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
                        print!("{}", rendered);
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

fn cargo_build_args<'a>(matches: &'a ArgMatches<'a>, cargo: &mut Command) -> (BuildType<'a>, u64) {
    if matches.is_present("quiet") {
        cargo.arg("--quiet");
    }

    if let Some(package) = matches.value_of("package") {
        cargo.arg("--package");
        cargo.arg(package);
    }

    if let Some(jobs) = matches.value_of("jobs") {
        cargo.arg("-j");
        cargo.arg(jobs);
    }

    let build_type = if matches.is_present("lib") {
        cargo.args(&["--lib"]);
        BuildType::Lib
    } else if let Some(bin_name) = matches.value_of("bin") {
        cargo.args(&["--bin", bin_name]);
        BuildType::Bin(bin_name)
    } else if let Some(example_name) = matches.value_of("example") {
        cargo.args(&["--example", example_name]);
        BuildType::Example(example_name)
    } else if let Some(test_name) = matches.value_of("test") {
        cargo.args(&["--test", test_name]);
        BuildType::Test(test_name)
    } else if let Some(bench_name) = matches.value_of("bench") {
        cargo.args(&["--bench", bench_name]);
        BuildType::Bench(bench_name)
    } else {
        BuildType::Any
    };

    if matches.is_present("release") {
        cargo.arg("--release");
    }

    if let Some(profile) = matches.value_of("profile") {
        cargo.arg("--profile");
        cargo.arg(profile);
    }

    if let Some(features) = matches.values_of("features") {
        for feature in features {
            cargo.args(&["--features", feature]);
        }
    }
    if matches.is_present("no-default-features") {
        cargo.arg("--no-default-features");
    }
    if matches.is_present("all-features") {
        cargo.arg("--all-features");
    }

    // NOTE we do *not* use `project.target()` here because Cargo will figure things out on
    // its own (i.e. it will search and parse .cargo/config, etc.)
    if let Some(target) = matches.value_of("target") {
        cargo.args(&["--target", target]);
    }

    let verbose = matches.occurrences_of("verbose");
    if verbose > 1 {
        cargo.arg(format!("-{}", "v".repeat((verbose - 1) as usize)));
    }

    if let Some(color) = matches.value_of("color") {
        cargo.arg("--color");
        cargo.arg(color);
    }

    if matches.is_present("frozen") {
        cargo.arg("--frozen");
    }

    if matches.is_present("locked") {
        cargo.arg("--locked");
    }

    if matches.is_present("offline") {
        cargo.arg("--offline");
    }

    if let Some(unstable_features) = matches.values_of("Z") {
        for unstable_feature in unstable_features {
            cargo.args(&["-Z", unstable_feature]);
        }
    }

    (build_type, verbose)
}
