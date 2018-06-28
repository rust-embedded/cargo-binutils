set -euxo pipefail

main() {
    cargo check
    cargo install --path . -f

    cargo nm -v -- target/release/cargo-nm > /dev/null
    cargo objdump -v -- -d target/release/cargo-objdump > /dev/null
    cargo objcopy -v -- -O binary target/release/cargo-objdump objdump.bin > /dev/null
    cargo size -v -- target/release/cargo-size
}

main
