use rustc_cfg::Cfg;

// Here we map Rust arches to LLVM arches
//
// Rust knows these arches as of 1.28 (from librustc_target/abi/call/mod.rs)
//
// - aarch64
// - arm
// - asmjs (emscripten fork only)
// - hexagon
// - mips
// - mips64
// - msp430
// - nvptx
// - nvptx64
// - powerpc
// - riscv32
// - riscv64
// - s390x
// - sparc
// - sparc64
// - wasm32
// - x86
// - x86_64
//
// Rust LLVM knows these arches of 7.0 (from `llvm-objdump -version`)
//
// - aarch64
// - aarch64_be
// - arm
// - arm64
// - armeb
// - hexagon
// - mips
// - mips64
// - mips64el
// - mipsel
// - msp430
// - nvptx
// - nvptx64
// - ppc32
// - ppc64
// - ppc64le
// - riscv32
// - riscv64
// - sparc
// - sparcel
// - sparcv9
// - systemz
// - thumb
// - thumbeb
// - wasm32
// - wasm64
// - x86
// - x86-64
pub fn arch_name<'a>(cfg: &'a Cfg, target: &'a str) -> &'a str {
    const BIG: &str = "big";
    const LITTLE: &str = "little";

    let endian = &*cfg.target_endian;
    let arch = &*cfg.target_arch;

    if target.starts_with("thumb") {
        // no way to tell from `--print cfg` that the target is thumb only so we
        // completely rely on the target name here
        if endian == BIG {
            "thumbeb"
        } else {
            "thumb"
        }
    } else {
        match (arch, endian) {
            // non standard endianness
            ("aarch64", BIG) => "aarch64_be",
            ("arm", BIG) => "armeb",
            ("mips", LITTLE) => "mipsel",
            ("mips64", LITTLE) => "mips64el",
            ("powerpc64", LITTLE) => "ppc64le",
            ("sparc", LITTLE) => "sparcel",

            // names that match
            ("powerpc", _) => "ppc32",
            ("powerpc64", BIG) => "ppc64",
            ("sparc64", _) => "sparcv9",
            ("s390x", _) => "systemz",
            ("x86_64", _) => "x86-64",

            // all the other names match as of 1.28
            _ => arch,
        }
    }
}
