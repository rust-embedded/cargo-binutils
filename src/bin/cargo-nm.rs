use std::process;

use cargo_binutils::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo nm --lib`                           - lists all symbols
`cargo nm --lib -- -print-size -size-sort` - lists all symbols sorted by size (smallest first)";

fn main() {
    match cargo_binutils::run(Tool::Nm, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
