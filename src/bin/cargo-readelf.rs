const EXAMPLES: &str = "

EXAMPLES

`cargo readelf --bin app -- -t` - Display the section details
`cargo readelf --bin app -- -s` - Display the symbol table";

fn main() {
    cargo_binutils::Tool::Readelf.cargo_exec(Some(EXAMPLES))
}
