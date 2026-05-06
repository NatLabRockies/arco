//! IPOPT solver implementation over solver-facing targets.

use crate::problem::ArcoProblem;
use crate::solution::Solution;
use arco_contracts::SolverError as CoreSolverError;
use arco_contracts::{Solve, SolverConfig, SolverError as GenericSolverError};
use arco_targets::AlgebraicProblem;
use tracing::debug;

pub type SolverError = CoreSolverError;

pub struct Solver {
    problem: AlgebraicProblem,
    config: SolverConfig,
}

impl Solver {
    pub fn new(problem: AlgebraicProblem) -> Result<Self, SolverError> {
        ArcoProblem::validate_supported_target(&problem)?;
        debug!(
            component = "solver",
            operation = "init",
            status = "success",
            solver = "ipopt",
            variables = problem.variable_instances.len() as u64,
            constraints = problem.constraints.len() as u64,
            "Creating IPOPT solver from target"
        );
        Ok(Self {
            problem,
            config: SolverConfig::new(),
        })
    }

    fn update_config(&mut self, update: impl FnOnce(SolverConfig) -> SolverConfig) {
        self.config = update(std::mem::take(&mut self.config));
    }

    pub fn set_log_to_console(&mut self, enabled: bool) {
        self.update_config(|config| config.with_log_to_console(enabled));
    }

    pub fn set_time_limit(&mut self, seconds: f64) {
        self.update_config(|config| config.with_time_limit(seconds));
    }

    pub fn set_verbosity(&mut self, level: u32) {
        self.update_config(|config| config.with_verbosity(level));
    }

    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.update_config(|config| config.with_tolerance(tolerance));
    }

    pub fn config(&self) -> &SolverConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: SolverConfig) {
        self.config = config;
    }

    pub fn solve(&mut self) -> Result<Solution, SolverError> {
        solve_problem(&self.problem, &self.config)
    }

    pub fn solve_with_config(&mut self, config: &SolverConfig) -> Result<Solution, SolverError> {
        solve_problem(&self.problem, config)
    }
}

impl Solve for Solver {
    type Solution = Solution;

    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, GenericSolverError> {
        self.solve_with_config(config)
    }
}

fn solve_problem(
    _problem: &AlgebraicProblem,
    _config: &SolverConfig,
) -> Result<Solution, SolverError> {
    Err(SolverError::SolverNotAvailable(
        "IPOPT target adapter is not implemented yet".to_string(),
    ))
}
