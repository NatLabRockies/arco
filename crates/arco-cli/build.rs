fn main() {
    // When the xpress feature is enabled, embed an rpath so the final binary
    // can locate libxprs.dylib at runtime without requiring DYLD_LIBRARY_PATH.
    #[cfg(feature = "xpress")]
    {
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
            if cfg!(unix) {
                println!("cargo:rustc-link-arg-bins=-Wl,-rpath,{lib_dir}");
            }
        }
    }
}
