//! IPOPT target adapter placeholder.
//!
//! IPOPT is being migrated from canonical-model input to solver-target input.
//! Nonlinear target support will live here once `arco-targets` exposes the
//! required nonlinear structures.

use arco_solver::SolverError;
use arco_targets::{AlgebraicProblem, VariableKind};

pub struct ArcoProblem;

impl ArcoProblem {
    pub fn validate_supported_target(problem: &AlgebraicProblem) -> Result<(), SolverError> {
        if problem.variable_instances.is_empty() {
            return Err(SolverError::EmptyModel);
        }
        if problem
            .variable_instances
            .iter()
            .any(|variable| matches!(variable.kind, VariableKind::Integer | VariableKind::Binary))
        {
            return Err(SolverError::SolverSpecific(
                "IPOPT does not support integer variables".to_string(),
            ));
        }
        Err(SolverError::SolverNotAvailable(
            "IPOPT target adapter is not implemented yet".to_string(),
        ))
    }
}
