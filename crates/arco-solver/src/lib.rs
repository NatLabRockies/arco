//! Shared solver abstractions for Arco optimization.
//!
//! This crate provides common traits that solver implementations
//! (like `arco-highs`) use to integrate with the Arco ecosystem.
//!
//! # Architecture
//!
//! - [`SolverConfig`]: Configuration options for solver behavior
//! - [`SolverStatus`]: Common status values across solvers (from `arco-solver-types`)
//! - [`SolverError`]: Error types for solver operations (from `arco-solver-types`)
//! - [`Solve`]: Trait for solver implementations
//! - [`SolutionView`]: Trait for accessing solution data
//! - [`SolverBackend`]: Trait for dispatching solves through a unified interface
//!
//! # Dependency Structure
//!
//! This crate depends on:
//! - `arco-solver-types`: Base solver types (Solution, SolverError, SolverStatus)
//! - `arco-core`: Model types for the SolverBackend trait
//! - `arco-expr`: Expression types

mod backend;
mod config;
mod registry;
mod traits;

pub use backend::SolverBackend;
pub use config::SolverConfig;
pub use registry::{
    PreflightError, ResolvedSelection, SelectionError, SolverCapabilityModel, SolverConfigDocument,
    SolverFamily, SolverProfile, SolverRegistry, SolverRequirements, SolverSelection,
    SolverTransport, merged_profiles, preflight_selection, resolve_selection,
};
pub use traits::{SolutionView, Solve};

// Re-export solver types from arco-solver-types for convenience
pub use arco_solver_types::{Solution, SolverError, SolverStatus};
