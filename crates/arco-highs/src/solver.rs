//! HiGHS solver implementation.

use crate::async_matrix::{AsyncCrsBuilder, ConstraintEntries};
use crate::ffi::{HighsModel, HighsModelError, HighsOption, HighsStatus, ObjectiveSense};
use crate::solution::Solution;
use crate::status::{core_to_generic_status, highs_has_solution, highs_to_core_status};
use arco_core::solver::SolverError as CoreSolverError;
use arco_core::{Model, Sense};
use arco_expr::{ConstraintId, VariableId};
use arco_solver::{Solve, SolverConfig, SolverError as GenericSolverError};
use arco_tools::memory::capture_rss_bytes;
use std::collections::BTreeMap;
use std::time::Instant;
use tracing::{debug, trace, warn};

/// Re-export of arco_core::SolverError for backward compatibility.
pub type SolverError = CoreSolverError;

/// Convert a HighsModelError into a SolverError.
fn highs_model_error_to_solver_error(err: HighsModelError) -> SolverError {
    SolverError::SolverSpecific(err.to_string())
}

/// Convert a arco_core::SolverError to a arco_solver::SolverError.
fn core_error_to_generic(err: CoreSolverError) -> GenericSolverError {
    match err {
        CoreSolverError::EmptyModel => GenericSolverError::EmptyModel,
        CoreSolverError::NoObjective => GenericSolverError::NoObjective,
        CoreSolverError::InvalidObjectiveSense => GenericSolverError::InvalidObjectiveSense,
        CoreSolverError::InvalidVariableId(id) => GenericSolverError::InvalidVariableId(id),
        CoreSolverError::SolverNotAvailable(msg) => GenericSolverError::InternalError(msg),
        CoreSolverError::SolverSpecific(msg) => GenericSolverError::InternalError(msg),
        CoreSolverError::SolveFailure { status } => GenericSolverError::SolveFailure {
            status: core_to_generic_status(status),
        },
    }
}

/// Zero-copy bridge from arco-core::Model to HiGHS
pub struct Solver {
    model: Model,
    config: SolverConfig,
    /// Warm-start primal hints (stored but not yet forwarded to HiGHS)
    primal_start: Option<Vec<(VariableId, f64)>>,
    /// Use async CRS matrix building
    use_async_crs: bool,
}

impl Solver {
    /// Create a new solver from a Model.
    pub fn new(model: Model) -> Result<Self, SolverError> {
        validate_model(&model)?;

        debug!(
            component = "solver",
            operation = "init",
            status = "success",
            variables = model.num_variables() as u64,
            constraints = model.num_constraints() as u64,
            nnz = model.num_coefficients() as u64,
            "Creating solver from model"
        );

        Ok(Solver {
            model,
            config: SolverConfig::new(),
            primal_start: None,
            use_async_crs: false,
        })
    }

    fn update_config(&mut self, update: impl FnOnce(SolverConfig) -> SolverConfig) {
        self.config = update(std::mem::take(&mut self.config));
    }

    /// Enable or disable HiGHS logging to console for the next solve.
    pub fn set_log_to_console(&mut self, enabled: bool) {
        self.update_config(|config| config.with_log_to_console(enabled));
    }

    /// Set a time limit in seconds for the next solve.
    pub fn set_time_limit(&mut self, seconds: f64) {
        self.update_config(|config| config.with_time_limit(seconds));
    }

    /// Set a relative MIP gap for the next solve.
    pub fn set_mip_gap(&mut self, gap: f64) {
        self.update_config(|config| config.with_mip_gap(gap));
    }

    /// Set verbosity level for the next solve.
    pub fn set_verbosity(&mut self, level: u32) {
        self.update_config(|config| config.with_verbosity(level));
    }

    /// Enable or disable presolve for the next solve.
    pub fn set_presolve(&mut self, enabled: bool) {
        self.update_config(|config| config.with_presolve(enabled));
    }

    /// Set thread count for the next solve.
    pub fn set_threads(&mut self, threads: u32) {
        self.update_config(|config| config.with_threads(threads));
    }

    /// Set feasibility tolerance for the next solve.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.update_config(|config| config.with_tolerance(tolerance));
    }

    /// Set primal start values (warm-start hints).
    ///
    /// Hints are forwarded to HiGHS as an initial solution.
    pub fn set_primal_start(&mut self, hints: &[(VariableId, f64)]) -> Result<(), SolverError> {
        // Validate that all variable IDs exist in model
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

    /// Enable async CRS matrix building for the next solve.
    ///
    /// When enabled, the solver uses an async-aware partitioned approach to build
    /// the coefficient matrix, which can improve performance on large models.
    pub fn set_async_crs(&mut self, enabled: bool) {
        self.use_async_crs = enabled;
        if enabled {
            debug!(
                component = "solver",
                operation = "config",
                status = "success",
                feature = "async_crs",
                "Enabled async CRS matrix building"
            );
        }
    }

    /// Get access to the current solver configuration.
    pub fn config(&self) -> &SolverConfig {
        &self.config
    }

    /// Set the solver configuration.
    pub fn set_config(&mut self, config: SolverConfig) {
        self.config = config;
    }

    /// Solve the model and return the solution
    pub fn solve(&mut self) -> Result<Solution, SolverError> {
        self.solve_with_config(&self.config.clone())
    }

    /// Solve the model with a specific configuration
    pub fn solve_with_config(&mut self, config: &SolverConfig) -> Result<Solution, SolverError> {
        solve_model(
            &self.model,
            config,
            self.primal_start.as_deref(),
            self.use_async_crs,
        )
    }
}

// Implement the arco_core::Solver trait
impl arco_core::solver::Solver for Solver {
    fn solve(&mut self, model: &Model) -> Result<arco_core::solver::Solution, CoreSolverError> {
        // Build a temporary solver that borrows the model for solving
        // We use our own config and primal_start
        let highs_solution = solve_model(
            model,
            &self.config,
            self.primal_start.as_deref(),
            self.use_async_crs,
        )?;
        Ok(highs_solution.into_core_solution())
    }
}

// Implement the Solve trait from arco-solver
impl Solve for Solver {
    type Solution = Solution;

    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, GenericSolverError> {
        self.solve_with_config(config)
            .map_err(core_error_to_generic)
    }
}

/// Validate that a model is ready for solving.
fn validate_model(model: &Model) -> Result<(), SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    Ok(())
}

fn collect_objective_coefficients(
    model: &Model,
) -> Result<(Sense, BTreeMap<VariableId, f64>), SolverError> {
    let objective = model.objective();
    let Some(sense) = objective.sense else {
        return Err(SolverError::NoObjective);
    };

    let mut objective_coeffs: BTreeMap<VariableId, f64> = BTreeMap::new();
    for (var_id, coeff) in &objective.terms {
        let var = model
            .get_variable(*var_id)
            .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;
        if !var.is_active {
            continue;
        }
        *objective_coeffs.entry(*var_id).or_insert(0.0) += *coeff;
    }

    Ok((sense, objective_coeffs))
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
        let presolve_str = if presolve { "on" } else { "off" };
        highs_model.set_option("presolve", HighsOption::Str(presolve_str.to_string()));
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

fn add_variables_to_highs(
    model: &Model,
    highs_model: &mut HighsModel,
    objective_coeffs: &BTreeMap<VariableId, f64>,
) -> BTreeMap<VariableId, usize> {
    let mut var_id_to_col = BTreeMap::new();

    for index in 0..model.num_variables() {
        let var_id = VariableId::new(index as u32);

        if let Ok(var) = model.get_variable(var_id) {
            let obj_coeff = objective_coeffs.get(&var_id).copied().unwrap_or(0.0);

            let (lower, upper, objective_coeff) = if var.is_active {
                (var.bounds.lower, var.bounds.upper, obj_coeff)
            } else {
                (0.0, 0.0, 0.0)
            };

            let col_idx = if var.is_integer {
                highs_model.add_integer_col(lower, upper, objective_coeff)
            } else {
                highs_model.add_col(lower, upper, objective_coeff)
            };
            var_id_to_col.insert(var_id, col_idx);

            trace!(
                component = "solver",
                operation = "add_variable",
                status = "success",
                var_id = var_id.inner(),
                col_idx,
                lower = var.bounds.lower,
                upper = var.bounds.upper,
                obj_coeff,
                is_integer = var.is_integer,
                "Added variable to HiGHS"
            );
        }
    }

    debug!(
        component = "solver",
        operation = "add_variables",
        status = "success",
        num_vars = model.num_variables(),
        "Added all variables to HiGHS"
    );

    var_id_to_col
}

fn prepare_warm_start_columns(
    model: &Model,
    var_id_to_col: &BTreeMap<VariableId, usize>,
    hints: &[(VariableId, f64)],
) -> Result<Vec<f64>, SolverError> {
    let mut cols = Vec::with_capacity(model.num_variables());
    for index in 0..model.num_variables() {
        let var_id = VariableId::new(index as u32);
        let var = model
            .get_variable(var_id)
            .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;
        cols.push(default_primal_value(var.bounds.lower, var.bounds.upper));
    }

    for (var_id, value) in hints {
        let Some(&col_idx) = var_id_to_col.get(var_id) else {
            return Err(SolverError::InvalidVariableId(var_id.inner()));
        };
        if let Some(slot) = cols.get_mut(col_idx) {
            *slot = *value;
        }
    }

    debug!(
        component = "solver",
        operation = "prepare_warm_start",
        status = "success",
        num_hints = hints.len(),
        "Prepared warm-start solution"
    );

    Ok(cols)
}

fn build_constraint_entries(
    model: &Model,
    var_id_to_col: &BTreeMap<VariableId, usize>,
    use_async_crs: bool,
) -> ConstraintEntries {
    if use_async_crs {
        let builder = AsyncCrsBuilder::new();
        let result = builder.build_blocking(model, var_id_to_col);

        debug!(
            component = "solver",
            operation = "build_rows",
            status = "success",
            method = "async",
            num_constraints = result.constraint_entries.len(),
            duration_ms = result.duration_ms,
            "Built constraint matrix asynchronously"
        );

        result.constraint_entries
    } else {
        build_constraint_entries_sequential(model, var_id_to_col)
    }
}

fn build_constraint_entries_sequential(
    model: &Model,
    var_id_to_col: &BTreeMap<VariableId, usize>,
) -> ConstraintEntries {
    let matrix_build_started = Instant::now();
    let mut constraint_entries: ConstraintEntries = BTreeMap::new();

    for (var_id, column) in model.columns() {
        let var = if let Ok(var) = model.get_variable(var_id) {
            var
        } else {
            warn!(
                component = "solver",
                operation = "build_rows",
                status = "warn",
                var_id = var_id.inner(),
                "Variable missing from model; skipping coefficients"
            );
            continue;
        };
        if !var.is_active {
            continue;
        }

        let Some(&col_idx) = var_id_to_col.get(&var_id) else {
            warn!(
                component = "solver",
                operation = "build_rows",
                status = "warn",
                var_id = var_id.inner(),
                "Variable missing HiGHS column index; skipping coefficients"
            );
            continue;
        };

        for (constraint_id, coeff) in column {
            let entry = constraint_entries
                .entry(*constraint_id)
                .or_insert_with(|| (Vec::new(), Vec::new()));
            entry.0.push(col_idx);
            entry.1.push(*coeff);
        }
    }

    let duration_ms = matrix_build_started.elapsed().as_secs_f64() * 1000.0;
    debug!(
        component = "solver",
        operation = "build_rows",
        status = "success",
        method = "sequential",
        num_constraints = constraint_entries.len(),
        duration_ms = duration_ms,
        "Built constraint matrix sequentially"
    );

    constraint_entries
}

fn add_constraints_to_highs(
    model: &Model,
    highs_model: &mut HighsModel,
    constraint_entries: &mut ConstraintEntries,
) -> Result<(), SolverError> {
    for index in 0..model.num_constraints() {
        let constraint_id = ConstraintId::new(index as u32);

        if let Ok(constraint) = model.get_constraint(constraint_id) {
            let (col_indices, coefficients) = constraint_entries
                .remove(&constraint_id)
                .unwrap_or_else(|| (Vec::new(), Vec::new()));

            highs_model
                .add_row(
                    constraint.bounds.lower,
                    constraint.bounds.upper,
                    &col_indices,
                    &coefficients,
                )
                .map_err(highs_model_error_to_solver_error)?;

            trace!(
                component = "solver",
                operation = "add_constraint",
                status = "success",
                constraint_id = constraint_id.inner(),
                lower = constraint.bounds.lower,
                upper = constraint.bounds.upper,
                num_coeffs = col_indices.len(),
                "Added constraint to HiGHS"
            );
        }
    }

    debug!(
        component = "solver",
        operation = "add_constraints",
        status = "success",
        num_constraints = model.num_constraints(),
        "Added all constraints to HiGHS"
    );

    Ok(())
}

/// Solve a model with the given config. This is the shared implementation used by
/// both the `arco_core::Solver` trait and the internal `solve_with_config` method.
fn solve_model(
    model: &Model,
    config: &SolverConfig,
    primal_start: Option<&[(VariableId, f64)]>,
    use_async_crs: bool,
) -> Result<Solution, SolverError> {
    validate_model(model)?;

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

    let (sense, objective_coeffs) = collect_objective_coefficients(model)?;

    // Create HiGHS model
    let mut highs_model = HighsModel::new();
    apply_solver_config(&mut highs_model, config)?;
    trace!(
        component = "solver",
        operation = "init_highs",
        status = "success",
        "Created HiGHS model"
    );

    // Set objective sense
    let highs_sense = match sense {
        Sense::Minimize => ObjectiveSense::Minimize,
        Sense::Maximize => ObjectiveSense::Maximize,
    };
    highs_model.set_objective_sense(highs_sense);
    debug!(
        component = "solver",
        operation = "set_objective_sense",
        status = "success",
        sense = ?sense,
        "Set objective sense"
    );

    let var_id_to_col = add_variables_to_highs(model, &mut highs_model, &objective_coeffs);

    let warm_start_cols = primal_start
        .map(|hints| prepare_warm_start_columns(model, &var_id_to_col, hints))
        .transpose()?;

    let mut constraint_entries = build_constraint_entries(model, &var_id_to_col, use_async_crs);
    add_constraints_to_highs(model, &mut highs_model, &mut constraint_entries)?;

    if let Some(cols) = warm_start_cols {
        highs_model
            .set_primal_start(cols)
            .map_err(highs_model_error_to_solver_error)?;
    }

    // Solve
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
    let heap_bytes: Option<u64> = None;

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
        heap_bytes = ?heap_bytes,
        "HiGHS solve completed"
    );

    // Check status - allow time/iteration limits as they may have feasible solutions
    let is_acceptable = highs_has_solution(status);

    if !is_acceptable {
        warn!(
            component = "solver",
            operation = "solve",
            status = "warn",
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
            heap_bytes = ?heap_bytes,
            "Solver did not find optimal solution"
        );
        return Err(SolverError::SolveFailure {
            status: highs_to_core_status(status),
        });
    }

    // Log if we hit a limit but still have a solution
    if status != HighsStatus::Optimal {
        warn!(
            component = "solver",
            operation = "solve",
            status = "warn",
            solver = "highs",
            solver_version = %solver_version,
            solver_status = ?status,
            simplex_iterations,
            barrier_iterations,
            total_iterations = simplex_iterations + barrier_iterations,
            objective_value = objective_value_log,
            optimality_gap,
            duration_ms = solve_ms,
            "Solver hit limit but returning best solution found"
        );
    }

    // Extract solution
    let snapshot = highs_model
        .solution_snapshot()
        .map_err(highs_model_error_to_solver_error)?;
    let objective_value = highs_model
        .objective_value()
        .map_err(highs_model_error_to_solver_error)?;
    let primal_values = snapshot.col_values().to_vec();
    let variable_duals = snapshot.col_duals().to_vec();
    let constraint_duals = snapshot.row_duals().to_vec();
    let row_values = snapshot.row_values().to_vec();

    // Extract additional solution metadata
    let mip_gap = highs_model.mip_gap();
    let primal_feasibility_tolerance = highs_model.primal_feasibility_tolerance();
    let dual_feasibility_tolerance = highs_model.dual_feasibility_tolerance();
    let presolved_rows = highs_model.presolved_num_rows();
    let presolved_cols = highs_model.presolved_num_cols();

    debug!(
        component = "solver",
        operation = "extract_solution",
        status = "success",
        objective_value,
        num_primal_values = primal_values.len(),
        num_variable_duals = variable_duals.len(),
        num_constraint_duals = constraint_duals.len(),
        mip_gap,
        presolved_rows,
        presolved_cols,
        "Solution extracted"
    );

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
        mip_gap,
        primal_feasibility_tolerance,
        dual_feasibility_tolerance,
        presolved_rows,
        presolved_cols,
    })
}

fn default_primal_value(lower: f64, upper: f64) -> f64 {
    if lower.is_finite() && upper.is_finite() {
        if lower <= 0.0 && 0.0 <= upper {
            0.0
        } else if 0.0 < lower {
            lower
        } else {
            upper
        }
    } else if lower.is_finite() {
        if 0.0 < lower { lower } else { 0.0 }
    } else if upper.is_finite() {
        if 0.0 > upper { upper } else { 0.0 }
    } else {
        0.0
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_core::solver::SolverStatus as CoreSolverStatus;
    use arco_core::types::Bounds;
    use arco_core::{Objective, Variable};
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn test_solver_error_display_empty_model() {
        let err = SolverError::EmptyModel;
        assert!(err.to_string().contains("no variables"));
    }

    #[test]
    fn test_solver_error_display_no_objective() {
        let err = SolverError::NoObjective;
        assert!(err.to_string().contains("no objective"));
    }

    #[test]
    fn test_solver_error_display_solve_failure() {
        let err = SolverError::SolveFailure {
            status: CoreSolverStatus::Infeasible,
        };
        assert!(err.to_string().contains("infeasible"));

        let err = SolverError::SolveFailure {
            status: CoreSolverStatus::Unbounded,
        };
        assert!(err.to_string().contains("unbounded"));
    }

    #[test]
    fn test_solver_new_rejects_empty_model() {
        let model = Model::new();
        let result = Solver::new(model);
        assert!(result.is_err());
        if let Err(SolverError::EmptyModel) = result {
            // Expected
        } else {
            panic!("Expected EmptyModel error");
        }
    }

    #[test]
    fn test_default_primal_value() {
        // Bounded with 0 in range
        assert_eq!(default_primal_value(-10.0, 10.0), 0.0);
        assert_eq!(default_primal_value(0.0, 10.0), 0.0);
        assert_eq!(default_primal_value(-10.0, 0.0), 0.0);

        // Bounded positive
        assert_eq!(default_primal_value(1.0, 10.0), 1.0);

        // Bounded negative
        assert_eq!(default_primal_value(-10.0, -1.0), -1.0);

        // Lower bound only
        assert_eq!(default_primal_value(5.0, f64::INFINITY), 5.0);
        assert_eq!(default_primal_value(-5.0, f64::INFINITY), 0.0);

        // Upper bound only
        assert_eq!(default_primal_value(f64::NEG_INFINITY, -5.0), -5.0);
        assert_eq!(default_primal_value(f64::NEG_INFINITY, 5.0), 0.0);

        // Unbounded
        assert_eq!(default_primal_value(f64::NEG_INFINITY, f64::INFINITY), 0.0);
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
    fn test_solve_with_negative_time_limit_does_not_panic() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).expect("solver");
        let config = SolverConfig::new().with_time_limit(-1.0);

        let outcome = catch_unwind(AssertUnwindSafe(|| solver.solve_with_config(&config)));
        assert!(outcome.is_ok(), "solve_with_config must not panic");
        let result = outcome.expect("caught panic");
        assert!(result.is_err(), "invalid time limit should return error");
    }

    #[test]
    fn test_solve_with_negative_mip_gap_does_not_panic() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).expect("solver");
        let config = SolverConfig::new().with_mip_gap(-0.1);

        let outcome = catch_unwind(AssertUnwindSafe(|| solver.solve_with_config(&config)));
        assert!(outcome.is_ok(), "solve_with_config must not panic");
        let result = outcome.expect("caught panic");
        assert!(result.is_err(), "invalid mip gap should return error");
    }

    #[test]
    fn test_solve_with_negative_tolerance_does_not_panic() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).expect("solver");
        let config = SolverConfig::new().with_tolerance(-1e-9);

        let outcome = catch_unwind(AssertUnwindSafe(|| solver.solve_with_config(&config)));
        assert!(outcome.is_ok(), "solve_with_config must not panic");
        let result = outcome.expect("caught panic");
        assert!(result.is_err(), "invalid tolerance should return error");
    }

    #[test]
    fn test_solve_with_zero_threads_does_not_panic() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).expect("solver");
        let config = SolverConfig::new().with_threads(0);

        let outcome = catch_unwind(AssertUnwindSafe(|| solver.solve_with_config(&config)));
        assert!(outcome.is_ok(), "solve_with_config must not panic");
        let result = outcome.expect("caught panic");
        assert!(result.is_err(), "zero threads should return error");
    }
}
