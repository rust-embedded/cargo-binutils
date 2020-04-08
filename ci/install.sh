set -euxo pipefail

main() {
    if [ -z $TRAVIS_TAG ]; then
        rustup component add llvm-tools-preview
    fi

    if [ $T = x86_64-pc-windows-msvc ]; then
        rustup target add $T
    fi
    if [ $T = x86_64-unknown-linux-musl ]; then
        rustup target add $T
        curl -L https://github.com/japaric/musl-bin/raw/master/14.04.tar.gz | tar xz -C $HOME
    fi
}

main
