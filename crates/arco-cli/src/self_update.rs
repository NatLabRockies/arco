use std::io::IsTerminal;

use axoupdater::{AxoUpdater, AxoupdateError, UpdateRequest};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::cli_io::{should_colorize_stdout, write_stderr_line};

const APP_NAME: &str = "arco";
const RELEASE_URL_PREFIX: &str = "https://github.com/NatLabRockies/arco/releases/tag/";

pub fn update(version: Option<String>, token: Option<String>, verbose: u8) -> Result<i32> {
    tokio::runtime::Runtime::new()
        .into_diagnostic()?
        .block_on(update_async(version, token, verbose))
}

async fn update_async(version: Option<String>, token: Option<String>, verbose: u8) -> Result<i32> {
    let color = should_colorize_stdout(std::io::stderr().is_terminal());
    let mut updater = AxoUpdater::new_for(APP_NAME);

    if verbose > 0 {
        updater.enable_installer_output();
    } else {
        updater.disable_installer_output();
    }

    if let Some(ref token) = token {
        updater.set_github_token(token);
    }

    let Ok(updater) = updater.load_receipt() else {
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
        .into_diagnostic()?;
        return Ok(1);
    };

    if !updater
        .check_receipt_is_for_this_executable()
        .into_diagnostic()?
    {
        write_stderr_line(&labelled(
            "error",
            "Self-update is only available for arco binaries installed via the standalone installation scripts.",
            color,
        ))
        .into_diagnostic()?;
        write_stderr_line(&labelled(
            "hint",
            "A cargo-dist receipt exists, but it does not belong to this executable.",
            color,
        ))
        .into_diagnostic()?;
        return Ok(1);
    }

    write_stderr_line(&labelled("info", "Checking for updates...", color)).into_diagnostic()?;

    let update_request = version.map_or(UpdateRequest::Latest, UpdateRequest::SpecificTag);
    updater.configure_version_specifier(update_request);

    match updater.run().await {
        Ok(Some(result)) => {
            let version_information = result.old_version.map_or_else(
                || format!("to v{}", result.new_version),
                |old_version| format!("from v{old_version} to v{}", result.new_version),
            );
            write_stderr_line(&labelled(
                "success",
                &format!(
                    "Upgraded arco {version_information}! {RELEASE_URL_PREFIX}{}",
                    result.new_version_tag
                ),
                color,
            ))
            .into_diagnostic()?;
        }
        Ok(None) => {
            write_stderr_line(&labelled(
                "success",
                &format!(
                    "You're on the latest version of arco (v{})",
                    env!("CARGO_PKG_VERSION")
                ),
                color,
            ))
            .into_diagnostic()?;
        }
        Err(error) => {
            if let AxoupdateError::Reqwest(error) = &error {
                if error.status() == Some(http::StatusCode::FORBIDDEN) && token.is_none() {
                    write_stderr_line(&labelled(
                        "error",
                        "GitHub API rate limit exceeded. Please provide a GitHub token via --token.",
                        color,
                    ))
                    .into_diagnostic()?;
                    return Ok(1);
                }
            }
            return Err(error).into_diagnostic();
        }
    }

    Ok(0)
}

fn labelled(label: &str, message: &str, color: bool) -> String {
    if color {
        format!("{}{} {message}", label.bold().cyan(), ":".bold())
    } else {
        format!("{label}: {message}")
    }
}
