extern crate cargo_binutils as binutils;

fn main() {
    binutils::Tool::Strip.rust_exec()
}
