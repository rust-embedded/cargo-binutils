use std::process;

use cargo_binutils::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo objdump --lib --release -- -d`                   - disassemble
`cargo objdump --bin foo --release -- -s -j .rodata`    - prints the contents of the .rodata section";

fn main() {
    match cargo_binutils::run(Tool::Objdump, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
