fn main() {
    if target_family().as_deref() == Some("unix") {
        if let Ok(libscip_dir) = std::env::var("DEP_SCIP_LIBDIR") {
            println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,{libscip_dir}");
        }
        for libscip_dir in bundled_scip_lib_dirs() {
            println!(
                "cargo:rustc-link-arg-cdylib=-Wl,-rpath,{}",
                libscip_dir.display()
            );
        }
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        for gcc_lib_dir in [
            "/opt/homebrew/opt/gcc/lib/gcc/current",
            "/usr/local/opt/gcc/lib/gcc/current",
        ] {
            if std::path::Path::new(gcc_lib_dir).exists() {
                println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,{gcc_lib_dir}");
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

fn target_family() -> Option<String> {
    std::env::var("CARGO_CFG_TARGET_FAMILY").ok()
}

fn bundled_scip_lib_dirs() -> Vec<std::path::PathBuf> {
    let manifest_dir =
        std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    let repo_root = manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .unwrap_or(manifest_dir);
    let target_dir = repo_root.join("target");
    let mut dirs = Vec::new();
    for profile in ["debug", "release"] {
        if let Some(path) = newest_bundled_scip_lib_dir(&target_dir.join(profile).join("build")) {
            dirs.push(path);
        }
    }
    dirs
}

fn newest_bundled_scip_lib_dir(build_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(build_dir).ok()?;
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path().join("out").join("scip_install").join("lib");
            if !path.join("libscip.dylib").exists() && !path.join("libscip.so").exists() {
                return None;
            }
            let modified = path
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}
