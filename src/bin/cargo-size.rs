extern crate cargo_binutils as cbu;

use std::process;

use cbu::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo size --bin foo --release`        - prints binary size in Berkeley format
`cargo size --bin foo --release -- -A`  - prints binary size in System V format";

fn main() {
    match cbu::run(Tool::Size, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
