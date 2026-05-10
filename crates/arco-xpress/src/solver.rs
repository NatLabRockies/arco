//! Xpress solver implementation over primitive model views.

use crate::solution::Solution;
use arco_model::ModelView;
use arco_solver::{
    ModelViewBackend, ModelViewSolveResult, Solve, SolverConfig, SolverError as CoreSolverError,
};
use tracing::debug;

pub type SolverError = CoreSolverError;

/// Xpress backend registration object for primitive model views.
#[derive(Debug, Default, Clone, Copy)]
pub struct XpressModelViewBackend;

impl ModelViewBackend for XpressModelViewBackend {
    fn family(&self) -> &'static str {
        "xpress"
    }

    fn solve_model_view(
        &self,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        solve_model_view(model, config)
    }
}

pub struct Solver {
    config: SolverConfig,
}

impl Solver {
    pub fn new(model: &impl ModelView) -> Result<Self, SolverError> {
        if model.num_variables() == 0 {
            return Err(SolverError::EmptyModel);
        }
        debug!(
            component = "solver",
            operation = "init",
            status = "success",
            solver = "xpress",
            variables = model.num_variables() as u64,
            constraints = model.num_constraints() as u64,
            "Creating Xpress solver from model view"
        );
        Ok(Self {
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

    pub fn set_mip_gap(&mut self, gap: f64) {
        self.update_config(|config| config.with_mip_gap(gap));
    }

    pub fn set_verbosity(&mut self, level: u32) {
        self.update_config(|config| config.with_verbosity(level));
    }

    pub fn set_presolve(&mut self, enabled: bool) {
        self.update_config(|config| config.with_presolve(enabled));
    }

    pub fn set_threads(&mut self, threads: u32) {
        self.update_config(|config| config.with_threads(threads));
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
        solve_problem(&self.config)
    }

    pub fn solve_with_config(&mut self, config: &SolverConfig) -> Result<Solution, SolverError> {
        solve_problem(config)
    }
}

impl Solve for Solver {
    type Solution = Solution;

    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, CoreSolverError> {
        self.solve_with_config(config)
    }
}

/// Attempt to solve a primitive model view with Xpress.
pub fn solve_model_view(
    model: &(impl ModelView + ?Sized),
    _config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    Err(SolverError::SolverNotAvailable(
        "Xpress model-view solve execution is not linked in this build".to_string(),
    ))
}

fn solve_problem(_config: &SolverConfig) -> Result<Solution, SolverError> {
    Err(SolverError::SolverNotAvailable(
        "Xpress model-view solve execution is not linked in this build".to_string(),
    ))
}
