//! Solver platform orchestration for Arco.
//!
//! Solver contracts (selection, config, status, traits) and platform behavior
//! live in this crate.

mod backend;
mod config;
mod conformance;
mod model_view_backend;
mod preflight;
mod profile;
mod registry;
mod request;
mod selection;
mod traits;
mod types;

pub use backend::SolverBackend as GenericSolverBackend;
pub use config::SolverConfig;
pub use conformance::{
    BackendConformanceReport, check_empty_model_rejected, check_no_objective_rejected,
    check_small_lp, check_small_milp, small_lp_model, small_milp_model,
};
pub use model_view_backend::{
    ModelViewBackend, ModelViewBackendRegistry, validate_model_view_solve_result,
};
pub use preflight::{
    PreflightError, SolverRequirements, preflight_model_view, preflight_selection,
};
pub use profile::{SolverConfigDocument, SolverProfile, merged_profiles};
pub use registry::{SolverCapabilityModel, SolverFamily, SolverRegistry, SolverTransport};
pub use request::SolveRequest;
pub use selection::{ResolvedSelection, SelectionError, SolverSelection, resolve_selection};
pub use traits::{SolutionView, Solve};
pub use types::{
    Solution, SolverDiagnostic, SolverError, SolverModelStats, SolverStatus, SolverStatusMapping,
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
    /// Reduced costs in model variable-id order when the backend reports them.
    pub variable_duals: Vec<f64>,
    /// Row activities in model constraint-id order when the backend reports them.
    pub row_values: Vec<f64>,
    /// Constraint duals in model constraint-id order when the backend reports them.
    pub constraint_duals: Vec<f64>,
    /// Backend-reported numeric metadata such as timings and matrix dimensions.
    pub metadata: std::collections::BTreeMap<String, f64>,
}

/// Platform-facing backend trait for Arco's core model type.
///
/// Solver adapters implement the generic `backend::SolverBackend` contract
/// directly; this alias trait keeps orchestration callers on the concrete Arco
/// model signature.
pub trait SolverBackend: backend::SolverBackend<arco_model::Model, arco_model::VariableId> {}

impl<T> SolverBackend for T where
    T: backend::SolverBackend<arco_model::Model, arco_model::VariableId>
{
}
