const EXAMPLES: &str = "

EXAMPLES

`cargo objdump --lib --release -- -d`                   - disassemble
`cargo objdump --bin foo --release -- -s -j .rodata`    - prints the contents of the .rodata section";

fn main() {
    cargo_binutils::Tool::Objdump.cargo_exec(Some(EXAMPLES))
}
