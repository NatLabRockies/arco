//! Solver traits for abstraction over different solver backends.

use crate::{SolverConfig, SolverError, SolverStatus};

/// Trait for accessing solution data from a solver.
///
/// This trait provides a common interface for reading solution values
/// regardless of the underlying solver implementation.
pub trait SolutionView {
    /// Get the objective value of the solution.
    fn objective_value(&self) -> f64;

    /// Get the solver status.
    fn status(&self) -> SolverStatus;

    /// Get the primal value at the given index.
    fn get_primal(&self, index: usize) -> Option<f64>;

    /// Get the variable dual (reduced cost) at the given index.
    fn get_variable_dual(&self, index: usize) -> Option<f64>;

    /// Get the constraint dual (shadow price) at the given index.
    fn get_constraint_dual(&self, index: usize) -> Option<f64>;

    /// Get all primal values as a slice.
    fn primal_values(&self) -> &[f64];

    /// Get all variable dual values as a slice.
    fn variable_duals(&self) -> &[f64];

    /// Get all constraint dual values as a slice.
    fn constraint_duals(&self) -> &[f64];

    /// Get the solve time in seconds.
    fn solve_time_seconds(&self) -> f64;

    /// Check if the solution is optimal.
    fn is_optimal(&self) -> bool {
        self.status().is_optimal()
    }

    /// Check if the solution is feasible.
    fn is_feasible(&self) -> bool {
        self.status().is_feasible()
    }

    /// Check if the solution is infeasible.
    fn is_infeasible(&self) -> bool {
        self.status().is_infeasible()
    }

    /// Check if the solution is unbounded.
    fn is_unbounded(&self) -> bool {
        self.status().is_unbounded()
    }
}

/// Trait for solver implementations.
///
/// This trait defines the interface that all solver backends must implement.
/// It allows for a unified API across different solvers (HiGHS, Gurobi, etc.).
pub trait Solve {
    /// The solution type returned by this solver.
    type Solution: SolutionView;

    /// Solve the model with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns a `SolverError` if:
    /// - The model is empty
    /// - No objective is set
    /// - The solver fails to find a solution
    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, SolverError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixtureSolution {
        status: SolverStatus,
    }

    impl SolutionView for FixtureSolution {
        fn objective_value(&self) -> f64 {
            0.0
        }

        fn status(&self) -> SolverStatus {
            self.status
        }

        fn get_primal(&self, _index: usize) -> Option<f64> {
            None
        }

        fn get_variable_dual(&self, _index: usize) -> Option<f64> {
            None
        }

        fn get_constraint_dual(&self, _index: usize) -> Option<f64> {
            None
        }

        fn primal_values(&self) -> &[f64] {
            &[]
        }

        fn variable_duals(&self) -> &[f64] {
            &[]
        }

        fn constraint_duals(&self) -> &[f64] {
            &[]
        }

        fn solve_time_seconds(&self) -> f64 {
            0.0
        }
    }

    #[test]
    fn test_solution_view_default_is_optimal() {
        let solution = FixtureSolution {
            status: SolverStatus::Optimal,
        };
        assert!(solution.is_optimal());
        assert!(solution.is_feasible());
        assert!(!solution.is_infeasible());
        assert!(!solution.is_unbounded());
    }

    #[test]
    fn test_solution_view_default_is_infeasible() {
        let solution = FixtureSolution {
            status: SolverStatus::Infeasible,
        };
        assert!(!solution.is_optimal());
        assert!(!solution.is_feasible());
        assert!(solution.is_infeasible());
        assert!(!solution.is_unbounded());
    }

    #[test]
    fn test_solution_view_default_is_unbounded() {
        let solution = FixtureSolution {
            status: SolverStatus::Unbounded,
        };
        assert!(!solution.is_optimal());
        assert!(!solution.is_feasible());
        assert!(!solution.is_infeasible());
        assert!(solution.is_unbounded());
    }

    #[test]
    fn test_solution_view_time_limit_is_feasible() {
        let solution = FixtureSolution {
            status: SolverStatus::ReachedTimeLimit,
        };
        assert!(!solution.is_optimal());
        assert!(solution.is_feasible()); // time limit may have feasible solution
        assert!(!solution.is_infeasible());
        assert!(!solution.is_unbounded());
    }
}
