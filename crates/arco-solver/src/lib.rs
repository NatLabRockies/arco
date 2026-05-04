//! Solver platform orchestration for Arco.
//!
//! Contract types (selection, config, status, traits) live in `arco-contracts`.
//! This crate focuses on platform behavior such as preflight.

mod backend;
mod preflight;

pub use arco_contracts::{
    ResolvedSelection, SelectionError, Solution, SolutionView, Solve, SolveRequest,
    SolverCapabilityModel, SolverConfig, SolverConfigDocument, SolverError, SolverFamily,
    SolverProfile, SolverRegistry, SolverSelection, SolverStatus, SolverTransport, merged_profiles,
    resolve_selection,
};
pub use backend::SolverBackend;
pub use preflight::{PreflightError, SolverRequirements, preflight_selection};
