extern crate cargo_binutils as binutils;

fn main() {
    binutils::Tool::Lld.rust_exec()
}
