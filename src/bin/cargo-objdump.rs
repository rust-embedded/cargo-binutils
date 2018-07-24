extern crate cargo_binutils as cbu;

use std::process;

fn main() {
    match cbu::run(
        true,
        |ctxt| ctxt.objdump(ctxt.tool_args().iter().all(|s| !s.ends_with(".wasm"))),
        |_ctxt, stdout| stdout.into(),
    ) {
        Err(e) => eprintln!("error: {}", e),
        Ok(ec) => process::exit(ec),
    }
}
