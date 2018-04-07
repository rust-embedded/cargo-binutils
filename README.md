# `cargo-binutils`

> Cargo subcommands to invoke the LLVM tools shipped (\*) with the Rust toolchain

(\*) Except that they won't be shipped with the Rust toolchain until [rust-lang/rust#49584] is
approved. So for now these subcommands invoke the LLVM tools in the user's $PATH.

[rust-lang/rust#49584]: https://github.com/rust-lang/rust/issues/49584

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
$ cargo nm -- target/thumbv7m-none-eabi/release/app
20000074 D _sgot
20000074 D _sheap
08000774 A _sidata
20005000 R _stack_start
08000400 R _stext
08000000 R _svector_table
20000008 B errno
080005f4 T free
20000014 d impure_data
080005e4 T malloc

$ cargo nm -- -print-size -size-sort target/thumbv7m-none-eabi/release/app
080005e4 00000010 T malloc
0800059c 00000020 T _sbrk
08000750 00000020 T _sbrk_r
080005c0 00000024 t app::main::h8f28ebffab84c118
08000008 00000038 R EXCEPTIONS
20000014 00000060 d impure_data
08000604 00000098 T _free_r
0800069c 000000b4 T _malloc_r
08000400 0000018e t cortex_m_rt::reset_handler::h2bf1df29cde02662
08000040 000003c0 r app::INTERRUPTS::h4487021ef2e905fd
```

### `objcopy`

``` console
$ cargo objcopy -- -O binary target/thumbv7m-none-eabi/release/app app.bin

$ stat --printf="%s\n" app.bin
8716
```

### `objdump`

``` console
$ cargo objdump -- -disassemble -no-show-raw-insn target/thumbv7m-none-eabi/debug/app
target/thumbv7m-none-eabi/debug/app:    file format ELF32-arm-little

Disassembly of section .text:
cortex_m_rt::reset_handler::h69c216b4053d343c:
 8000400:       push    {r7, lr}
 8000402:       mov     r7, sp
 8000404:       sub     sp, #16
 8000406:       movw    r0, #0
 800040a:       movt    r0, #8192
```

### `size`

``` console
$ cargo size -- -A -x target/thumbv7m-none-eabi/release/app
target/thumbv7m-none-eabi/release/app  :
section              size         addr
.vector_table       0x400    0x8000000
.text               0x374    0x8000400
.rodata                 0    0x8000774
.bss                  0xc   0x20000000
.data                0x68   0x2000000c
.ARM.attributes      0x34            0
.debug_frame        0x14c            0
Total               0x968
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
