use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use toml::de;

use Result;

/// Search for `file` in `path` and its parent directories
pub fn search<'p>(mut path: &'p Path, file: &str) -> Option<&'p Path> {
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

pub fn parse<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    Ok(de::from_str(&s)?)
}
