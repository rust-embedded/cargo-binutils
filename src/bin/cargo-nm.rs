extern crate cargo_binutils as cbu;
extern crate clap;

use std::process;

use cbu::Tool;

fn main() {
    match cbu::run(Tool::Nm) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
