use std::path::Path;

fn main() {
    // Unlike highs-sys, which builds a static libhighs.a, scip-sys bundled
    // installs dynamic SCIP libraries. On Unix targets, emit runtime search
    // paths for those libraries; on macOS, also include Homebrew GCC runtime
    // paths used by the bundled SCIP binary.
    if target_family().as_deref() == Ok("unix") {
        if let Ok(libscip_dir) = std::env::var("DEP_SCIP_LIBDIR") {
            emit_rpath(&libscip_dir);
        }
    }

    if target_os().as_deref() == Ok("macos") {
        for gcc_lib_dir in [
            "/opt/homebrew/opt/gcc/lib/gcc/current",
            "/usr/local/opt/gcc/lib/gcc/current",
        ] {
            if Path::new(gcc_lib_dir).exists() {
                println!("cargo:rustc-link-search=native={gcc_lib_dir}");
                println!("cargo:rustc-link-lib=dylib=gfortran");
                println!("cargo:rustc-link-lib=dylib=quadmath");
                println!("cargo:rustc-link-lib=dylib=gcc_s.1.1");
                emit_rpath(gcc_lib_dir);
            }
        }
    }
}

fn target_family() -> Result<String, std::env::VarError> {
    std::env::var("CARGO_CFG_TARGET_FAMILY")
}

fn target_os() -> Result<String, std::env::VarError> {
    std::env::var("CARGO_CFG_TARGET_OS")
}

fn emit_rpath(path: &str) {
    println!("cargo:rustc-link-arg=-Wl,-rpath,{path}");
}
