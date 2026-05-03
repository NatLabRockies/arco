use std::collections::BTreeMap;

use crate::profile::SolverProfile;
use crate::registry::{SolverRegistry, SolverTransport};

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

        // Synthesized built-in profile path: no explicit profile, derive family-default transport.
        let transport = registry
            .family(selection_token)
            .and_then(|family| {
                if family.transports.contains(&SolverTransport::Embedded) {
                    Some(SolverTransport::Embedded)
                } else {
                    family.transports.iter().next().copied()
                }
            })
            .unwrap_or(SolverTransport::Embedded);

        return Ok(ResolvedSelection {
            token: selection_token.to_string(),
            family: selection_token.to_string(),
            profile: None,
            transport,
        });
    }

    Err(SelectionError::UnknownSelection {
        token: selection_token.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::SolverConfig;
    use crate::SolverProfile;
    use crate::registry::{SolverCapabilityModel, SolverFamily, SolverRegistry, SolverTransport};

    use super::{SelectionError, resolve_selection};

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
    fn resolve_external_process_family_defaults_transport() {
        let mut registry = SolverRegistry::new();
        registry.add_family(SolverFamily::external_process(
            "example",
            "Example",
            SolverCapabilityModel::lp_mip_default(),
        ));
        let profiles = BTreeMap::new();

        let resolved = resolve_selection(&registry, &profiles, "example")
            .unwrap_or_else(|err| panic!("unexpected resolve error: {err}"));
        assert_eq!(resolved.transport, SolverTransport::ExternalProcess);
        assert!(resolved.profile.is_none());
    }
}
