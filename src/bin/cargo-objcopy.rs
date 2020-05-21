const EXAMPLES: &str = "

EXAMPLES

`cargo objcopy --bin foo -- -O binary foo.hex`  - converts the output (e.g. ELF) into binary format";

fn main() {
    cargo_binutils::Tool::Objcopy.cargo_exec(Some(EXAMPLES))
}
