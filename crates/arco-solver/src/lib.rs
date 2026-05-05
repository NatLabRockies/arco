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
pub use preflight::{PreflightError, SolverRequirements, preflight_selection};

/// Platform-facing backend trait for Arco's core model type.
///
/// Solver adapters implement the generic `arco_contracts::SolverBackend`
/// contract directly; this alias trait keeps orchestration callers on the
/// concrete Arco model signature.
pub trait SolverBackend:
    arco_contracts::SolverBackend<arco_core::Model, arco_expr::VariableId>
{
}

impl<T> SolverBackend for T where
    T: arco_contracts::SolverBackend<arco_core::Model, arco_expr::VariableId>
{
}
