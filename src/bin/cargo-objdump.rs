extern crate cargo_binutils as cbu;

use std::process;

use crate::cbu::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo objdump --lib --release -- -d`                   - disassemble
`cargo objdump --bin foo --release -- -s -j .rodata`    - prints the contents of the .rodata section";

fn main() {
    match cbu::run(Tool::Objdump, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
