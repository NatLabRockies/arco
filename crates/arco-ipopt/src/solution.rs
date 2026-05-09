//! IPOPT solution type and trait implementations.

use crate::status::{
    ipopt_has_solution, ipopt_status_string, ipopt_to_core_status, ipopt_to_generic_status,
};
use arco_solver::{Solution as CoreSolution, SolverStatus as CoreSolverStatus};
use arco_solver::{SolutionView, SolverStatus};
use ipopt::SolveStatus;
use std::collections::BTreeMap;

/// Solution from the IPOPT solver.
#[derive(Debug, Clone)]
pub struct Solution {
    pub(crate) primal_values: Vec<f64>,
    pub(crate) variable_duals: Vec<f64>,
    pub(crate) constraint_duals: Vec<f64>,
    pub(crate) row_values: Vec<f64>,
    pub(crate) objective_value: f64,
    pub(crate) status: SolveStatus,
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
    pub fn ipopt_status(&self) -> SolveStatus {
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
            SolveStatus::SolveSucceeded | SolveStatus::SolvedToAcceptableLevel
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
            SolveStatus::InfeasibleProblemDetected | SolveStatus::RestorationFailed
        )
    }

    /// Check if solution is unbounded.
    pub fn is_unbounded(&self) -> bool {
        matches!(self.status, SolveStatus::DivergingIterates)
    }

    /// Get solution status as a human-readable string.
    pub fn status_string(&self) -> &'static str {
        ipopt_status_string(self.status)
    }

    /// Convert the IPOPT status to an `arco_model::SolverStatus`.
    pub fn core_status(&self) -> CoreSolverStatus {
        ipopt_to_core_status(self.status)
    }

    /// Convert this IPOPT-specific solution into a solver-agnostic `arco_model::Solution`.
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
