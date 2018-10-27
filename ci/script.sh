set -euxo pipefail

main() {
    cargo check
    cargo install --path . -f

    cargo nm --bin cargo-nm -v > /dev/null
    cargo objdump --bin cargo-objdump -v -- -d > /dev/null
    cargo objcopy --bin cargo-objdump -v -- -O binary objdump.bin > /dev/null
    cargo size --bin cargo-size -v
    cargo strip --bin cargo-strip -v
}

main
