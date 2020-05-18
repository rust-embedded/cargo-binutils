use std::process;

use cargo_binutils::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo readobj --bin app -- -s` - Displays the section headers
`cargo readobj --bin app -- -t` - Displays the symbol table";

fn main() {
    match cargo_binutils::run(Tool::Readobj, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
