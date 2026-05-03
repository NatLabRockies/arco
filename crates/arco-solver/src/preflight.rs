use arco_core::Model;

use crate::registry::{SolverRegistry, SolverTransport};
use crate::selection::ResolvedSelection;

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

fn has_integer_variables(model: &Model) -> bool {
    for idx in 0..model.num_variables() {
        let variable_id = arco_expr::VariableId::new(idx as u32);
        if model
            .get_variable(variable_id)
            .map(|variable| variable.is_active && variable.is_integer)
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Validate resolved selection against model + explicit requirements.
pub fn preflight_selection(
    registry: &SolverRegistry,
    resolved: &ResolvedSelection,
    model: &Model,
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
