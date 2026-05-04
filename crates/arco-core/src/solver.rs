//! Solver trait for `arco-core` model backends.
//!
//! Shared solver contracts are re-exported from `arco-contracts`.

use crate::Model;

pub use arco_contracts::{Solution, SolverConfig, SolverError, SolverStatus};

/// Trait that all solver backends must implement.
pub trait Solver {
    /// Solve the given model and return a solver-agnostic solution.
    fn solve(&mut self, model: &Model) -> Result<Solution, SolverError>;
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
    fn solver_status_is_infeasible() {
        assert!(SolverStatus::Infeasible.is_infeasible());
        assert!(!SolverStatus::Optimal.is_infeasible());
    }

    #[test]
    fn solver_status_is_unbounded() {
        assert!(SolverStatus::Unbounded.is_unbounded());
        assert!(!SolverStatus::Optimal.is_unbounded());
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
    fn solver_error_display() {
        assert_eq!(SolverError::EmptyModel.code(), "SOLVER_EMPTY_MODEL");
        assert!(SolverError::EmptyModel.to_string().contains("no variables"));
        assert!(
            SolverError::NoObjective
                .to_string()
                .contains("no objective")
        );
        assert!(
            SolverError::InvalidObjectiveSense
                .to_string()
                .contains("Invalid objective sense")
        );
        assert!(
            SolverError::InvalidVariableId(42)
                .to_string()
                .contains("42")
        );
        assert!(
            SolverError::SolverNotAvailable("Xpress".to_string())
                .to_string()
                .contains("Xpress")
        );
        assert!(
            SolverError::SolverSpecific("oops".to_string())
                .to_string()
                .contains("oops")
        );

        let err = SolverError::SolveFailure {
            status: SolverStatus::Infeasible,
        };
        assert!(err.to_string().contains("infeasible"));
    }

    #[test]
    fn solution_accessors() {
        use std::collections::BTreeMap;

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
        assert_eq!(solution.get_constraint_dual(1), None);
        assert_eq!(solution.objective_value, 10.0);
        assert!(solution.is_optimal());
        assert!(solution.is_feasible());
        assert!(!solution.is_infeasible());
        assert!(!solution.is_unbounded());
        assert_eq!(solution.status_string(), "optimal");
    }
}
