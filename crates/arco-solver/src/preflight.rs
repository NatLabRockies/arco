use crate::{ResolvedSelection, SolverRegistry, SolverTransport};
use arco_model::{ModelView, VariableId};

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

fn has_integer_variables(model: &impl ModelView) -> bool {
    for idx in 0..model.num_variables() {
        let variable_id = VariableId::new(idx as u32);
        if model
            .variable(variable_id)
            .is_some_and(|variable| variable.is_active && variable.is_integer)
        {
            return true;
        }
    }
    false
}

/// Validate resolved selection against a primitive model view + explicit requirements.
pub fn preflight_model_view(
    registry: &SolverRegistry,
    resolved: &ResolvedSelection,
    model: &impl ModelView,
    requirements: &SolverRequirements,
) -> Result<(), PreflightError> {
    if let Some(required_transport) = requirements.transport {
        if required_transport != resolved.transport {
            return Err(PreflightError::TransportMismatch {
                required: required_transport,
                actual: resolved.transport,
            });
        }
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

/// Validate resolved selection against the concrete model type.
pub fn preflight_selection(
    registry: &SolverRegistry,
    resolved: &ResolvedSelection,
    model: &arco_model::Model,
    requirements: &SolverRequirements,
) -> Result<(), PreflightError> {
    preflight_model_view(registry, resolved, model, requirements)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        SolverCapabilityModel, SolverFamily, SolverRegistry, SolverTransport, resolve_selection,
        small_lp_model, small_milp_model,
    };

    use super::{PreflightError, SolverRequirements, preflight_model_view};

    #[test]
    fn preflight_allows_continuous_model_for_continuous_only_family() {
        let registry = SolverRegistry::with_builtin_families();
        let resolved =
            resolve_selection(&registry, &BTreeMap::new(), "ipopt").expect("ipopt should resolve");
        let model = small_lp_model();

        preflight_model_view(&registry, &resolved, &model, &SolverRequirements::default())
            .expect("continuous model should pass continuous-only family preflight");
    }

    #[test]
    fn preflight_rejects_integer_model_for_continuous_only_family() {
        let registry = SolverRegistry::with_builtin_families();
        let resolved =
            resolve_selection(&registry, &BTreeMap::new(), "ipopt").expect("ipopt should resolve");
        let model = small_milp_model();

        let error =
            preflight_model_view(&registry, &resolved, &model, &SolverRequirements::default())
                .expect_err("binary model should fail continuous-only family preflight");

        assert_eq!(
            error,
            PreflightError::IntegerNotSupported {
                family: "ipopt".to_string()
            }
        );
    }

    #[test]
    fn preflight_rejects_missing_required_capabilities() {
        let mut registry = SolverRegistry::new();
        let mut capabilities = SolverCapabilityModel::lp_mip_default();
        capabilities.warm_start = false;
        capabilities.iis = false;
        registry.add_family(SolverFamily::embedded("tiny", "Tiny", capabilities));
        let resolved =
            resolve_selection(&registry, &BTreeMap::new(), "tiny").expect("tiny should resolve");
        let model = small_lp_model();

        let warm_start_error = preflight_model_view(
            &registry,
            &resolved,
            &model,
            &SolverRequirements {
                require_warm_start: true,
                ..SolverRequirements::default()
            },
        )
        .expect_err("warm-start requirement should fail");
        let iis_error = preflight_model_view(
            &registry,
            &resolved,
            &model,
            &SolverRequirements {
                require_iis: true,
                ..SolverRequirements::default()
            },
        )
        .expect_err("IIS requirement should fail");

        assert_eq!(
            warm_start_error,
            PreflightError::WarmStartNotSupported {
                family: "tiny".to_string()
            }
        );
        assert_eq!(
            iis_error,
            PreflightError::IisNotSupported {
                family: "tiny".to_string()
            }
        );
    }

    #[test]
    fn preflight_rejects_transport_mismatch() {
        let registry = SolverRegistry::with_builtin_families();
        let resolved =
            resolve_selection(&registry, &BTreeMap::new(), "highs").expect("highs should resolve");
        let model = small_lp_model();

        let error = preflight_model_view(
            &registry,
            &resolved,
            &model,
            &SolverRequirements {
                transport: Some(SolverTransport::ExternalProcess),
                ..SolverRequirements::default()
            },
        )
        .expect_err("external-process requirement should fail embedded selection");

        assert_eq!(
            error,
            PreflightError::TransportMismatch {
                required: SolverTransport::ExternalProcess,
                actual: SolverTransport::Embedded
            }
        );
    }
}
