//! HiGHS solver adapter for primitive Arco model views.

pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

pub use ffi::{
    HighsModel, HighsModelError, HighsOption, HighsStatus, ObjectiveSense, SolutionSnapshot,
    highs_version,
};
pub use solution::Solution;
pub use solver::{HighsModelViewBackend, SolverError, solve_model_view, solve_owned_model};
