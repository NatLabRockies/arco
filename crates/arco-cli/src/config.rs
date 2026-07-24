// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use arco_diagnostics::codes;
use arco_ops::ArcoOps;
use arco_ops::solve::{
    ResolvedSelection, SelectionError, SolverConfigDocument, SolverProfile, SolverRegistry,
    merged_profiles, resolve_selection,
};
use miette::Diagnostic;
use std::collections::BTreeMap;
use std::env;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct SolverConfigState {
    pub(crate) registry: SolverRegistry,
    pub(crate) project: SolverConfigDocument,
    pub(crate) user: SolverConfigDocument,
    pub(crate) merged_profiles: BTreeMap<String, SolverProfile>,
    pub(crate) selection: String,
    pub(crate) resolved: ResolvedSelection,
    pub(crate) user_path: PathBuf,
    pub(crate) project_path: PathBuf,
}

impl SolverConfigState {
    pub fn live_status_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("selection: {}", self.selection));
        lines.push(format!("resolved.family: {}", self.resolved.family));
        if let Some(profile) = &self.resolved.profile {
            lines.push(format!("resolved.profile: {}", profile));
        } else {
            lines.push("resolved.profile: <synthesized built-in>".to_string());
        }
        lines.push(format!("resolved.transport: {:?}", self.resolved.transport));

        let family = self.registry.family(&self.resolved.family);
        if let Some(family) = family {
            lines.push("availability.registered: true".to_string());
            lines.push("availability.compiled: true".to_string());
            let usable = selection_is_supported_in_cli(&self.resolved);
            lines.push(format!("availability.usable: {}", usable));
            lines.push(format!("family.display_name: {}", family.display_name));
        } else {
            lines.push("availability.registered: false".to_string());
            lines.push("availability.compiled: false".to_string());
            lines.push("availability.usable: false".to_string());
        }
        lines.push(format!("path.user: {}", self.user_path.display()));
        lines.push(format!("path.project: {}", self.project_path.display()));
        lines
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not determine an arco config directory")]
    MissingConfigDirectory,
    #[error("could not determine current project directory")]
    MissingProjectDirectory,
    #[error("failed to read solver config {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write solver config {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse solver config {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to serialize solver config for {path}: {source}")]
    Serialize {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
    #[error("invalid solver selection: {source}")]
    InvalidSelection {
        #[source]
        source: SelectionError,
    },
    #[error("solver profile '{profile}' contains non-reference secret value for env var '{key}'")]
    RawSecretValue { profile: String, key: String },
}

impl Diagnostic for ConfigError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::MissingConfigDirectory => Some(Box::new(codes::CONFIG_MISSING_DIRECTORY)),
            Self::MissingProjectDirectory => {
                Some(Box::new(codes::CONFIG_MISSING_PROJECT_DIRECTORY))
            }
            Self::Read { .. } | Self::Write { .. } => Some(Box::new(codes::CONFIG_IO)),
            Self::Parse { .. } | Self::Serialize { .. } => Some(Box::new(codes::CONFIG_TOML)),
            Self::InvalidSelection { .. } => Some(Box::new(codes::CONFIG_SELECTION)),
            Self::RawSecretValue { .. } => Some(Box::new(codes::CONFIG_SECRET_REFERENCE_REQUIRED)),
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::MissingConfigDirectory => Some(Box::new(
                "set ARCO_CONFIG_DIR, XDG_CONFIG_HOME, HOME, or APPDATA before running the command",
            )),
            Self::MissingProjectDirectory => Some(Box::new(
                "run from a project directory or set ARCO_PROJECT_CONFIG_DIR",
            )),
            Self::Read { .. } => Some(Box::new("verify the config path exists and is readable")),
            Self::Write { .. } => Some(Box::new("verify the config directory is writable")),
            Self::Parse { .. } => Some(Box::new(
                "delete or repair the solver TOML config file and retry",
            )),
            Self::Serialize { .. } => Some(Box::new(
                "inspect the solver configuration payload for unsupported values",
            )),
            Self::InvalidSelection { .. } => Some(Box::new(
                "choose a known family/profile name or update solver profiles in solver.toml",
            )),
            Self::RawSecretValue { .. } => Some(Box::new(
                "store only references (for example ${{ENV_VAR}} or file:/path/to/secret) in solver profile environment values",
            )),
        }
    }
}

pub fn load_solver_config() -> Result<SolverConfigState, ConfigError> {
    let user_path = solver_config_path()?;
    let project_path = project_solver_config_path()?;

    let project = read_config_document(&project_path)?;
    let user = read_config_document(&user_path)?;

    let merged_profiles = merged_profiles(&project, &user);
    validate_secret_references(&merged_profiles)?;
    let selection = user
        .default_selection
        .clone()
        .or_else(|| project.default_selection.clone())
        .unwrap_or_else(|| "highs".to_string());

    let registry = ArcoOps::solver_registry_with_builtin_families();
    let resolved = resolve_selection(&registry, &merged_profiles, &selection)
        .map_err(|source| ConfigError::InvalidSelection { source })?;

    Ok(SolverConfigState {
        registry,
        project,
        user,
        merged_profiles,
        selection,
        resolved,
        user_path,
        project_path,
    })
}

fn selection_is_supported_in_cli(resolved: &ResolvedSelection) -> bool {
    ArcoOps::builtin_adapter_for_selection(resolved, false, None).is_ok()
}

pub fn save_solver_selection(selection: &str) -> Result<PathBuf, ConfigError> {
    let path = solver_config_path()?;

    let mut document = read_config_document(&path)?;
    document.version = 1;
    document.default_selection = Some(selection.to_string());

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let text = toml::to_string_pretty(&document).map_err(|source| ConfigError::Serialize {
        path: path.clone(),
        source,
    })?;

    std::fs::write(&path, text).map_err(|source| ConfigError::Write {
        path: path.clone(),
        source,
    })?;

    Ok(path)
}

pub(crate) fn solver_config_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("solver.toml"))
}

pub(crate) fn project_solver_config_path() -> Result<PathBuf, ConfigError> {
    if let Some(path) = env::var_os("ARCO_PROJECT_CONFIG_DIR") {
        return Ok(PathBuf::from(path).join("solver.toml"));
    }
    let cwd = env::current_dir().map_err(|_| ConfigError::MissingProjectDirectory)?;
    Ok(cwd.join(".arco").join("solver.toml"))
}

fn value_looks_like_reference(value: &str) -> bool {
    value.starts_with("${") || value.starts_with("env:") || value.starts_with("file:")
}

fn validate_secret_references(
    profiles: &BTreeMap<String, SolverProfile>,
) -> Result<(), ConfigError> {
    for (name, profile) in profiles {
        for (key, value) in &profile.environment {
            if !value_looks_like_reference(value) {
                return Err(ConfigError::RawSecretValue {
                    profile: name.clone(),
                    key: key.clone(),
                });
            }
        }
    }
    Ok(())
}

fn read_config_document(path: &Path) -> Result<SolverConfigDocument, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let document: SolverConfigDocument =
                toml::from_str(&text).map_err(|source| ConfigError::Parse {
                    path: path.to_path_buf(),
                    source,
                })?;
            Ok(document)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(SolverConfigDocument::default())
        }
        Err(source) => Err(ConfigError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "arco-cli-config-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("failed to create temp dir {}: {err}", path.display()));
        path
    }

    #[test]
    fn read_config_document_defaults_when_missing() {
        let dir = temp_dir("missing");
        let path = dir.join("solver.toml");
        let document = read_config_document(&path)
            .unwrap_or_else(|err| panic!("read should succeed for missing file: {err}"));
        assert_eq!(document.version, 1);
        assert!(document.default_selection.is_none());
        assert!(document.profiles.is_empty());
    }

    #[test]
    fn read_config_document_ignores_legacy_json_sidecar() {
        let dir = temp_dir("legacy");
        let toml_path = dir.join("solver.toml");
        let legacy = dir.join("solver.json");
        std::fs::write(&legacy, "{\"backend\":\"highs\"}")
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", legacy.display()));

        let document = read_config_document(&toml_path)
            .unwrap_or_else(|err| panic!("read should ignore legacy json sidecar: {err}"));
        assert_eq!(document, SolverConfigDocument::default());
    }

    #[test]
    fn merged_profile_keeps_project_and_user_values() {
        let project = SolverConfigDocument {
            version: 1,
            default_selection: None,
            profiles: BTreeMap::from([(
                "xpress".to_string(),
                SolverProfile {
                    name: "xpress".to_string(),
                    family: "xpress".to_string(),
                    transport: arco_ops::solve::SolverTransport::ExternalProcess,
                    executable: Some("/opt/xpress/bin/xprs".to_string()),
                    arguments: vec!["--quiet".to_string()],
                    environment: BTreeMap::new(),
                    options: arco_ops::solve::SolverConfig::new().with_threads(4),
                },
            )]),
        };

        let user = SolverConfigDocument {
            version: 1,
            default_selection: Some("xpress".to_string()),
            profiles: BTreeMap::from([(
                "xpress".to_string(),
                SolverProfile {
                    name: "xpress".to_string(),
                    family: "xpress".to_string(),
                    transport: arco_ops::solve::SolverTransport::ExternalProcess,
                    executable: Some("/home/user/xprs".to_string()),
                    arguments: vec!["--nolog".to_string()],
                    environment: BTreeMap::new(),
                    options: arco_ops::solve::SolverConfig::new().with_threads(8),
                },
            )]),
        };

        let merged = merged_profiles(&project, &user);
        let profile = merged
            .get("xpress")
            .unwrap_or_else(|| panic!("missing xpress merged profile"));
        assert_eq!(profile.arguments, vec!["--nolog".to_string()]);
        assert_eq!(profile.executable.as_deref(), Some("/home/user/xprs"));
        assert_eq!(profile.options.threads, Some(8));
    }

    #[test]
    fn selection_support_check_accepts_external_process_selection() {
        let resolved = ResolvedSelection {
            token: "scip".to_string(),
            family: "scip".to_string(),
            profile: None,
            transport: arco_ops::solve::SolverTransport::Embedded,
        };

        assert!(super::selection_is_supported_in_cli(&resolved));
    }

    #[test]
    fn validate_secret_references_rejects_raw_values() {
        let profiles = BTreeMap::from([(
            "xpress".to_string(),
            SolverProfile {
                name: "xpress".to_string(),
                family: "xpress".to_string(),
                transport: arco_ops::solve::SolverTransport::ExternalProcess,
                executable: None,
                arguments: Vec::new(),
                environment: BTreeMap::from([(
                    "XPRESS_TOKEN".to_string(),
                    "plaintext-secret".to_string(),
                )]),
                options: arco_ops::solve::SolverConfig::default(),
            },
        )]);

        let result = validate_secret_references(&profiles);
        assert!(matches!(result, Err(ConfigError::RawSecretValue { .. })));
    }
}
