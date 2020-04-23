extern crate cargo_binutils as binutils;

fn main() {
    binutils::Tool::Nm.rust_exec()
}
