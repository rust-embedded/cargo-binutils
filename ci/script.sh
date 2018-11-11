set -euxo pipefail

main() {
    cargo install --target $T --path . -f --debug

    cargo nm --bin cargo-nm -v > /dev/null
    cargo objdump --bin cargo-objdump -v -- -d > /dev/null
    if [ $TRAVIS_OS_NAME = linux ]; then
        cargo objcopy --bin cargo-objdump -v -- -O binary objdump.bin > /dev/null
    fi
    cargo size --bin cargo-size -v
    if [ $TRAVIS_OS_NAME = linux ]; then
        cargo strip --bin cargo-strip -v
    fi
}

# skip tests when building binaries and on successful merges to master
if [ -z ${TRAVIS_TAG:-} ] && [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_PULL_REQUEST != false ]; then
    main
fi
