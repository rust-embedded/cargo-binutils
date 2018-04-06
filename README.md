# `cargo-binutils`

> Cargo subcommands to invoke the LLVM tools shipped (\*) with the Rust toolchain

(\*) Except that they won't be shipped with the Rust toolchain until rust-lang/rust#49584 is
approved. So for now these subcommands invoke the LLVM tools in the user's $PATH.

## Features

- All Rust symbols in the output of the LLVM tools are automatically demangled.

## Installation

``` console
$ cargo install --git https://github.com/japaric/cargo-binutils
```

## Usage

This

``` console
$ cargo $tool -- ${args[@]}
```

is basically sugar for

``` console
$ $(find $(rustc --print sysroot) -name llvm-$tool) ${args[@]}
```

## Examples

### `nm`

``` console
$ cargo nm -- target/thumbv7m-none-eabi/app

$ cargo nm -- -print-size -size-sort target/thumbv7m-none-eabi/release/app
```

### `objcopy`

``` console
$ cargo objcopy -- -O binary target/thumbv7m-none-eabi/debug/app app.bin
```

### `objdump`

``` console
$ cargo objdump -- -disassemble -no-show-raw-insn target/thumbv7m-none-eabi/debug/app
```

### `size`

``` console
$ cargo size -- target/thumbv7m-none-eabi/release/app

$ cargo size -- -A -x target/thumbv7m-none-eabi/release/app
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
