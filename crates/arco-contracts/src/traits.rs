use crate::{SolverConfig, SolverError, SolverStatus};

pub trait SolutionView {
    fn objective_value(&self) -> f64;

    fn status(&self) -> SolverStatus;

    fn get_primal(&self, index: usize) -> Option<f64>;

    fn get_variable_dual(&self, index: usize) -> Option<f64>;

    fn get_constraint_dual(&self, index: usize) -> Option<f64>;

    fn primal_values(&self) -> &[f64];

    fn variable_duals(&self) -> &[f64];

    fn constraint_duals(&self) -> &[f64];

    fn solve_time_seconds(&self) -> f64;

    fn is_optimal(&self) -> bool {
        self.status().is_optimal()
    }

    fn is_feasible(&self) -> bool {
        self.status().is_feasible()
    }

    fn is_infeasible(&self) -> bool {
        self.status().is_infeasible()
    }

    fn is_unbounded(&self) -> bool {
        self.status().is_unbounded()
    }
}

pub trait Solve {
    type Solution: SolutionView;

    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, SolverError>;
}
