# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.3...HEAD
[v0.1.3]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.0...v0.1.1
