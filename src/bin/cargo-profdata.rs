use std::process;

use cargo_binutils::Tool;

fn main() {
    match cargo_binutils::run(Tool::Profdata, None) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
