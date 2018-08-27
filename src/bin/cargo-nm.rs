extern crate cargo_binutils as cbu;
extern crate clap;

use std::process;

use cbu::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo nm --lib`                           - lists all symbols
`cargo nm --lib -- -print-size -size-sort` - lists all symbols sorted by size (smallest first)";

fn main() {
    match cbu::run(Tool::Nm, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
