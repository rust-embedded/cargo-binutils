const EXAMPLES: &str = "

EXAMPLES

`cargo strip --bin foo --release -- -strip-all -o stripped`     - strips all symbols";

fn main() {
    cargo_binutils::Tool::Strip.cargo_exec(Some(EXAMPLES))
}
