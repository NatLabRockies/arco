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
    fn solution_view_default_status_helpers_match_solver_status() {
        let optimal = FixtureSolution {
            status: SolverStatus::Optimal,
        };
        let infeasible = FixtureSolution {
            status: SolverStatus::Infeasible,
        };
        let unbounded = FixtureSolution {
            status: SolverStatus::Unbounded,
        };
        let time_limit = FixtureSolution {
            status: SolverStatus::TimeLimit,
        };

        assert!(optimal.is_optimal());
        assert!(optimal.is_feasible());
        assert!(infeasible.is_infeasible());
        assert!(unbounded.is_unbounded());
        assert!(time_limit.is_feasible());
        assert!(!time_limit.is_optimal());
    }
}
