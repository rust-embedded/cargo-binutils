use std::env::consts::EXE_SUFFIX;
use std::path::PathBuf;
use std::process::Command;
use std::{env, process};

use anyhow::Result;

use crate::rustc::rustlib;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Ar,
    As,
    Cov,
    Lld,
    Nm,
    Objcopy,
    Objdump,
    Profdata,
    Readobj,
    Size,
    Strip,
}

impl Tool {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ar => "ar",
            Self::As => "as",
            Self::Cov => "cov",
            Self::Lld => "lld",
            Self::Nm => "nm",
            Self::Objcopy => "objcopy",
            Self::Objdump => "objdump",
            Self::Profdata => "profdata",
            Self::Readobj => "readobj",
            Self::Size => "size",
            Self::Strip => "strip",
        }
    }

    #[must_use]
    pub fn exe(self) -> String {
        match self {
            Self::Lld => format!("rust-lld{EXE_SUFFIX}"),
            _ => format!("llvm-{}{}", self.name(), EXE_SUFFIX),
        }
    }

    pub fn path(self) -> Result<PathBuf> {
        let mut path = rustlib()?;
        path.push(self.exe());
        Ok(path)
    }

    /// Forwards execution to the specified tool.
    /// If the tool fails to start or is not found this process exits with
    /// status code 101 the same as if the process has a panic!
    pub fn rust_exec(self) -> ! {
        let path = match self.path() {
            Err(e) => {
                eprintln!("Failed to find tool: {}\n{}", self.name(), e);
                process::exit(101)
            }
            Ok(p) => p,
        };

        if !path.exists() {
            eprintln!(
                "Could not find tool: {}\nat: {}\nConsider `rustup component add llvm-tools`",
                self.name(),
                path.to_string_lossy()
            );
            process::exit(102)
        }

        // Note: The first argument is the name of the binary (e.g. `rust-nm`)
        let args = env::args().skip(1);

        // Spawn the process and check if the process did spawn
        let status = match Command::new(path).args(args).status() {
            Err(e) => {
                eprintln!("Failed to execute tool: {}\n{}", self.name(), e);
                process::exit(101)
            }
            Ok(s) => s,
        };

        // Forward the exit code from the tool
        process::exit(status.code().unwrap_or(101));
    }

    /// Parses arguments for `cargo $tool` and then if needed executes `cargo build`
    /// before parsing the required arguments to `rust-$tool`.
    /// If the tool fails to start or is not found this process exits with
    /// status code 101 the same as if the process has a panic!
    pub fn cargo_exec(self, examples: Option<&str>) -> ! {
        let matches = crate::args(self, examples);

        match crate::run(self, &matches) {
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(101)
            }
            Ok(ec) => process::exit(ec),
        }
    }

    // Whether this tool requires the project to be previously built
    #[must_use]
    pub const fn needs_build(self) -> bool {
        match self {
            Self::Ar | Self::As | Self::Cov | Self::Lld | Self::Profdata => false,
            Self::Nm | Self::Objcopy | Self::Objdump | Self::Readobj | Self::Size | Self::Strip => {
                true
            }
        }
    }
}
