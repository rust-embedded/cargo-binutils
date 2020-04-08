extern crate cargo_binutils as cbu;

use std::process;

use crate::cbu::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo strip --bin foo --release -- -strip-all -o stripped`     - strips all symbols";

fn main() {
    match cbu::run(Tool::Strip, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
