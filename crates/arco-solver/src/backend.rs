//! Solver backend trait for dispatching solves through a unified interface.

use crate::{Solution, SolverConfig, SolverError};
use arco_core::Model;
use arco_expr::VariableId;

/// A solver backend that can solve a model with a given configuration.
///
/// Each solver crate (e.g., `arco-highs`, `arco-ipopt`) exports a zero-sized
/// struct implementing this trait. The Python bindings dispatch through
/// `&dyn SolverBackend` instead of per-solver match arms.
pub trait SolverBackend {
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
