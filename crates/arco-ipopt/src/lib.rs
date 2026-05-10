//! IPOPT solver backend for Arco solver targets.
//!
//! IPOPT support is migrating to the target-based adapter seam. The current
//! crate rejects solve attempts until nonlinear target support is available.

pub mod problem;
pub mod solution;
pub mod solver;
mod status;

pub use solution::Solution;
pub use solver::{IpoptModelViewBackend, Solver, SolverError, solve_model_view};
