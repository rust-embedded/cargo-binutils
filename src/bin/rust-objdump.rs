extern crate cargo_binutils as binutils;

fn main() {
    binutils::Tool::Objdump.rust_exec()
}
