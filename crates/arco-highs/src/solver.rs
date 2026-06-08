//! HiGHS solver implementation over solver-facing targets.

use crate::status::highs_model_status;
use arco_model::{ConstraintId, ModelFingerprint, ModelView, Sense, VariableId};
use arco_solver::{
    ModelViewBackend, ModelViewSolveResult, SolverConfig, SolverStatus, SolverStatusMapping,
    validate_model_view_solve_result_with_config,
};
use highs::SolvedModel as RawSolvedHighsModel;
use highs::{ColProblem, HighsOptionValue, Model as RawHighsModel, Sense as HighsSense};
use std::collections::BTreeMap;
use std::ffi::{CString, c_void};
use std::time::Instant;

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

fn apply_solver_config(
    highs_model: &mut RawHighsModel,
    config: &SolverConfig,
) -> Result<(), SolverError> {
    validate_solver_config(config)?;

    if config.verbosity.unwrap_or(0) == 0 && !config.log_to_console.unwrap_or(false) {
        try_set_highs_option(highs_model, "output_flag", false)?;
        try_set_highs_option(highs_model, "log_to_console", false)?;
    }
    if let Some(level) = config.verbosity {
        try_set_highs_option(highs_model, "output_flag", level > 0)?;
    }
    if config.log_to_console.unwrap_or(false) {
        try_set_highs_option(highs_model, "log_to_console", true)?;
        try_set_highs_option(highs_model, "output_flag", true)?;
    }
    if let Some(limit) = config.time_limit {
        try_set_highs_option(highs_model, "time_limit", limit)?;
    }
    if let Some(gap) = config.mip_gap {
        try_set_highs_option(highs_model, "mip_rel_gap", gap)?;
    }
    if let Some(presolve) = config.presolve {
        try_set_highs_option(highs_model, "presolve", if presolve { "on" } else { "off" })?;
    }
    if let Some(threads) = config.threads {
        try_set_highs_option(highs_model, "threads", threads as i32)?;
    }
    if let Some(tolerance) = config.tolerance {
        try_set_highs_option(highs_model, "primal_feasibility_tolerance", tolerance)?;
        try_set_highs_option(highs_model, "dual_feasibility_tolerance", tolerance)?;
    }
    for (key, value) in &config.parameters {
        if key.starts_with("arco.") {
            continue;
        }
        try_set_highs_option(highs_model, key.as_str(), value.as_str())?;
    }
    Ok(())
}

fn try_set_highs_option<V: HighsOptionValue>(
    highs_model: &mut RawHighsModel,
    key: &str,
    value: V,
) -> Result<(), SolverError> {
    highs_model.try_set_option(key, value).map_err(|_| {
        SolverError::InvalidSettings(format!("invalid HiGHS option or value for {key:?}"))
    })
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
    let prepared_problem = prepare_highs_problem(model, config)?;
    let result =
        finish_prepared_solve(optimise_prepared_problem(prepared_problem, config)?, config)?;
    validate_model_view_solve_result_with_config(model, &result, config)?;
    Ok(result)
}

/// Solve an owned core model, allowing callers to release model memory before
/// the HiGHS algorithm starts.
pub fn solve_owned_model(
    mut model: arco_model::Model,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let prepared_problem = prepare_owned_highs_problem(&mut model, config)?;
    drop(model);
    let prepared = optimise_prepared_problem(prepared_problem, config)?;
    finish_prepared_solve(prepared, config)
}

struct PreparedHighsProblem {
    problem: PreparedHighsLoad,
    load_path: HighsLoadPath,
    sense: HighsSense,
    fingerprint: ModelFingerprint,
    matrix_build_seconds: f64,
    fingerprint_seconds: f64,
    num_variables: usize,
    num_constraints: usize,
    num_coefficients: usize,
}

struct PreparedHighsSolve {
    highs_model: PreparedHighsModel,
    load_path: HighsLoadPath,
    fingerprint: ModelFingerprint,
    matrix_build_seconds: f64,
    fingerprint_seconds: f64,
    num_variables: usize,
    num_constraints: usize,
    num_coefficients: usize,
}

enum PreparedHighsLoad {
    Wrapper(ColProblem),
    Direct(DirectHighsLoadData),
}

enum PreparedHighsModel {
    Wrapper(RawHighsModel),
    Direct(DirectHighsModel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HighsLoadPath {
    Wrapper,
    Direct,
}

enum SolvedHighsModel {
    Wrapper(RawSolvedHighsModel),
    Direct {
        model: DirectHighsModel,
        status: SolverStatus,
    },
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
            unsafe {
                highs_sys::Highs_destroy(self.ptr);
            }
        }
    }
}

fn prepare_highs_problem(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<PreparedHighsProblem, SolverError> {
    prepare_highs_problem_with_objective_terms(model, model.objective().terms.as_slice(), config)
}

fn prepare_owned_highs_problem(
    model: &mut arco_model::Model,
    config: &SolverConfig,
) -> Result<PreparedHighsProblem, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let fingerprint_start = Instant::now();
    let fingerprint = if config
        .parameters
        .get("arco.fingerprint")
        .is_none_or(|value| value != "false")
    {
        model.fingerprint()
    } else {
        ModelFingerprint(0)
    };
    let fingerprint_seconds = fingerprint_start.elapsed().as_secs_f64();

    let objective_terms = model.take_objective_terms_for_consumed_solve();
    match requested_load_path(config)? {
        HighsLoadPath::Wrapper => prepare_highs_problem_with_objective_terms_and_fingerprint(
            model,
            objective_terms.as_slice(),
            config,
            fingerprint,
            fingerprint_seconds,
        ),
        HighsLoadPath::Direct => prepare_consumed_direct_highs_problem(
            model,
            objective_terms.as_slice(),
            fingerprint,
            fingerprint_seconds,
        ),
    }
}

fn prepare_highs_problem_with_objective_terms(
    model: &(impl ModelView + ?Sized),
    objective_terms: &[(VariableId, f64)],
    config: &SolverConfig,
) -> Result<PreparedHighsProblem, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let fingerprint_start = Instant::now();
    let fingerprint = if config
        .parameters
        .get("arco.fingerprint")
        .is_none_or(|value| value != "false")
    {
        model.fingerprint()
    } else {
        ModelFingerprint(0)
    };
    let fingerprint_seconds = fingerprint_start.elapsed().as_secs_f64();

    prepare_highs_problem_with_objective_terms_and_fingerprint(
        model,
        objective_terms,
        config,
        fingerprint,
        fingerprint_seconds,
    )
}

fn prepare_highs_problem_with_objective_terms_and_fingerprint(
    model: &(impl ModelView + ?Sized),
    objective_terms: &[(VariableId, f64)],
    config: &SolverConfig,
    fingerprint: ModelFingerprint,
    fingerprint_seconds: f64,
) -> Result<PreparedHighsProblem, SolverError> {
    let objective_sense = model.objective().sense.ok_or(SolverError::NoObjective)?;

    let matrix_start = Instant::now();
    let load_path = requested_load_path(config)?;
    let problem = match load_path {
        HighsLoadPath::Wrapper => {
            PreparedHighsLoad::Wrapper(build_wrapper_highs_problem(model, objective_terms)?)
        }
        HighsLoadPath::Direct => {
            PreparedHighsLoad::Direct(build_direct_highs_load_data(model, objective_terms)?)
        }
    };
    let matrix_build_seconds = matrix_start.elapsed().as_secs_f64();

    let sense = match objective_sense {
        Sense::Minimize => HighsSense::Minimise,
        Sense::Maximize => HighsSense::Maximise,
    };

    Ok(PreparedHighsProblem {
        problem,
        load_path,
        sense,
        fingerprint,
        matrix_build_seconds,
        fingerprint_seconds,
        num_variables: model.num_variables(),
        num_constraints: model.num_constraints(),
        num_coefficients: model.num_coefficients(),
    })
}

fn prepare_consumed_direct_highs_problem(
    model: &mut arco_model::Model,
    objective_terms: &[(VariableId, f64)],
    fingerprint: ModelFingerprint,
    fingerprint_seconds: f64,
) -> Result<PreparedHighsProblem, SolverError> {
    let objective_sense = model.objective().sense.ok_or(SolverError::NoObjective)?;
    let num_variables = model.num_variables();
    let num_constraints = model.num_constraints();
    let num_coefficients = model.num_coefficients();

    let matrix_start = Instant::now();
    let problem = PreparedHighsLoad::Direct(build_direct_highs_load_data_consuming_model(
        model,
        objective_terms,
        num_coefficients,
    )?);
    let matrix_build_seconds = matrix_start.elapsed().as_secs_f64();

    let sense = match objective_sense {
        Sense::Minimize => HighsSense::Minimise,
        Sense::Maximize => HighsSense::Maximise,
    };

    Ok(PreparedHighsProblem {
        problem,
        load_path: HighsLoadPath::Direct,
        sense,
        fingerprint,
        matrix_build_seconds,
        fingerprint_seconds,
        num_variables,
        num_constraints,
        num_coefficients,
    })
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

fn build_wrapper_highs_problem(
    model: &(impl ModelView + ?Sized),
    objective_terms: &[(VariableId, f64)],
) -> Result<ColProblem, SolverError> {
    let mut problem = ColProblem::new();
    let rows = (0..model.num_constraints())
        .map(|index| {
            let constraint = model
                .constraint(ConstraintId::new(index as u32))
                .ok_or_else(|| {
                    SolverError::SolverSpecific(format!("constraint ID {index} does not exist"))
                })?;
            Ok(problem.add_row(constraint.bounds.lower..=constraint.bounds.upper))
        })
        .collect::<Result<Vec<_>, SolverError>>()?;

    let mut objective_terms = objective_terms.iter().peekable();
    for index in 0..model.num_variables() {
        let variable_id = VariableId::new(index as u32);
        let objective = loop {
            match objective_terms.peek().copied() {
                Some((term_variable_id, _)) if term_variable_id.inner() as usize == index => {
                    let coefficient = objective_terms.next().map_or(0.0, |(_, value)| *value);
                    break coefficient;
                }
                Some((term_variable_id, _)) if (term_variable_id.inner() as usize) < index => {
                    let _ = objective_terms.next();
                }
                _ => break 0.0,
            }
        };
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(index as u32))?;
        let column = model.column(variable_id).unwrap_or(&[]);
        for (constraint_id, _) in column {
            let row_index = constraint_id.inner() as usize;
            if row_index >= rows.len() {
                return Err(SolverError::SolverSpecific(format!(
                    "constraint ID {row_index} does not exist"
                )));
            }
        }
        let factors = column.iter().map(|(constraint_id, coefficient)| {
            (rows[constraint_id.inner() as usize], *coefficient)
        });
        if variable.is_integer {
            problem.add_integer_column(
                objective,
                variable.bounds.lower..=variable.bounds.upper,
                factors,
            );
        } else {
            problem.add_column(
                objective,
                variable.bounds.lower..=variable.bounds.upper,
                factors,
            );
        }
    }

    Ok(problem)
}

fn build_direct_highs_load_data(
    model: &(impl ModelView + ?Sized),
    objective_terms: &[(VariableId, f64)],
) -> Result<DirectHighsLoadData, SolverError> {
    let num_cols = checked_highs_int(model.num_variables(), "variables")?;
    let num_rows = checked_highs_int(model.num_constraints(), "constraints")?;
    let num_nonzeros = checked_highs_int(model.num_coefficients(), "coefficients")?;
    let ncols = model.num_variables();
    let nrows = model.num_constraints();
    let mut col_cost = Vec::with_capacity(ncols);
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

    let mut objective_terms = objective_terms.iter().peekable();
    for index in 0..ncols {
        let variable_id = VariableId::new(index as u32);
        let objective = objective_coefficient_for_index(&mut objective_terms, index);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(index as u32))?;
        if variable.is_integer && integrality.is_none() {
            integrality = Some(vec![highs_sys::kHighsVarTypeContinuous; index]);
        }
        if let Some(integrality) = &mut integrality {
            integrality.push(if variable.is_integer {
                highs_sys::kHighsVarTypeInteger
            } else {
                highs_sys::kHighsVarTypeContinuous
            });
        }
        col_cost.push(objective);
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
        col_cost,
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

fn build_direct_highs_load_data_consuming_model(
    model: &mut arco_model::Model,
    objective_terms: &[(VariableId, f64)],
    num_coefficients: usize,
) -> Result<DirectHighsLoadData, SolverError> {
    let num_cols = checked_highs_int(model.num_variables(), "variables")?;
    let num_rows = checked_highs_int(model.num_constraints(), "constraints")?;
    let num_nonzeros = checked_highs_int(num_coefficients, "coefficients")?;
    let ncols = model.num_variables();
    let nrows = model.num_constraints();
    let mut col_cost = Vec::with_capacity(ncols);
    let mut col_lower = Vec::with_capacity(ncols);
    let mut col_upper = Vec::with_capacity(ncols);
    let mut row_lower = Vec::with_capacity(nrows);
    let mut row_upper = Vec::with_capacity(nrows);
    let mut a_start = Vec::with_capacity(ncols);
    let mut a_index = Vec::with_capacity(num_coefficients);
    let mut a_value = Vec::with_capacity(num_coefficients);
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

    let mut objective_terms = objective_terms.iter().peekable();
    for index in 0..ncols {
        let variable_id = VariableId::new(index as u32);
        let objective = objective_coefficient_for_index(&mut objective_terms, index);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(index as u32))?;
        if variable.is_integer && integrality.is_none() {
            integrality = Some(vec![highs_sys::kHighsVarTypeContinuous; index]);
        }
        if let Some(integrality) = &mut integrality {
            integrality.push(if variable.is_integer {
                highs_sys::kHighsVarTypeInteger
            } else {
                highs_sys::kHighsVarTypeContinuous
            });
        }
        col_cost.push(objective);
        col_lower.push(variable.bounds.lower);
        col_upper.push(variable.bounds.upper);
        a_start.push(checked_highs_int(a_index.len(), "column start")?);

        let mut column_error = None;
        let found_column =
            model.drain_column_for_consumed_solve(variable_id, |constraint_id, coefficient| {
                if column_error.is_some() {
                    return;
                }
                let row_index = constraint_id.inner() as usize;
                if row_index >= nrows {
                    column_error = Some(SolverError::SolverSpecific(format!(
                        "constraint ID {row_index} does not exist"
                    )));
                    return;
                }
                match checked_highs_int(row_index, "row index") {
                    Ok(row_index) => {
                        a_index.push(row_index);
                        a_value.push(coefficient);
                    }
                    Err(error) => column_error = Some(error),
                }
            });
        if !found_column {
            return Err(SolverError::InvalidVariableId(index as u32));
        }
        if let Some(error) = column_error {
            return Err(error);
        }
    }

    Ok(DirectHighsLoadData {
        num_cols,
        num_rows,
        num_nonzeros,
        col_cost,
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

fn objective_coefficient_for_index(
    objective_terms: &mut std::iter::Peekable<std::slice::Iter<'_, (VariableId, f64)>>,
    index: usize,
) -> f64 {
    loop {
        match objective_terms.peek().copied() {
            Some((term_variable_id, _)) if term_variable_id.inner() as usize == index => {
                return objective_terms.next().map_or(0.0, |(_, value)| *value);
            }
            Some((term_variable_id, _)) if (term_variable_id.inner() as usize) < index => {
                let _ = objective_terms.next();
            }
            _ => return 0.0,
        }
    }
}

fn checked_highs_int(value: usize, name: &str) -> Result<highs_sys::HighsInt, SolverError> {
    highs_sys::HighsInt::try_from(value)
        .map_err(|_| SolverError::SolverSpecific(format!("{name} count is too large for HiGHS")))
}

fn optimise_prepared_problem(
    prepared: PreparedHighsProblem,
    config: &SolverConfig,
) -> Result<PreparedHighsSolve, SolverError> {
    let highs_model = match prepared.problem {
        PreparedHighsLoad::Wrapper(problem) => {
            let mut highs_model = problem.optimise(prepared.sense);
            apply_solver_config(&mut highs_model, config)?;
            PreparedHighsModel::Wrapper(highs_model)
        }
        PreparedHighsLoad::Direct(load_data) => {
            let mut highs_model = DirectHighsModel::load(load_data, prepared.sense)?;
            apply_direct_solver_config(&mut highs_model, config)?;
            PreparedHighsModel::Direct(highs_model)
        }
    };
    Ok(PreparedHighsSolve {
        highs_model,
        load_path: prepared.load_path,
        fingerprint: prepared.fingerprint,
        matrix_build_seconds: prepared.matrix_build_seconds,
        fingerprint_seconds: prepared.fingerprint_seconds,
        num_variables: prepared.num_variables,
        num_constraints: prepared.num_constraints,
        num_coefficients: prepared.num_coefficients,
    })
}

#[allow(unsafe_code)]
impl DirectHighsModel {
    fn load(load_data: DirectHighsLoadData, sense: HighsSense) -> Result<Self, SolverError> {
        let ptr = unsafe { highs_sys::Highs_create() };
        if ptr.is_null() {
            return Err(SolverError::SolverNotAvailable(
                "HiGHS_create returned NULL".to_string(),
            ));
        }
        let mut model = Self { ptr };
        model.set_bool_option("output_flag", false)?;
        model.set_bool_option("log_to_console", false)?;
        let sense = match sense {
            HighsSense::Minimise => highs_sys::OBJECTIVE_SENSE_MINIMIZE,
            HighsSense::Maximise => highs_sys::OBJECTIVE_SENSE_MAXIMIZE,
        };
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

    fn solve(&mut self) -> Result<SolverStatus, SolverError> {
        ensure_highs_ok(unsafe { highs_sys::Highs_run(self.ptr) }, "Highs_run")?;
        Ok(raw_highs_model_status_to_solver_status(unsafe {
            highs_sys::Highs_getModelStatus(self.ptr)
        }))
    }

    fn objective_value(&self) -> f64 {
        unsafe { highs_sys::Highs_getObjectiveValue(self.ptr) }
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
            unsafe {
                highs_sys::Highs_setBoolOptionValue(self.ptr, key.as_ptr(), i32::from(value))
            },
            key.to_string_lossy().as_ref(),
        )
    }

    fn set_int_option(&mut self, key: &str, value: i32) -> Result<(), SolverError> {
        let key = cstring_option_key(key)?;
        ensure_highs_ok(
            unsafe { highs_sys::Highs_setIntOptionValue(self.ptr, key.as_ptr(), value) },
            key.to_string_lossy().as_ref(),
        )
    }

    fn set_double_option(&mut self, key: &str, value: f64) -> Result<(), SolverError> {
        let key = cstring_option_key(key)?;
        ensure_highs_ok(
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
    for (key, value) in &config.parameters {
        if key.starts_with("arco.") {
            continue;
        }
        highs_model.set_string_option(key.as_str(), value.as_str())?;
    }
    Ok(())
}

fn ensure_highs_ok(status: highs_sys::HighsInt, operation: &str) -> Result<(), SolverError> {
    if status == highs_sys::STATUS_OK || status == highs_sys::STATUS_WARNING {
        Ok(())
    } else {
        Err(SolverError::InvalidSettings(format!(
            "invalid HiGHS option or value for {operation:?}"
        )))
    }
}

fn cstring_option_key(key: &str) -> Result<CString, SolverError> {
    CString::new(key).map_err(|_| {
        SolverError::InvalidSettings(format!("invalid HiGHS option name {key:?}: contains NUL"))
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

fn finish_prepared_solve(
    prepared: PreparedHighsSolve,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let extract_solution = config
        .parameters
        .get("arco.extract_solution")
        .is_none_or(|value| value != "false");
    let highs_run_start = Instant::now();
    let solved_model = match prepared.highs_model {
        PreparedHighsModel::Wrapper(model) => SolvedHighsModel::Wrapper(model.solve()),
        PreparedHighsModel::Direct(mut model) => {
            let status = model.solve()?;
            SolvedHighsModel::Direct { model, status }
        }
    };
    let mapped_status = match &solved_model {
        SolvedHighsModel::Wrapper(model) => highs_model_status(model.status()).to_solver_status(),
        SolvedHighsModel::Direct { status, .. } => *status,
    };
    let objective_value = match &solved_model {
        SolvedHighsModel::Wrapper(model) => model.objective_value(),
        SolvedHighsModel::Direct { model, .. } => model.objective_value(),
    };
    let highs_run_seconds = highs_run_start.elapsed().as_secs_f64();
    if !mapped_status.is_feasible() {
        return Err(SolverError::SolveFailure {
            status: mapped_status,
        });
    }
    let solution_extract_start = Instant::now();
    let (primal_values, variable_duals, row_values, constraint_duals) = if extract_solution {
        match &solved_model {
            SolvedHighsModel::Wrapper(model) => {
                let solution = model.get_solution();
                (
                    solution.columns().to_vec(),
                    solution.dual_columns().to_vec(),
                    solution.rows().to_vec(),
                    solution.dual_rows().to_vec(),
                )
            }
            SolvedHighsModel::Direct { model, .. } => {
                model.solution_vectors(prepared.num_variables, prepared.num_constraints)?
            }
        }
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };
    let solution_extract_seconds = solution_extract_start.elapsed().as_secs_f64();

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "highs_matrix_build_s".to_string(),
        prepared.matrix_build_seconds,
    );
    metadata.insert(
        "highs_direct_load_path".to_string(),
        match prepared.load_path {
            HighsLoadPath::Wrapper => 0.0,
            HighsLoadPath::Direct => 1.0,
        },
    );
    metadata.insert("highs_run_s".to_string(), highs_run_seconds);
    metadata.insert("solution_extract_s".to_string(), solution_extract_seconds);
    metadata.insert("fingerprint_s".to_string(), prepared.fingerprint_seconds);
    metadata.insert("num_variables".to_string(), prepared.num_variables as f64);
    metadata.insert(
        "num_constraints".to_string(),
        prepared.num_constraints as f64,
    );
    metadata.insert(
        "num_coefficients".to_string(),
        prepared.num_coefficients as f64,
    );

    let result = ModelViewSolveResult {
        fingerprint: prepared.fingerprint,
        status: mapped_status,
        objective_value,
        primal_values,
        variable_duals,
        row_values,
        constraint_duals,
        metadata,
    };
    validate_result_shape_counts(
        &result,
        config,
        prepared.num_variables,
        prepared.num_constraints,
    )?;
    Ok(result)
}

fn validate_result_shape_counts(
    result: &ModelViewSolveResult,
    config: &SolverConfig,
    num_variables: usize,
    num_constraints: usize,
) -> Result<(), SolverError> {
    let allow_omitted_primal_values = config
        .parameters
        .get("arco.extract_solution")
        .is_some_and(|value| value == "false");
    if allow_omitted_primal_values {
        validate_optional_result_len("primal_values", result.primal_values.len(), num_variables)?;
    } else {
        validate_required_result_len("primal_values", result.primal_values.len(), num_variables)?;
    }
    validate_optional_result_len("variable_duals", result.variable_duals.len(), num_variables)?;
    validate_optional_result_len("row_values", result.row_values.len(), num_constraints)?;
    validate_optional_result_len(
        "constraint_duals",
        result.constraint_duals.len(),
        num_constraints,
    )?;
    Ok(())
}

fn validate_required_result_len(
    name: &str,
    actual: usize,
    expected: usize,
) -> Result<(), SolverError> {
    if actual == expected {
        return Ok(());
    }
    Err(SolverError::InvalidResultShape(format!(
        "{name} length {actual} does not match expected {expected}"
    )))
}

fn validate_optional_result_len(
    name: &str,
    actual: usize,
    expected: usize,
) -> Result<(), SolverError> {
    if actual == 0 || actual == expected {
        return Ok(());
    }
    Err(SolverError::InvalidResultShape(format!(
        "{name} length {actual} must be 0 or match expected {expected}"
    )))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_model::{
        Bounds, Constraint, Model, ModelFingerprint, ModelView, Objective, Sense, Variable,
    };
    use arco_solver::{
        check_empty_model_rejected, check_no_objective_rejected, check_small_lp, check_small_milp,
    };

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
    fn invalid_highs_option_value_returns_solver_error_instead_of_panicking() {
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

        let config = SolverConfig::new().with_parameter("run_crossover", "false");
        let error = solve_model_view(&model, &config)
            .expect_err("invalid HiGHS option value should be reported");

        assert!(matches!(
            error,
            SolverError::InvalidSettings(message)
                if message.contains("invalid HiGHS option or value")
                    && message.contains("run_crossover")
        ));
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
        assert_eq!(result.metadata.get("num_variables"), Some(&1.0));
        assert_eq!(result.metadata.get("num_constraints"), Some(&1.0));
        assert_eq!(result.metadata.get("num_coefficients"), Some(&1.0));
    }

    #[test]
    fn model_view_solver_supports_objective_only_result_when_solution_extraction_is_disabled() {
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
        let result = solve_model_view(&model, &config)
            .expect("objective-only result should be accepted when explicitly requested");

        assert_eq!(result.fingerprint, ModelFingerprint(0));
        assert!(result.primal_values.is_empty());
        assert!(result.variable_duals.is_empty());
        assert!(result.row_values.is_empty());
        assert!(result.constraint_duals.is_empty());
        assert_eq!(result.objective_value, 2.0);
    }

    #[test]
    fn owned_model_solver_supports_objective_only_result() {
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

        let result = solve_owned_model(model, &config).expect("owned solve succeeds");

        assert_eq!(result.fingerprint, ModelFingerprint(0));
        assert!(result.primal_values.is_empty());
        assert_eq!(result.objective_value, 2.0);
        assert_eq!(result.metadata.get("highs_direct_load_path"), Some(&0.0));
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

        let result = solve_model_view(&model, &config).expect("direct HiGHS solve succeeds");

        assert_eq!(result.fingerprint, ModelFingerprint(0));
        assert!(result.status.is_feasible());
        assert_eq!(result.primal_values, vec![1.0]);
        assert_eq!(result.objective_value, 2.0);
        assert_eq!(result.metadata.get("highs_direct_load_path"), Some(&1.0));
    }

    #[test]
    fn direct_highs_load_path_supports_owned_objective_only_result() {
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
            .with_parameter("arco.fingerprint", "false")
            .with_parameter("arco.extract_solution", "false");

        let result = solve_owned_model(model, &config).expect("direct owned solve succeeds");

        assert_eq!(result.fingerprint, ModelFingerprint(0));
        assert!(result.primal_values.is_empty());
        assert_eq!(result.objective_value, 2.0);
        assert_eq!(result.metadata.get("highs_direct_load_path"), Some(&1.0));
        assert_eq!(result.metadata.get("num_coefficients"), Some(&1.0));
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
            SolverError::InvalidSettings(message)
                if message.contains("invalid HiGHS option or value")
                    && message.contains("run_crossover")
        ));
    }
}
