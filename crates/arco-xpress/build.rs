fn main() {
    println!("cargo:rerun-if-env-changed=XPRESSDIR");

    let xpress_dir = std::env::var("XPRESSDIR").ok().or_else(|| {
        for path in &["/opt/xpressmp", "/Library/xpressmp", "C:\\xpressmp"] {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        None
    });

    if let Some(dir) = xpress_dir {
        let lib_dir = format!("{dir}/lib");
        println!("cargo:rustc-link-search=native={lib_dir}");
        println!("cargo:rustc-link-lib=dylib=xprs");
        println!("cargo:rustc-cfg=has_xpress");
    } else {
        println!(
            "cargo:warning=XPRESSDIR not set and Xpress not found in default locations. \
             FFI functions will not link. Set XPRESSDIR to your Xpress installation directory."
        );
    }
}
