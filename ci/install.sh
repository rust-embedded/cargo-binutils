set -euxo pipefail

main() {
    if [ -z $TRAVIS_TAG ]; then
        rustup component add llvm-tools-preview
    else
        if [ $T = x86_64-unknown-linux-musl ]; then
            rustup target add $T
        fi
    fi
}

main
