//! IPOPT solution type and trait implementations.

use crate::status::{
    IpoptSolveStatus, ipopt_has_solution, ipopt_status_string, ipopt_to_core_status,
    ipopt_to_generic_status,
};
use arco_solver::{Solution as CoreSolution, SolverStatus as CoreSolverStatus};
use arco_solver::{SolutionView, SolverStatus};
use std::collections::BTreeMap;

/// Solution from the IPOPT solver.
#[derive(Debug, Clone)]
pub struct Solution {
    pub(crate) primal_values: Vec<f64>,
    pub(crate) variable_duals: Vec<f64>,
    pub(crate) constraint_duals: Vec<f64>,
    pub(crate) row_values: Vec<f64>,
    pub(crate) objective_value: f64,
    pub(crate) status: IpoptSolveStatus,
    pub(crate) solve_time_seconds: f64,
}

impl Solution {
    /// Get the primal value of a variable at the given index.
    pub fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    /// Get the dual value (reduced cost) of a variable at the given index.
    pub fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    /// Get the dual value (shadow price) of a constraint at the given index.
    pub fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    /// Get the objective value.
    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    /// Get the IPOPT-specific status.
    pub fn ipopt_status(&self) -> IpoptSolveStatus {
        self.status
    }

    /// Get all primal values.
    pub fn primal_values(&self) -> &[f64] {
        &self.primal_values
    }

    /// Get all variable dual values.
    pub fn variable_duals(&self) -> &[f64] {
        &self.variable_duals
    }

    /// Get all constraint dual values.
    pub fn constraint_duals(&self) -> &[f64] {
        &self.constraint_duals
    }

    /// Get solve time in seconds.
    pub fn solve_time_seconds(&self) -> f64 {
        self.solve_time_seconds
    }

    /// Check if solution is optimal.
    pub fn is_optimal(&self) -> bool {
        matches!(
            self.status,
            IpoptSolveStatus::SolveSucceeded | IpoptSolveStatus::SolvedToAcceptableLevel
        )
    }

    /// Check if solution is feasible (includes optimal and limit-reached).
    pub fn is_feasible(&self) -> bool {
        ipopt_has_solution(self.status)
    }

    /// Check if solution is infeasible.
    pub fn is_infeasible(&self) -> bool {
        matches!(
            self.status,
            IpoptSolveStatus::InfeasibleProblemDetected | IpoptSolveStatus::RestorationFailed
        )
    }

    /// Check if solution is unbounded.
    pub fn is_unbounded(&self) -> bool {
        matches!(self.status, IpoptSolveStatus::DivergingIterates)
    }

    /// Get solution status as a human-readable string.
    pub fn status_string(&self) -> &'static str {
        ipopt_status_string(self.status)
    }

    /// Convert the IPOPT status to an `arco_solver::SolverStatus`.
    pub fn core_status(&self) -> CoreSolverStatus {
        ipopt_to_core_status(self.status)
    }

    /// Convert this IPOPT-specific solution into a solver-agnostic `arco_solver::Solution`.
    pub fn into_core_solution(self) -> CoreSolution {
        CoreSolution {
            primal_values: self.primal_values,
            variable_duals: self.variable_duals,
            constraint_duals: self.constraint_duals,
            row_values: self.row_values,
            objective_value: self.objective_value,
            status: ipopt_to_core_status(self.status),
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
        ipopt_to_generic_status(self.status)
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
mod tests {
    use super::*;

    #[test]
    fn test_solution_status_methods() {
        let optimal = Solution {
            primal_values: vec![1.0],
            variable_duals: vec![0.0],
            constraint_duals: vec![0.0],
            row_values: vec![0.0],
            objective_value: 42.0,
            status: IpoptSolveStatus::SolveSucceeded,
            solve_time_seconds: 0.1,
        };
        assert!(optimal.is_optimal());
        assert!(optimal.is_feasible());
        assert!(!optimal.is_infeasible());
        assert!(!optimal.is_unbounded());
        assert_eq!(optimal.core_status(), CoreSolverStatus::Optimal);
        assert_eq!(optimal.status_string(), "optimal");

        let infeasible = Solution {
            status: IpoptSolveStatus::InfeasibleProblemDetected,
            ..optimal.clone()
        };
        assert!(!infeasible.is_optimal());
        assert!(!infeasible.is_feasible());
        assert!(infeasible.is_infeasible());
        assert!(!infeasible.is_unbounded());
        assert_eq!(infeasible.core_status(), CoreSolverStatus::Infeasible);
        assert_eq!(infeasible.status_string(), "infeasible");
    }

    #[test]
    fn test_into_core_solution() {
        let solution = Solution {
            primal_values: vec![1.0, 2.0],
            variable_duals: vec![0.0, 0.0],
            constraint_duals: vec![3.0],
            row_values: vec![4.0],
            objective_value: 42.0,
            status: IpoptSolveStatus::SolveSucceeded,
            solve_time_seconds: 0.5,
        };

        let core = solution.into_core_solution();
        assert_eq!(core.primal_values, vec![1.0, 2.0]);
        assert!((core.objective_value - 42.0).abs() < f64::EPSILON);
        assert_eq!(core.status, CoreSolverStatus::Optimal);
        assert!((core.solve_time_seconds - 0.5).abs() < f64::EPSILON);
    }
}
