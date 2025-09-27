use std::borrow::Cow;
use std::str;

use regex::{Captures, Regex};

// Here we post process the output of some tools to improve. If the output of the tool is not valid
// UTF-8 then we don't touch it.

// This pass demangles *all* the Rust symbols in the input
pub fn demangle(bytes: &[u8]) -> Cow<'_, [u8]> {
    let re = Regex::new(r"_Z.+?E\b").expect("BUG: Malformed Regex");

    str::from_utf8(bytes).map_or_else(
        |_| bytes.into(),
        |text| match re.replace_all(text, |cs: &Captures<'_>| {
            format!("{}", rustc_demangle::demangle(cs.get(0).unwrap().as_str()))
        }) {
            Cow::Borrowed(s) => s.as_bytes().into(),
            Cow::Owned(s) => s.into_bytes().into(),
        },
    )
}

// This pass turns the addresses in the output of `size -A` into hexadecimal format
pub fn size(bytes: &[u8]) -> Cow<'_, [u8]> {
    str::from_utf8(bytes).map_or_else(
        |_| bytes.into(),
        |text| {
            let mut s = text
                .lines()
                .map(|line| -> Cow<'_, str> {
                    match line
                        .split_whitespace()
                        .nth(2)
                        .and_then(|part| part.parse::<u64>().ok().map(|addr| (part, addr)))
                    {
                        // the lines to postprocess have the form ".section_name 100 1024" where
                        // the second number is the address
                        Some((needle, addr)) if line.starts_with('.') => {
                            let pos = line.rfind(needle).unwrap();
                            let hex_addr = format!("{addr:#x}");
                            let start = pos + needle.len() - hex_addr.len();

                            format!("{}{}", &line[..start], hex_addr).into()
                        }
                        _ => line.into(),
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            // `text.lines()` loses the trailing newline so we restore it here
            s.push('\n');

            s.into_bytes().into()
        },
    )
}
