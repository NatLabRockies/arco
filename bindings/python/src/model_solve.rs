use crate::errors;
use crate::solver::{
    SolveOverrides, detect_default_backend, extract_solver_settings, resolve_backend,
    solve_failure_solution,
};
use crate::{PyModel, PySolveResult};
use arco_expr::VariableId;
use arco_ops::ArcoOps;
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
    let overrides = SolveOverrides {
        log_to_console,
        time_limit,
        mip_gap,
        verbosity,
    };

    let hints: Option<Vec<(VariableId, f64)>> = primal_start.map(|ps| {
        ps.into_iter()
            .map(|(var_id, value)| (VariableId::new(var_id), value))
            .collect()
    });

    let effective_settings = if let Some(s) = solver_obj {
        extract_solver_settings(Some(s))?
    } else {
        model.solver_settings.clone()
    };
    let effective_settings = effective_settings.with_overrides(overrides)?;

    let selected_backend = solver_obj.map_or_else(
        || model.default_backend.clone(),
        |solver| detect_default_backend(Some(solver)),
    );
    let config = effective_settings.to_solver_config();

    let backend = resolve_backend(solver_obj, &selected_backend)?;

    let registry = arco_solver::SolverRegistry::with_builtin_families();
    let profiles = std::collections::BTreeMap::new();
    let resolved = ArcoOps::resolve_solver_selection(&registry, &profiles, &selected_backend)
        .map_err(|error| errors::SolverInternalError::new_err(error.to_string()))?;
    let requirements = arco_solver::SolverRequirements {
        transport: None,
        require_warm_start: hints.is_some(),
        require_iis: false,
    };
    ArcoOps::preflight_solver_selection(&registry, &resolved, &model.inner, &requirements)
        .map_err(|error| errors::SolverInternalError::new_err(error.to_string()))?;

    let result = match ArcoOps::solve_model_backend(
        backend.as_ref(),
        &model.inner,
        &config,
        hints.as_deref(),
    ) {
        Ok(solution) => Ok(PySolveResult::new(solution)),
        Err(arco_solver::SolverError::SolveFailure { status }) => {
            Ok(PySolveResult::new(solve_failure_solution(status)))
        }
        Err(error) => Err(errors::generic_solver_error_to_py(error)),
    }?;

    Py::new(py, result)
}
