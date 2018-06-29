# `cargo-binutils`

> Cargo subcommands to invoke the LLVM tools shipped with the Rust toolchain

**NOTE** This is **not** an official Rust project.

## Features

- All Rust symbols in the output of the LLVM tools are automatically demangled.

## Installation

``` console
$ cargo install cargo-binutils

$ rustup component add llvm-tools
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

In the case of `cargo-objdump` the compilation target is passed as `-triple=$target` to
`llvm-objdump`. `-triple` specifies to which architecture disassemble the object file to.

You can get more information about the CLI of each tool by running `cargo $tool -- --help`.

`cargo $tool` accepts the flags: `--target` and `--verbose` / `-v`. In verbose mode the `llvm-$tool`
invocation will be printed to stderr.

*Disclaimer* Note that `cargo-binutils` simply proxies the LLVM tools in the `llvm-tools` component
and the Rust project makes no guarantee about the availability and the CLI of these tools -- i.e.
the availability and CLI of these tools may change as new Rust releases are made.

## Examples

### `nm`

``` console
$ cargo nm -- target/thumbv7m-none-eabi/release/app
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

``` console
$ cargo nm -- -print-size -size-sort target/thumbv7m-none-eabi/release/app
0800040a 00000002 T DefaultHandler
08000408 00000002 T UserHardFault
08000004 00000004 R __RESET_VECTOR
08000400 00000008 T main
08000008 00000038 R __EXCEPTIONS
0800040c 00000252 T Reset
08000040 000003c0 R __INTERRUPTS
```

### `objcopy`

``` console
$ cargo objcopy -- -O binary target/thumbv7m-none-eabi/release/app app.bin

$ stat --printf="%s\n" app.bin
1642
```

### `objdump`

``` console
$ cargo objdump -- -disassemble -no-show-raw-insn target/thumbv7m-none-eabi/debug/app
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

``` console
$ cargo size -- -A -x target/thumbv7m-none-eabi/release/app
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
