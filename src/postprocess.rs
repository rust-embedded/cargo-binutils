use std::borrow::Cow;
use std::str;

use regex::{Captures, Regex};
use rustc_demangle;

// Here we post process the output of some tools to improve. If the output of the tool is not valid
// UTF-8 then we don't touch it.

// This pass demangles *all* the Rust symbols in the input
pub fn demangle(bytes: &[u8]) -> Cow<[u8]> {
    let re = Regex::new(r#"_Z.+?E\b"#).expect("BUG: Malformed Regex");

    if let Ok(text) = str::from_utf8(bytes) {
        match re.replace_all(text, |cs: &Captures| {
            format!("{}", rustc_demangle::demangle(cs.get(0).unwrap().as_str()))
        }) {
            Cow::Borrowed(s) => s.as_bytes().into(),
            Cow::Owned(s) => s.into_bytes().into(),
        }
    } else {
        bytes.into()
    }
}

// This pass turns the addresses in the output of `size` into hexadecimal format
pub fn size(bytes: &[u8]) -> Cow<[u8]> {
    if let Ok(text) = str::from_utf8(bytes) {
        let mut s = text
            .lines()
            .map(|line| -> Cow<str> {
                let mut parts = line.split_whitespace();

                if let Some((needle, addr)) = parts
                    .nth(2)
                    .and_then(|part| part.parse::<u64>().ok().map(|addr| (part, addr)))
                {
                    let pos = line.rfind(needle).unwrap();
                    let hex_addr = format!("{:#x}", addr);
                    let start = pos + needle.as_bytes().len() - hex_addr.as_bytes().len();

                    format!("{}{}", &line[..start], hex_addr).into()
                } else {
                    line.into()
                }
            }).collect::<Vec<_>>()
            .join("\n");

        // `text.lines()` loses the trailing newline so we restore it here
        s.push('\n');

        s.into_bytes().into()
    } else {
        bytes.into()
    }
}
