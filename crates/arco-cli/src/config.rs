// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use clap::ValueEnum;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SolverBackend {
    #[default]
    Highs,
    Ipopt,
    Xpress,
}

impl SolverBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Highs => "highs",
            Self::Ipopt => "ipopt",
            Self::Xpress => "xpress",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolverConfig {
    pub backend: SolverBackend,
}

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    #[error("could not determine an arco config directory")]
    #[diagnostic(
        code(arco::config::missing_directory),
        help("set ARCO_CONFIG_DIR, XDG_CONFIG_HOME, HOME, or APPDATA before running the command")
    )]
    MissingConfigDirectory,
    #[error("failed to read solver config {path}: {source}")]
    #[diagnostic(
        code(arco::config::io),
        help("verify the config path exists and is readable")
    )]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write solver config {path}: {source}")]
    #[diagnostic(
        code(arco::config::io),
        help("verify the config directory is writable")
    )]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse solver config {path}: {source}")]
    #[diagnostic(
        code(arco::config::json),
        help("delete or repair the solver config file and retry")
    )]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialize solver config for {path}: {source}")]
    #[diagnostic(
        code(arco::config::json),
        help("inspect the solver configuration payload for unsupported values")
    )]
    Serialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub fn load_solver_config() -> Result<SolverConfig, ConfigError> {
    let path = solver_config_path()?;
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            serde_json::from_str(&text).map_err(|source| ConfigError::Parse { path, source })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(SolverConfig::default()),
        Err(source) => Err(ConfigError::Read { path, source }),
    }
}

pub fn save_solver_config(config: &SolverConfig) -> Result<PathBuf, ConfigError> {
    let path = solver_config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let text = serde_json::to_string_pretty(config).map_err(|source| ConfigError::Serialize {
        path: path.clone(),
        source,
    })?;
    std::fs::write(&path, text).map_err(|source| ConfigError::Write {
        path: path.clone(),
        source,
    })?;

    Ok(path)
}

pub fn solver_config_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("solver.json"))
}

fn config_dir() -> Result<PathBuf, ConfigError> {
    if let Some(path) = env::var_os("ARCO_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("arco"));
    }
    if cfg!(windows) {
        return env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("arco"))
            .ok_or(ConfigError::MissingConfigDirectory);
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|path| path.join(".config").join("arco"))
        .ok_or(ConfigError::MissingConfigDirectory)
}
