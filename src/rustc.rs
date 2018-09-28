use std::process::Command;

use failure;

use Endian;

/// Parsed `rustc --print cfg`
pub struct Cfg {
    arch: String,
    endian: Endian,
}

impl Cfg {
    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl Cfg {
    pub fn parse(target: &str) -> Result<Self, failure::Error> {
        const MSG: &str = "parsing `rustc --print cfg`";

        let mut rustc = Command::new("rustc");
        rustc.args(&["--target", target]);

        let stdout = String::from_utf8(rustc.arg("--print").arg("cfg").output()?.stdout)?;

        let mut arch = None;
        let mut endian = None;
        for line in stdout.lines() {
            if line.starts_with("target_arch") {
                arch = Some(
                    line.split('"')
                        .nth(1)
                        .map(|s| s.to_owned())
                        .ok_or_else(|| failure::err_msg(MSG))?,
                );
            } else if line.starts_with("target_endian") {
                endian = Some(if line.ends_with("\"little\"") {
                    Endian::Little
                } else {
                    Endian::Big
                });
            }
        }

        if let (Some(arch), Some(endian)) = (arch, endian) {
            Ok(Cfg { arch, endian })
        } else {
            Err(failure::err_msg(MSG))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cfg;
    use Endian;

    #[test]
    fn x86_64() {
        let cfg = Cfg::parse("x86_64-unknown-linux-gnu").unwrap();

        assert_eq!(cfg.arch, "x86_64");
        assert_eq!(cfg.endian, Endian::Little);
    }
}
