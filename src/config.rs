use std::fs::File;
use std::io::Read;
use std::path::Path;

use toml;

use Result;

#[derive(Deserialize)]
pub struct Config {
    pub build: Option<Build>,
}

impl Config {
    pub fn get(cwd: &Path) -> Result<Option<Self>> {
        if let Some(root) = search(cwd, ".cargo/config") {
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

/// Search for `file` in `path` and its parent directories
fn search<'p>(mut path: &'p Path, file: &str) -> Option<&'p Path> {
    loop {
        if path.join(file).exists() {
            return Some(path);
        }

        if let Some(p) = path.parent() {
            path = p;
        } else {
            return None;
        }
    }
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
