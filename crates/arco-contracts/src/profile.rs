use std::collections::BTreeMap;

use crate::{SolverConfig, SolverTransport};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SolverProfile {
    pub name: String,
    pub family: String,
    pub transport: SolverTransport,
    pub executable: Option<String>,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub options: SolverConfig,
}

impl SolverProfile {
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SolverConfigDocument {
    pub version: u32,
    pub default_selection: Option<String>,
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
