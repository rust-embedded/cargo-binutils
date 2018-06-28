extern crate cargo_binutils as cbu;

use std::process;

fn main() {
    match cbu::run(|ctxt| ctxt.profdata(), false) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
