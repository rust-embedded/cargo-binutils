set -euxo pipefail

main() {
    rustup component add llvm-tools-preview
}

main
