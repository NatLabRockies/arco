//! Zero-copy bridge from Arco model to HiGHS solver
//!
//! This crate provides efficient conversion from `arco-core::Model` to HiGHS,
//! leveraging the model's column-first (CSC) storage format to minimize copying.

pub mod async_matrix;
pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

// Re-export bridge for backward compatibility
#[deprecated(since = "0.2.0", note = "Use solver module directly")]
pub mod bridge {
    pub use crate::solution::Solution;
    pub use crate::solver::{Solver, SolverError};
}

pub use async_matrix::{AsyncCrsBuilder, CrsMatrixResult};
pub use ffi::{
    HighsModel, HighsModelError, HighsOption, HighsStatus, ObjectiveSense, SolutionSnapshot,
    highs_version,
};
pub use solution::Solution;
pub use solver::{Solver, SolverError};
