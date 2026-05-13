fn main() {
    if target_family().as_deref() != Ok("unix") {
        return;
    }

    if let Ok(libscip_dir) = std::env::var("DEP_SCIP_LIBDIR") {
        emit_arco_rpath(&libscip_dir);
    }
    for libscip_dir in bundled_scip_lib_dirs() {
        emit_arco_rpath(&libscip_dir.display().to_string());
    }
}

fn target_family() -> Result<String, std::env::VarError> {
    std::env::var("CARGO_CFG_TARGET_FAMILY")
}

fn emit_arco_rpath(path: &str) {
    println!("cargo:rustc-link-arg-bin=arco=-Wl,-rpath,{path}");
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
