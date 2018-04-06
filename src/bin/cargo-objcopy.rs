extern crate cargo_binutils as cbu;

use std::process;

fn main() {
    match cbu::run(|ctxt| ctxt.objcopy(), false) {
        Err(e) => eprintln!("Error: {:?}", e),
        Ok(ec) => process::exit(ec),
    }
}
