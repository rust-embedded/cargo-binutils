set -euxo pipefail

main() {
    local bins=(
        nm
        objcopy
        objdump
        profdata
        readobj
        size
        strip
    )

    mkdir stage
    for bin in ${bins[@]}; do
        cargo rustc --target $T --bin cargo-$bin --release -- -C lto
        cp target/$T/release/cargo-$bin stage
    done

    pushd stage
    tar czf ../$CRATE_NAME-$TRAVIS_TAG-$T.tar.gz *
    popd

    rm -rf stage
}

main
