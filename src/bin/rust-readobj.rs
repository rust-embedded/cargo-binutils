extern crate cargo_binutils as binutils;

fn main() {
    binutils::Tool::Readobj.rust_exec()
}
