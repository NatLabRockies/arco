fn main() {
    // highs-sys builds HiGHS with /MD (dynamic CRT) but does not emit link
    // directives for the MSVC C runtime. Without these, the linker cannot
    // resolve standard C math/runtime symbols (`ldexp`, `round`, `malloc`, …).
    // See: https://github.com/rust-or/highs-sys/issues/21
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-lib=ucrt");
        println!("cargo:rustc-link-lib=msvcrt");
    }
}
