//! Shared solver types and status enums for Arco optimization.
//!
//! This crate provides solver-agnostic types that are used by both the model
//! core and solver implementations. Extracting these types into a separate
//! crate breaks the diamond dependency pattern and enables proper layering.
//!
//! # Design Rationale
//!
//! Previously, these types were in `arco-core`, which created a diamond
//! dependency: solver backends needed both `arco-core` (for Model) and
//! `arco-solver` (for traits), but `arco-solver` also depended on `arco-core`.
//!
//! By extracting types into this crate, we achieve:
//! - `arco-solver-types`: Base types (no internal deps)
//! - `arco-core`: Model implementation, depends on `arco-solver-types`
//! - `arco-solver`: Traits, depends on `arco-solver-types` and `arco-core`
//! - Solver backends: Depend on `arco-solver-types`, `arco-core`, `arco-solver`

use std::collections::BTreeMap;

/// Status of a solver solution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SolverStatus {
    /// Optimal solution found.
    Optimal,
    /// Problem is infeasible.
    Infeasible,
    /// Problem is unbounded.
    Unbounded,
    /// Solver reached time limit (may have feasible solution).
    TimeLimit,
    /// Solver reached iteration limit (may have feasible solution).
    IterationLimit,
    /// Status is unknown or solver did not complete.
    Unknown,
}

impl SolverStatus {
    /// Check if the status indicates an optimal solution.
    pub const fn is_optimal(self) -> bool {
        matches!(self, SolverStatus::Optimal)
    }

    /// Check if the status indicates a feasible solution.
    pub const fn is_feasible(self) -> bool {
        matches!(
            self,
            SolverStatus::Optimal | SolverStatus::TimeLimit | SolverStatus::IterationLimit
        )
    }

    /// Check if the status indicates infeasibility.
    pub const fn is_infeasible(self) -> bool {
        matches!(self, SolverStatus::Infeasible)
    }

    /// Check if the status indicates unboundedness.
    pub const fn is_unbounded(self) -> bool {
        matches!(self, SolverStatus::Unbounded)
    }

    /// Get a human-readable string representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            SolverStatus::Optimal => "optimal",
            SolverStatus::Infeasible => "infeasible",
            SolverStatus::Unbounded => "unbounded",
            SolverStatus::TimeLimit => "time_limit",
            SolverStatus::IterationLimit => "iteration_limit",
            SolverStatus::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for SolverStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error type for solver operations.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SolverError {
    /// Model has no variables.
    EmptyModel,
    /// No objective function set.
    NoObjective,
    /// Invalid objective sense.
    InvalidObjectiveSense,
    /// Invalid variable ID.
    InvalidVariableId(u32),
    /// Solver is not available (e.g., library not installed).
    SolverNotAvailable(String),
    /// Solver failed to find optimal solution.
    SolveFailure {
        /// The solver status that caused the failure.
        status: SolverStatus,
    },
    /// Solver-specific error not covered by other variants.
    SolverSpecific(String),
}

impl SolverError {
    /// Returns a semantic error code for programmatic handling.
    pub const fn code(&self) -> &'static str {
        match self {
            SolverError::EmptyModel => "SOLVER_EMPTY_MODEL",
            SolverError::NoObjective => "SOLVER_NO_OBJECTIVE",
            SolverError::InvalidObjectiveSense => "SOLVER_INVALID_OBJECTIVE_SENSE",
            SolverError::InvalidVariableId(_) => "SOLVER_INVALID_VARIABLE_ID",
            SolverError::SolverNotAvailable(_) => "SOLVER_NOT_AVAILABLE",
            SolverError::SolveFailure { .. } => "SOLVER_SOLVE_FAILURE",
            SolverError::SolverSpecific(_) => "SOLVER_SPECIFIC",
        }
    }
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverError::EmptyModel => write!(f, "[{}] Model has no variables", self.code()),
            SolverError::NoObjective => write!(f, "[{}] Model has no objective", self.code()),
            SolverError::InvalidObjectiveSense => {
                write!(f, "[{}] Invalid objective sense", self.code())
            }
            SolverError::InvalidVariableId(id) => {
                write!(f, "[{}] Variable ID {} does not exist", self.code(), id)
            }
            SolverError::SolverNotAvailable(msg) => {
                write!(f, "[{}] Solver not available: {}", self.code(), msg)
            }
            SolverError::SolveFailure { status } => {
                write!(f, "[{}] Solve failed with status: {}", self.code(), status)
            }
            SolverError::SolverSpecific(msg) => {
                write!(f, "[{}] Solver error: {}", self.code(), msg)
            }
        }
    }
}

impl std::error::Error for SolverError {}

/// Solver-agnostic solution from an optimization solve.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Solution {
    /// Primal values of variables indexed by their internal position.
    pub primal_values: Vec<f64>,
    /// Dual values of variables (reduced costs) indexed by their internal position.
    pub variable_duals: Vec<f64>,
    /// Dual values of constraints (shadow prices) indexed by their internal position.
    pub constraint_duals: Vec<f64>,
    /// Row activity values (constraint LHS evaluated at the solution) indexed by constraint position.
    pub row_values: Vec<f64>,
    /// Objective value of the solution.
    pub objective_value: f64,
    /// Status of the solution.
    pub status: SolverStatus,
    /// Solve time in seconds.
    pub solve_time_seconds: f64,
    /// Solver-agnostic metadata (e.g., iteration counts, gaps).
    pub metadata: BTreeMap<String, f64>,
}

impl Solution {
    /// Get the primal value at the given index.
    pub fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    /// Get the variable dual (reduced cost) at the given index.
    pub fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    /// Get the constraint dual (shadow price) at the given index.
    pub fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    /// Get the row activity value (constraint LHS at solution) at the given index.
    pub fn get_row_value(&self, index: usize) -> Option<f64> {
        self.row_values.get(index).copied()
    }

    /// Check if the solution is optimal.
    pub fn is_optimal(&self) -> bool {
        self.status.is_optimal()
    }

    /// Check if the solution is feasible.
    pub fn is_feasible(&self) -> bool {
        self.status.is_feasible()
    }

    /// Check if the solution is infeasible.
    pub fn is_infeasible(&self) -> bool {
        self.status.is_infeasible()
    }

    /// Check if the solution is unbounded.
    pub fn is_unbounded(&self) -> bool {
        self.status.is_unbounded()
    }

    /// Get a human-readable status string.
    pub fn status_string(&self) -> &'static str {
        self.status.as_str()
    }
}

/// Configuration options for solver behavior.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolverConfig {
    /// Time limit in seconds (None = no limit).
    pub time_limit_seconds: Option<f64>,
    /// Iteration limit (None = no limit).
    pub iteration_limit: Option<u32>,
    /// Enable presolve.
    pub presolve: bool,
    /// Verbosity level (0 = silent, higher = more output).
    pub verbosity: u8,
    /// Solver-specific parameters as key-value pairs.
    pub parameters: BTreeMap<String, String>,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            time_limit_seconds: None,
            iteration_limit: None,
            presolve: true,
            verbosity: 0,
            parameters: BTreeMap::new(),
        }
    }
}

impl SolverConfig {
    /// Create a new solver config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the time limit in seconds.
    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        self.time_limit_seconds = Some(seconds);
        self
    }

    /// Set the iteration limit.
    pub fn with_iteration_limit(mut self, limit: u32) -> Self {
        self.iteration_limit = Some(limit);
        self
    }

    /// Set whether to use presolve.
    pub fn with_presolve(mut self, enable: bool) -> Self {
        self.presolve = enable;
        self
    }

    /// Set the verbosity level.
    pub fn with_verbosity(mut self, level: u8) -> Self {
        self.verbosity = level;
        self
    }

    /// Add a solver-specific parameter.
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn solver_status_is_optimal() {
        assert!(SolverStatus::Optimal.is_optimal());
        assert!(!SolverStatus::Infeasible.is_optimal());
        assert!(!SolverStatus::Unbounded.is_optimal());
        assert!(!SolverStatus::TimeLimit.is_optimal());
        assert!(!SolverStatus::IterationLimit.is_optimal());
        assert!(!SolverStatus::Unknown.is_optimal());
    }

    #[test]
    fn solver_status_is_feasible() {
        assert!(SolverStatus::Optimal.is_feasible());
        assert!(SolverStatus::TimeLimit.is_feasible());
        assert!(SolverStatus::IterationLimit.is_feasible());
        assert!(!SolverStatus::Infeasible.is_feasible());
        assert!(!SolverStatus::Unbounded.is_feasible());
        assert!(!SolverStatus::Unknown.is_feasible());
    }

    #[test]
    fn solver_status_as_str() {
        assert_eq!(SolverStatus::Optimal.as_str(), "optimal");
        assert_eq!(SolverStatus::Infeasible.as_str(), "infeasible");
        assert_eq!(SolverStatus::Unbounded.as_str(), "unbounded");
        assert_eq!(SolverStatus::TimeLimit.as_str(), "time_limit");
        assert_eq!(SolverStatus::IterationLimit.as_str(), "iteration_limit");
        assert_eq!(SolverStatus::Unknown.as_str(), "unknown");
    }

    #[test]
    fn solver_status_display() {
        assert_eq!(format!("{}", SolverStatus::Optimal), "optimal");
        assert_eq!(format!("{}", SolverStatus::Infeasible), "infeasible");
    }

    #[test]
    fn solver_error_code() {
        assert_eq!(SolverError::EmptyModel.code(), "SOLVER_EMPTY_MODEL");
        assert_eq!(SolverError::NoObjective.code(), "SOLVER_NO_OBJECTIVE");
        assert_eq!(
            SolverError::InvalidVariableId(42).code(),
            "SOLVER_INVALID_VARIABLE_ID"
        );
    }

    #[test]
    fn solver_error_display() {
        let err = SolverError::EmptyModel;
        assert!(err.to_string().contains("SOLVER_EMPTY_MODEL"));
        assert!(err.to_string().contains("no variables"));
    }

    #[test]
    fn solution_accessors() {
        let solution = Solution {
            primal_values: vec![1.0, 2.0, 3.0],
            variable_duals: vec![0.1, 0.2, 0.3],
            constraint_duals: vec![0.5],
            row_values: vec![4.0],
            objective_value: 10.0,
            status: SolverStatus::Optimal,
            solve_time_seconds: 0.1,
            metadata: BTreeMap::new(),
        };

        assert_eq!(solution.get_primal(0), Some(1.0));
        assert_eq!(solution.get_primal(3), None);
        assert_eq!(solution.get_variable_dual(1), Some(0.2));
        assert_eq!(solution.get_constraint_dual(0), Some(0.5));
        assert_eq!(solution.objective_value, 10.0);
        assert!(solution.is_optimal());
        assert!(solution.is_feasible());
    }

    #[test]
    fn solver_config_builder() {
        let config = SolverConfig::new()
            .with_time_limit(60.0)
            .with_verbosity(2)
            .with_parameter("threads", "4");

        assert_eq!(config.time_limit_seconds, Some(60.0));
        assert_eq!(config.verbosity, 2);
        assert_eq!(config.parameters.get("threads"), Some(&"4".to_string()));
    }

    #[test]
    fn solver_config_default() {
        let config = SolverConfig::default();
        assert!(config.time_limit_seconds.is_none());
        assert!(config.iteration_limit.is_none());
        assert!(config.presolve);
        assert_eq!(config.verbosity, 0);
    }
}
