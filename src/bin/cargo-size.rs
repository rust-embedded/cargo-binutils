const EXAMPLES: &str = "

EXAMPLES

`cargo size --bin foo --release`        - prints binary size in Berkeley format
`cargo size --bin foo --release -- -A`  - prints binary size in System V format";

fn main() {
    cargo_binutils::Tool::Size.cargo_exec(Some(EXAMPLES))
}
