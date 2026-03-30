fn main() {
    println!("cargo:rerun-if-env-changed=XPRESSDIR");

    let xpress_dir = std::env::var("XPRESSDIR").ok().or_else(|| {
        for path in &["/opt/xpressmp", "/Library/xpressmp", "C:\\xpressmp"] {
            if std::path::Path::new(path).exists() {
                return Some((*path).to_owned());
            }
        }
        None
    });

    match xpress_dir {
        Some(dir) => {
            let lib_dir = format!("{dir}/lib");
            println!("cargo:rustc-link-search=native={lib_dir}");
            println!("cargo:rustc-link-lib=dylib=xprs");
        }
        None => {
            panic!(
                "XPRESSDIR not set and Xpress SDK not found in default locations.\n\
                 Set XPRESSDIR to your FICO Xpress installation directory.\n\
                 \n\
                 On macOS (DMG install):\n\
                     export XPRESSDIR=\"/Applications/FICO Xpress/xpressmp\"\n\
                 \n\
                 On Linux:\n\
                     export XPRESSDIR=\"/opt/xpressmp\"\n\
                 \n\
                 To use HiGHS instead (no extra dependencies):\n\
                     cargo build without the xpress feature"
            );
        }
    }
}
