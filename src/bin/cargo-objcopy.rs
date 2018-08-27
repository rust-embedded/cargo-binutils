extern crate cargo_binutils as cbu;

use std::process;

use cbu::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo objcopy --bin foo -- -O binary foo.hex`  - converts the output (e.g. ELF) into binary format";

fn main() {
    match cbu::run(Tool::Objcopy, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
