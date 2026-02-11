//! Shared solver abstractions for Arco optimization.
//!
//! This crate provides common types and traits that solver implementations
//! (like `arco-highs`) use to integrate with the Arco ecosystem.
//!
//! # Overview
//!
//! - [`SolverConfig`]: Configuration options for solver behavior
//! - [`SolverStatus`]: Common status values across solvers
//! - [`SolverError`]: Error types for solver operations
//! - [`Solve`]: Trait for solver implementations
//! - [`SolutionView`]: Trait for accessing solution data

mod config;
mod error;
mod status;
mod traits;

pub use config::SolverConfig;
pub use error::SolverError;
pub use status::SolverStatus;
pub use traits::{SolutionView, Solve};
