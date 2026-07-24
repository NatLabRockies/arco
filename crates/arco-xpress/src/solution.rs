//! Solution extraction from Xpress solver results.

use arco_solver::{Solution as CoreSolution, SolverStatus as CoreSolverStatus};
use arco_solver::{SolutionView, SolverStatus};
use std::collections::BTreeMap;

/// Solution obtained from the Xpress solver.
///
/// Xpress handles maximize natively, so no objective sign correction is
/// needed. The solver status is stored as [`CoreSolverStatus`] directly.
#[derive(Debug, Clone)]
pub struct Solution {
    pub(crate) primal_values: Vec<f64>,
    pub(crate) variable_duals: Vec<f64>,
    pub(crate) constraint_duals: Vec<f64>,
    pub(crate) row_values: Vec<f64>,
    pub(crate) objective_value: f64,
    pub(crate) core_status: CoreSolverStatus,
    pub(crate) is_mip: bool,
    pub(crate) solve_time_seconds: f64,
}

impl Solution {
    /// Objective function value.
    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    /// Core solver status.
    pub(crate) fn core_status(&self) -> CoreSolverStatus {
        self.core_status
    }

    /// Whether this solution came from a MIP solve.
    pub(crate) fn is_mip(&self) -> bool {
        self.is_mip
    }

    /// All primal variable values.
    pub fn primal_values(&self) -> &[f64] {
        &self.primal_values
    }

    /// Variable dual values (reduced costs).
    pub(crate) fn variable_duals(&self) -> &[f64] {
        &self.variable_duals
    }

    /// Constraint dual values (shadow prices).
    pub(crate) fn constraint_duals(&self) -> &[f64] {
        &self.constraint_duals
    }

    /// Solve wall-clock time in seconds.
    pub fn solve_time_seconds(&self) -> f64 {
        self.solve_time_seconds
    }

    /// Primal value of variable at `index`, or `None` if out of bounds.
    pub(crate) fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    /// Reduced cost of variable at `index`, or `None` if out of bounds.
    pub(crate) fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    /// Shadow price of constraint at `index`, or `None` if out of bounds.
    pub(crate) fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    /// Whether the solution status is optimal.
    pub(crate) fn is_optimal(&self) -> bool {
        matches!(self.core_status, CoreSolverStatus::Optimal)
    }

    /// Whether the solver found a feasible solution (optimal or limit-reached).
    pub fn is_feasible(&self) -> bool {
        matches!(
            self.core_status,
            CoreSolverStatus::Optimal
                | CoreSolverStatus::TimeLimit
                | CoreSolverStatus::IterationLimit
        )
    }

    /// Whether the problem was proven infeasible.
    pub(crate) fn is_infeasible(&self) -> bool {
        matches!(self.core_status, CoreSolverStatus::Infeasible)
    }

    /// Whether the problem was proven unbounded.
    pub(crate) fn is_unbounded(&self) -> bool {
        matches!(self.core_status, CoreSolverStatus::Unbounded)
    }

    /// Convert this Xpress-specific solution into a solver-agnostic `arco_model::Solution`.
    pub(crate) fn into_core_solution(self) -> CoreSolution {
        CoreSolution {
            primal_values: self.primal_values,
            variable_duals: self.variable_duals,
            constraint_duals: self.constraint_duals,
            row_values: self.row_values,
            objective_value: self.objective_value,
            status: self.core_status,
            solve_time_seconds: self.solve_time_seconds,
            metadata: BTreeMap::new(),
        }
    }
}

impl SolutionView for Solution {
    fn objective_value(&self) -> f64 {
        self.objective_value
    }

    fn status(&self) -> SolverStatus {
        self.core_status
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

    /// Helper to create a test solution with sensible defaults.
    fn make_solution(status: CoreSolverStatus, is_mip: bool) -> Solution {
        Solution {
            primal_values: vec![1.0, 2.0, 3.0],
            variable_duals: vec![0.1, 0.2, 0.3],
            constraint_duals: vec![0.5, 0.6],
            row_values: vec![4.0, 5.0],
            objective_value: 42.0,
            core_status: status,
            is_mip,
            solve_time_seconds: 1.5,
        }
    }

    #[test]
    fn solution_view_objective_value() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::objective_value(&sol), 42.0);
    }

    #[test]
    fn solution_view_status_optimal() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::status(&sol), SolverStatus::Optimal);
    }

    #[test]
    fn solution_view_status_infeasible() {
        let sol = make_solution(CoreSolverStatus::Infeasible, false);
        assert_eq!(SolutionView::status(&sol), SolverStatus::Infeasible);
    }

    #[test]
    fn solution_view_status_unbounded() {
        let sol = make_solution(CoreSolverStatus::Unbounded, false);
        assert_eq!(SolutionView::status(&sol), SolverStatus::Unbounded);
    }

    #[test]
    fn solution_view_status_time_limit() {
        let sol = make_solution(CoreSolverStatus::TimeLimit, false);
        assert_eq!(SolutionView::status(&sol), SolverStatus::TimeLimit);
    }

    #[test]
    fn solution_view_status_iteration_limit() {
        let sol = make_solution(CoreSolverStatus::IterationLimit, false);
        assert_eq!(SolutionView::status(&sol), SolverStatus::IterationLimit);
    }

    #[test]
    fn solution_view_status_unknown() {
        let sol = make_solution(CoreSolverStatus::Unknown, false);
        assert_eq!(SolutionView::status(&sol), SolverStatus::Unknown);
    }

    #[test]
    fn solution_view_primal_values() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::primal_values(&sol), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn solution_view_variable_duals() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::variable_duals(&sol), &[0.1, 0.2, 0.3]);
    }

    #[test]
    fn solution_view_constraint_duals() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::constraint_duals(&sol), &[0.5, 0.6]);
    }

    #[test]
    fn solution_view_solve_time() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::solve_time_seconds(&sol), 1.5);
    }

    #[test]
    fn solution_view_get_primal_valid() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::get_primal(&sol, 0), Some(1.0));
        assert_eq!(SolutionView::get_primal(&sol, 1), Some(2.0));
        assert_eq!(SolutionView::get_primal(&sol, 2), Some(3.0));
    }

    #[test]
    fn solution_view_get_variable_dual_valid() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::get_variable_dual(&sol, 0), Some(0.1));
        assert_eq!(SolutionView::get_variable_dual(&sol, 2), Some(0.3));
    }

    #[test]
    fn solution_view_get_constraint_dual_valid() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(SolutionView::get_constraint_dual(&sol, 0), Some(0.5));
        assert_eq!(SolutionView::get_constraint_dual(&sol, 1), Some(0.6));
    }

    #[test]
    fn get_primal_out_of_bounds_returns_none() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(sol.get_primal(3), None);
        assert_eq!(sol.get_primal(100), None);
    }

    #[test]
    fn get_variable_dual_out_of_bounds_returns_none() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(sol.get_variable_dual(3), None);
        assert_eq!(sol.get_variable_dual(100), None);
    }

    #[test]
    fn get_constraint_dual_out_of_bounds_returns_none() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert_eq!(sol.get_constraint_dual(2), None);
        assert_eq!(sol.get_constraint_dual(100), None);
    }

    #[test]
    fn is_optimal_returns_true_for_optimal() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert!(sol.is_optimal());
    }

    #[test]
    fn is_optimal_returns_false_for_non_optimal() {
        assert!(!make_solution(CoreSolverStatus::Infeasible, false).is_optimal());
        assert!(!make_solution(CoreSolverStatus::Unbounded, false).is_optimal());
        assert!(!make_solution(CoreSolverStatus::TimeLimit, false).is_optimal());
        assert!(!make_solution(CoreSolverStatus::IterationLimit, false).is_optimal());
        assert!(!make_solution(CoreSolverStatus::Unknown, false).is_optimal());
    }

    #[test]
    fn is_feasible_returns_true_for_optimal_and_limits() {
        assert!(make_solution(CoreSolverStatus::Optimal, false).is_feasible());
        assert!(make_solution(CoreSolverStatus::TimeLimit, false).is_feasible());
        assert!(make_solution(CoreSolverStatus::IterationLimit, false).is_feasible());
    }

    #[test]
    fn is_feasible_returns_false_for_infeasible_unbounded_unknown() {
        assert!(!make_solution(CoreSolverStatus::Infeasible, false).is_feasible());
        assert!(!make_solution(CoreSolverStatus::Unbounded, false).is_feasible());
        assert!(!make_solution(CoreSolverStatus::Unknown, false).is_feasible());
    }

    #[test]
    fn is_infeasible_returns_true_only_for_infeasible() {
        assert!(make_solution(CoreSolverStatus::Infeasible, false).is_infeasible());
        assert!(!make_solution(CoreSolverStatus::Optimal, false).is_infeasible());
        assert!(!make_solution(CoreSolverStatus::Unbounded, false).is_infeasible());
    }

    #[test]
    fn is_unbounded_returns_true_only_for_unbounded() {
        assert!(make_solution(CoreSolverStatus::Unbounded, false).is_unbounded());
        assert!(!make_solution(CoreSolverStatus::Optimal, false).is_unbounded());
        assert!(!make_solution(CoreSolverStatus::Infeasible, false).is_unbounded());
    }

    #[test]
    fn is_mip_returns_correct_value() {
        assert!(make_solution(CoreSolverStatus::Optimal, true).is_mip());
        assert!(!make_solution(CoreSolverStatus::Optimal, false).is_mip());
    }

    #[test]
    fn core_status_returns_stored_status() {
        assert_eq!(
            make_solution(CoreSolverStatus::Optimal, false).core_status(),
            CoreSolverStatus::Optimal
        );
        assert_eq!(
            make_solution(CoreSolverStatus::Infeasible, false).core_status(),
            CoreSolverStatus::Infeasible
        );
        assert_eq!(
            make_solution(CoreSolverStatus::Unknown, true).core_status(),
            CoreSolverStatus::Unknown
        );
    }

    #[test]
    fn into_core_solution_preserves_all_fields() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        let core = sol.into_core_solution();

        assert_eq!(core.primal_values, vec![1.0, 2.0, 3.0]);
        assert_eq!(core.variable_duals, vec![0.1, 0.2, 0.3]);
        assert_eq!(core.constraint_duals, vec![0.5, 0.6]);
        assert_eq!(core.row_values, vec![4.0, 5.0]);
        assert_eq!(core.objective_value, 42.0);
        assert_eq!(core.status, CoreSolverStatus::Optimal);
        assert_eq!(core.solve_time_seconds, 1.5);
        assert!(core.metadata.is_empty());
    }

    #[test]
    fn into_core_solution_preserves_infeasible_status() {
        let sol = make_solution(CoreSolverStatus::Infeasible, true);
        let core = sol.into_core_solution();
        assert_eq!(core.status, CoreSolverStatus::Infeasible);
    }

    #[test]
    fn into_core_solution_preserves_time_limit_status() {
        let sol = make_solution(CoreSolverStatus::TimeLimit, false);
        let core = sol.into_core_solution();
        assert_eq!(core.status, CoreSolverStatus::TimeLimit);
    }

    #[test]
    fn solution_view_default_is_optimal() {
        let sol = make_solution(CoreSolverStatus::Optimal, false);
        assert!(SolutionView::is_optimal(&sol));
        assert!(SolutionView::is_feasible(&sol));
        assert!(!SolutionView::is_infeasible(&sol));
        assert!(!SolutionView::is_unbounded(&sol));
    }

    #[test]
    fn solution_view_default_is_infeasible() {
        let sol = make_solution(CoreSolverStatus::Infeasible, false);
        assert!(!SolutionView::is_optimal(&sol));
        assert!(!SolutionView::is_feasible(&sol));
        assert!(SolutionView::is_infeasible(&sol));
        assert!(!SolutionView::is_unbounded(&sol));
    }

    #[test]
    fn solution_view_default_is_unbounded() {
        let sol = make_solution(CoreSolverStatus::Unbounded, false);
        assert!(!SolutionView::is_optimal(&sol));
        assert!(!SolutionView::is_feasible(&sol));
        assert!(!SolutionView::is_infeasible(&sol));
        assert!(SolutionView::is_unbounded(&sol));
    }

    #[test]
    fn solution_view_default_time_limit_is_feasible() {
        let sol = make_solution(CoreSolverStatus::TimeLimit, false);
        assert!(!SolutionView::is_optimal(&sol));
        assert!(SolutionView::is_feasible(&sol));
        assert!(!SolutionView::is_infeasible(&sol));
        assert!(!SolutionView::is_unbounded(&sol));
    }

    #[test]
    fn empty_solution_returns_none_for_all_indices() {
        let sol = Solution {
            primal_values: vec![],
            variable_duals: vec![],
            constraint_duals: vec![],
            row_values: vec![],
            objective_value: 0.0,
            core_status: CoreSolverStatus::Unknown,
            is_mip: false,
            solve_time_seconds: 0.0,
        };

        assert_eq!(sol.get_primal(0), None);
        assert_eq!(sol.get_variable_dual(0), None);
        assert_eq!(sol.get_constraint_dual(0), None);
        assert!(sol.primal_values().is_empty());
        assert!(sol.variable_duals().is_empty());
        assert!(sol.constraint_duals().is_empty());
    }
}
