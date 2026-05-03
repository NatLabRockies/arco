use crate::errors;
use crate::helpers;
use crate::solver::{self, SolverSettings};
use crate::{PyModel, PySimplifyLevel};
use arco_core::Model;
use arco_core::model::CscInput;
use pyo3::prelude::*;
use pyo3::types::PyAny;

pub(crate) fn new_model(
    simplify_level: Option<PySimplifyLevel>,
    solver: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyModel> {
    let inner = if let Some(level) = simplify_level {
        Model::with_simplify_level(level.into())
    } else {
        Model::new()
    };
    let default_backend = solver::detect_default_backend(solver);
    let solver_settings = solver::extract_solver_settings(solver)?;
    Ok(PyModel::from_parts(inner, solver_settings, default_backend))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn from_csc_model(
    num_constraints: usize,
    num_variables: usize,
    col_ptrs: &Bound<'_, PyAny>,
    row_indices: &Bound<'_, PyAny>,
    values: &Bound<'_, PyAny>,
    var_lower: &Bound<'_, PyAny>,
    var_upper: &Bound<'_, PyAny>,
    con_lower: &Bound<'_, PyAny>,
    con_upper: &Bound<'_, PyAny>,
    is_integer: &Bound<'_, PyAny>,
    simplify_level: Option<PySimplifyLevel>,
) -> PyResult<PyModel> {
    let col_ptrs = helpers::extract_indices(col_ptrs, "col_ptrs")?;
    let row_indices = helpers::extract_indices(row_indices, "row_indices")?;
    let values = helpers::extract_f32(values, "values")?;
    let var_lower = helpers::extract_f32(var_lower, "var_lower")?;
    let var_upper = helpers::extract_f32(var_upper, "var_upper")?;
    let con_lower = helpers::extract_f32(con_lower, "con_lower")?;
    let con_upper = helpers::extract_f32(con_upper, "con_upper")?;
    let is_integer = helpers::extract_bool(is_integer, "is_integer")?;
    let simplify_level = simplify_level.map(Into::into).unwrap_or_default();

    let inner = Model::from_csc(
        CscInput {
            num_constraints,
            num_variables,
            col_ptrs: &col_ptrs,
            row_indices: &row_indices,
            values: &values,
            var_lower: &var_lower,
            var_upper: &var_upper,
            con_lower: &con_lower,
            con_upper: &con_upper,
            is_integer: &is_integer,
        },
        simplify_level,
    )
    .map_err(errors::model_error_to_py)?;

    Ok(PyModel::from_parts(
        inner,
        SolverSettings::default(),
        "highs".to_string(),
    ))
}
