extern crate cargo_binutils as binutils;

fn main() {
    binutils::Tool::Objcopy.rust_exec()
}
