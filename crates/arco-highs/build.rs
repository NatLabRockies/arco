use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-lib=ucrt");
        println!("cargo:rustc-link-lib=msvcrt");
    }

    println!("cargo:rerun-if-env-changed=ARCO_HIGHS_ROOT");
    println!("cargo:rerun-if-env-changed=HIGHS_ROOT");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_ALLOW_CROSS");

    if env::var_os("CARGO_FEATURE_BUNDLED_HIGHS").is_some() {
        return;
    }

    if let Some(root) = env::var_os("ARCO_HIGHS_ROOT").or_else(|| env::var_os("HIGHS_ROOT")) {
        let root = PathBuf::from(root);
        if link_from_pkg_config_file(&root).is_ok() {
            return;
        }
        link_from_root(&root);
        return;
    }

    if link_from_pkg_config_command() {
        return;
    }

    eprintln!(
        "could not locate HiGHS; run through scripts/with_solver_build_env.sh or set ARCO_HIGHS_ROOT"
    );
    std::process::exit(1);
}

fn link_from_pkg_config_file(root: &Path) -> Result<(), String> {
    let pc_file = root.join("lib/pkgconfig/highs.pc");
    let contents = std::fs::read_to_string(&pc_file).map_err(|error| error.to_string())?;
    let mut variables = BTreeMap::new();
    let mut libs = None;

    for line in contents.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            variables.insert(key.trim().to_string(), value.trim().to_string());
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == "Libs" {
                libs = Some(value.trim().to_string());
            }
        }
    }

    let Some(libs) = libs else {
        return Err("highs.pc did not define Libs".to_string());
    };
    let expanded = expand_pkg_config_variables(&libs, &variables);
    emit_link_args(expanded.split_whitespace());
    Ok(())
}

fn expand_pkg_config_variables(value: &str, variables: &BTreeMap<String, String>) -> String {
    let mut expanded = value.to_string();
    for _ in 0..8 {
        let mut changed = false;
        for (key, replacement) in variables {
            let token = format!("${{{key}}}");
            if expanded.contains(&token) {
                expanded = expanded.replace(&token, replacement);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    expanded
}

fn link_from_root(root: &Path) {
    println!(
        "cargo:rustc-link-search=native={}",
        root.join("lib").display()
    );
    println!("cargo:rustc-link-lib=highs");

    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("linux") || target.contains("windows-gnu") {
        println!("cargo:rustc-link-lib=stdc++");
    }
}

fn link_from_pkg_config_command() -> bool {
    let output = Command::new("pkg-config")
        .args(["--libs", "highs"])
        .output();
    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let libs = String::from_utf8_lossy(&output.stdout);
    emit_link_args(libs.split_whitespace());
    true
}

fn emit_link_args<'a>(args: impl IntoIterator<Item = &'a str>) {
    for arg in args {
        if let Some(path) = arg.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(lib) = arg.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        } else if let Some(link_arg) = arg.strip_prefix("-Wl,") {
            println!("cargo:rustc-link-arg=-Wl,{link_arg}");
        }
    }
}
