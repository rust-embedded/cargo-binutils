# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- Added command suggestion messages if no target artifact is matched

### Fixed

- Fixed `cargo build` messages being buffered and causing an output delay for larger projects
- Fixed implicit artifact matching to allow for library crates to be matched

### Changed

- Improved error messaging when `llvm-tools-preview` hasn't been installed

## [v0.3.1] - 2020-07-30

### Fixed

- Fixed confusion caused by `build.rs` in the project generating an virtual
  artifact resulting in `error: Can only have one matching artifact but found
  several`

### Changed

- Allow the combination of `--features` `--no-default-features` and `--all-features` flags

## [v0.3.0] - 2020-05-28

### Added

- Added `--quiet` and `--color` arguments to be passed to `cargo build`
- Added `--test` and `--bench` build arguments to allow targeting testing artifacts
- Added `--package` argument so its possible to specify a target package in a workspace
- Added `--no-default-features` cargo argument support
- Added `--profile` argument to allow specifying the profile to build the target package with
- Added `--frozen`, `--locked`, `--offline` cargo argument support
- Added `-Z` argument to allow use of unstable cargo features

### Fixed

- Fixed handling of `--lib` argument to reflect how its used with `cargo build`
- Fixed `--` argument handling to ensure argument validation
- Fixed `--lib` to be able to support `lib`, `rlib`, `dylib`, `cdylib`, etc.
- Fixed panic due to broken pipe, caused by `bail!` while `cargo build` is running.
  Additionally fixes broken output format due to interrupted stderr output from `cargo build`
- Fixed `rust-*` binaries exiting with exit code 0 if the tool was not found
- Fixed `cargo build` running for `cargo profdata` when its not required
- Fixed `--features` not allowing multiple

### Changed

- Changed help output to more closely reflect the help command of `cargo` subcommands
- Removed `walkdir` dependency by using expected path to tool executable
- Updated `cargo_metadata 0.9 -> 0.10`
- Replaced `failure` dependency with [`anyhow`](https://github.com/dtolnay/anyhow)
- Allowed  multiple levels of verbosity and verbose cargo output via `-vv` and `-vvv`

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

[Unreleased]: https://github.com/rust-embedded/cargo-binutils/compare/v0.3.1...HEAD
[v0.3.1]: https://github.com/rust-embedded/cargo-binutils/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rust-embedded/cargo-binutils/compare/v0.2.0...v0.3.0
[v0.2.0]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.7...v0.2.0
[v0.1.7]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.6...v0.1.7
[v0.1.6]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.5...v0.1.6
[v0.1.5]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.4...v0.1.5
[v0.1.4]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.3...v0.1.4
[v0.1.3]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/rust-embedded/cargo-binutils/compare/v0.1.0...v0.1.1
