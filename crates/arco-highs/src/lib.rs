//! Zero-copy bridge from Arco model to HiGHS solver
//!
//! This crate provides efficient conversion from `arco-core::Model` to HiGHS,
//! leveraging the model's column-first (CSC) storage format to minimize copying.

pub mod async_matrix;
pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

pub use async_matrix::{AsyncCrsBuilder, CrsMatrixResult};
pub use ffi::{
    HighsModel, HighsModelError, HighsOption, HighsStatus, ObjectiveSense, SolutionSnapshot,
    highs_version,
};
pub use solution::Solution;
pub use solver::{HiGHSBackend, Solver, SolverError};
