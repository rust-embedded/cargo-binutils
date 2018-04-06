#![deny(warnings)]

extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate rustc_demangle;
extern crate rustc_version;
extern crate toml;

use std::borrow::Cow;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, str};

use regex::{Captures, Regex};

pub use failure::Error;

use config::Config;

mod config;

pub type Result<T> = std::result::Result<T, failure::Error>;

/// Execution context
// TODO this should be some sort of initialize once, read-only singleton
pub struct Context {
    /// Directory within the Rust sysroot where the llvm tools reside
    bindir: Option<PathBuf>,
    host: String,
    /// Regex to find mangled Rust symbols
    re: Regex,
    // NOTE `None` means that the target is the host
    target: Option<String>,
}

impl Context {
    /* Constructors */
    fn new() -> Result<Self> {
        let cwd = env::current_dir()?;

        let config = Config::get(&cwd)?;
        let meta = rustc_version::version_meta()?;
        let host = meta.host;

        let mut target = config.and_then(|c| c.build.and_then(|b| b.target));

        if target.as_ref() == Some(&host) {
            target = None;
        }

        Ok(Context {
            // TODO
            bindir: None,
            host,
            target,
            re: Regex::new(r#"_Z.+?E\b"#).expect("BUG: Malformed Regex"),
        })
    }

    /* Public API */
    pub fn nm(&self) -> Command {
        self.tool("llvm-nm")
    }

    pub fn objcopy(&self) -> Command {
        self.tool("llvm-objcopy")
    }

    pub fn objdump(&self) -> Command {
        let mut objdump = self.tool("llvm-objdump");
        objdump.arg("-triple");
        // NOTE assumes that the target name equates its "llvm-target" option. This may not be true
        // for custom targets.
        objdump.arg(self.target());
        objdump
    }

    pub fn size(&self) -> Command {
        self.tool("llvm-size")
    }

    /* Private API */
    fn bindir(&self) -> Option<&Path> {
        self.bindir.as_ref().map(|p| &**p)
    }

    fn demangle<'i>(&self, input: &'i str) -> Cow<'i, str> {
        self.re.replace_all(input, |cs: &Captures| {
            format!("{}", rustc_demangle::demangle(cs.get(0).unwrap().as_str()))
        })
    }

    #[cfg(unused)]
    fn host(&self) -> &str {
        &self.host
    }

    fn target(&self) -> &str {
        self.target.as_ref().unwrap_or(&self.host)
    }

    fn tool(&self, name: &str) -> Command {
        if let Some(bindir) = self.bindir() {
            Command::new(bindir.join(name))
        } else {
            Command::new(name)
        }
    }
}

/// Shared entry point for the Cargo subcommands
pub fn run<F>(tool: F, demangle: bool) -> Result<i32>
where
    F: FnOnce(&Context) -> Command,
{
    let ctxt = Context::new()?;
    let mut tool = tool(&ctxt);

    tool.args(env::args().skip_while(|arg| arg != "--").skip(1));

    let output = tool.output()?;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    let tool_stdout = if demangle {
        match ctxt.demangle(str::from_utf8(&output.stdout)?) {
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(s) => Cow::Owned(s.into_bytes()),
        }
    } else {
        Cow::from(output.stdout)
    };
    Ok(if output.status.success() {
        stdout.write_all(&*tool_stdout)?;
        stderr.write_all(&output.stderr)?;
        0
    } else {
        stdout.write_all(&*tool_stdout)?;
        stderr.write_all(&output.stderr)?;
        output.status.code().unwrap_or(1)
    })
}
