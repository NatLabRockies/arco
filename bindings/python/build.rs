fn main() {
    #[cfg(feature = "xpress")]
    {
        println!("cargo:rerun-if-env-changed=XPRESSDIR");

        let Some(dir) = detect_xpress_dir() else {
            return;
        };

        if target_family().as_deref() == Some("unix") {
            let lib_dir = format!("{dir}/lib");
            println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,{lib_dir}");
        }
    }
}

#[cfg(feature = "xpress")]
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

#[cfg(feature = "xpress")]
fn target_family() -> Option<String> {
    std::env::var("CARGO_CFG_TARGET_FAMILY").ok()
}
