use std::process;

use cargo_binutils::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo objcopy --bin foo -- -O binary foo.hex`  - converts the output (e.g. ELF) into binary format";

fn main() {
    match cargo_binutils::run(Tool::Objcopy, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
