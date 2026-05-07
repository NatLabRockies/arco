//! HiGHS solver implementation over solver-facing targets.

use crate::ffi::{HighsModel, HighsModelError, HighsOption, HighsStatus};
use crate::solution::Solution;
use crate::status::{highs_has_solution, highs_to_core_status};
use arco_model::{ConstraintId, ModelView, Sense, VariableId};
use arco_solver::ModelViewSolveResult;
use arco_solver::SolverError as GenericSolverError;
use arco_solver::{Solve, SolverConfig};
use arco_targets::{AlgebraicProblem, ConstraintSense, ObjectiveSense, VariableKind};
use arco_tools::memory::capture_rss_bytes;
use std::collections::BTreeMap;
use std::time::Instant;
use tracing::{debug, trace, warn};

/// Re-export of contract solver error for backward compatibility.
pub type SolverError = arco_solver::SolverError;

fn highs_model_error_to_solver_error(err: HighsModelError) -> SolverError {
    SolverError::SolverSpecific(err.to_string())
}

/// HiGHS adapter for an already-lowered algebraic solve target.
pub struct Solver {
    problem: AlgebraicProblem,
    config: SolverConfig,
}

impl Solver {
    /// Create a new solver from a lowered algebraic problem.
    pub fn new(problem: AlgebraicProblem) -> Result<Self, SolverError> {
        validate_problem(&problem)?;
        debug!(
            component = "solver",
            operation = "init",
            status = "success",
            variables = problem.variable_instances.len() as u64,
            constraints = problem.constraints.len() as u64,
            "Creating HiGHS solver from algebraic target"
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

fn validate_problem(problem: &AlgebraicProblem) -> Result<(), SolverError> {
    if problem.variable_instances.is_empty() {
        return Err(SolverError::EmptyModel);
    }
    Ok(())
}

fn validate_solver_config(config: &SolverConfig) -> Result<(), SolverError> {
    if let Some(limit) = config.time_limit {
        if !limit.is_finite() || limit < 0.0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: time_limit must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(gap) = config.mip_gap {
        if !gap.is_finite() || gap < 0.0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: mip_gap must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(tolerance) = config.tolerance {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: tolerance must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(threads) = config.threads {
        if threads == 0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: threads must be >= 1".to_string(),
            ));
        }
    }
    Ok(())
}

fn apply_solver_config(
    highs_model: &mut HighsModel,
    config: &SolverConfig,
) -> Result<(), SolverError> {
    validate_solver_config(config)?;
    highs_model.set_log_to_console(config.log_to_console.unwrap_or(false));

    if let Some(limit) = config.time_limit {
        highs_model.set_option("time_limit", HighsOption::Float(limit));
    }
    if let Some(gap) = config.mip_gap {
        highs_model.set_option("mip_rel_gap", HighsOption::Float(gap));
    }
    if let Some(level) = config.verbosity {
        highs_model.set_verbosity(level);
    }
    if let Some(presolve) = config.presolve {
        highs_model.set_option(
            "presolve",
            HighsOption::Str(if presolve { "on" } else { "off" }.to_string()),
        );
    }
    if let Some(threads) = config.threads {
        highs_model.set_option("threads", HighsOption::Int(threads as i32));
    }
    if let Some(tolerance) = config.tolerance {
        highs_model.set_option(
            "primal_feasibility_tolerance",
            HighsOption::Float(tolerance),
        );
        highs_model.set_option("dual_feasibility_tolerance", HighsOption::Float(tolerance));
    }
    Ok(())
}

fn objective_coefficients(problem: &AlgebraicProblem) -> BTreeMap<&str, f64> {
    let mut coefficients = BTreeMap::new();
    for term in &problem.objective.terms {
        *coefficients
            .entry(term.variable_name.as_str())
            .or_insert(0.0) += term.coefficient;
    }
    coefficients
}

fn add_variables_to_highs(
    problem: &AlgebraicProblem,
    highs_model: &mut HighsModel,
) -> BTreeMap<String, usize> {
    let objective_coeffs = objective_coefficients(problem);
    let mut columns = BTreeMap::new();

    for variable in &problem.variable_instances {
        let obj_coeff = objective_coeffs
            .get(variable.name.as_str())
            .copied()
            .unwrap_or(0.0);
        let upper = variable.upper.unwrap_or(f64::INFINITY);
        let is_integer = matches!(variable.kind, VariableKind::Integer | VariableKind::Binary);
        let col_idx = if is_integer {
            highs_model.add_integer_col(variable.lower, upper, obj_coeff)
        } else {
            highs_model.add_col(variable.lower, upper, obj_coeff)
        };
        columns.insert(variable.name.clone(), col_idx);
        trace!(
            component = "solver",
            operation = "add_variable",
            status = "success",
            variable = variable.name.as_str(),
            col_idx,
            lower = variable.lower,
            upper,
            obj_coeff,
            is_integer,
            "Added variable to HiGHS"
        );
    }

    columns
}

fn constraint_bounds(sense: ConstraintSense, rhs: f64) -> (f64, f64) {
    match sense {
        ConstraintSense::GreaterEqual => (rhs, f64::INFINITY),
        ConstraintSense::LessEqual => (f64::NEG_INFINITY, rhs),
        ConstraintSense::Equal => (rhs, rhs),
    }
}

fn add_constraints_to_highs(
    problem: &AlgebraicProblem,
    highs_model: &mut HighsModel,
    columns: &BTreeMap<String, usize>,
) -> Result<(), SolverError> {
    for constraint in &problem.constraints {
        let (lower, upper) = constraint_bounds(constraint.sense, constraint.rhs);
        let mut col_indices = Vec::with_capacity(constraint.terms.len());
        let mut coefficients = Vec::with_capacity(constraint.terms.len());
        for term in &constraint.terms {
            let Some(&col_idx) = columns.get(&term.variable_name) else {
                return Err(SolverError::InvalidVariableId(0));
            };
            col_indices.push(col_idx);
            coefficients.push(term.coefficient);
        }
        highs_model
            .add_row(lower, upper, &col_indices, &coefficients)
            .map_err(highs_model_error_to_solver_error)?;
    }
    Ok(())
}

/// Solve a primitive model view directly with HiGHS.
pub fn solve_model_view(
    model: &impl ModelView,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }

    let mut highs_model = HighsModel::new();
    apply_solver_config(&mut highs_model, config)?;
    highs_model.set_objective_sense(match model.objective().sense.unwrap_or(Sense::Minimize) {
        Sense::Minimize => ObjectiveSense::Minimize,
        Sense::Maximize => ObjectiveSense::Maximize,
    });

    let objective_terms = model
        .objective()
        .terms
        .iter()
        .copied()
        .collect::<BTreeMap<VariableId, f64>>();
    for index in 0..model.num_variables() {
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(index as u32))?;
        let objective = objective_terms.get(&variable_id).copied().unwrap_or(0.0);
        if variable.is_integer {
            highs_model.add_integer_col(variable.bounds.lower, variable.bounds.upper, objective);
        } else {
            highs_model.add_col(variable.bounds.lower, variable.bounds.upper, objective);
        }
    }

    let mut rows: Vec<(Vec<usize>, Vec<f64>)> =
        vec![(Vec::new(), Vec::new()); model.num_constraints()];
    for index in 0..model.num_variables() {
        let variable_id = VariableId::new(index as u32);
        let Some(column) = model.column(variable_id) else {
            continue;
        };
        for (constraint_id, coefficient) in column {
            let row_index = constraint_id.inner() as usize;
            if row_index >= rows.len() {
                return Err(SolverError::SolverSpecific(format!(
                    "constraint ID {row_index} does not exist"
                )));
            }
            rows[row_index].0.push(index);
            rows[row_index].1.push(*coefficient);
        }
    }

    for (index, (columns, coefficients)) in rows.iter().enumerate() {
        let constraint = model
            .constraint(ConstraintId::new(index as u32))
            .ok_or_else(|| {
                SolverError::SolverSpecific(format!("constraint ID {index} does not exist"))
            })?;
        highs_model
            .add_row(
                constraint.bounds.lower,
                constraint.bounds.upper,
                columns,
                coefficients,
            )
            .map_err(highs_model_error_to_solver_error)?;
    }

    let status = highs_model.solve();
    if !highs_has_solution(status) {
        return Err(SolverError::SolveFailure {
            status: highs_to_core_status(status),
        });
    }
    let snapshot = highs_model
        .solution_snapshot()
        .map_err(highs_model_error_to_solver_error)?;
    let objective_value = highs_model
        .objective_value()
        .map_err(highs_model_error_to_solver_error)?;
    let (primal_values, _, _, _) = snapshot.into_vecs();

    Ok(ModelViewSolveResult {
        fingerprint: model.fingerprint(),
        status: highs_to_core_status(status),
        objective_value,
        primal_values,
    })
}

fn solve_problem(
    problem: &AlgebraicProblem,
    config: &SolverConfig,
) -> Result<Solution, SolverError> {
    validate_problem(problem)?;

    let solver_version = crate::ffi::highs_version().unwrap_or_else(|| "unknown".to_string());
    let rss_before = capture_rss_bytes("solve_start");
    let solve_started = Instant::now();

    debug!(
        component = "solver",
        operation = "solve",
        status = "success",
        solver = "highs",
        solver_version = %solver_version,
        rss_bytes = ?rss_before,
        "Starting solve process"
    );

    let mut highs_model = HighsModel::new();
    apply_solver_config(&mut highs_model, config)?;
    highs_model.set_objective_sense(problem.objective.sense);

    let columns = add_variables_to_highs(problem, &mut highs_model);
    add_constraints_to_highs(problem, &mut highs_model, &columns)?;

    let status = highs_model.solve();
    let solve_ms = solve_started.elapsed().as_secs_f64() * 1000.0;
    let rss_after = capture_rss_bytes("solve_end");
    let rss_delta = match (rss_before, rss_after) {
        (Some(before), Some(after)) => Some(after as i64 - before as i64),
        _ => None,
    };
    let simplex_iterations = highs_model.simplex_iteration_count();
    let barrier_iterations = highs_model.barrier_iteration_count();
    let optimality_gap = highs_model.mip_gap();
    let objective_value_log = highs_model.objective_value().unwrap_or(f64::NAN);

    debug!(
        component = "solver",
        operation = "solve",
        status = "success",
        solver = "highs",
        solver_version = %solver_version,
        solver_status = ?status,
        simplex_iterations,
        barrier_iterations,
        total_iterations = simplex_iterations + barrier_iterations,
        objective_value = objective_value_log,
        optimality_gap,
        duration_ms = solve_ms,
        rss_bytes = ?rss_after,
        rss_delta_bytes = ?rss_delta,
        "HiGHS solve completed"
    );

    if !highs_has_solution(status) {
        warn!(
            component = "solver",
            operation = "solve",
            status = "warn",
            solver = "highs",
            solver_version = %solver_version,
            solver_status = ?status,
            "Solver did not find a feasible solution"
        );
        return Err(SolverError::SolveFailure {
            status: highs_to_core_status(status),
        });
    }

    if status != HighsStatus::Optimal {
        warn!(
            component = "solver",
            operation = "solve",
            status = "warn",
            solver = "highs",
            solver_version = %solver_version,
            solver_status = ?status,
            "Solver hit limit but returning best solution found"
        );
    }

    let snapshot = highs_model
        .solution_snapshot()
        .map_err(highs_model_error_to_solver_error)?;
    let objective_value = highs_model
        .objective_value()
        .map_err(highs_model_error_to_solver_error)?;
    let (primal_values, variable_duals, row_values, constraint_duals) = snapshot.into_vecs();

    Ok(Solution {
        primal_values,
        variable_duals,
        constraint_duals,
        row_values,
        objective_value,
        status,
        solve_time_seconds: solve_started.elapsed().as_secs_f64(),
        simplex_iterations,
        barrier_iterations,
        mip_gap: highs_model.mip_gap(),
        primal_feasibility_tolerance: highs_model.primal_feasibility_tolerance(),
        dual_feasibility_tolerance: highs_model.dual_feasibility_tolerance(),
        presolved_rows: highs_model.presolved_num_rows().unwrap_or(0),
        presolved_cols: highs_model.presolved_num_cols().unwrap_or(0),
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_model::{Bounds, Constraint, Model, ModelView, Objective, Sense, Variable};
    use arco_targets::{
        LinearConstraint, LinearObjective, LinearTerm, ObjectiveSense, VariableInstance,
    };

    fn fixture_problem() -> AlgebraicProblem {
        AlgebraicProblem {
            variable_instances: vec![VariableInstance {
                name: "x".to_string(),
                family: "x".to_string(),
                lower: 0.0,
                upper: None,
                kind: VariableKind::Continuous,
            }],
            constraints: vec![LinearConstraint {
                name: "demand".to_string(),
                sense: ConstraintSense::GreaterEqual,
                rhs: 1.0,
                terms: vec![LinearTerm {
                    variable_name: "x".to_string(),
                    coefficient: 1.0,
                }],
            }],
            objective: LinearObjective {
                name: "cost".to_string(),
                sense: ObjectiveSense::Minimize,
                constant: 0.0,
                terms: vec![LinearTerm {
                    variable_name: "x".to_string(),
                    coefficient: 2.0,
                }],
            },
            reports: Vec::new(),
        }
    }

    #[test]
    fn solver_rejects_empty_problem() {
        let mut problem = fixture_problem();
        problem.variable_instances.clear();
        assert!(matches!(Solver::new(problem), Err(SolverError::EmptyModel)));
    }

    #[test]
    fn target_problem_solves() {
        let mut solver = Solver::new(fixture_problem()).expect("solver initializes");
        let solution = solver.solve().expect("solve succeeds");
        assert!(solution.is_feasible());
        assert_eq!(solution.get_primal(0), Some(1.0));
        assert_eq!(solution.objective_value(), 2.0);
    }

    #[test]
    fn model_view_problem_solves_directly() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("variable");
        let demand = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        model.set_coefficient(x, demand, 1.0).expect("coefficient");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 2.0)],
            })
            .expect("objective");

        let result = solve_model_view(&model, &SolverConfig::new()).expect("solve succeeds");
        assert!(result.status.is_feasible());
        assert_eq!(result.primal_values, vec![1.0]);
        assert_eq!(result.objective_value, 2.0);
        assert_eq!(result.fingerprint, model.fingerprint());
    }
}
