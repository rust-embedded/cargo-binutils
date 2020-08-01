#![deny(warnings)]

use std::collections::HashMap;
use std::io::{self, BufReader, Write};
use std::path::Component;
use std::process::{Command, Stdio};
use std::{env, str};

use anyhow::{bail, Result};
use cargo_metadata::{Artifact, CargoOpt, Message, MetadataCommand, PackageId};
use clap::{App, AppSettings, Arg, ArgMatches};
use rustc_cfg::Cfg;

pub use tool::Tool;

mod llvm;
mod postprocess;
mod rustc;
mod tool;

#[derive(Debug)]
enum BuildType<'a> {
    Any,
    Bin(&'a str),
    Example(&'a str),
    Test(&'a str),
    Bench(&'a str),
    CustomBuild, // Note: Only currently used for filtering when BuildType::Any is used
    Lib,
}

impl<'a> BuildType<'a> {
    fn matches(&self, artifact: &Artifact) -> bool {
        let name = &artifact.target.name;
        // For info about 'kind' values see:
        // https://github.com/rust-lang/cargo/blob/95b22d2874ea82a91e55db6286e87aaf40f8d4d5/src/cargo/core/manifest.rs#L111-L126
        let kind = &artifact.target.kind;
        match self {
            // Note: BuildType::Any matches all artifacts but is then later filtered to find the most appropriate artifact
            BuildType::Any => true,
            BuildType::Bin(target_name) => kind[0] == "bin" && name == *target_name,
            BuildType::Example(target_name) => kind[0] == "example" && name == *target_name,
            BuildType::Test(target_name) => kind[0] == "test" && name == *target_name,
            BuildType::Bench(target_name) => kind[0] == "bench" && name == *target_name,
            BuildType::CustomBuild => kind[0] == "custom-build",
            // Since LibKind can be an arbitrary string `LibKind:Other(String)` we filter by what it can't be
            BuildType::Lib => kind.iter().any(|s| {
                s != "bin" && s != "example" && s != "test" && s != "custom-build" && s != "bench"
            }),
        }
    }

    fn from_artifact(artifact: &'a Artifact) -> BuildType<'a> {
        let name = &artifact.target.name;
        match artifact.target.kind[0].as_str() {
            "bin" => BuildType::Bin(name),
            "example" => BuildType::Example(name),
            "test" => BuildType::Test(name),
            "bench" => BuildType::Bench(name),
            "custom-build" => BuildType::CustomBuild,
            _ => {
                if BuildType::Lib.matches(artifact) {
                    return BuildType::Lib;
                }
                BuildType::Any
            }
        }
    }

    fn generate_command(tool: Tool, artifact: &Artifact) -> String {
        match BuildType::from_artifact(artifact) {
            BuildType::Any => format!(
                "Unknown target: {}, {:?}",
                artifact.target.name, artifact.target.kind
            ),
            BuildType::CustomBuild => format!(
                "Unable to create command for custom-build target: {}",
                artifact.target.name
            ),
            BuildType::Bin(target_name) => format!("cargo {} --bin {}", tool.name(), target_name),
            BuildType::Example(target_name) => {
                format!("cargo {} --example {}", tool.name(), target_name)
            }
            BuildType::Test(target_name) => format!("cargo {} --test {}", tool.name(), target_name),
            BuildType::Bench(target_name) => {
                format!("cargo {} --bench {}", tool.name(), target_name)
            }
            BuildType::Lib => format!("cargo {} --lib {}", tool.name(), artifact.target.name),
        }
    }

    fn generate_command_with_package(tool: Tool, artifact: &Artifact) -> String {
        match BuildType::from_artifact(artifact) {
            BuildType::Any => format!(
                "Unknown target: {} {}, {:?}",
                artifact.package_id, artifact.target.name, artifact.target.kind,
            ),
            BuildType::CustomBuild => format!(
                "Unable to create command for custom-build target: {} {}",
                artifact.package_id, artifact.target.name,
            ),
            BuildType::Bin(target_name) => format!(
                "cargo {} --package {} --bin {}",
                tool.name(),
                artifact.package_id,
                target_name,
            ),
            BuildType::Example(target_name) => format!(
                "cargo {} --package {} --example {}",
                tool.name(),
                artifact.package_id,
                target_name,
            ),
            BuildType::Test(target_name) => format!(
                "cargo {} --package {} --test {}",
                tool.name(),
                artifact.package_id,
                target_name,
            ),
            BuildType::Bench(target_name) => format!(
                "cargo {} --package {} --bench {}",
                tool.name(),
                artifact.package_id,
                target_name,
            ),
            BuildType::Lib => format!(
                "cargo {} --package {} --lib {}",
                tool.name(),
                artifact.package_id,
                artifact.target.name,
            ),
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
    let mut lltool = Command::new(format!("rust-{}", tool.name()));

    if tool.needs_build() {
        cargo_build(tool, &matches, &mut lltool)?
    }

    // User flags
    if let Some(args) = matches.values_of("args") {
        lltool.args(args);
    }

    if matches.is_present("verbose") {
        eprintln!("{:?}", lltool);
    }

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let output = lltool.stderr(Stdio::inherit()).output()?;

    // post process output
    let processed_output = match tool {
        Tool::Ar | Tool::Lld | Tool::Objcopy | Tool::Profdata | Tool::Strip => output.stdout.into(),
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

fn cargo_build(tool: Tool, matches: &ArgMatches, lltool: &mut Command) -> Result<()> {
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

    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cargo = Command::new(cargo);
    cargo.arg("build");

    let build_type = cargo_build_args(matches, &mut cargo);

    cargo.arg("--message-format=json");
    cargo.stdout(Stdio::piped());

    if matches.is_present("verbose") {
        eprintln!("{:?}", cargo);
    }

    let mut child = cargo.spawn()?;
    let stdout = BufReader::new(child.stdout.take().expect("Pipe to cargo process failed"));

    // Note: To ensure we don't corrupt the output messages from the compiler we wait until it has
    //   completed before doing processing that could terminate the process.
    let mut artifacts: Vec<Artifact> = vec![];
    for message in Message::parse_stream(stdout) {
        match message? {
            Message::CompilerArtifact(artifact) => {
                if metadata.workspace_members.contains(&artifact.package_id)
                    && build_type.matches(&artifact)
                {
                    artifacts.push(artifact);
                }
            }
            Message::CompilerMessage(msg) => {
                if let Some(rendered) = msg.message.rendered {
                    // Note: We want to send these as soon as possible since larger projects
                    //   can take some time to build so don't buffer them
                    print!("{}", rendered);
                }
            }
            _ => (),
        }
    }

    let status = child.wait()?;
    if !status.success() {
        if matches.is_present("verbose") {
            bail!("Failed to run `{:?}`: {:?}", cargo, status);
        }
        match status.code() {
            Some(code) => bail!(
                "Failed to run `cargo build` got exit code: {}\n\
                Use `cargo {} -v ...` to display the full `cargo build` command",
                code,
                tool.name(),
            ),
            None => bail!(
                "Failed to run `cargo build`: terminated by signal\n\
                Use `cargo {} -v ...` to display the full `cargo build` command",
                tool.name()
            ),
        }
    }

    if artifacts.is_empty() {
        if matches.is_present("verbose") {
            bail!("`{:?}` didn't compile any targets", cargo);
        }
        bail!(
            "`cargo build` didn't compile any targets\n\
            Use `cargo {} -v ...` to display the full `cargo build` command",
            tool.name()
        );
    }

    // If no build type was not explicitly set and we have more than 1 artifact then we filter out what we don't want to match
    let mut filtered_artifacts = artifacts.clone();
    if let BuildType::Any = build_type {
        if filtered_artifacts.len() > 1 {
            filtered_artifacts = filtered_artifacts
                .into_iter()
                .filter(|a| !BuildType::CustomBuild.matches(a))
                .collect();
        }
    }

    if filtered_artifacts.is_empty() {
        bail!(
            "No matching targets.\n\
            Specified build type: {:?}\n\
            Possible targets: \n\
            {}",
            build_type,
            format_targets(tool, artifacts)
        );
    }

    // Note: Only allow a single artifact for now but we can handle this per tool easily later
    if filtered_artifacts.len() > 1 {
        bail!(
            "Can only have one matching target but found several.\n\
            Specified build type: {:?}\n\
            Possible targets: \n\
            {}",
            build_type,
            format_targets(tool, artifacts)
        );
    }

    let artifact = &filtered_artifacts[0];

    // Extra flags
    match tool {
        Tool::Objdump => {
            // Currently there is no clean way to get the target triple from cargo so we can only make
            // an approximation, we do this by extracting the target triple from the artifacts path.
            // For more info on the path structure see: https://doc.rust-lang.org/cargo/guide/build-cache.html

            // In the future it may be possible to replace this code and use a cargo feature:
            // See: https://github.com/rust-lang/cargo/issues/5579, https://github.com/rust-lang/cargo/issues/8002

            // Should always succeed.
            let target_path = artifact.filenames[0].strip_prefix(metadata.target_directory)?;
            let target_name = if let Some(Component::Normal(path)) = target_path.components().next()
            {
                let path = path.to_string_lossy();
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

            let cfg = Cfg::of(&target_name).map_err(|e| e.compat())?;

            let arch_name = llvm::arch_name(&cfg, &target_name);

            if arch_name == "thumb" {
                // `-arch-name=thumb` doesn't produce the right output so instead we pass
                // `-triple=$target`, which contains more information about the target
                lltool.args(&["-triple", &target_name]);
            } else {
                lltool.args(&["-arch-name", arch_name]);
            }
        }
        Tool::Readobj => {
            // The default output style of `readobj` is JSON-like, which is not user friendly, so we
            // change it to the human readable GNU style
            lltool.arg("-elf-output-style=GNU");
        }
        _ => {}
    }

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
        Tool::Ar | Tool::Lld | Tool::Profdata => {}
        // for some tools we change the current working directory and
        // make the artifact path relative. This makes the path that the
        // tool will print easier to read. e.g. `libfoo.rlib` instead of
        // `/home/user/rust/project/target/$T/debug/libfoo.rlib`.
        Tool::Objdump | Tool::Nm | Tool::Readobj | Tool::Size => {
            let dir = file.parent().unwrap();
            if matches.is_present("verbose") {
                eprintln!("Running rust-{} from: {:?}", tool.name(), dir)
            }
            lltool.current_dir(dir).arg(file.file_name().unwrap());
        }
        Tool::Objcopy | Tool::Strip => {
            lltool.arg(file);
        }
    }

    Ok(())
}

fn cargo_build_args<'a>(matches: &'a ArgMatches<'a>, cargo: &mut Command) -> BuildType<'a> {
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

    build_type
}

fn format_targets(tool: Tool, artifacts: Vec<Artifact>) -> String {
    // Group targets by package id
    let mut map: HashMap<PackageId, Vec<&Artifact>> = HashMap::default();
    for artifact in &artifacts {
        if let Some(vec) = map.get_mut(&artifact.package_id) {
            vec.push(artifact);
        } else {
            map.insert(artifact.package_id.clone(), vec![artifact]);
        }
    }

    // If only one package exists then we don't need to
    if map.len() == 1 {
        return format!(
            "  {}",
            artifacts
                .iter()
                .map(|a| BuildType::generate_command(tool, a))
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }

    map.iter()
        .map(|(k, v)| {
            let str = v
                .iter()
                .map(|a| BuildType::generate_command_with_package(tool, a))
                .collect::<Vec<_>>()
                .join("\n  ");
            format!("Package: {}\n  {}", k, str)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
