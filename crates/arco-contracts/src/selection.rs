use std::collections::BTreeMap;

use crate::{SolverProfile, SolverRegistry, SolverTransport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverSelection {
    Family(String),
    Profile(String),
}

impl SolverSelection {
    pub fn family(name: impl Into<String>) -> Self {
        Self::Family(name.into())
    }

    pub fn profile(name: impl Into<String>) -> Self {
        Self::Profile(name.into())
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Family(name) | Self::Profile(name) => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSelection {
    pub token: String,
    pub family: String,
    pub profile: Option<String>,
    pub transport: SolverTransport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    UnknownSelection {
        token: String,
    },
    NameCollision {
        token: String,
    },
    AmbiguousFamilySelection {
        family: String,
        profiles: Vec<String>,
    },
    UnknownProfileFamily {
        profile: String,
        family: String,
    },
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
