use std::env;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use rustc_version::VersionMeta;

struct RefError<'a, T>(&'a T);

impl<'a, T: Debug> Debug for RefError<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(self.0, f)
    }
}

impl<'a, T: Display> Display for RefError<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self.0, f)
    }
}

impl<'a, T: Error> Error for RefError<'a, T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

lazy_static::lazy_static! {
    static ref VERSION_META: Result<VersionMeta, rustc_version::Error> = rustc_version::version_meta();
}

pub fn version_meta() -> Result<&'static VersionMeta> {
    VERSION_META
        .as_ref()
        .map_err(|error| RefError(error).into())
}

pub fn sysroot() -> Result<String> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc).arg("--print").arg("sysroot").output()?;
    // Note: We must trim() to remove the `\n` from the end of stdout
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

// See: https://github.com/rust-lang/rust/blob/564758c4c329e89722454dd2fbb35f1ac0b8b47c/src/bootstrap/dist.rs#L2334-L2341
pub fn rustlib() -> Result<PathBuf> {
    let sysroot = sysroot()?;
    let mut pathbuf = PathBuf::from(sysroot);
    pathbuf.push("lib");
    pathbuf.push("rustlib");
    pathbuf.push(&version_meta()?.host);
    pathbuf.push("bin");
    Ok(pathbuf)
}
