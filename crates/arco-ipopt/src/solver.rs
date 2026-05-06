//! IPOPT solver implementation.

use crate::problem::ArcoProblem;
use crate::solution::Solution;
use crate::status::{ipopt_has_solution, ipopt_to_core_status};
use arco_contracts::SolverError as CoreSolverError;
use arco_contracts::{Solve, SolverBackend, SolverConfig, SolverError as GenericSolverError};
use arco_core::Model;
use arco_expr::VariableId;
use ipopt::Ipopt;
use std::time::Instant;
use tracing::{debug, warn};

/// Re-export of contract solver error for backward compatibility.
pub type SolverError = CoreSolverError;

/// IPOPT solver wrapper.
pub struct Solver {
    model: Model,
    config: SolverConfig,
    primal_start: Option<Vec<(VariableId, f64)>>,
}

impl Solver {
    /// Create a new IPOPT solver from a Model.
    pub fn new(model: Model) -> Result<Self, SolverError> {
        validate_model(&model)?;

        debug!(
            component = "solver",
            operation = "init",
            status = "success",
            solver = "ipopt",
            variables = model.num_variables() as u64,
            constraints = model.num_constraints() as u64,
            "Creating IPOPT solver from model"
        );

        Ok(Solver {
            model,
            config: SolverConfig::new(),
            primal_start: None,
        })
    }

    fn update_config(&mut self, update: impl FnOnce(SolverConfig) -> SolverConfig) {
        self.config = update(std::mem::take(&mut self.config));
    }

    /// Enable or disable IPOPT logging to console for the next solve.
    pub fn set_log_to_console(&mut self, enabled: bool) {
        self.update_config(|config| config.with_log_to_console(enabled));
    }

    /// Set a time limit in seconds for the next solve.
    pub fn set_time_limit(&mut self, seconds: f64) {
        self.update_config(|config| config.with_time_limit(seconds));
    }

    /// Set verbosity level for the next solve.
    pub fn set_verbosity(&mut self, level: u32) {
        self.update_config(|config| config.with_verbosity(level));
    }

    /// Set feasibility tolerance for the next solve.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.update_config(|config| config.with_tolerance(tolerance));
    }

    /// Set primal start values (warm-start hints).
    pub fn set_primal_start(&mut self, hints: &[(VariableId, f64)]) -> Result<(), SolverError> {
        for (var_id, _) in hints {
            if self.model.get_variable(*var_id).is_err() {
                return Err(SolverError::InvalidVariableId(var_id.inner()));
            }
        }
        self.primal_start = Some(hints.to_vec());
        debug!(
            component = "solver",
            operation = "set_primal_start",
            status = "success",
            num_hints = hints.len(),
            "Stored warm-start hints"
        );
        Ok(())
    }

    /// Clear primal start hints.
    pub fn clear_primal_start(&mut self) {
        self.primal_start = None;
    }

    /// Get current primal start hints.
    pub fn get_primal_start(&self) -> Option<&[(VariableId, f64)]> {
        self.primal_start.as_deref()
    }

    /// Get access to the current solver configuration.
    pub fn config(&self) -> &SolverConfig {
        &self.config
    }

    /// Set the solver configuration.
    pub fn set_config(&mut self, config: SolverConfig) {
        self.config = config;
    }

    /// Solve the model and return the solution.
    pub fn solve(&mut self) -> Result<Solution, SolverError> {
        solve_model(&self.model, &self.config, self.primal_start.as_deref())
    }

    /// Solve the model with a specific configuration.
    pub fn solve_with_config(&mut self, config: &SolverConfig) -> Result<Solution, SolverError> {
        solve_model(&self.model, config, self.primal_start.as_deref())
    }
}

impl Solve for Solver {
    type Solution = Solution;

    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, GenericSolverError> {
        self.solve_with_config(config)
    }
}

/// Zero-sized backend for trait-based dispatch from the Python bindings.
pub struct IpoptBackend;

impl SolverBackend<Model, VariableId> for IpoptBackend {
    fn solve(
        &self,
        model: &Model,
        config: &SolverConfig,
        primal_start: Option<&[(VariableId, f64)]>,
    ) -> Result<Solution, GenericSolverError> {
        solve_model(model, config, primal_start).map(|s| s.into_core_solution())
    }

    fn name(&self) -> &'static str {
        "IPOPT"
    }

    fn supports_integer(&self) -> bool {
        false
    }
}

/// Validate that a model is ready for solving.
fn validate_model(model: &Model) -> Result<(), SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    Ok(())
}

/// Apply `SolverConfig` to an IPOPT solver instance.
fn apply_ipopt_config<P: ipopt::ConstrainedProblem>(ipopt: &mut Ipopt<P>, config: &SolverConfig) {
    // Always use limited-memory BFGS for LP (zero Hessian)
    ipopt.set_option("hessian_approximation", "limited-memory");

    // Suppress output by default; use level 5 when console logging is enabled
    let log_enabled = config.log_to_console.unwrap_or(false);
    let default_level = if log_enabled { 5 } else { 0 };
    let print_level = config.verbosity.map_or(default_level, |v| v.min(12) as i32);
    ipopt.set_option("print_level", print_level);

    if let Some(limit) = config.time_limit {
        ipopt.set_option("max_cpu_time", limit);
    }
    if let Some(tol) = config.tolerance {
        ipopt.set_option("tol", tol);
        ipopt.set_option("constr_viol_tol", tol);
    }
}

/// Solve a model with the given config.
fn solve_model(
    model: &Model,
    config: &SolverConfig,
    primal_start: Option<&[(VariableId, f64)]>,
) -> Result<Solution, SolverError> {
    validate_model(model)?;

    let solve_started = Instant::now();

    debug!(
        component = "solver",
        operation = "solve",
        solver = "ipopt",
        variables = model.num_variables() as u64,
        constraints = model.num_constraints() as u64,
        "Starting IPOPT solve"
    );

    let problem = ArcoProblem::from_model(model, primal_start)?;

    let mut ipopt = Ipopt::new(problem).map_err(|e| {
        SolverError::SolverSpecific(format!("Failed to create IPOPT problem: {e:?}"))
    })?;

    apply_ipopt_config(&mut ipopt, config);

    let result = ipopt.solve();
    let solve_time = solve_started.elapsed().as_secs_f64();

    let status = result.status;
    let raw_objective = result.objective_value;

    debug!(
        component = "solver",
        operation = "solve",
        solver = "ipopt",
        solver_status = ?status,
        objective_value = raw_objective,
        duration_ms = solve_time * 1000.0,
        "IPOPT solve completed"
    );

    if !ipopt_has_solution(status) {
        warn!(
            component = "solver",
            operation = "solve",
            status = "warn",
            solver = "ipopt",
            solver_status = ?status,
            duration_ms = solve_time * 1000.0,
            "Solver did not find optimal solution"
        );
        return Err(SolverError::SolveFailure {
            status: ipopt_to_core_status(status),
        });
    }

    let sol = &result.solver_data.solution;

    Ok(Solution::from_ipopt(
        sol.primal_variables,
        sol.lower_bound_multipliers,
        sol.upper_bound_multipliers,
        sol.constraint_multipliers,
        result.constraint_values,
        raw_objective,
        status,
        solve_time,
        result.solver_data.problem,
    ))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_core::types::Bounds;
    use arco_core::{Objective, Sense, Variable};

    #[test]
    fn test_solver_new_rejects_empty_model() {
        let model = Model::new();
        assert!(matches!(Solver::new(model), Err(SolverError::EmptyModel)));
    }

    fn build_single_variable_model() -> Model {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("variable");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.0)],
            })
            .expect("objective");
        model
    }

    #[test]
    fn test_primal_start_storage() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).unwrap();
        let hints = vec![(VariableId::new(0), 5.0)];
        assert!(solver.set_primal_start(&hints).is_ok());
        assert_eq!(solver.get_primal_start(), Some(hints.as_slice()));
    }

    #[test]
    fn test_primal_start_validation() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).unwrap();
        let invalid_hints = vec![(VariableId::new(9999), 0.5)];
        assert!(solver.set_primal_start(&invalid_hints).is_err());
    }

    #[test]
    fn test_primal_start_clear() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).unwrap();
        let hints = vec![(VariableId::new(0), 5.0)];
        solver.set_primal_start(&hints).unwrap();
        assert!(solver.get_primal_start().is_some());
        solver.clear_primal_start();
        assert!(solver.get_primal_start().is_none());
    }
}
