//! HiGHS solver implementation over solver-facing targets.

use arco_model::{ConstraintId, ModelFingerprint, ModelView, Sense, VariableId};
use arco_solver::{
    LpAlgorithm, ModelViewBackend, ModelViewSolveResult, SolverConfig, SolverStatus,
    validate_model_view_solve_result, validate_model_view_solve_result_shape,
};
use std::collections::BTreeMap;
use std::ffi::{CString, c_void};
use std::time::Instant;

use crate::sys as highs_sys;

/// Re-export of contract solver error for backward compatibility.
pub type SolverError = arco_solver::SolverError;

fn validate_solver_config(config: &SolverConfig) -> Result<(), SolverError> {
    if let Some(limit) = config.time_limit {
        if !limit.is_finite() || limit < 0.0 {
            return Err(SolverError::InvalidSettings(
                "time_limit must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(gap) = config.mip_gap {
        if !gap.is_finite() || gap < 0.0 {
            return Err(SolverError::InvalidSettings(
                "mip_gap must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(tolerance) = config.tolerance {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(SolverError::InvalidSettings(
                "tolerance must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(threads) = config.threads {
        if threads == 0 {
            return Err(SolverError::InvalidSettings(
                "threads must be >= 1".to_string(),
            ));
        }
    }
    Ok(())
}

/// Adapter implementation for primitive model-view solves through HiGHS.
#[derive(Debug, Default, Clone, Copy)]
pub struct HighsModelViewBackend;

impl ModelViewBackend for HighsModelViewBackend {
    fn family(&self) -> &'static str {
        "highs"
    }

    fn solve_model_view(
        &self,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        solve_model_view(model, config)
    }
}

/// Solve a primitive model view directly with HiGHS.
pub fn solve_model_view(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let prepared = PreparedHighsModel::prepare(model, config)?;
    let result = prepared.solve()?;
    validate_model_view_solve_result(model, &result)?;
    Ok(result)
}

/// An owned HiGHS problem whose native state no longer borrows the source model.
pub struct PreparedHighsModel {
    highs_model: DirectHighsModel,
    fingerprint: ModelFingerprint,
    extract_solution: bool,
    num_variables: usize,
    num_constraints: usize,
    num_coefficients: usize,
    matrix_build_seconds: f64,
    preparation_seconds: f64,
    fingerprint_seconds: f64,
}

impl PreparedHighsModel {
    /// Build an owned native HiGHS problem from a model view.
    pub fn prepare(
        model: &(impl ModelView + ?Sized),
        config: &SolverConfig,
    ) -> Result<Self, SolverError> {
        if model.num_variables() == 0 {
            return Err(SolverError::EmptyModel);
        }
        if model.objective().sense.is_none() && model.objective().terms.is_empty() {
            return Err(SolverError::NoObjective);
        }
        validate_solver_config(config)?;
        let _requested_load_path = requested_load_path(config)?;
        let prepare_start = Instant::now();
        let (fingerprint, fingerprint_seconds) = if config
            .parameters
            .get("arco.fingerprint")
            .is_none_or(|value| value != "false")
        {
            let fingerprint_start = Instant::now();
            let fingerprint = model.fingerprint();
            (fingerprint, fingerprint_start.elapsed().as_secs_f64())
        } else {
            (ModelFingerprint(0), 0.0)
        };
        let num_variables = model.num_variables();
        let num_constraints = model.num_constraints();
        let num_coefficients = model.num_coefficients();
        let sense = match model.objective().sense.unwrap_or(Sense::Minimize) {
            Sense::Minimize => highs_sys::OBJECTIVE_SENSE_MINIMIZE,
            Sense::Maximize => highs_sys::OBJECTIVE_SENSE_MAXIMIZE,
        };
        let objective_coefficients = objective_coefficients(model);
        let matrix_start = Instant::now();
        let load_data = build_direct_highs_load_data(model, objective_coefficients)?;
        let mut highs_model = DirectHighsModel::load(load_data, sense)?;
        apply_direct_solver_config(&mut highs_model, config)?;
        let matrix_build_seconds = matrix_start.elapsed().as_secs_f64();
        let extract_solution = config
            .parameters
            .get("arco.extract_solution")
            .is_none_or(|value| value != "false");

        Ok(Self {
            highs_model,
            fingerprint,
            extract_solution,
            num_variables,
            num_constraints,
            num_coefficients,
            matrix_build_seconds,
            preparation_seconds: prepare_start.elapsed().as_secs_f64(),
            fingerprint_seconds,
        })
    }

    /// Return the fingerprint captured while the source model was borrowed.
    pub fn fingerprint(&self) -> ModelFingerprint {
        self.fingerprint
    }

    /// Optimize the prepared native problem without retaining the source model.
    pub fn solve(self) -> Result<ModelViewSolveResult, SolverError> {
        let Self {
            mut highs_model,
            fingerprint,
            extract_solution,
            num_variables,
            num_constraints,
            num_coefficients,
            matrix_build_seconds,
            preparation_seconds,
            fingerprint_seconds,
        } = self;

        let highs_run_start = Instant::now();
        let model_status = highs_model.solve()?;
        let highs_run_seconds = highs_run_start.elapsed().as_secs_f64();
        let mapped_status = raw_highs_model_status_to_solver_status(model_status);
        let highs_model_status = model_status as f64;
        let highs_primal_solution_status =
            highs_model.int_info_value("primal_solution_status")? as f64;
        let objective_value = highs_model.objective_value();
        let objective_value = objective_value_for_primal_solution_status(
            objective_value,
            highs_primal_solution_status,
        );
        let reported_status =
            status_with_primal_solution_availability(mapped_status, highs_primal_solution_status);
        if !reported_status.is_feasible() {
            return Err(SolverError::SolveFailure {
                status: reported_status,
            });
        }
        let solution_extract_start = Instant::now();
        let (primal_values, variable_duals, row_values, constraint_duals) = if extract_solution {
            highs_model.solution_vectors(num_variables, num_constraints)?
        } else {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        };
        let solution_extract_seconds = solution_extract_start.elapsed().as_secs_f64();

        let mut metadata = BTreeMap::new();
        metadata.insert("highs_matrix_build_s".to_string(), matrix_build_seconds);
        metadata.insert("highs_prepare_s".to_string(), preparation_seconds);
        metadata.insert("highs_direct_load_path".to_string(), 1.0);
        metadata.insert("highs_run_s".to_string(), highs_run_seconds);
        metadata.insert("solution_extract_s".to_string(), solution_extract_seconds);
        metadata.insert("fingerprint_s".to_string(), fingerprint_seconds);
        metadata.insert("highs_model_status".to_string(), highs_model_status);
        metadata.insert(
            "highs_primal_solution_status".to_string(),
            highs_primal_solution_status,
        );
        metadata.insert("num_variables".to_string(), num_variables as f64);
        metadata.insert("num_constraints".to_string(), num_constraints as f64);
        metadata.insert("num_coefficients".to_string(), num_coefficients as f64);

        let result = ModelViewSolveResult {
            fingerprint,
            status: reported_status,
            objective_value,
            primal_values,
            variable_duals,
            row_values,
            constraint_duals,
            metadata,
        };
        validate_model_view_solve_result_shape(&result, num_variables, num_constraints)?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HighsLoadPath {
    Wrapper,
    Direct,
}

struct DirectHighsLoadData {
    num_cols: highs_sys::HighsInt,
    num_rows: highs_sys::HighsInt,
    num_nonzeros: highs_sys::HighsInt,
    col_cost: Vec<f64>,
    col_lower: Vec<f64>,
    col_upper: Vec<f64>,
    row_lower: Vec<f64>,
    row_upper: Vec<f64>,
    a_start: Vec<highs_sys::HighsInt>,
    a_index: Vec<highs_sys::HighsInt>,
    a_value: Vec<f64>,
    integrality: Option<Vec<highs_sys::HighsInt>>,
}

struct DirectHighsModel {
    ptr: *mut c_void,
}

type SolutionVectors = (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>);

#[allow(unsafe_code)]
impl Drop for DirectHighsModel {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: `ptr` is created by `Highs_create`, owned exclusively by this
            // wrapper, and destroyed exactly once here.
            unsafe {
                highs_sys::Highs_destroy(self.ptr);
            }
        }
    }
}

fn requested_load_path(config: &SolverConfig) -> Result<HighsLoadPath, SolverError> {
    match config
        .parameters
        .get("arco.highs_load_path")
        .map(String::as_str)
    {
        None | Some("wrapper") => Ok(HighsLoadPath::Wrapper),
        Some("direct") => Ok(HighsLoadPath::Direct),
        Some(value) => Err(SolverError::InvalidSettings(format!(
            "unsupported arco.highs_load_path value {value:?}"
        ))),
    }
}

fn objective_coefficients(model: &(impl ModelView + ?Sized)) -> Vec<f64> {
    let mut coefficients = vec![0.0; model.num_variables()];
    for (variable_id, coefficient) in &model.objective().terms {
        let index = variable_id.inner() as usize;
        if index < coefficients.len() {
            coefficients[index] = *coefficient;
        }
    }
    coefficients
}

fn build_direct_highs_load_data(
    model: &(impl ModelView + ?Sized),
    objective_coefficients: Vec<f64>,
) -> Result<DirectHighsLoadData, SolverError> {
    let num_cols = checked_highs_int(model.num_variables(), "variables")?;
    let num_rows = checked_highs_int(model.num_constraints(), "constraints")?;
    let num_nonzeros = checked_highs_int(model.num_coefficients(), "coefficients")?;
    let ncols = model.num_variables();
    let nrows = model.num_constraints();
    let mut col_lower = Vec::with_capacity(ncols);
    let mut col_upper = Vec::with_capacity(ncols);
    let mut row_lower = Vec::with_capacity(nrows);
    let mut row_upper = Vec::with_capacity(nrows);
    let mut a_start = Vec::with_capacity(ncols);
    let mut a_index = Vec::with_capacity(model.num_coefficients());
    let mut a_value = Vec::with_capacity(model.num_coefficients());
    let mut integrality: Option<Vec<highs_sys::HighsInt>> = None;

    for index in 0..nrows {
        let constraint = model
            .constraint(ConstraintId::new(index as u32))
            .ok_or_else(|| {
                SolverError::SolverSpecific(format!("constraint ID {index} does not exist"))
            })?;
        row_lower.push(constraint.bounds.lower);
        row_upper.push(constraint.bounds.upper);
    }

    for index in 0..ncols {
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(index as u32))?;
        if variable.is_integer && integrality.is_none() {
            integrality = Some(vec![highs_sys::VAR_TYPE_CONTINUOUS; index]);
        }
        if let Some(integrality) = &mut integrality {
            integrality.push(if variable.is_integer {
                highs_sys::VAR_TYPE_INTEGER
            } else {
                highs_sys::VAR_TYPE_CONTINUOUS
            });
        }
        col_lower.push(variable.bounds.lower);
        col_upper.push(variable.bounds.upper);
        a_start.push(checked_highs_int(a_index.len(), "column start")?);
        if let Some(column) = model.column(variable_id) {
            for (constraint_id, coefficient) in column {
                let row_index = constraint_id.inner() as usize;
                if row_index >= nrows {
                    return Err(SolverError::SolverSpecific(format!(
                        "constraint ID {row_index} does not exist"
                    )));
                }
                a_index.push(checked_highs_int(row_index, "row index")?);
                a_value.push(*coefficient);
            }
        }
    }

    Ok(DirectHighsLoadData {
        num_cols,
        num_rows,
        num_nonzeros,
        col_cost: objective_coefficients,
        col_lower,
        col_upper,
        row_lower,
        row_upper,
        a_start,
        a_index,
        a_value,
        integrality,
    })
}

fn checked_highs_int(value: usize, name: &str) -> Result<highs_sys::HighsInt, SolverError> {
    highs_sys::HighsInt::try_from(value)
        .map_err(|_| SolverError::SolverSpecific(format!("{name} count is too large for HiGHS")))
}

#[allow(unsafe_code)]
impl DirectHighsModel {
    fn load(
        load_data: DirectHighsLoadData,
        sense: highs_sys::HighsInt,
    ) -> Result<Self, SolverError> {
        // SAFETY: `Highs_create` takes no arguments and returns either a valid
        // HiGHS handle or NULL, which is checked immediately.
        let ptr = unsafe { highs_sys::Highs_create() };
        if ptr.is_null() {
            return Err(SolverError::SolverNotAvailable(
                "HiGHS_create returned NULL".to_string(),
            ));
        }
        let mut model = Self { ptr };
        model.set_bool_option("output_flag", false)?;
        model.set_bool_option("log_to_console", false)?;
        // SAFETY: all slices passed to HiGHS are owned by `load_data` and stay
        // alive for the entire call. Dimensions are converted with
        // `checked_highs_int`, and `MATRIX_FORMAT_COLUMN_WISE` matches `a_start`
        // column offsets plus row indices.
        let status = unsafe {
            if let Some(integrality) = load_data.integrality.as_ref() {
                highs_sys::Highs_passMip(
                    model.ptr,
                    load_data.num_cols,
                    load_data.num_rows,
                    load_data.num_nonzeros,
                    highs_sys::MATRIX_FORMAT_COLUMN_WISE,
                    sense,
                    0.0,
                    load_data.col_cost.as_ptr(),
                    load_data.col_lower.as_ptr(),
                    load_data.col_upper.as_ptr(),
                    load_data.row_lower.as_ptr(),
                    load_data.row_upper.as_ptr(),
                    load_data.a_start.as_ptr(),
                    load_data.a_index.as_ptr(),
                    load_data.a_value.as_ptr(),
                    integrality.as_ptr(),
                )
            } else {
                highs_sys::Highs_passLp(
                    model.ptr,
                    load_data.num_cols,
                    load_data.num_rows,
                    load_data.num_nonzeros,
                    highs_sys::MATRIX_FORMAT_COLUMN_WISE,
                    sense,
                    0.0,
                    load_data.col_cost.as_ptr(),
                    load_data.col_lower.as_ptr(),
                    load_data.col_upper.as_ptr(),
                    load_data.row_lower.as_ptr(),
                    load_data.row_upper.as_ptr(),
                    load_data.a_start.as_ptr(),
                    load_data.a_index.as_ptr(),
                    load_data.a_value.as_ptr(),
                )
            }
        };
        ensure_highs_ok(status, "Highs_passLp/Highs_passMip")?;
        Ok(model)
    }

    fn solve(&mut self) -> Result<highs_sys::HighsInt, SolverError> {
        // SAFETY: `self.ptr` is a live HiGHS handle owned by this wrapper.
        ensure_highs_ok(unsafe { highs_sys::Highs_run(self.ptr) }, "Highs_run")?;
        // SAFETY: `Highs_run` completed successfully on this live handle.
        Ok(unsafe { highs_sys::Highs_getModelStatus(self.ptr) })
    }

    fn objective_value(&self) -> f64 {
        // SAFETY: `self.ptr` is a live HiGHS handle and objective value is a
        // scalar query that does not outlive the handle.
        unsafe { highs_sys::Highs_getObjectiveValue(self.ptr) }
    }

    fn int_info_value(&self, key: &str) -> Result<highs_sys::HighsInt, SolverError> {
        let key = cstring_highs_key(key)?;
        let mut value = -1;
        ensure_highs_ok(
            // SAFETY: `key` is a NUL-terminated CString and `value` points to
            // valid writable storage for the duration of the call.
            unsafe { highs_sys::Highs_getIntInfoValue(self.ptr, key.as_ptr(), &mut value) },
            key.to_string_lossy().as_ref(),
        )?;
        Ok(value)
    }

    fn solution_vectors(
        &self,
        num_variables: usize,
        num_constraints: usize,
    ) -> Result<SolutionVectors, SolverError> {
        let mut primal_values = vec![0.0; num_variables];
        let mut variable_duals = vec![0.0; num_variables];
        let mut row_values = vec![0.0; num_constraints];
        let mut constraint_duals = vec![0.0; num_constraints];
        ensure_highs_ok(
            // SAFETY: output vectors are sized to the model dimensions and their
            // buffers remain valid and mutable for the duration of the call.
            unsafe {
                highs_sys::Highs_getSolution(
                    self.ptr,
                    primal_values.as_mut_ptr(),
                    variable_duals.as_mut_ptr(),
                    row_values.as_mut_ptr(),
                    constraint_duals.as_mut_ptr(),
                )
            },
            "Highs_getSolution",
        )?;
        Ok((primal_values, variable_duals, row_values, constraint_duals))
    }

    fn set_bool_option(&mut self, key: &str, value: bool) -> Result<(), SolverError> {
        let key = cstring_option_key(key)?;
        ensure_highs_ok(
            // SAFETY: `key` is a NUL-terminated CString and the handle is live.
            unsafe {
                highs_sys::Highs_setBoolOptionValue(self.ptr, key.as_ptr(), i32::from(value))
            },
            key.to_string_lossy().as_ref(),
        )
    }

    fn set_int_option(&mut self, key: &str, value: i32) -> Result<(), SolverError> {
        let key = cstring_option_key(key)?;
        ensure_highs_ok(
            // SAFETY: `key` is a NUL-terminated CString and the handle is live.
            unsafe { highs_sys::Highs_setIntOptionValue(self.ptr, key.as_ptr(), value) },
            key.to_string_lossy().as_ref(),
        )
    }

    fn set_double_option(&mut self, key: &str, value: f64) -> Result<(), SolverError> {
        let key = cstring_option_key(key)?;
        ensure_highs_ok(
            // SAFETY: `key` is a NUL-terminated CString and the handle is live.
            unsafe { highs_sys::Highs_setDoubleOptionValue(self.ptr, key.as_ptr(), value) },
            key.to_string_lossy().as_ref(),
        )
    }

    fn set_string_option(&mut self, key: &str, value: &str) -> Result<(), SolverError> {
        let key = cstring_option_key(key)?;
        let value = CString::new(value).map_err(|_| {
            SolverError::InvalidSettings(format!(
                "invalid HiGHS option value for {:?}: contains NUL",
                key.to_string_lossy()
            ))
        })?;
        ensure_highs_ok(
            // SAFETY: `key` and `value` are NUL-terminated CStrings and both
            // remain alive for the duration of the call.
            unsafe {
                highs_sys::Highs_setStringOptionValue(self.ptr, key.as_ptr(), value.as_ptr())
            },
            key.to_string_lossy().as_ref(),
        )
    }
}

fn apply_direct_solver_config(
    highs_model: &mut DirectHighsModel,
    config: &SolverConfig,
) -> Result<(), SolverError> {
    validate_solver_config(config)?;

    if config.verbosity.unwrap_or(0) == 0 && !config.log_to_console.unwrap_or(false) {
        highs_model.set_bool_option("output_flag", false)?;
        highs_model.set_bool_option("log_to_console", false)?;
    }
    if let Some(level) = config.verbosity {
        highs_model.set_bool_option("output_flag", level > 0)?;
    }
    if config.log_to_console.unwrap_or(false) {
        highs_model.set_bool_option("log_to_console", true)?;
        highs_model.set_bool_option("output_flag", true)?;
    }
    if let Some(limit) = config.time_limit {
        highs_model.set_double_option("time_limit", limit)?;
    }
    if let Some(gap) = config.mip_gap {
        highs_model.set_double_option("mip_rel_gap", gap)?;
    }
    if let Some(presolve) = config.presolve {
        highs_model.set_string_option("presolve", if presolve { "on" } else { "off" })?;
    }
    if let Some(threads) = config.threads {
        highs_model.set_int_option("threads", threads as i32)?;
    }
    if let Some(tolerance) = config.tolerance {
        highs_model.set_double_option("primal_feasibility_tolerance", tolerance)?;
        highs_model.set_double_option("dual_feasibility_tolerance", tolerance)?;
    }
    if let Some(algorithm) = config.lp_algorithm {
        apply_lp_algorithm(highs_model, algorithm)?;
    }
    for (key, value) in &config.parameters {
        if key.starts_with("arco.") {
            continue;
        }
        highs_model.set_string_option(key.as_str(), value.as_str())?;
    }
    Ok(())
}

fn apply_lp_algorithm(
    highs_model: &mut DirectHighsModel,
    algorithm: LpAlgorithm,
) -> Result<(), SolverError> {
    match algorithm {
        LpAlgorithm::Automatic => highs_model.set_string_option("solver", "choose")?,
        LpAlgorithm::PrimalSimplex => {
            highs_model.set_string_option("solver", "simplex")?;
            highs_model.set_int_option("simplex_strategy", 4)?;
        }
        LpAlgorithm::DualSimplex => {
            highs_model.set_string_option("solver", "simplex")?;
            highs_model.set_int_option("simplex_strategy", 1)?;
        }
        LpAlgorithm::Barrier => {
            highs_model.set_string_option("solver", "ipm")?;
            highs_model.set_string_option("run_crossover", "off")?;
        }
        LpAlgorithm::BarrierWithCrossover => {
            highs_model.set_string_option("solver", "ipm")?;
            highs_model.set_string_option("run_crossover", "on")?;
        }
        LpAlgorithm::PrimalDualFirstOrder => {
            highs_model.set_string_option("solver", "pdlp")?;
        }
        LpAlgorithm::Concurrent => {
            return Err(SolverError::InvalidSettings(
                "lp_algorithm 'concurrent' is not supported by the HiGHS backend".to_string(),
            ));
        }
    }
    Ok(())
}

fn ensure_highs_ok(status: highs_sys::HighsInt, operation: &str) -> Result<(), SolverError> {
    if status == highs_sys::STATUS_OK || status == highs_sys::STATUS_WARNING {
        Ok(())
    } else {
        Err(SolverError::SolverSpecific(format!(
            "{operation} returned HiGHS status {status}"
        )))
    }
}

fn cstring_option_key(key: &str) -> Result<CString, SolverError> {
    CString::new(key).map_err(|_| {
        SolverError::InvalidSettings(format!("invalid HiGHS option name {key:?}: contains NUL"))
    })
}

fn cstring_highs_key(key: &str) -> Result<CString, SolverError> {
    CString::new(key).map_err(|_| {
        SolverError::SolverSpecific(format!("invalid HiGHS info key {key:?}: contains NUL"))
    })
}

fn raw_highs_model_status_to_solver_status(status: highs_sys::HighsInt) -> SolverStatus {
    match status {
        highs_sys::MODEL_STATUS_OPTIMAL => SolverStatus::Optimal,
        highs_sys::MODEL_STATUS_INFEASIBLE => SolverStatus::Infeasible,
        highs_sys::MODEL_STATUS_UNBOUNDED => SolverStatus::Unbounded,
        highs_sys::MODEL_STATUS_REACHED_TIME_LIMIT => SolverStatus::TimeLimit,
        highs_sys::MODEL_STATUS_REACHED_ITERATION_LIMIT => SolverStatus::IterationLimit,
        _ => SolverStatus::Unknown,
    }
}

fn primal_solution_is_feasible(highs_primal_solution_status: f64) -> bool {
    let feasible_status = highs_sys::SOLUTION_STATUS_FEASIBLE as f64;
    (highs_primal_solution_status - feasible_status).abs() <= f64::EPSILON
}

fn objective_value_for_primal_solution_status(
    objective_value: f64,
    highs_primal_solution_status: f64,
) -> f64 {
    if primal_solution_is_feasible(highs_primal_solution_status) {
        objective_value
    } else {
        f64::NAN
    }
}

fn status_with_primal_solution_availability(
    mapped_status: SolverStatus,
    highs_primal_solution_status: f64,
) -> SolverStatus {
    if mapped_status.is_feasible() && !primal_solution_is_feasible(highs_primal_solution_status) {
        SolverStatus::Unknown
    } else {
        mapped_status
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_model::{Bounds, Constraint, Model, ModelView, Objective, Sense, Variable};
    use arco_solver::{
        check_empty_model_rejected, check_no_objective_rejected, check_small_lp, check_small_milp,
        small_lp_model, small_milp_model,
    };
    use std::cell::Cell;

    struct CountingModelView<'a> {
        model: &'a Model,
        fingerprint_calls: Cell<usize>,
    }

    impl ModelView for CountingModelView<'_> {
        fn num_variables(&self) -> usize {
            self.model.num_variables()
        }

        fn num_constraints(&self) -> usize {
            self.model.num_constraints()
        }

        fn num_coefficients(&self) -> usize {
            self.model.num_coefficients()
        }

        fn variable(&self, id: VariableId) -> Option<arco_model::Variable> {
            self.model.variable(id)
        }

        fn constraint(&self, id: ConstraintId) -> Option<arco_model::Constraint> {
            self.model.constraint(id)
        }

        fn objective(&self) -> &arco_model::Objective {
            self.model.objective()
        }

        fn column(&self, id: VariableId) -> Option<&[(ConstraintId, f64)]> {
            self.model.column(id)
        }

        fn fingerprint(&self) -> ModelFingerprint {
            self.fingerprint_calls
                .set(self.fingerprint_calls.get().saturating_add(1));
            self.model.fingerprint()
        }
    }

    #[test]
    fn model_view_solver_rejects_empty_problem() {
        let backend = HighsModelViewBackend;
        check_empty_model_rejected(&backend).expect("HiGHS should reject empty model");
    }

    #[test]
    fn model_view_solver_rejects_no_objective_problem() {
        let backend = HighsModelViewBackend;
        check_no_objective_rejected(&backend).expect("HiGHS should reject missing objective");
    }

    #[test]
    fn shared_solver_setting_validation_uses_stable_error_variant() {
        for (config, expected) in [
            (
                SolverConfig::new().with_time_limit(-1.0),
                "time_limit must be finite and >= 0",
            ),
            (
                SolverConfig::new().with_mip_gap(f64::NAN),
                "mip_gap must be finite and >= 0",
            ),
            (
                SolverConfig::new().with_tolerance(-0.5),
                "tolerance must be finite and >= 0",
            ),
            (SolverConfig::new().with_threads(0), "threads must be >= 1"),
        ] {
            let error = validate_solver_config(&config)
                .expect_err("invalid shared setting should be rejected");

            assert!(matches!(
                error,
                SolverError::InvalidSettings(message) if message == expected
            ));
        }
    }

    #[test]
    fn model_view_problem_solves_directly() {
        let backend = HighsModelViewBackend;
        let report =
            check_small_lp(&backend, &SolverConfig::new()).expect("HiGHS should solve small LP");
        let milp_report = check_small_milp(&backend, &SolverConfig::new())
            .expect("HiGHS should solve small MILP");

        assert_eq!(report.family, "highs");
        assert_eq!(report.variables, 1);
        assert_eq!(report.constraints, 1);
        assert_eq!(report.coefficients, 1);
        assert_eq!(milp_report.family, "highs");
        assert_eq!(milp_report.variables, 1);
        assert_eq!(milp_report.constraints, 1);
        assert_eq!(milp_report.coefficients, 1);

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
        assert_eq!(result.metadata.get("highs_direct_load_path"), Some(&1.0));
        assert_eq!(result.metadata.get("num_variables"), Some(&1.0));
        assert_eq!(result.metadata.get("num_constraints"), Some(&1.0));
        assert_eq!(result.metadata.get("num_coefficients"), Some(&1.0));
    }

    #[test]
    fn model_view_solver_can_skip_fingerprint_and_solution_extraction() {
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

        let config = SolverConfig::new()
            .with_parameter("arco.fingerprint", "false")
            .with_parameter("arco.extract_solution", "false");
        let result = solve_model_view(&model, &config).expect("solve succeeds");

        assert!(result.status.is_feasible());
        assert_eq!(result.fingerprint.0, 0);
        assert!(result.primal_values.is_empty());
    }

    #[test]
    fn direct_highs_load_path_solves_model_view_problem() {
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

        let config = SolverConfig::new()
            .with_parameter("arco.highs_load_path", "direct")
            .with_parameter("arco.fingerprint", "false");
        let result = solve_model_view(&model, &config).expect("direct solve succeeds");

        assert!(result.status.is_feasible());
        assert_eq!(result.primal_values, vec![1.0]);
        assert_eq!(result.objective_value, 2.0);
        assert_eq!(result.fingerprint, ModelFingerprint(0));
        assert_eq!(result.metadata.get("highs_direct_load_path"), Some(&1.0));
        assert_eq!(
            result.metadata.get("highs_model_status"),
            Some(&(highs_sys::MODEL_STATUS_OPTIMAL as f64))
        );
        assert_eq!(
            result.metadata.get("highs_primal_solution_status"),
            Some(&(highs_sys::SOLUTION_STATUS_FEASIBLE as f64))
        );
    }

    #[test]
    fn direct_load_preserves_sparse_empty_and_duplicate_objectives() {
        let config = SolverConfig::new().with_parameter("arco.fingerprint", "false");

        let mut sparse_model = Model::new();
        sparse_model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("first variable");
        sparse_model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("second variable");
        let sparse_x2 = sparse_model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("third variable");
        let sparse_constraint = sparse_model
            .add_constraint(Constraint {
                bounds: Bounds::new(2.0, f64::INFINITY),
            })
            .expect("sparse constraint");
        sparse_model
            .set_coefficient(sparse_x2, sparse_constraint, 1.0)
            .expect("sparse coefficient");
        sparse_model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(sparse_x2, 3.0)],
            })
            .expect("sparse objective");

        let sparse_result = solve_model_view(&sparse_model, &config).expect("sparse solve");
        assert_eq!(sparse_result.primal_values, vec![0.0, 0.0, 2.0]);
        assert_eq!(sparse_result.objective_value, 6.0);

        let mut empty_model = Model::new();
        empty_model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("empty-objective variable");
        empty_model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: Vec::new(),
            })
            .expect("empty objective");

        let empty_result = solve_model_view(&empty_model, &config).expect("empty solve");
        assert_eq!(empty_result.objective_value, 0.0);

        let mut duplicate_model = Model::new();
        let duplicate_x = duplicate_model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("duplicate-term variable");
        let duplicate_constraint = duplicate_model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("duplicate-term constraint");
        duplicate_model
            .set_coefficient(duplicate_x, duplicate_constraint, 1.0)
            .expect("duplicate-term coefficient");
        duplicate_model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(duplicate_x, 2.0), (duplicate_x, 3.0)],
            })
            .expect("duplicate objective");

        let duplicate_result =
            solve_model_view(&duplicate_model, &config).expect("duplicate-term solve");
        assert_eq!(duplicate_result.primal_values, vec![1.0]);
        assert_eq!(duplicate_result.objective_value, 5.0);
    }

    #[test]
    fn direct_load_reuses_dense_objective_allocation() {
        let mut model = Model::new();
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("first variable");
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("second variable");

        let objective_coefficients = vec![2.0, 0.0];
        let allocation = objective_coefficients.as_ptr();
        let load_data = build_direct_highs_load_data(&model, objective_coefficients)
            .expect("load data should build");

        assert_eq!(load_data.col_cost.as_ptr(), allocation);
        assert_eq!(load_data.col_cost, [2.0, 0.0]);
    }

    #[test]
    fn legacy_wrapper_load_path_alias_solves_with_direct_backend() {
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

        let config = SolverConfig::new().with_parameter("arco.highs_load_path", "wrapper");
        let result = solve_model_view(&model, &config).expect("legacy wrapper alias succeeds");

        assert!(result.status.is_feasible());
        assert_eq!(result.primal_values, vec![1.0]);
        assert_eq!(result.objective_value, 2.0);
        assert_eq!(result.metadata.get("highs_direct_load_path"), Some(&1.0));
    }

    #[test]
    fn direct_highs_load_path_reports_invalid_options() {
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
                terms: vec![(x, 1.0)],
            })
            .expect("objective");
        let config = SolverConfig::new()
            .with_parameter("arco.highs_load_path", "direct")
            .with_parameter("run_crossover", "false");

        let error = solve_model_view(&model, &config)
            .expect_err("invalid direct HiGHS option value should be reported");

        assert!(matches!(
            error,
            SolverError::SolverSpecific(message)
                if message.contains("Highs_setStringOptionValue")
                    || message.contains("run_crossover")
                    || message.contains("returned HiGHS status")
        ));
    }

    #[test]
    fn prepared_model_solves_after_source_drop_for_lp_and_mip() {
        let config = SolverConfig::new().with_threads(1);

        let lp_model = small_lp_model();
        let lp_fingerprint = lp_model.fingerprint();
        let prepared_lp =
            PreparedHighsModel::prepare(&lp_model, &config).expect("prepare HiGHS LP");
        drop(lp_model);
        let lp_result = prepared_lp.solve().expect("solve prepared HiGHS LP");
        assert_eq!(lp_result.fingerprint, lp_fingerprint);
        assert_eq!(lp_result.primal_values, [1.0]);
        assert_eq!(lp_result.objective_value, 2.0);

        let mip_model = small_milp_model();
        let mip_fingerprint = mip_model.fingerprint();
        let prepared_mip =
            PreparedHighsModel::prepare(&mip_model, &config).expect("prepare HiGHS MILP");
        drop(mip_model);
        let mip_result = prepared_mip.solve().expect("solve prepared HiGHS MILP");
        assert_eq!(mip_result.fingerprint, mip_fingerprint);
        assert_eq!(mip_result.primal_values, [1.0]);
        assert_eq!(mip_result.objective_value, 1.0);
    }

    #[test]
    fn prepared_model_honors_disabled_fingerprint_and_extraction() {
        let model = small_lp_model();
        let view = CountingModelView {
            model: &model,
            fingerprint_calls: Cell::new(0),
        };
        let config = SolverConfig::new()
            .with_threads(1)
            .with_parameter("arco.fingerprint", "false")
            .with_parameter("arco.extract_solution", "false");
        let prepared = PreparedHighsModel::prepare(&view, &config).expect("prepare HiGHS model");
        assert_eq!(view.fingerprint_calls.get(), 0);
        let result = prepared.solve().expect("solve prepared HiGHS model");
        assert_eq!(view.fingerprint_calls.get(), 0);

        assert_eq!(result.fingerprint, ModelFingerprint(0));
        assert!(result.primal_values.is_empty());
        assert!(result.variable_duals.is_empty());
        assert!(result.row_values.is_empty());
        assert!(result.constraint_duals.is_empty());
    }

    #[test]
    fn prepared_model_captures_enabled_fingerprint_once_before_source_drop() {
        let model = small_lp_model();
        let prepared = {
            let view = CountingModelView {
                model: &model,
                fingerprint_calls: Cell::new(0),
            };
            let prepared = PreparedHighsModel::prepare(&view, &SolverConfig::new())
                .expect("prepare HiGHS model");
            assert_eq!(view.fingerprint_calls.get(), 1);
            prepared
        };
        drop(model);

        prepared.solve().expect("solve prepared HiGHS model");
    }

    #[test]
    fn prepared_model_releases_native_state_on_drop_and_rejects_invalid_config_early() {
        let model = small_lp_model();
        let prepared =
            PreparedHighsModel::prepare(&model, &SolverConfig::new()).expect("prepare HiGHS model");
        drop(prepared);

        let retry = PreparedHighsModel::prepare(&model, &SolverConfig::new())
            .expect("prepare after dropping unsolved HiGHS model");
        drop(retry);

        let invalid_config = SolverConfig::new().with_threads(0);
        let error = match PreparedHighsModel::prepare(&model, &invalid_config) {
            Err(error) => error,
            Ok(_) => panic!("invalid configuration should fail before native load"),
        };
        assert!(
            matches!(error, SolverError::InvalidSettings(message) if message == "threads must be >= 1")
        );

        let invalid_native_option = SolverConfig::new().with_parameter("run_crossover", "false");
        let error = match PreparedHighsModel::prepare(&model, &invalid_native_option) {
            Err(error) => error,
            Ok(_) => panic!("invalid native option should fail during preparation"),
        };
        assert!(
            matches!(error, SolverError::SolverSpecific(message) if message.contains("run_crossover") || message.contains("Highs_setStringOptionValue") || message.contains("returned HiGHS status"))
        );
        PreparedHighsModel::prepare(&model, &SolverConfig::new())
            .expect("native state should be released after preparation failure");
    }

    #[test]
    fn prepared_model_releases_native_state_after_optimization_failure() {
        let mut infeasible_model = Model::new();
        let variable = infeasible_model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("variable");
        let constraint = infeasible_model
            .add_constraint(Constraint {
                bounds: Bounds::new(2.0, f64::INFINITY),
            })
            .expect("constraint");
        infeasible_model
            .set_coefficient(variable, constraint, 1.0)
            .expect("coefficient");
        infeasible_model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(variable, 1.0)],
            })
            .expect("objective");

        let prepared = PreparedHighsModel::prepare(&infeasible_model, &SolverConfig::new())
            .expect("prepare infeasible HiGHS model");
        drop(infeasible_model);
        let error = prepared
            .solve()
            .expect_err("infeasible HiGHS model must fail to solve");
        assert!(matches!(
            error,
            SolverError::SolveFailure {
                status: SolverStatus::Infeasible
            }
        ));

        let retry_model = small_lp_model();
        let retry = PreparedHighsModel::prepare(&retry_model, &SolverConfig::new())
            .expect("prepare after failed HiGHS optimization");
        drop(retry_model);
        let result = retry.solve().expect("retry HiGHS solve");
        assert_eq!(result.objective_value, 2.0);
    }
}
