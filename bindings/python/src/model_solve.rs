use crate::py_modules::errors;
use crate::py_modules::solver::{
    SolveOverrides, detect_default_backend, extract_solver_settings, solve_failure_solution,
};
use crate::{PyModel, PySolveResult};
use arco_ops::solver::{ModelViewSolveResult, Solution, SolverError};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::collections::BTreeMap;

pub(crate) fn solve_model(
    model: &PyModel,
    py: Python<'_>,
    solver_obj: Option<&Bound<'_, PyAny>>,
    log_to_console: Option<bool>,
    primal_start: Option<Vec<(u32, f64)>>,
    time_limit: Option<f64>,
    mip_gap: Option<f64>,
    verbosity: Option<u32>,
) -> PyResult<Py<PySolveResult>> {
    if model.inner.num_variables() == 0 {
        return Err(errors::generic_solver_error_to_py(SolverError::EmptyModel));
    }
    let _warm_start_hint_count = primal_start.as_ref().map_or(0, Vec::len);

    let selected_backend = solver_obj.map_or_else(
        || model.default_backend.clone(),
        |solver| detect_default_backend(Some(solver)),
    );

    let overrides = SolveOverrides {
        log_to_console,
        time_limit,
        mip_gap,
        verbosity,
    };
    let effective_settings = if let Some(s) = solver_obj {
        extract_solver_settings(Some(s))?
    } else {
        model.solver_settings.clone()
    };
    let config = effective_settings
        .with_overrides(overrides)?
        .to_solver_config();

    let result = match arco_ops::ArcoOps::solve_model_view_with_builtin_backend(
        &selected_backend,
        &model.inner,
        &config,
    ) {
        Ok(solution) => Ok(PySolveResult::new(solution_from_model_view_result(
            solution,
        ))),
        Err(SolverError::SolveFailure { status }) => {
            Ok(PySolveResult::new(solve_failure_solution(status)))
        }
        Err(error) => Err(errors::generic_solver_error_to_py(error)),
    }?;

    Py::new(py, result)
}

fn solution_from_model_view_result(result: ModelViewSolveResult) -> Solution {
    Solution {
        primal_values: result.primal_values,
        variable_duals: result.variable_duals,
        constraint_duals: result.constraint_duals,
        row_values: result.row_values,
        objective_value: result.objective_value,
        status: result.status,
        solve_time_seconds: 0.0,
        metadata: BTreeMap::new(),
    }
}
