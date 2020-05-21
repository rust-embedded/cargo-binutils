const EXAMPLES: &str = "

EXAMPLES

`cargo nm --lib`                           - lists all symbols
`cargo nm --lib -- -print-size -size-sort` - lists all symbols sorted by size (smallest first)";

fn main() {
    cargo_binutils::Tool::Nm.cargo_exec(Some(EXAMPLES))
}
