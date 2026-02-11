//! Solver error types.

use crate::SolverStatus;

/// Error type for solver operations.
#[derive(Debug, Clone)]
pub enum SolverError {
    /// Model has no variables.
    EmptyModel,
    /// No objective function set.
    NoObjective,
    /// Invalid objective sense.
    InvalidObjectiveSense,
    /// Invalid variable ID in warm-start hints or other operations.
    InvalidVariableId(u32),
    /// Internal solver error.
    InternalError(String),
    /// Solver failed to find optimal solution.
    SolveFailure {
        /// The solver status that caused the failure.
        status: SolverStatus,
    },
}

impl SolverError {
    /// Returns a semantic error code for programmatic handling.
    pub fn code(&self) -> &'static str {
        match self {
            SolverError::EmptyModel => "MODEL_EMPTY",
            SolverError::NoObjective => "OBJECTIVE_MISSING",
            SolverError::InvalidObjectiveSense => "OBJECTIVE_INVALID_SENSE",
            SolverError::InvalidVariableId(_) => "VARIABLE_INVALID_ID",
            SolverError::InternalError(_) => "SOLVER_INTERNAL",
            SolverError::SolveFailure { status } => match status {
                SolverStatus::Infeasible => "SOLVER_INFEASIBLE",
                SolverStatus::Unbounded => "SOLVER_UNBOUNDED",
                SolverStatus::ReachedTimeLimit => "SOLVER_TIME_LIMIT",
                SolverStatus::ReachedIterationLimit => "SOLVER_ITERATION_LIMIT",
                _ => "SOLVER_INTERNAL",
            },
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
            SolverError::InvalidVariableId(var_id) => {
                write!(f, "[{}] Variable ID {} does not exist", self.code(), var_id)
            }
            SolverError::InternalError(msg) => {
                write!(f, "[{}] Solver internal error: {}", self.code(), msg)
            }
            SolverError::SolveFailure { status } => {
                write!(f, "[{}] {}", self.code(), status_message(*status))
            }
        }
    }
}

fn status_message(status: SolverStatus) -> &'static str {
    match status {
        SolverStatus::Infeasible => "Problem is infeasible",
        SolverStatus::Unbounded => "Problem is unbounded",
        SolverStatus::ReachedTimeLimit => "Solver reached time limit",
        SolverStatus::ReachedIterationLimit => "Solver reached iteration limit",
        SolverStatus::Unknown => "Solver status unknown",
        SolverStatus::Optimal => "Solver returned optimal",
    }
}

impl std::error::Error for SolverError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_empty_model() {
        let err = SolverError::EmptyModel;
        let msg = format!("{}", err);
        assert!(msg.contains("MODEL_EMPTY"));
        assert!(msg.contains("no variables"));
    }

    #[test]
    fn test_error_display_no_objective() {
        let err = SolverError::NoObjective;
        let msg = format!("{}", err);
        assert!(msg.contains("OBJECTIVE_MISSING"));
    }

    #[test]
    fn test_error_display_invalid_variable_id() {
        let err = SolverError::InvalidVariableId(42);
        let msg = format!("{}", err);
        assert!(msg.contains("VARIABLE_INVALID_ID"));
        assert!(msg.contains("42"));
    }

    #[test]
    fn test_error_display_internal_error() {
        let err = SolverError::InternalError("something went wrong".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("SOLVER_INTERNAL"));
        assert!(msg.contains("something went wrong"));
    }

    #[test]
    fn test_error_display_solve_failure_infeasible() {
        let err = SolverError::SolveFailure {
            status: SolverStatus::Infeasible,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("SOLVER_INFEASIBLE"));
        assert!(msg.contains("infeasible"));
    }

    #[test]
    fn test_error_display_solve_failure_unbounded() {
        let err = SolverError::SolveFailure {
            status: SolverStatus::Unbounded,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("SOLVER_UNBOUNDED"));
        assert!(msg.contains("unbounded"));
    }

    #[test]
    fn test_error_display_solve_failure_time_limit() {
        let err = SolverError::SolveFailure {
            status: SolverStatus::ReachedTimeLimit,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("SOLVER_TIME_LIMIT"));
        assert!(msg.contains("time limit"));
    }

    #[test]
    fn test_error_code() {
        assert_eq!(SolverError::EmptyModel.code(), "MODEL_EMPTY");
        assert_eq!(SolverError::NoObjective.code(), "OBJECTIVE_MISSING");
        assert_eq!(
            SolverError::InvalidObjectiveSense.code(),
            "OBJECTIVE_INVALID_SENSE"
        );
        assert_eq!(
            SolverError::InvalidVariableId(0).code(),
            "VARIABLE_INVALID_ID"
        );
        assert_eq!(
            SolverError::InternalError(String::new()).code(),
            "SOLVER_INTERNAL"
        );
    }
}
