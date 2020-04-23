use std::path::PathBuf;
use std::process::Command;

use failure::Error;
use std::env;

pub fn sysroot() -> Result<String, Error> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc).arg("--print").arg("sysroot").output()?;
    // Note: We must trim() to remove the `\n` from the end of stdout
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

// See: https://github.com/rust-lang/rust/blob/564758c4c329e89722454dd2fbb35f1ac0b8b47c/src/bootstrap/dist.rs#L2334-L2341
pub fn rustlib() -> Result<PathBuf, Error> {
    let sysroot = sysroot()?;
    let mut pathbuf = PathBuf::from(sysroot);
    pathbuf.push("lib");
    pathbuf.push("rustlib");
    pathbuf.push(rustc_version::version_meta()?.host); // TODO: Prevent calling rustc_version::version_meta() multiple times
    pathbuf.push("bin");
    Ok(pathbuf)
}
