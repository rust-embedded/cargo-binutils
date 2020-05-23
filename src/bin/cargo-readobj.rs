const EXAMPLES: &str = "

EXAMPLES

`cargo readobj --bin app -- -s` - Displays the section headers
`cargo readobj --bin app -- -t` - Displays the symbol table";

fn main() {
    cargo_binutils::Tool::Readobj.cargo_exec(Some(EXAMPLES))
}
