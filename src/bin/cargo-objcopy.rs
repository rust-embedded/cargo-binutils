extern crate cargo_binutils as cbu;

use std::process;

fn main() {
    match cbu::run(false, |ctxt| ctxt.objcopy(), |_ctxt, stdout| stdout.into()) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
