//! Shared solver backend adapter contract.

use crate::{Solution, SolverConfig, SolverError};

/// A solver backend that can solve a model with a given configuration.
///
/// Solver adapter crates export zero-sized structs implementing this trait so
/// platform orchestration can dispatch through `&dyn SolverBackend` without
/// forcing contract types to depend on a concrete model representation.
pub trait SolverBackend<Model, VariableId> {
    /// Solve the model and return a solver-agnostic solution.
    fn solve(
        &self,
        model: &Model,
        config: &SolverConfig,
        primal_start: Option<&[(VariableId, f64)]>,
    ) -> Result<Solution, SolverError>;

    /// Human-readable solver name (e.g., `"HiGHS"`, `"IPOPT"`).
    fn name(&self) -> &'static str;

    /// Whether this backend supports integer/binary variables.
    fn supports_integer(&self) -> bool {
        true
    }
}
