fn main() {
    let xpress_dir = std::env::var("XPRESSDIR").unwrap_or_else(|_| {
        for path in &["/opt/xpressmp", "/Library/xpressmp", "C:\\xpressmp"] {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }
        panic!(
            "XPRESSDIR environment variable not set and Xpress not found in default locations. \
             Set XPRESSDIR to your Xpress installation directory."
        );
    });

    let lib_dir = format!("{xpress_dir}/lib");
    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-lib=dylib=xprs");
    println!("cargo:rerun-if-env-changed=XPRESSDIR");
}
