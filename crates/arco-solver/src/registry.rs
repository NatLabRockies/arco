//! Solver registry, selection, profile, and preflight types.

use std::collections::{BTreeMap, BTreeSet};

use arco_core::Model;

use crate::SolverConfig;

/// Supported solver transports in v1.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SolverTransport {
    /// In-process linked backend.
    Embedded,
    /// External process invocation backend.
    ExternalProcess,
}

/// Capability support model for a solver family.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SolverCapabilityModel {
    /// Whether integer/binary variables are supported.
    pub supports_integer: bool,
    /// Whether warm starts are supported.
    pub warm_start: bool,
    /// Whether quadratic objective terms are supported.
    pub quadratic_objective: bool,
    /// Whether quadratic constraints are supported.
    pub quadratic_constraints: bool,
    /// Whether multi-objective solve is supported.
    pub multi_objective: bool,
    /// Whether IIS extraction is supported.
    pub iis: bool,
}

impl SolverCapabilityModel {
    /// Conservative defaults for LP/MIP-only families.
    pub fn lp_mip_default() -> Self {
        Self {
            supports_integer: true,
            warm_start: true,
            quadratic_objective: false,
            quadratic_constraints: false,
            multi_objective: false,
            iis: false,
        }
    }

    /// Conservative defaults for continuous-only NLP families.
    pub fn continuous_default() -> Self {
        Self {
            supports_integer: false,
            warm_start: true,
            quadratic_objective: true,
            quadratic_constraints: true,
            multi_objective: false,
            iis: false,
        }
    }
}

/// Family-level metadata in the solver registry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SolverFamily {
    /// Canonical family name (unique).
    pub name: String,
    /// Human-readable label.
    pub display_name: String,
    /// Supported transports.
    pub transports: BTreeSet<SolverTransport>,
    /// Capability model.
    pub capabilities: SolverCapabilityModel,
}

impl SolverFamily {
    /// Build an embedded family descriptor.
    pub fn embedded(
        name: impl Into<String>,
        display_name: impl Into<String>,
        capabilities: SolverCapabilityModel,
    ) -> Self {
        let mut transports = BTreeSet::new();
        transports.insert(SolverTransport::Embedded);
        Self {
            name: name.into(),
            display_name: display_name.into(),
            transports,
            capabilities,
        }
    }
}

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

/// Static registry model for known families.
#[derive(Debug, Clone, Default)]
pub struct SolverRegistry {
    families: BTreeMap<String, SolverFamily>,
}

impl SolverRegistry {
    /// Build an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the in-tree known family registry.
    pub fn with_builtin_families() -> Self {
        let mut registry = Self::new();
        registry.add_family(SolverFamily::embedded(
            "highs",
            "HiGHS",
            SolverCapabilityModel::lp_mip_default(),
        ));
        registry.add_family(SolverFamily::embedded(
            "xpress",
            "Xpress",
            SolverCapabilityModel::lp_mip_default(),
        ));
        registry.add_family(SolverFamily::embedded(
            "ipopt",
            "Ipopt",
            SolverCapabilityModel::continuous_default(),
        ));
        registry
    }

    /// Register a family.
    pub fn add_family(&mut self, family: SolverFamily) {
        self.families.insert(family.name.clone(), family);
    }

    /// Lookup family by name.
    pub fn family(&self, name: &str) -> Option<&SolverFamily> {
        self.families.get(name)
    }

    /// Iterate families.
    pub fn families(&self) -> impl Iterator<Item = &SolverFamily> {
        self.families.values()
    }
}

/// Selection kind, parsed from user token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverSelection {
    /// Family name selection.
    Family(String),
    /// Profile name selection.
    Profile(String),
}

/// Resolved selection result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSelection {
    /// Original token.
    pub token: String,
    /// Resolved family.
    pub family: String,
    /// Optional resolved profile name.
    pub profile: Option<String>,
    /// Effective transport.
    pub transport: SolverTransport,
}

/// Selection resolution errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    /// Selection does not match family or profile.
    UnknownSelection { token: String },
    /// Family name collides with profile name.
    NameCollision { token: String },
    /// Family selection has multiple profiles and no explicit default profile configured.
    AmbiguousFamilySelection {
        family: String,
        profiles: Vec<String>,
    },
    /// A profile references a family that is not registered.
    UnknownProfileFamily { profile: String, family: String },
}

impl std::fmt::Display for SelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionError::UnknownSelection { token } => {
                write!(f, "unknown solver selection '{token}'")
            }
            SelectionError::NameCollision { token } => write!(
                f,
                "solver selection '{token}' is ambiguous because a family and profile share that name",
            ),
            SelectionError::AmbiguousFamilySelection { family, profiles } => write!(
                f,
                "solver family '{family}' has multiple profiles ({}) and no explicit profile was selected",
                profiles.join(", ")
            ),
            SelectionError::UnknownProfileFamily { profile, family } => write!(
                f,
                "solver profile '{profile}' references unknown family '{family}'",
            ),
        }
    }
}

impl std::error::Error for SelectionError {}

/// Preflight requirement constraints.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SolverRequirements {
    /// Optional transport requirement.
    pub transport: Option<SolverTransport>,
    /// Require warm-start support.
    pub require_warm_start: bool,
    /// Require IIS support.
    pub require_iis: bool,
}

/// Preflight validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightError {
    /// Integer model sent to continuous-only family.
    IntegerNotSupported { family: String },
    /// Required transport differs from resolved transport.
    TransportMismatch {
        required: SolverTransport,
        actual: SolverTransport,
    },
    /// Warm start required but unsupported.
    WarmStartNotSupported { family: String },
    /// IIS required but unsupported.
    IisNotSupported { family: String },
}

impl std::fmt::Display for PreflightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreflightError::IntegerNotSupported { family } => write!(
                f,
                "solver family '{family}' does not support integer/binary variables"
            ),
            PreflightError::TransportMismatch { required, actual } => write!(
                f,
                "solver transport mismatch: required '{required:?}', resolved '{actual:?}'"
            ),
            PreflightError::WarmStartNotSupported { family } => {
                write!(f, "solver family '{family}' does not support warm starts")
            }
            PreflightError::IisNotSupported { family } => {
                write!(
                    f,
                    "solver family '{family}' does not support IIS extraction"
                )
            }
        }
    }
}

impl std::error::Error for PreflightError {}

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

fn has_integer_variables(model: &Model) -> bool {
    for idx in 0..model.num_variables() {
        let variable_id = arco_expr::VariableId::new(idx as u32);
        if let Ok(variable) = model.get_variable(variable_id)
            && variable.is_active
            && variable.is_integer
        {
            return true;
        }
    }
    false
}

/// Resolve selection token to family/profile/transport.
///
/// The selection token is interpreted as profile if it exists uniquely as profile,
/// family if it exists as family, and errors on collisions.
pub fn resolve_selection(
    registry: &SolverRegistry,
    profiles: &BTreeMap<String, SolverProfile>,
    selection_token: &str,
) -> Result<ResolvedSelection, SelectionError> {
    let family_exists = registry.family(selection_token).is_some();

    if let Some(profile) = profiles.get(selection_token) {
        if family_exists {
            return Err(SelectionError::NameCollision {
                token: selection_token.to_string(),
            });
        }
        if registry.family(&profile.family).is_none() {
            return Err(SelectionError::UnknownProfileFamily {
                profile: profile.name.clone(),
                family: profile.family.clone(),
            });
        }
        return Ok(ResolvedSelection {
            token: selection_token.to_string(),
            family: profile.family.clone(),
            profile: Some(profile.name.clone()),
            transport: profile.transport,
        });
    }

    if family_exists {
        let mut family_profiles: Vec<&SolverProfile> = profiles
            .values()
            .filter(|profile| profile.family == selection_token)
            .collect();

        if family_profiles.len() == 1 {
            if let Some(profile) = family_profiles.pop() {
                return Ok(ResolvedSelection {
                    token: selection_token.to_string(),
                    family: selection_token.to_string(),
                    profile: Some(profile.name.clone()),
                    transport: profile.transport,
                });
            }
        } else if family_profiles.len() > 1 {
            let mut names: Vec<String> = family_profiles
                .iter()
                .map(|profile| profile.name.clone())
                .collect();
            names.sort_unstable();
            return Err(SelectionError::AmbiguousFamilySelection {
                family: selection_token.to_string(),
                profiles: names,
            });
        }

        // Synthesized built-in profile path: no explicit profile, derive embedded transport.
        return Ok(ResolvedSelection {
            token: selection_token.to_string(),
            family: selection_token.to_string(),
            profile: None,
            transport: SolverTransport::Embedded,
        });
    }

    Err(SelectionError::UnknownSelection {
        token: selection_token.to_string(),
    })
}

/// Validate resolved selection against model + explicit requirements.
pub fn preflight_selection(
    registry: &SolverRegistry,
    resolved: &ResolvedSelection,
    model: &Model,
    requirements: &SolverRequirements,
) -> Result<(), PreflightError> {
    if let Some(required_transport) = requirements.transport
        && required_transport != resolved.transport
    {
        return Err(PreflightError::TransportMismatch {
            required: required_transport,
            actual: resolved.transport,
        });
    }

    let capabilities = registry
        .family(&resolved.family)
        .map(|family| &family.capabilities);

    if let Some(caps) = capabilities {
        if has_integer_variables(model) && !caps.supports_integer {
            return Err(PreflightError::IntegerNotSupported {
                family: resolved.family.clone(),
            });
        }
        if requirements.require_warm_start && !caps.warm_start {
            return Err(PreflightError::WarmStartNotSupported {
                family: resolved.family.clone(),
            });
        }
        if requirements.require_iis && !caps.iis {
            return Err(PreflightError::IisNotSupported {
                family: resolved.family.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_profile_selection() {
        let registry = SolverRegistry::with_builtin_families();
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "xpress-local".to_string(),
            SolverProfile {
                name: "xpress-local".to_string(),
                family: "xpress".to_string(),
                transport: SolverTransport::Embedded,
                executable: None,
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                options: SolverConfig::default(),
            },
        );

        let resolved = resolve_selection(&registry, &profiles, "xpress-local")
            .unwrap_or_else(|err| panic!("unexpected resolve error: {err}"));
        assert_eq!(resolved.family, "xpress");
        assert_eq!(resolved.profile.as_deref(), Some("xpress-local"));
    }

    #[test]
    fn resolve_family_with_single_profile_autoresolves() {
        let registry = SolverRegistry::with_builtin_families();
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "highs-fast".to_string(),
            SolverProfile {
                name: "highs-fast".to_string(),
                family: "highs".to_string(),
                transport: SolverTransport::Embedded,
                executable: None,
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                options: SolverConfig::default(),
            },
        );

        let resolved = resolve_selection(&registry, &profiles, "highs")
            .unwrap_or_else(|err| panic!("unexpected resolve error: {err}"));
        assert_eq!(resolved.profile.as_deref(), Some("highs-fast"));
    }

    #[test]
    fn resolve_family_with_many_profiles_errors() {
        let registry = SolverRegistry::with_builtin_families();
        let mut profiles = BTreeMap::new();
        for profile_name in ["highs-a", "highs-b"] {
            profiles.insert(
                profile_name.to_string(),
                SolverProfile {
                    name: profile_name.to_string(),
                    family: "highs".to_string(),
                    transport: SolverTransport::Embedded,
                    executable: None,
                    arguments: Vec::new(),
                    environment: BTreeMap::new(),
                    options: SolverConfig::default(),
                },
            );
        }

        let result = resolve_selection(&registry, &profiles, "highs");
        assert!(matches!(
            result,
            Err(SelectionError::AmbiguousFamilySelection { .. })
        ));
    }

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
