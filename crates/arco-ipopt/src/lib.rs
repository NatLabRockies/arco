//! IPOPT solver backend for Arco optimization.
//!
//! This crate provides a bridge from `arco-core::Model` to the IPOPT nonlinear
//! interior-point solver. It currently supports LP problems but is architecturally
//! designed for future NLP extension.
//!
//! IPOPT is a continuous-only solver and will reject models with integer or
//! binary variables.

pub mod problem;
pub mod solution;
pub mod solver;
mod status;

pub use solution::Solution;
pub use solver::{IpoptBackend, Solver, SolverError};
