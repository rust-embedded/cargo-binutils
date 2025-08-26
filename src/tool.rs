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
    pub fn name(self) -> &'static str {
        match self {
            Tool::Ar => "ar",
            Tool::As => "as",
            Tool::Cov => "cov",
            Tool::Lld => "lld",
            Tool::Nm => "nm",
            Tool::Objcopy => "objcopy",
            Tool::Objdump => "objdump",
            Tool::Profdata => "profdata",
            Tool::Readobj => "readobj",
            Tool::Size => "size",
            Tool::Strip => "strip",
        }
    }

    pub fn exe(self) -> String {
        match self {
            Tool::Lld => format!("rust-lld{EXE_SUFFIX}"),
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
        };

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

        match crate::run(self, matches) {
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(101)
            }
            Ok(ec) => process::exit(ec),
        }
    }

    // Whether this tool requires the project to be previously built
    pub fn needs_build(self) -> bool {
        match self {
            Tool::Ar | Tool::As | Tool::Cov | Tool::Lld | Tool::Profdata => false,
            Tool::Nm | Tool::Objcopy | Tool::Objdump | Tool::Readobj | Tool::Size | Tool::Strip => {
                true
            }
        }
    }
}
