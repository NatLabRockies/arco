//! FICO Xpress solver backend for Arco solver targets.
//!
//! Xpress is a commercial solver and requires a valid license. The adapter is
//! migrating to the target-based solver seam.

pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

pub use solution::Solution;
pub use solver::{
    Solver, XpressModelViewBackend, detect_xpress_dir, solve_model_view, xpress_runtime_available,
};
