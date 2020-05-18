use std::process;

use cargo_binutils::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo size --bin foo --release`        - prints binary size in Berkeley format
`cargo size --bin foo --release -- -A`  - prints binary size in System V format";

fn main() {
    match cargo_binutils::run(Tool::Size, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
