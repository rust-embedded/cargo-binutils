# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- Added `--quiet` and `--color` arguments to be passed to `cargo build`
- Added `--test` and `--bench` build arguments to allow targeting testing artifacts

### Fixed

- Fixed handling of `--lib` argument to reflect how its used with `cargo build`
- Fixed `--` argument handling to ensure argument validation
- Fixed `--lib` to be able to support `lib`, `rlib`, `dylib`, `cdylib`, etc.

### Changed

- Changed help output to more closely reflect the help command of `cargo` subcommands

## [v0.2.0] - 2020-04-11

### Added

- Implement the typical cargo behaviour to determine an artifact to use if none was explictly specified

### Changed

- Use edition 2018 and bump some dependencies
- Compare artifact against the requested artifact instead of package name
- Use `cargo-metadata` instead of `cargo-project` (potentially a **breaking change**)

## [v0.1.7] - 2019-11-15

### Added

- Add rust-* direct aliases to the llvm tools (e.g. rust-ar, rust-ld, rust-lld etc).

### Fixed

- Fixed detection of workspaces (via cargo-project dependency)

## [v0.1.6] - 2018-12-18

### Added

- All the `cargo-$tool` subcommands that include a build step now accept the
  flags: `--features` and `--all-features`. These flags are passed as is to the
  `cargo build` command that these subcommands invoke.

- This project now produces binary artifacts for each new release. You can find
  the binaries in the [releases] page.

[releases]: https://github.com/rust-embedded/cargo-binutils/releases

## [v0.1.5] - 2018-10-28

### Added

- Path inference support for WASM binaries

### Fixed

- Path inference on windows hosts

## [v0.1.4] - 2018-09-09

### Fixed

- `cargo-objdump` now produces appropriate output when disassembling for Thumb
  and PowerPC targets.

- Cargo now respects the `build.target-dir` setting in `.cargo/config`.

## [v0.1.3] - 2018-09-01

### Added

- A `cargo-readobj` subcommand that proxies the `llvm-readobj` tool, which is
  similar to GNU's `readelf`. Note that this subcommand requires nightly from
  2018-09-01 or newer.

## [v0.1.2] - 2018-08-28

### Added

- Build and inspect mode. Some tools don't require that the path to the artifact
  is passed as an argument in this mode. See README for more details.

### Fixed

- `cargo-objdump`: More robust detection of the target architecture. The
  `riscv32imac-unknown-none-elf` is now properly supported.

- More robust post processing. If the output of the LLVM tool can *not* be
  processed then the original output is shown instead of just showing an error
  message.

## [v0.1.1] - 2018-07-15

### Added

- `cargo-strip` subcommand

### Changed

- The `llvm-tools` component was renamed to `llvm-tools-preview`

## v0.1.0 - 2018-06-28

Initial release

[Unreleased]: https://github.com/rust-embedded/cargo-binutils/compare/v0.2.0...HEAD
[v0.2.0]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.7...v0.2.0
[v0.1.7]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.6...v0.1.7
[v0.1.6]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.5...v0.1.6
[v0.1.5]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.4...v0.1.5
[v0.1.4]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.3...v0.1.4
[v0.1.3]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.0...v0.1.1
