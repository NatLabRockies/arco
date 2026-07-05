#[cfg(feature = "bundled")]
use std::env;
#[cfg(feature = "bundled")]
use std::error::Error;
#[cfg(feature = "bundled")]
use std::fs;
#[cfg(feature = "bundled")]
use std::path::{Path, PathBuf};
#[cfg(feature = "bundled")]
use std::process::Command;

#[cfg(feature = "bundled")]
const SCIP_DEPLOY_VERSION: &str = "0.12.0";

#[cfg(feature = "bundled")]
const PYTHON_EXTRACT: &str = r#"
from pathlib import Path
import shutil
import sys
import zipfile

archive = Path(sys.argv[1])
install = Path(sys.argv[2])
staging = Path(sys.argv[3])

def unpack(zip_path: Path, dest: Path) -> None:
    shutil.rmtree(dest, ignore_errors=True)
    dest.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(zip_path) as zip_file:
        zip_file.extractall(dest)

def entries(path: Path):
    return [entry for entry in path.iterdir() if entry.name != "__MACOSX"]

def normalize(source: Path, dest: Path) -> None:
    shutil.rmtree(dest, ignore_errors=True)
    current_entries = entries(source)
    if len(current_entries) == 1 and current_entries[0].is_dir():
        shutil.move(str(current_entries[0]), dest)
        return

    dest.mkdir(parents=True, exist_ok=True)
    for entry in current_entries:
        shutil.move(str(entry), dest / entry.name)

unpack(archive, staging)
top_entries = entries(staging)
if len(top_entries) == 1 and top_entries[0].suffix.lower() == ".zip":
    nested = staging.with_name(staging.name + "-nested")
    unpack(top_entries[0], nested)
    normalize(nested, install)
else:
    normalize(staging, install)

if not (install / "lib").exists():
    raise SystemExit(f"{install / 'lib'} does not exist after extraction")
if not (install / "include").exists():
    raise SystemExit(f"{install / 'include'} does not exist after extraction")
"#;

/// Map the current target OS/arch to the platform tag used both for the
/// prebuilt SCIP download and for selecting the matching prebuilt bindings in
/// `src/bindings/<tag>.rs`. Keeping a single source of truth ensures the
/// downloaded library and the committed bindings always refer to the same
/// platform.
#[cfg(feature = "bundled")]
pub fn target_string() -> String {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let os_string = if os == "linux" && arch == "x86_64" {
        "linux"
    } else if os == "linux" && arch == "aarch64" {
        "linux-arm"
    } else if os == "macos" && arch == "x86_64" {
        "macos-intel"
    } else if os == "macos" && arch == "aarch64" {
        "macos-arm"
    } else if os == "windows" && arch == "x86_64" {
        "windows"
    } else {
        panic!("Unsupported OS-arch combination: {}-{}", os, arch);
    };

    os_string.to_string()
}

#[cfg(feature = "bundled")]
pub fn prepare_scip() -> PathBuf {
    if let Some(path) = bundled_dir_from_env() {
        return path;
    }

    download_scip()
}

#[cfg(feature = "bundled")]
fn bundled_dir_from_env() -> Option<PathBuf> {
    let target = env::var("TARGET").ok();
    let target_env = target
        .as_deref()
        .map(|target| format!("SCIP_SYS_BUNDLED_DIR_{}", target.replace('-', "_")));

    if let Some(name) = target_env.as_deref() {
        println!("cargo:rerun-if-env-changed={name}");
        if let Some(path) = env_path(name) {
            return Some(path);
        }
    }

    println!("cargo:rerun-if-env-changed=SCIP_SYS_BUNDLED_DIR");
    env_path("SCIP_SYS_BUNDLED_DIR")
}

#[cfg(feature = "bundled")]
fn env_path(name: &str) -> Option<PathBuf> {
    let value = env::var_os(name)?;
    if value.is_empty() {
        return None;
    }

    let path = PathBuf::from(value);
    println!("cargo:warning=Using bundled SCIP from {}", path.display());
    Some(path)
}

#[cfg(feature = "bundled")]
fn download_scip() -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let install_path = out_dir.join("scip_install");

    if install_path.exists() {
        println!("cargo:warning=SCIP was previously downloaded, skipping download");
        return install_path;
    }

    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    println!("cargo:warning=Detected OS: {}", os);
    println!("cargo:warning=Detected arch: {}", arch);

    let os_string = target_string();
    let archive_path = out_dir.join(format!("libscip-{os_string}.zip"));
    let url = format!(
        "https://github.com/scipopt/scipoptsuite-deploy/releases/download/v{SCIP_DEPLOY_VERSION}/libscip-{os_string}.zip",
    );

    download_archive(&url, &archive_path)
        .unwrap_or_else(|err| panic!("Failed to download SCIP from {url}: {err}"));
    extract_archive(&archive_path, &install_path)
        .unwrap_or_else(|err| panic!("Failed to extract SCIP from {archive_path:?}: {err}"));

    install_path
}

#[cfg(feature = "bundled")]
fn download_archive(url: &str, archive_path: &Path) -> Result<(), Box<dyn Error>> {
    if archive_path.exists() {
        println!(
            "cargo:warning=Using cached SCIP archive {}",
            archive_path.display()
        );
        return Ok(());
    }

    println!("cargo:warning=Downloading from {url}");
    let status = Command::new("curl")
        .arg("--proto")
        .arg("=https")
        .arg("--tlsv1.2")
        .arg("-fsSL")
        .arg(url)
        .arg("-o")
        .arg(archive_path)
        .status()?;

    if !status.success() {
        return Err(format!("curl exited with status {status}").into());
    }

    Ok(())
}

#[cfg(feature = "bundled")]
fn extract_archive(archive_path: &Path, install_path: &Path) -> Result<(), Box<dyn Error>> {
    let staging = install_path.with_file_name("scip_extract");

    for python in ["python3", "python"] {
        if run_python_extract(python, &[], archive_path, install_path, &staging)? {
            return Ok(());
        }
    }
    if run_python_extract("py", &["-3"], archive_path, install_path, &staging)? {
        return Ok(());
    }

    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }

    Err("could not find python3, python, or py -3 to extract the SCIP archive".into())
}

#[cfg(feature = "bundled")]
fn run_python_extract(
    python: &str,
    leading_args: &[&str],
    archive_path: &Path,
    install_path: &Path,
    staging: &Path,
) -> Result<bool, Box<dyn Error>> {
    let mut command = Command::new(python);
    command.args(leading_args);
    command
        .arg("-c")
        .arg(PYTHON_EXTRACT)
        .arg(archive_path)
        .arg(install_path)
        .arg(staging);

    match command.status() {
        Ok(status) if status.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(Box::new(err)),
    }
}
