//! FICO Xpress solver backend for Arco optimization.
//!
//! This crate provides a bridge from `arco-core::Model` to the FICO Xpress
//! solver. It supports LP, MIP, and QP problems.
//!
//! Xpress is a commercial solver and requires a valid license.

pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

pub use solution::Solution;
pub use solver::{Solver, XpressBackend};
