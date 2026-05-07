//! Solver platform orchestration for Arco.
//!
//! Contract types (selection, config, status, traits) live in `arco-contracts`.
//! This crate focuses on platform behavior such as preflight.

mod preflight;

pub use arco_contracts::{
    ResolvedSelection, SelectionError, Solution, SolutionView, Solve, SolveRequest,
    SolverCapabilityModel, SolverConfig, SolverConfigDocument, SolverError, SolverFamily,
    SolverProfile, SolverRegistry, SolverSelection, SolverStatus, SolverTransport, merged_profiles,
    resolve_selection,
};
pub use preflight::{
    PreflightError, SolverRequirements, preflight_model_view, preflight_selection,
};

/// Minimal result envelope for direct solves over primitive model views.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelViewSolveResult {
    /// Fingerprint of the model view used for the solve.
    pub fingerprint: arco_model::ModelFingerprint,
    /// Solver status mapped to the shared solver status contract.
    pub status: SolverStatus,
    /// Objective value reported by the solver.
    pub objective_value: f64,
    /// Primal values in model variable-id order.
    pub primal_values: Vec<f64>,
}

/// Platform-facing backend trait for Arco's core model type.
///
/// Solver adapters implement the generic `arco_contracts::SolverBackend`
/// contract directly; this alias trait keeps orchestration callers on the
/// concrete Arco model signature.
pub trait SolverBackend:
    arco_contracts::SolverBackend<arco_model::Model, arco_expr::VariableId>
{
}

impl<T> SolverBackend for T where
    T: arco_contracts::SolverBackend<arco_model::Model, arco_expr::VariableId>
{
}
