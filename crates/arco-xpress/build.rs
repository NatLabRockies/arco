fn main() {
    println!("cargo:rerun-if-env-changed=XPRESSDIR");

    let xpress_dir = detect_xpress_dir();

    match xpress_dir {
        Some(dir) => {
            let lib_dir = format!("{dir}/lib");
            println!("cargo:rustc-link-search=native={lib_dir}");
            println!("cargo:rustc-link-lib=dylib=xprs");
            if target_family().as_deref() == Some("unix") {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
                println!("cargo:rustc-link-arg-tests=-Wl,-rpath,{lib_dir}");
            }
        }
        None => {
            panic!(
                "XPRESSDIR not set and Xpress SDK not found in default locations.\n\
                 Set XPRESSDIR to your FICO Xpress installation directory.\n\
                 \n\
                 On macOS (DMG install):\n\
                     export XPRESSDIR=\"$HOME/opt/xpressmp\"\n\
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

fn detect_xpress_dir() -> Option<String> {
    if let Some(path) = std::env::var("XPRESSDIR")
        .ok()
        .filter(|value| !value.is_empty())
    {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = std::path::PathBuf::from(home);
        candidates.push(home.join("User Apps").join("FICO Xpress").join("xpressmp"));
        candidates.push(home.join("opt").join("xpressmp"));
    }
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let user_profile = std::path::PathBuf::from(user_profile);
        candidates.push(
            user_profile
                .join("AppData")
                .join("Local")
                .join("FICO Xpress")
                .join("xpressmp"),
        );
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let program_files = std::path::PathBuf::from(program_files);
        candidates.push(program_files.join("FICO Xpress").join("xpressmp"));
    }
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        let program_files_x86 = std::path::PathBuf::from(program_files_x86);
        candidates.push(program_files_x86.join("FICO Xpress").join("xpressmp"));
    }
    candidates.extend([
        std::path::PathBuf::from("/Applications/FICO Xpress/xpressmp"),
        std::path::PathBuf::from("/Volumes/FICO Xpress Installer/FICO Xpress/xpressmp"),
        std::path::PathBuf::from("/opt/xpressmp"),
        std::path::PathBuf::from("/Library/xpressmp"),
        std::path::PathBuf::from("C:\\xpressmp"),
    ]);

    candidates
        .into_iter()
        .find(|path| path.exists())
        .map(|path| path.display().to_string())
}

fn target_family() -> Option<String> {
    std::env::var("CARGO_CFG_TARGET_FAMILY").ok()
}
