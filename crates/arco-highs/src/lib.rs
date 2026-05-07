//! HiGHS solver adapter for Arco solver targets.

pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

pub use ffi::{
    HighsModel, HighsModelError, HighsOption, HighsStatus, ObjectiveSense, SolutionSnapshot,
    highs_version,
};
pub use solution::Solution;
pub use solver::{Solver, SolverError, solve_model_view};
