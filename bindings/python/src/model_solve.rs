use crate::py_modules::errors;
use crate::py_modules::solver::{
    SolveOverrides, detect_default_backend, extract_solver_settings, solve_failure_solution,
    validate_backend_settings,
};
use crate::{PyModel, PySolveResult};
use arco_ops::solve::{ModelViewSolveResult, Solution, SolverError};
use pyo3::prelude::*;
use pyo3::types::PyAny;

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
    reject_unsupported_primal_start(primal_start.as_deref())
        .map_err(errors::generic_solver_error_to_py)?;

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
    let effective_settings = effective_settings.with_overrides(overrides)?;
    validate_backend_settings(&selected_backend, &effective_settings)?;
    if selected_backend == "xpress" && !crate::py_modules::solver::xpress_backend_enabled() {
        return Err(errors::generic_solver_error_to_py(SolverError::SolverNotAvailable(
            "Python bindings were built without the xpress feature. Rebuild with: uv run --with maturin maturin develop --features xpress".to_string(),
        )));
    }

    let config = effective_settings.to_solver_config();

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

fn reject_unsupported_primal_start(primal_start: Option<&[(u32, f64)]>) -> Result<(), SolverError> {
    if primal_start.is_some_and(|values| !values.is_empty()) {
        return Err(SolverError::InvalidSettings(
            "primal_start is not supported on the model-view solve path".to_string(),
        ));
    }
    Ok(())
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
        metadata: result.metadata,
    }
}
