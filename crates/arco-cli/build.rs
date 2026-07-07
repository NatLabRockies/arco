fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");

    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux") => {
            // SCIP's libgfortran dependency also needs to resolve from the install dir.
            println!("cargo:rustc-link-arg-bin=arco=-Wl,--disable-new-dtags,-rpath,$ORIGIN");
        }
        Ok("macos") => {
            println!("cargo:rustc-link-arg-bin=arco=-Wl,-rpath,@loader_path");
        }
        _ => {}
    }
}
