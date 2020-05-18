use std::process;

use cargo_binutils::Tool;

const EXAMPLES: &str = "

EXAMPLES

`cargo strip --bin foo --release -- -strip-all -o stripped`     - strips all symbols";

fn main() {
    match cargo_binutils::run(Tool::Strip, Some(EXAMPLES)) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
