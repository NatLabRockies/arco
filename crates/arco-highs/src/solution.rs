//! Solution type and trait implementations.

use crate::ffi::HighsStatus;
use crate::status::{
    highs_has_solution, highs_status_string, highs_to_core_status, highs_to_generic_status,
};
use arco_core::solver::{Solution as CoreSolution, SolverStatus as CoreSolverStatus};
use arco_solver::{SolutionView, SolverStatus};
use std::collections::BTreeMap;

/// Solution from HiGHS solver
#[derive(Debug, Clone)]
pub struct Solution {
    /// Primal values of variables indexed by their internal position
    pub(crate) primal_values: Vec<f64>,
    /// Dual values of variables (reduced costs) indexed by their internal position
    pub(crate) variable_duals: Vec<f64>,
    /// Dual values of constraints (shadow prices) indexed by their internal position
    pub(crate) constraint_duals: Vec<f64>,
    /// Row activity values (constraint LHS evaluated at the solution)
    pub(crate) row_values: Vec<f64>,
    /// Objective value of the solution
    pub(crate) objective_value: f64,
    /// Status of the solution
    pub(crate) status: HighsStatus,
    /// Solve time in seconds
    pub(crate) solve_time_seconds: f64,
    /// Number of simplex iterations (0 for pure feasibility problems)
    pub(crate) simplex_iterations: u64,
    /// Number of barrier iterations (0 if simplex was used)
    pub(crate) barrier_iterations: u64,
    /// Relative MIP gap (0.0 for LP problems)
    pub(crate) mip_gap: f64,
    /// Primal feasibility tolerance achieved
    pub(crate) primal_feasibility_tolerance: f64,
    /// Dual feasibility tolerance achieved
    pub(crate) dual_feasibility_tolerance: f64,
    /// Number of rows after presolve (0 if presolve not used)
    pub(crate) presolved_rows: u64,
    /// Number of cols after presolve (0 if presolve not used)
    pub(crate) presolved_cols: u64,
}

impl Solution {
    /// Get the primal value of a variable at the given index
    pub fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    /// Get the dual value (reduced cost) of a variable at the given index
    pub fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    /// Get the dual value (shadow price) of a constraint at the given index
    pub fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    /// Get the objective value
    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    /// Get the HiGHS-specific status
    pub fn highs_status(&self) -> HighsStatus {
        self.status
    }

    /// Get all primal values
    pub fn primal_values(&self) -> &[f64] {
        &self.primal_values
    }

    /// Get all variable dual values
    pub fn variable_duals(&self) -> &[f64] {
        &self.variable_duals
    }

    /// Get all constraint dual values
    pub fn constraint_duals(&self) -> &[f64] {
        &self.constraint_duals
    }

    /// Get solve time in seconds
    pub fn solve_time_seconds(&self) -> f64 {
        self.solve_time_seconds
    }

    /// Get number of simplex iterations
    pub fn simplex_iterations(&self) -> u64 {
        self.simplex_iterations
    }

    /// Get number of barrier iterations
    pub fn barrier_iterations(&self) -> u64 {
        self.barrier_iterations
    }

    /// Get total iterations (simplex + barrier)
    pub fn total_iterations(&self) -> u64 {
        self.simplex_iterations + self.barrier_iterations
    }

    /// Get relative MIP gap (0.0 for LP problems)
    pub fn mip_gap(&self) -> f64 {
        self.mip_gap
    }

    /// Get primal feasibility tolerance achieved
    pub fn primal_feasibility_tolerance(&self) -> f64 {
        self.primal_feasibility_tolerance
    }

    /// Get dual feasibility tolerance achieved
    pub fn dual_feasibility_tolerance(&self) -> f64 {
        self.dual_feasibility_tolerance
    }

    /// Get number of rows after presolve (0 if presolve not used or problem fully reduced)
    pub fn presolved_rows(&self) -> u64 {
        self.presolved_rows
    }

    /// Get number of cols after presolve (0 if presolve not used or problem fully reduced)
    pub fn presolved_cols(&self) -> u64 {
        self.presolved_cols
    }

    /// Check if solution is optimal
    pub fn is_optimal(&self) -> bool {
        matches!(self.status, HighsStatus::Optimal)
    }

    /// Check if solution is feasible (includes optimal)
    pub fn is_feasible(&self) -> bool {
        highs_has_solution(self.status)
    }

    /// Check if solution is infeasible
    pub fn is_infeasible(&self) -> bool {
        matches!(self.status, HighsStatus::Infeasible)
    }

    /// Check if solution is unbounded
    pub fn is_unbounded(&self) -> bool {
        matches!(self.status, HighsStatus::Unbounded)
    }

    /// Get solution status as a human-readable string
    pub fn status_string(&self) -> &'static str {
        highs_status_string(self.status)
    }

    /// Convert the HiGHS status to a arco_core::SolverStatus.
    pub fn core_status(&self) -> CoreSolverStatus {
        highs_to_core_status(self.status)
    }

    /// Convert this HiGHS-specific solution into a solver-agnostic `arco_core::Solution`.
    pub fn into_core_solution(self) -> CoreSolution {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "simplex_iterations".to_string(),
            self.simplex_iterations as f64,
        );
        metadata.insert(
            "barrier_iterations".to_string(),
            self.barrier_iterations as f64,
        );
        metadata.insert("mip_gap".to_string(), self.mip_gap);
        metadata.insert(
            "primal_feasibility_tolerance".to_string(),
            self.primal_feasibility_tolerance,
        );
        metadata.insert(
            "dual_feasibility_tolerance".to_string(),
            self.dual_feasibility_tolerance,
        );
        metadata.insert("presolved_rows".to_string(), self.presolved_rows as f64);
        metadata.insert("presolved_cols".to_string(), self.presolved_cols as f64);

        CoreSolution {
            primal_values: self.primal_values,
            variable_duals: self.variable_duals,
            constraint_duals: self.constraint_duals,
            row_values: self.row_values,
            objective_value: self.objective_value,
            status: highs_to_core_status(self.status),
            solve_time_seconds: self.solve_time_seconds,
            metadata,
        }
    }
}

// Implement the SolutionView trait from arco-solver
impl SolutionView for Solution {
    fn objective_value(&self) -> f64 {
        self.objective_value
    }

    fn status(&self) -> SolverStatus {
        highs_to_generic_status(self.status)
    }

    fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    fn primal_values(&self) -> &[f64] {
        &self.primal_values
    }

    fn variable_duals(&self) -> &[f64] {
        &self.variable_duals
    }

    fn constraint_duals(&self) -> &[f64] {
        &self.constraint_duals
    }

    fn solve_time_seconds(&self) -> f64 {
        self.solve_time_seconds
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_solution_status_helpers() {
        let solution = Solution {
            primal_values: vec![1.0, 2.0],
            variable_duals: vec![0.0, 0.0],
            constraint_duals: vec![],
            row_values: vec![],
            objective_value: 10.0,
            status: HighsStatus::Optimal,
            solve_time_seconds: 0.1,
            simplex_iterations: 5,
            barrier_iterations: 0,
            mip_gap: 0.0,
            primal_feasibility_tolerance: 1e-7,
            dual_feasibility_tolerance: 1e-7,
            presolved_rows: 0,
            presolved_cols: 0,
        };

        assert!(solution.is_optimal());
        assert!(solution.is_feasible());
        assert!(!solution.is_infeasible());
        assert!(!solution.is_unbounded());
        assert_eq!(solution.status_string(), "optimal");
    }

    #[test]
    fn test_solution_limit_status_is_feasible() {
        let solution = Solution {
            primal_values: vec![1.0],
            variable_duals: vec![0.0],
            constraint_duals: vec![],
            row_values: vec![],
            objective_value: 1.0,
            status: HighsStatus::ReachedTimeLimit,
            solve_time_seconds: 0.1,
            simplex_iterations: 3,
            barrier_iterations: 0,
            mip_gap: 0.0,
            primal_feasibility_tolerance: 1e-7,
            dual_feasibility_tolerance: 1e-7,
            presolved_rows: 0,
            presolved_cols: 0,
        };

        assert!(!solution.is_optimal());
        assert!(solution.is_feasible());
    }

    #[test]
    fn test_solution_get_primal() {
        let solution = Solution {
            primal_values: vec![1.0, 2.0, 3.0],
            variable_duals: vec![0.1, 0.2, 0.3],
            constraint_duals: vec![0.5],
            row_values: vec![],
            objective_value: 6.0,
            status: HighsStatus::Optimal,
            solve_time_seconds: 0.05,
            simplex_iterations: 3,
            barrier_iterations: 0,
            mip_gap: 0.0,
            primal_feasibility_tolerance: 1e-7,
            dual_feasibility_tolerance: 1e-7,
            presolved_rows: 0,
            presolved_cols: 0,
        };

        assert_eq!(solution.get_primal(0), Some(1.0));
        assert_eq!(solution.get_primal(1), Some(2.0));
        assert_eq!(solution.get_primal(2), Some(3.0));
        assert_eq!(solution.get_primal(3), None); // Out of bounds

        assert_eq!(solution.get_variable_dual(0), Some(0.1));
        assert_eq!(solution.get_constraint_dual(0), Some(0.5));
        assert_eq!(solution.get_constraint_dual(1), None);
    }

    #[test]
    fn test_solution_iteration_counts() {
        let solution = Solution {
            primal_values: vec![],
            variable_duals: vec![],
            constraint_duals: vec![],
            row_values: vec![],
            objective_value: 0.0,
            status: HighsStatus::Optimal,
            solve_time_seconds: 1.0,
            simplex_iterations: 100,
            barrier_iterations: 50,
            mip_gap: 0.0,
            primal_feasibility_tolerance: 1e-7,
            dual_feasibility_tolerance: 1e-7,
            presolved_rows: 10,
            presolved_cols: 5,
        };

        assert_eq!(solution.simplex_iterations(), 100);
        assert_eq!(solution.barrier_iterations(), 50);
        assert_eq!(solution.total_iterations(), 150);
        assert_eq!(solution.presolved_rows(), 10);
        assert_eq!(solution.presolved_cols(), 5);
    }

    #[test]
    fn test_solution_view_trait() {
        use arco_solver::SolutionView;

        let solution = Solution {
            primal_values: vec![1.0, 2.0],
            variable_duals: vec![0.1, 0.2],
            constraint_duals: vec![0.5],
            row_values: vec![],
            objective_value: 10.0,
            status: HighsStatus::Optimal,
            solve_time_seconds: 0.1,
            simplex_iterations: 5,
            barrier_iterations: 0,
            mip_gap: 0.0,
            primal_feasibility_tolerance: 1e-7,
            dual_feasibility_tolerance: 1e-7,
            presolved_rows: 0,
            presolved_cols: 0,
        };

        // Test trait methods
        assert_eq!(SolutionView::objective_value(&solution), 10.0);
        assert_eq!(SolutionView::status(&solution), SolverStatus::Optimal);
        assert!(SolutionView::is_optimal(&solution));
        assert!(SolutionView::is_feasible(&solution));
        assert!(!SolutionView::is_infeasible(&solution));
        assert!(!SolutionView::is_unbounded(&solution));
    }
}
