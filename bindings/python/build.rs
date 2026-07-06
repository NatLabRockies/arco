fn main() {
    #[cfg(feature = "scip")]
    {
        println!("cargo:rerun-if-env-changed=ARCO_SCIP_LIBRARY_PATH");
        println!("cargo:rerun-if-env-changed=ARCO_SCIP_FORTRAN_RUNTIME_PATH");
        println!("cargo:rerun-if-env-changed=ARCO_SCIP_GCC_RUNTIME_PATH");

        if target_family().as_deref() == Some("unix") {
            for lib_dir in scip_runtime_paths() {
                println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,{lib_dir}");
            }
        }
    }

    #[cfg(feature = "xpress")]
    {
        println!("cargo:rerun-if-env-changed=XPRESSDIR");

        let Some(dir) = detect_xpress_dir() else {
            return;
        };

        if target_family().as_deref() == Some("unix") {
            // Help the extension module discover libxprs when it is loaded at runtime.
            let lib_dir = format!("{dir}/lib");
            println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,{lib_dir}");
        }
    }
}

#[cfg(feature = "scip")]
fn scip_runtime_paths() -> impl Iterator<Item = String> {
    [
        "ARCO_SCIP_LIBRARY_PATH",
        "ARCO_SCIP_FORTRAN_RUNTIME_PATH",
        "ARCO_SCIP_GCC_RUNTIME_PATH",
    ]
    .into_iter()
    .filter_map(|name| std::env::var(name).ok())
    .flat_map(|paths| {
        std::env::split_paths(&paths)
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
    })
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

#[cfg(any(feature = "scip", feature = "xpress"))]
fn target_family() -> Option<String> {
    std::env::var("CARGO_CFG_TARGET_FAMILY").ok()
}
