extern crate cargo_binutils as cbu;

use std::process;

use cbu::Tool;

fn main() {
    match cbu::run(Tool::Objcopy) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
