// #![deny(warnings)]

#[macro_use]
extern crate failure;
extern crate regex;
extern crate rustc_demangle;
extern crate rustc_version;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate walkdir;

use std::borrow::Cow;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, str};

pub use failure::Error;
use regex::{Captures, Regex};
use walkdir::WalkDir;

use config::Config;

mod config;

pub type Result<T> = std::result::Result<T, failure::Error>;

/// Execution context
// TODO this should be some sort of initialize once, read-only singleton
pub struct Context {
    /// Directory within the Rust sysroot where the llvm tools reside
    bindir: PathBuf,
    host: String,
    /// Regex to find mangled Rust symbols
    re: Regex,
    // NOTE `None` means that the target is the host
    target: Option<String>,
    // Arguments after `--`
    tool_args: Vec<String>,
    verbose: bool,
}

impl Context {
    /* Constructors */
    fn new() -> Result<Self> {
        let cwd = env::current_dir()?;

        let config = Config::get(&cwd)?;
        let meta = rustc_version::version_meta()?;
        let host = meta.host;

        let mut args = env::args().skip(2);

        let mut target = None;
        let mut error = false;
        let mut verbose = false;
        while let Some(arg) = args.next() {
            if arg == "--target" {
                // duplicated target
                if target.is_some() {
                    error = true;
                    break;
                }

                target = args.next();

                // malformed invocation: `cargo nm --target`
                if target.is_none() {
                    error = true;
                    break;
                }
            } else if arg.starts_with("--target=") {
                // duplicated target
                if target.is_some() {
                    error = true;
                    break;
                }

                target = arg.split('=').nth(1).map(|s| s.to_owned());

                // malformed invocation: `cargo nm --target=`
                if target.is_none() {
                    error = true;
                    break;
                }
            } else if arg == "--" {
                break;
            } else if arg == "--verbose" || arg == "-v" {
                verbose = true;
            } else {
                error = true;
                break;
            }
        }

        let tool_args = args.collect();

        if error {
            bail!("malformed Cargo arguments");
        }

        target = target.or_else(|| config.and_then(|c| c.build.and_then(|b| b.target)));

        if target.as_ref() == Some(&host) {
            target = None;
        }

        let sysroot = String::from_utf8(
            Command::new("rustc")
                .arg("--print")
                .arg("sysroot")
                .output()?
                .stdout,
        )?;

        for entry in WalkDir::new(sysroot.trim()).into_iter() {
            let entry = entry?;

            if entry.file_name() == &*exe("llvm-size") {
                let bindir = entry.path().parent().unwrap().to_owned();

                return Ok(Context {
                    bindir,
                    host,
                    re: Regex::new(r#"_Z.+?E\b"#).expect("BUG: Malformed Regex"),
                    target,
                    tool_args,
                    verbose,
                });
            }
        }

        bail!(
            "`llvm-tools` component is missing or empty. Install it with `rustup component add \
             llvm-tools`"
        );
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

    pub fn profdata(&self) -> Command {
        self.tool("llvm-profdata")
    }

    pub fn size(&self) -> Command {
        self.tool("llvm-size")
    }

    pub fn tool_args(&self) -> &[String] {
        &self.tool_args
    }

    /* Private API */
    fn bindir(&self) -> &Path {
        &self.bindir
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
        Command::new(self.bindir().join(&*exe(name)))
    }
}

/// Shared entry point for the Cargo subcommands
pub fn run<F, G>(demangle: bool, tool: F, post_process: G) -> Result<i32>
where
    F: FnOnce(&Context) -> Command,
    G: for<'s> FnOnce(&Context, &'s str) -> Cow<'s, str>,
{
    let ctxt = Context::new()?;
    let mut tool = tool(&ctxt);

    tool.args(ctxt.tool_args());

    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    if ctxt.verbose {
        writeln!(stderr, "{:?}", tool).ok();
    }

    let output = tool.output()?;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let tool_stdout = if demangle {
        match ctxt.demangle(str::from_utf8(&output.stdout)?) {
            Cow::Borrowed(s) => Cow::Borrowed(s),
            Cow::Owned(s) => Cow::Owned(s),
        }
    } else {
        Cow::from(String::from_utf8(output.stdout)?)
    };

    let post_stdout = post_process(&ctxt, &*tool_stdout);

    Ok(if output.status.success() {
        stdout.write_all(post_stdout.as_bytes())?;
        stderr.write_all(&output.stderr)?;
        0
    } else {
        stdout.write_all(post_stdout.as_bytes())?;
        stderr.write_all(&output.stderr)?;
        output.status.code().unwrap_or(1)
    })
}

#[cfg(target_os = "windows")]
fn exe(name: &str) -> Cow<str> {
    format!("{}.exe", name).into()
}

#[cfg(not(target_os = "windows"))]
fn exe(name: &str) -> Cow<str> {
    name.into()
}
