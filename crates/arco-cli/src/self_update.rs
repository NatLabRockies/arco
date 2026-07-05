use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Deserialize;

use crate::cli_io::{should_colorize_stdout, write_stderr_line};

const APP_NAME: &str = "arco";
const INSTALLER_BASENAME: &str = "arco-cli-installer";
const RELEASE_DOWNLOAD_PREFIX: &str = "https://github.com/NatLabRockies/arco/releases";

#[derive(Debug, Deserialize)]
struct InstallReceipt {
    install_prefix: PathBuf,
    #[serde(default)]
    provider: Option<ReceiptProvider>,
    #[serde(default = "default_modify_path")]
    modify_path: bool,
}

#[derive(Debug, Deserialize)]
struct ReceiptProvider {
    source: String,
    version: String,
}

pub fn update(version: Option<String>, _token: Option<String>, verbose: u8) -> Result<i32> {
    let color = should_colorize_stdout(std::io::stderr().is_terminal());
    let Some(receipt) = load_receipt()? else {
        write_standalone_requirement(color)?;
        return Ok(1);
    };

    if !receipt_matches_current_executable(&receipt)? {
        write_standalone_requirement(color)?;
        write_stderr_line(&labelled(
            "hint",
            "A cargo-dist receipt exists, but it does not belong to this executable.",
            color,
        ))
        .into_diagnostic()?;
        return Ok(1);
    }

    let installer_url = installer_url(version.as_deref())?;
    write_stderr_line(&labelled(
        "info",
        "Running standalone installer update...",
        color,
    ))
    .into_diagnostic()?;

    let install_root = install_prefix_root(&receipt);
    let code = run_installer(&installer_url, &install_root, receipt.modify_path, verbose)?;
    if code == 0 {
        write_stderr_line(&labelled("success", "Installer completed.", color)).into_diagnostic()?;
    } else {
        write_stderr_line(&labelled(
            "error",
            &format!("Installer exited with status {code}."),
            color,
        ))
        .into_diagnostic()?;
    }

    Ok(code)
}

fn default_modify_path() -> bool {
    true
}

fn write_standalone_requirement(color: bool) -> Result<()> {
    write_stderr_line(&labelled(
        "error",
        "Self-update is only available for arco binaries installed via the standalone installation scripts.",
        color,
    ))
    .into_diagnostic()?;
    write_stderr_line(&labelled(
        "hint",
        "If you installed arco with a package manager, update arco with that package manager instead.",
        color,
    ))
    .into_diagnostic()
}

fn load_receipt() -> Result<Option<InstallReceipt>> {
    for config_path in config_paths()? {
        let receipt_path = config_path.join(format!("{APP_NAME}-receipt.json"));
        if !receipt_path.exists() {
            continue;
        }
        let Ok(receipt_bytes) = fs::read(&receipt_path) else {
            return Ok(None);
        };
        let Ok(receipt) = serde_json::from_slice::<InstallReceipt>(&receipt_bytes) else {
            return Ok(None);
        };
        return Ok(Some(receipt));
    }

    Ok(None)
}

fn config_paths() -> Result<Vec<PathBuf>> {
    if env::var_os("AXOUPDATER_CONFIG_WORKING_DIR").is_some() {
        return Ok(vec![env::current_dir().into_diagnostic()?]);
    }
    if let Some(path) = env::var_os("AXOUPDATER_CONFIG_PATH") {
        return Ok(vec![PathBuf::from(path)]);
    }

    let mut paths = Vec::new();
    if let Some(xdg_home) = env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_home).join(APP_NAME);
        if path.exists() {
            paths.push(path);
        }
    }

    #[cfg(windows)]
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        paths.push(PathBuf::from(local_app_data).join(APP_NAME));
    }

    #[cfg(not(windows))]
    if let Some(home) = env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".config").join(APP_NAME));
    }

    Ok(paths)
}

fn receipt_matches_current_executable(receipt: &InstallReceipt) -> Result<bool> {
    let current_exe = canonicalize_or_original(env::current_exe().into_diagnostic()?);
    let mut current_root = current_exe
        .parent()
        .map_or(current_exe.clone(), Path::to_path_buf);
    let receipt_root = canonicalize_or_original(install_prefix_root(receipt));

    if path_file_name_is(&current_root, "bin") && !path_file_name_is(&receipt_root, "bin") {
        if let Some(parent) = current_root.parent() {
            current_root = parent.to_path_buf();
        }
    }

    Ok(current_root == receipt_root)
}

fn install_prefix_root(receipt: &InstallReceipt) -> PathBuf {
    if receipt
        .provider
        .as_ref()
        .is_some_and(provider_needs_bin_stripped)
    {
        return root_without_bin(&receipt.install_prefix);
    }

    receipt.install_prefix.clone()
}

fn provider_needs_bin_stripped(provider: &ReceiptProvider) -> bool {
    provider.source == "cargo-dist"
        && matches!(
            provider.version.trim_start_matches('v'),
            version if version.starts_with("0.10.")
                || version.starts_with("0.11.")
                || version.starts_with("0.12.")
                || version.starts_with("0.13.")
                || version.starts_with("0.14.")
                || version.starts_with("0.15.0-prerelease")
        )
}

fn root_without_bin(path: &Path) -> PathBuf {
    if path_file_name_is(path, "bin") {
        if let Some(parent) = path.parent() {
            return parent.to_path_buf();
        }
    }

    path.to_path_buf()
}

fn path_file_name_is(path: &Path, expected: &str) -> bool {
    path.file_name()
        .is_some_and(|value| value == OsStr::new(expected))
}

fn canonicalize_or_original(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}

fn installer_url(version: Option<&str>) -> Result<String> {
    let extension = if cfg!(windows) { "ps1" } else { "sh" };
    let installer_name = format!("{INSTALLER_BASENAME}.{extension}");
    match version {
        Some(tag) => {
            if !is_safe_release_tag(tag) {
                return Err(miette::miette!(
                    "release tag may only contain ASCII letters, numbers, '.', '_', '-', or '+'"
                ));
            }
            Ok(format!(
                "{RELEASE_DOWNLOAD_PREFIX}/download/{tag}/{installer_name}"
            ))
        }
        None => Ok(format!(
            "{RELEASE_DOWNLOAD_PREFIX}/latest/download/{installer_name}"
        )),
    }
}

fn is_safe_release_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
}

fn run_installer(url: &str, install_root: &Path, modify_path: bool, verbose: u8) -> Result<i32> {
    let installer = TempInstaller::new();
    download_installer(url, &installer.path)?;
    run_installer_file(&installer.path, install_root, modify_path, verbose)
}

struct TempInstaller {
    path: PathBuf,
}

impl TempInstaller {
    fn new() -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        let extension = if cfg!(windows) { "ps1" } else { "sh" };
        let path = env::temp_dir().join(format!(
            "{INSTALLER_BASENAME}-{}-{suffix}.{extension}",
            std::process::id()
        ));
        Self { path }
    }
}

impl Drop for TempInstaller {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(not(windows))]
fn download_installer(url: &str, path: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["--proto", "=https", "--tlsv1.2", "-LsSf", "-o"])
        .arg(path)
        .arg(url)
        .status()
        .into_diagnostic()?;
    if status.success() {
        return Ok(());
    }

    Err(miette::miette!("failed to download installer from {url}"))
}

#[cfg(windows)]
fn download_installer(url: &str, path: &Path) -> Result<()> {
    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "ByPass", "-Command"])
        .arg("Invoke-WebRequest")
        .arg("-Uri")
        .arg(url)
        .arg("-OutFile")
        .arg(path)
        .status()
        .into_diagnostic()?;
    if status.success() {
        return Ok(());
    }

    Err(miette::miette!("failed to download installer from {url}"))
}

#[cfg(not(windows))]
fn run_installer_file(
    path: &Path,
    install_root: &Path,
    modify_path: bool,
    verbose: u8,
) -> Result<i32> {
    let mut command = Command::new("sh");
    command.arg(path);
    run_installer_command(command, install_root, modify_path, verbose)
}

#[cfg(windows)]
fn run_installer_file(
    path: &Path,
    install_root: &Path,
    modify_path: bool,
    verbose: u8,
) -> Result<i32> {
    let mut command = Command::new("powershell");
    command
        .args(["-NoProfile", "-ExecutionPolicy", "ByPass", "-File"])
        .arg(path);
    run_installer_command(command, install_root, modify_path, verbose)
}

fn run_installer_command(
    mut command: Command,
    install_root: &Path,
    modify_path: bool,
    verbose: u8,
) -> Result<i32> {
    command.env("CARGO_DIST_FORCE_INSTALL_DIR", install_root);
    command.env("ARCO_INSTALL_DIR", install_root);
    if !modify_path {
        command.env("ARCO_NO_MODIFY_PATH", "1");
    }
    if verbose == 0 {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }

    let status = command.status().into_diagnostic()?;
    Ok(if status.success() {
        0
    } else {
        status.code().unwrap_or(1)
    })
}

fn labelled(label: &str, message: &str, color: bool) -> String {
    if color {
        format!("{}{} {message}", label.bold().cyan(), ":".bold())
    } else {
        format!("{label}: {message}")
    }
}

#[cfg(test)]
mod tests {
    use crate::self_update::installer_url;

    #[test]
    fn installer_url_uses_latest_download_for_default_update() {
        let url = installer_url(None).expect("latest URL should be valid");

        assert!(url.contains("/releases/latest/download/arco-cli-installer."));
    }

    #[test]
    fn installer_url_uses_tagged_download_for_version_update() {
        let url = installer_url(Some("v0.7.0")).expect("tagged URL should be valid");

        assert!(url.contains("/releases/download/v0.7.0/arco-cli-installer."));
    }

    #[test]
    fn installer_url_rejects_path_like_release_tags() {
        let error = installer_url(Some("../v0.7.0")).expect_err("path-like tags must be rejected");

        assert!(
            error
                .to_string()
                .contains("release tag may only contain ASCII")
        );
    }
}
