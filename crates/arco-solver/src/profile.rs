use std::collections::BTreeMap;

use crate::SolverConfig;
use crate::registry::SolverTransport;

/// Profile-scoped transport launch config.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SolverProfile {
    /// Globally unique profile name.
    pub name: String,
    /// Family this profile belongs to.
    pub family: String,
    /// Transport used by this profile.
    pub transport: SolverTransport,
    /// Optional executable path for external process transport.
    pub executable: Option<String>,
    /// Arguments for external-process transport.
    #[serde(default)]
    pub arguments: Vec<String>,
    /// Environment mapping where values are references (e.g., ${ENV_VAR}).
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    /// Common and family-specific options.
    #[serde(default)]
    pub options: SolverConfig,
}

impl SolverProfile {
    /// Merge an overlay profile onto a base profile.
    ///
    /// Scalars override, maps merge with overlay-wins, list fields replace.
    pub fn merged_with(&self, overlay: &Self) -> Self {
        let mut merged = self.clone();
        merged.family.clone_from(&overlay.family);
        merged.transport = overlay.transport;
        if overlay.executable.is_some() {
            merged.executable.clone_from(&overlay.executable);
        }
        if !overlay.arguments.is_empty() {
            merged.arguments.clone_from(&overlay.arguments);
        }
        for (key, value) in &overlay.environment {
            merged.environment.insert(key.clone(), value.clone());
        }
        merged.options = merged.options.merged_with(&overlay.options);
        merged
    }
}

/// Versioned TOML solver config document.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SolverConfigDocument {
    /// Schema version.
    pub version: u32,
    /// Exact selection token persisted by user (`family` or `profile`).
    pub default_selection: Option<String>,
    /// Named profile definitions.
    #[serde(default)]
    pub profiles: BTreeMap<String, SolverProfile>,
}

impl Default for SolverConfigDocument {
    fn default() -> Self {
        Self {
            version: 1,
            default_selection: None,
            profiles: BTreeMap::new(),
        }
    }
}

/// Build effective profile map by deep-merging project and user documents.
pub fn merged_profiles(
    project: &SolverConfigDocument,
    user: &SolverConfigDocument,
) -> BTreeMap<String, SolverProfile> {
    let mut merged = project.profiles.clone();
    for (name, profile) in &user.profiles {
        if let Some(base) = merged.get(name) {
            merged.insert(name.clone(), base.merged_with(profile));
        } else {
            merged.insert(name.clone(), profile.clone());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_profiles_overlays_values() {
        let mut project = SolverConfigDocument::default();
        project.profiles.insert(
            "xpress".to_string(),
            SolverProfile {
                name: "xpress".to_string(),
                family: "xpress".to_string(),
                transport: SolverTransport::ExternalProcess,
                executable: Some("/opt/xpress/bin/xprs".to_string()),
                arguments: vec!["--quiet".to_string()],
                environment: BTreeMap::from([(
                    "XPAUTH_PATH".to_string(),
                    "${XPAUTH_PATH}".to_string(),
                )]),
                options: SolverConfig::new().with_threads(4),
            },
        );

        let mut user = SolverConfigDocument::default();
        user.profiles.insert(
            "xpress".to_string(),
            SolverProfile {
                name: "xpress".to_string(),
                family: "xpress".to_string(),
                transport: SolverTransport::ExternalProcess,
                executable: Some("/home/user/xprs".to_string()),
                arguments: vec!["--nolog".to_string()],
                environment: BTreeMap::from([(
                    "XPAUTH_PATH".to_string(),
                    "${HOME}/.xpauth".to_string(),
                )]),
                options: SolverConfig::new().with_threads(8),
            },
        );

        let merged = merged_profiles(&project, &user);
        let merged_profile = merged.get("xpress");
        assert!(merged_profile.is_some());
        let merged_profile = merged_profile.unwrap_or_else(|| panic!("missing merged profile"));
        assert_eq!(
            merged_profile.executable.as_deref(),
            Some("/home/user/xprs")
        );
        assert_eq!(merged_profile.arguments, vec!["--nolog".to_string()]);
        assert_eq!(merged_profile.options.threads, Some(8));
    }
}
