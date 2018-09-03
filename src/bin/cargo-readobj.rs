extern crate cargo_binutils as cbu;
extern crate clap;

use std::process;

use cbu::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo readobj --bin app -- -s` - Displays the section headers
`cargo readobj --bin app -- -t` - Displays the symbol table";

fn main() {
    match cbu::run(Tool::Readobj, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
