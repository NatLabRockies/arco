//! HiGHS solver adapter for primitive Arco model views.

pub mod ffi;
pub mod solution;
pub mod solver;
mod status;
mod sys;

pub use ffi::{
    HighsModel, HighsModelError, HighsOption, HighsStatus, ObjectiveSense, SolutionSnapshot,
    highs_version,
};
pub use solution::Solution;
pub use solver::{HighsModelViewBackend, PreparedHighsModel, SolverError, solve_model_view};
