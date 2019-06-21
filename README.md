# `cargo-binutils`

> Cargo subcommands to invoke the LLVM tools shipped with the Rust toolchain

**NOTE** This is **not** an official Rust project.

This project is developed and maintained by the [Tools team][team].

## Features

- All Rust symbols in the output of the LLVM tools are automatically demangled.
- No need to pass to pass the path to the artifact as an argument if using the
  tool in "build and inspect" mode.

## Installation

``` console
$ cargo install cargo-binutils

$ rustup component add llvm-tools-preview
```

## Usage

This:

``` console
$ rust-$tool ${args[@]}
```

is basically sugar for:

``` console
$ $(find $(rustc --print sysroot) -name llvm-$tool) ${args[@]}
```

Apart from these `rust-*` tools, which are direct proxies for the llvm tools in
the `llvm-tools-preview` component, the crate also provides some Cargo
subcommands that will first build the project and then run the llvm tool on the
output artifact. This:

``` console
$ cargo size --example foo
```

is sugar for:

``` console
$ cargo build --example foo
$ rust-size target/examples/foo
```

In the case of `cargo-objdump` the architecture of the compilation target is
passed as `-arch-name=$target` to `llvm-objdump`. `-arch-name` specifies to
which architecture disassemble the object file to.

You can get more information about the CLI of each tool by running `rust-$tool
 -help`.

All the Cargo subcommands accept a `--verbose` / `-v` flag. In verbose mode the
`rust-$tool` invocation will be printed to stderr.

Build and inspect mode: Some subcommands accept the flags: `--bin`, `--example`,
`--lib`, `--target` and `--release`. These can be used to make the subcommand
first build the respective binary, example or library and have the path to the
artifact be automatically passed to the LLVM tool. This mode only works when the
subcommand is used from within a Cargo project.

*Disclaimer* Note that `cargo-binutils` simply proxies the LLVM tools in the
`llvm-tools-preview` component and the Rust project makes no guarantee about the
availability and the CLI of these tools -- i.e. the availability and CLI of
these tools may change as new Rust releases are made.

## Examples

### `nm`

List all symbols in an executable

``` console
$ cargo nm --bin app --release
0800040a T BusFault
0800040a T DebugMonitor
0800040a T DefaultHandler
0800065e T HardFault
0800040a T MemoryManagement
0800040a T NonMaskableInt
0800040a T PendSV
0800040c T Reset
0800040a T SVCall
0800040a T SysTick
0800040a T UsageFault
08000408 T UserHardFault
08000008 R __EXCEPTIONS
08000040 R __INTERRUPTS
08000004 R __RESET_VECTOR
08000000 R __STACK_START
```

List all symbols in an executable sorted by size (smallest first).

``` console
$ cargo nm --bin app --release -- -print-size -size-sort
0800040a 00000002 T DefaultHandler
08000408 00000002 T UserHardFault
08000004 00000004 R __RESET_VECTOR
08000400 00000008 T main
08000008 00000038 R __EXCEPTIONS
0800040c 00000252 T Reset
08000040 000003c0 R __INTERRUPTS
```

### `objcopy`

Transform the output of Cargo (ELF) into binary format.

``` console
$ cargo objcopy --bin app --release -- -O binary app.bin

$ stat --printf="%s\n" app.bin
1642
```

### `objdump`

Disassemble a binary.

``` console
$ cargo objdump --bin app --release -- -disassemble -no-show-raw-insn
target/thumbv7m-none-eabi/debug/app:    file format ELF32-arm-little

Disassembly of section .text:
main:
 8000400:       push    {r7, lr}
 8000402:       bl      #608
 8000406:       b       #-8 <main+0x2>

UserHardFault:
 8000408:       trap

UsageFault:
 800040a:       trap

Reset:
 800040c:       push.w  {r4, r5, r6, r7, r8, lr}
 8000410:       movw    r0, #0
 8000414:       movw    r2, #0
 8000418:       movt    r0, #8192
 800041c:       movt    r2, #8192
(..)
```

### `size`

Print binary size in System V format

``` console
$ cargo size --bin app --release -- -A -x
target/thumbv7m-none-eabi/release/app  :
section               size         addr
.vector_table        0x400    0x8000000
.text                0x26a    0x8000400
.rodata                0x2    0x800066a
.data                    0   0x20000000
.bss                     0   0x20000000
.debug_str          0x107e            0
.debug_loc           0x3e2            0
.debug_abbrev        0x31b            0
.debug_info         0x19f9            0
.debug_ranges         0xe8            0
.debug_macinfo         0x1            0
.debug_pubnames      0x9ff            0
.debug_pubtypes      0x8dd            0
.ARM.attributes       0x2e            0
.debug_frame          0x6c            0
.debug_line          0x69b            0
.debug_aranges        0x40            0
Total               0x531a
```

### `strip`

Strip all symbols from the build artifact

``` console
$ stat --printf="%s\n" target/release/hello
4094240

$ cargo-strip --bin hello --release -- -strip-all -O smaller-hello

$ stat --printf="%s\n" smaller-hello
424432
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], the maintainer of this crate, the [Tools team][team], promises
to intervene to uphold that code of conduct.

[CoC]: CODE_OF_CONDUCT.md
[team]: https://github.com/rust-embedded/wg#the-tools-team
