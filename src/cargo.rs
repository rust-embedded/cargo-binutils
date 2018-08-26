use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use failure;
use toml;

use {util, Artifact, Result};

pub fn artifact(
    kind: Artifact,
    release: bool,
    target_flag: Option<&str>,
    build_target: Option<&str>,
) -> Result<PathBuf> {
    let (mut p, crate_name) = target_dir()?;

    if let Some(target) = target_flag.or(build_target) {
        p.push(target)
    }

    if release {
        p.push("release")
    } else {
        p.push("debug")
    }

    match kind {
        Artifact::Bin(bin) => p.push(&*::exe(bin)),
        Artifact::Example(ex) => p.push(format!("examples/{}", ex)),
        Artifact::Lib => p.push(format!("lib{}.rlib", crate_name)),
    }

    Ok(p)
}

// Find where the build directory is
//
// This also return the name of the crate
pub fn target_dir() -> Result<(PathBuf, String)> {
    let cwd = PathBuf::from(env::current_dir()?);

    let crate_root = util::search(&cwd, "Cargo.toml")
        .ok_or_else(|| failure::err_msg("not in a Cargo project"))?;

    let crate_name = util::parse::<toml::Value>(&crate_root.join("Cargo.toml"))?
        .get("package")
        .and_then(|val| val.get("name"))
        .and_then(|val| val.as_str().map(|s| s.to_owned()))
        .ok_or_else(|| failure::err_msg("parsing Cargo.toml"))?;

    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        return Ok((PathBuf::from(target_dir), crate_name));
    }

    // we found the root of the crate we are in but we could be inside a workspace
    // so let's search for an outer crate
    if let Some(workspace_root) = crate_root
        .parent()
        .and_then(|parent| util::search(parent, "Cargo.toml"))
    {
        if util::parse::<toml::Value>(&workspace_root.join("Cargo.toml"))?
            .get("workspace")
            .is_some()
        {
            // this is indeed a workspace
            return Ok((workspace_root.join("target"), crate_name));
        }
    }

    Ok((crate_root.join("target"), crate_name))
}

#[derive(Deserialize)]
pub struct Config {
    pub build: Option<Build>,
}

impl Config {
    pub fn get(cwd: &Path) -> Result<Option<Self>> {
        if let Some(root) = util::search(cwd, ".cargo/config") {
            let p = root.join(".cargo/config");

            if p.exists() {
                let mut toml = String::new();
                File::open(p)?.read_to_string(&mut toml)?;
                Ok(Some(toml::from_str(&toml)?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Deserialize)]
pub struct Build {
    pub target: Option<String>,
}

#[cfg(test)]
mod tests {
    use toml;

    use super::Config;

    #[test]
    fn empty_config() {
        let config: Config = toml::from_str("").unwrap();

        assert!(config.build.is_none());
    }

    #[test]
    fn config_empty_build() {
        let config: Config = toml::from_str("[build]").unwrap();

        assert!(config.build.unwrap().target.is_none());
    }

    #[test]
    fn config_build_target() {
        let config: Config = toml::from_str(
            r#"
[build]
target = "thumbv7m-none-eabi"
"#,
        ).unwrap();

        assert_eq!(config.build.unwrap().target.unwrap(), "thumbv7m-none-eabi")
    }

    #[test]
    fn config_no_build_target() {
        let config: Config = toml::from_str(
            r#"
[target.thumbv7m-none-eabi]
runner = "arm-none-eabi-gdb"
"#,
        ).unwrap();

        assert!(config.build.is_none());
    }

    #[test]
    fn config_build_target_plus() {
        let config: Config = toml::from_str(
            r#"
[target.thumbv7m-none-eabi]
runner = "arm-none-eabi-gdb"

[build]
target = "thumbv7m-none-eabi"
"#,
        ).unwrap();

        assert_eq!(config.build.unwrap().target.unwrap(), "thumbv7m-none-eabi");
    }
}
