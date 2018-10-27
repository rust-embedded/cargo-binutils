set -euxo pipefail

main() {
    cargo install --path . -f --debug

    cargo nm --bin cargo-nm -v > /dev/null
    cargo objdump --bin cargo-objdump -v -- -d > /dev/null
    if [ $TRAVIS_OS_NAME = linux ]; then
        cargo objcopy --bin cargo-objdump -v -- -O binary objdump.bin > /dev/null
    fi
    cargo size --bin cargo-size -v
    cargo strip --bin cargo-strip -v
}

main
