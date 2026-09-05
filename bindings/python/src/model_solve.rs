use crate::py_modules::enums::PyLpAlgorithm;
use crate::py_modules::errors;
use crate::py_modules::solver::{
    SolveOverrides, detect_default_backend, extract_solver_settings, solve_failure_solution,
    validate_backend_settings,
};
use crate::{PyModel, PySolveResult};
use arco_model::ModelView;
use arco_solver::{
    ModelViewBackendRegistry, ModelViewSolveResult, Solution, SolverConfig, SolverError,
};
use pyo3::prelude::*;
use pyo3::types::PyAny;

#[allow(clippy::too_many_arguments)]
pub(crate) fn solve_model(
    model: &mut PyModel,
    py: Python<'_>,
    solver_obj: Option<&Bound<'_, PyAny>>,
    log_to_console: Option<bool>,
    primal_start: Option<Vec<(u32, f64)>>,
    time_limit: Option<f64>,
    mip_gap: Option<f64>,
    verbosity: Option<u32>,
    lp_algorithm: Option<PyLpAlgorithm>,
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
    let mut effective_settings = effective_settings.with_overrides(overrides)?;
    if let Some(lp_algorithm) = lp_algorithm {
        effective_settings.set_lp_algorithm(lp_algorithm);
    }
    let backend_family = normalize_model_view_backend_family(&selected_backend);
    validate_backend_settings(backend_family, &effective_settings)?;
    if backend_family == "xpress" && !crate::py_modules::solver::xpress_backend_enabled() {
        return Err(errors::generic_solver_error_to_py(SolverError::SolverNotAvailable(
            "Python bindings were built without the xpress feature. Rebuild with: uv run --with maturin maturin develop --features xpress".to_string(),
        )));
    }

    let config = effective_settings.to_solver_config();

    let consume_model = config
        .parameters
        .get("arco.consume_model")
        .is_some_and(|value| value == "true");

    let consumes_before_solve =
        consume_model && matches!(backend_family, "highs" | "xpress");
    let result = if consume_model && backend_family == "highs" {
        solve_consuming_highs_model(model, &config)?
    } else if consumes_before_solve {
        #[cfg(feature = "xpress")]
        {
            solve_consuming_xpress_model(model, &config)?
        }
        #[cfg(not(feature = "xpress"))]
        {
            solve_borrowed_model(&selected_backend, model, &config)?
        }
    } else {
        solve_borrowed_model(&selected_backend, model, &config)?
    };

    if consume_model && !consumes_before_solve {
        clear_consumed_model_state(model);
    }

    Py::new(py, result)
}

fn solve_borrowed_model(
    selected_backend: &str,
    model: &PyModel,
    config: &SolverConfig,
) -> PyResult<PySolveResult> {
    model_view_solve_result_to_py(solve_model_view_with_builtin_backend(
        selected_backend,
        &model.inner,
        config,
    ))
}

fn model_view_solve_result_to_py(
    result: Result<ModelViewSolveResult, SolverError>,
) -> PyResult<PySolveResult> {
    match result {
        Ok(solution) => Ok(PySolveResult::new(solution_from_model_view_result(solution))),
        Err(SolverError::SolveFailure { status }) => {
            Ok(PySolveResult::new(solve_failure_solution(status)))
        }
        Err(error) => Err(errors::generic_solver_error_to_py(error)),
    }
}

fn solve_consuming_highs_model(
    model: &mut PyModel,
    config: &SolverConfig,
) -> PyResult<PySolveResult> {
    let prepared = prepare_consuming_highs_model(model, config)
        .map_err(errors::generic_solver_error_to_py)?;
    model_view_solve_result_to_py(prepared.solve())
}

fn prepare_consuming_highs_model(
    model: &mut PyModel,
    config: &SolverConfig,
) -> Result<arco_highs::PreparedHighsModel, SolverError> {
    let prepared = arco_highs::PreparedHighsModel::prepare(&model.inner, config)?;
    clear_consumed_model_state(model);
    Ok(prepared)
}

#[cfg(feature = "xpress")]
fn solve_consuming_xpress_model(
    model: &mut PyModel,
    config: &SolverConfig,
) -> PyResult<PySolveResult> {
    let prepared = prepare_consuming_xpress_model(model, config)
        .map_err(errors::generic_solver_error_to_py)?;

    model_view_solve_result_to_py(prepared.solve_model_view())
}

#[cfg(feature = "xpress")]
fn prepare_consuming_xpress_model(
    model: &mut PyModel,
    config: &SolverConfig,
) -> Result<arco_xpress::PreparedXpressModel, SolverError> {
    let prepared = arco_xpress::PreparedXpressModel::prepare(&model.inner, config)?;
    clear_consumed_model_state(model);
    Ok(prepared)
}

fn clear_consumed_model_state(model: &mut PyModel) {
    model.inner = Default::default();
    model.last_solution = None;
    model.array_print_specs.clear();
    model.constraint_print_specs.clear();
    model.block_defs.clear();
    model.link_defs.clear();
    #[cfg(feature = "ipopt")]
    {
        model.nonlinear_state = crate::py_modules::nonlinear_state::NonlinearState::default();
    }
}

fn reject_unsupported_primal_start(primal_start: Option<&[(u32, f64)]>) -> Result<(), SolverError> {
    if primal_start.is_some_and(|values| !values.is_empty()) {
        return Err(SolverError::InvalidSettings(
            "primal_start is not supported on the model-view solve path".to_string(),
        ));
    }
    Ok(())
}

fn solve_model_view_with_builtin_backend(
    family: &str,
    model: &dyn ModelView,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let family = normalize_model_view_backend_family(family);
    if family == "ipopt" {
        return Err(SolverError::SolverNotAvailable(
            "IPOPT model-view backend is not implemented yet; use a supported backend such as 'highs'"
                .to_string(),
        ));
    }
    #[cfg(not(feature = "scip"))]
    if family == "scip" {
        return Err(SolverError::SolverNotAvailable(
            "SCIP model-view backend is not enabled; rebuild with --features scip".to_string(),
        ));
    }
    #[cfg(not(feature = "xpress"))]
    if family == "xpress" {
        return Err(SolverError::SolverNotAvailable(
            "Xpress model-view backend is not enabled; rebuild with --features xpress".to_string(),
        ));
    }

    let highs = arco_highs::HighsModelViewBackend;
    let mut registry = ModelViewBackendRegistry::new();
    registry.try_register(&highs)?;
    #[cfg(feature = "scip")]
    let scip = arco_scip::ScipModelViewBackend;
    #[cfg(feature = "scip")]
    registry.try_register(&scip)?;
    #[cfg(feature = "xpress")]
    let xpress = arco_xpress::XpressModelViewBackend;
    #[cfg(feature = "xpress")]
    registry.try_register(&xpress)?;
    registry.solve(family, model, config)
}

fn normalize_model_view_backend_family(family: &str) -> &str {
    match family {
        "arco-rust-highs" => "highs",
        "arco-rust-xpress" => "xpress",
        "arco-rust-scip" => "scip",
        other => other,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use arco_model::{Bounds, Constraint, Model, Objective, Sense, Variable};
    use crate::py_modules::index_set::{IndexMember, PyIndexSet};

    fn model_with_objective_for_backend(default_backend: &str) -> PyModel {
        let mut inner = Model::new();
        let variable = inner
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .expect("variable");
        let constraint = inner
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        inner
            .set_coefficient(variable, constraint, 1.0)
            .expect("coefficient");
        inner
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(variable, 2.0)],
            })
            .expect("objective");
        PyModel::from_parts(
            inner,
            crate::py_modules::solver::SolverSettings::default(),
            default_backend.to_string(),
        )
    }

    #[cfg(feature = "xpress")]
    fn model_with_objective() -> PyModel {
        model_with_objective_for_backend("xpress")
    }

    fn highs_model_with_objective() -> PyModel {
        model_with_objective_for_backend("highs")
    }

    fn infeasible_highs_model() -> PyModel {
        let mut inner = Model::new();
        let variable = inner
            .add_variable(Variable::continuous(Bounds::new(0.0, 0.0)))
            .expect("variable");
        let constraint = inner
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        inner
            .set_coefficient(variable, constraint, 1.0)
            .expect("coefficient");
        inner
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(variable, 1.0)],
            })
            .expect("objective");
        PyModel::from_parts(
            inner,
            crate::py_modules::solver::SolverSettings::default(),
            "highs".to_string(),
        )
    }

    fn set_previous_solution(model: &mut PyModel) {
        Python::initialize();
        let previous_solution = Python::attach(|py| {
            Py::new(
                py,
                PySolveResult::new(Solution {
                    primal_values: vec![1.0],
                    variable_duals: vec![0.0],
                    constraint_duals: vec![1.0],
                    row_values: vec![1.0],
                    objective_value: 2.0,
                    status: arco_solver::SolverStatus::Optimal,
                    solve_time_seconds: 0.0,
                    metadata: std::collections::BTreeMap::new(),
                }),
            )
            .expect("previous solution")
        });
        model.last_solution = Some(previous_solution);
    }

    fn add_model_metadata(model: &mut PyModel) {
        model
            .constraint_print_specs
            .push(crate::py_modules::model_pretty::ConstraintPrintSpec {
                start_constraint_id: 0,
                len: 1,
                base_name: "constraint".to_string(),
            });
        Python::initialize();
        Python::attach(|py| {
            let index_set = Py::new(
                py,
                PyIndexSet {
                    name: "index".to_string(),
                    members: vec![IndexMember::Int(0)],
                },
            )
            .expect("index set");
            model.register_array_print_spec(
                py,
                0,
                1,
                &[index_set],
                &[1],
                Some("variable"),
            );
        });
    }

    #[test]
    fn consuming_prepare_failure_preserves_highs_model_state() {
        let mut model = highs_model_with_objective();
        set_previous_solution(&mut model);
        add_model_metadata(&mut model);
        let error = match prepare_consuming_highs_model(
            &mut model,
            &SolverConfig::new().with_threads(0),
        ) {
            Ok(_) => panic!("invalid preparation settings should fail"),
            Err(error) => error,
        };

        assert!(matches!(error, SolverError::InvalidSettings(_)));
        assert_eq!(model.inner.num_variables(), 1);
        assert_eq!(model.inner.num_constraints(), 1);
        assert!(model.last_solution.is_some());
        assert_eq!(model.array_print_specs.len(), 1);
        assert_eq!(model.constraint_print_specs.len(), 1);
        assert_eq!(model.constraint_print_specs[0].base_name, "constraint");
        assert_eq!(model.array_print_specs[0].len, 1);
    }

    #[test]
    fn consuming_prepare_clears_highs_model_before_native_solve() {
        let mut model = highs_model_with_objective();
        set_previous_solution(&mut model);
        add_model_metadata(&mut model);
        let prepared = prepare_consuming_highs_model(
            &mut model,
            &SolverConfig::new()
                .with_threads(1)
                .with_parameter("arco.extract_solution", "false")
                .with_parameter("arco.fingerprint", "false"),
        )
        .unwrap_or_else(|error| panic!("unexpected HiGHS preparation failure: {error}"));

        assert_eq!(model.inner.num_variables(), 0);
        assert_eq!(model.inner.num_constraints(), 0);
        assert!(model.last_solution.is_none());
        assert!(model.array_print_specs.is_empty());
        assert!(model.constraint_print_specs.is_empty());
        let result = prepared
            .solve()
            .unwrap_or_else(|error| panic!("unexpected HiGHS solve failure: {error}"));
        assert_eq!(result.status, arco_solver::SolverStatus::Optimal);
        assert!((result.objective_value - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn consuming_prepare_keeps_highs_model_consumed_when_native_solve_fails() {
        let mut model = infeasible_highs_model();
        let prepared = prepare_consuming_highs_model(
            &mut model,
            &SolverConfig::new()
                .with_threads(1)
                .with_parameter("arco.extract_solution", "false")
                .with_parameter("arco.fingerprint", "false"),
        )
        .unwrap_or_else(|error| panic!("unexpected HiGHS preparation failure: {error}"));

        assert_eq!(model.inner.num_variables(), 0);
        assert_eq!(model.inner.num_constraints(), 0);
        let error = prepared
            .solve()
            .expect_err("infeasible HiGHS solve should report failure");
        assert!(matches!(
            error,
            SolverError::SolveFailure {
                status: arco_solver::SolverStatus::Infeasible
            }
        ));
    }

    #[cfg(feature = "xpress")]
    #[test]
    fn consuming_prepare_failure_preserves_model_state() {
        let mut model = model_with_objective();
        Python::initialize();
        let previous_solution = Python::attach(|py| {
            Py::new(
                py,
                PySolveResult::new(Solution {
                    primal_values: vec![1.0],
                    variable_duals: vec![0.0],
                    constraint_duals: vec![0.0],
                    row_values: vec![1.0],
                    objective_value: 2.0,
                    status: arco_solver::SolverStatus::Optimal,
                    solve_time_seconds: 0.0,
                    metadata: std::collections::BTreeMap::new(),
                }),
            )
            .expect("previous solution")
        });
        model.last_solution = Some(previous_solution);
        let error = match prepare_consuming_xpress_model(
            &mut model,
            &SolverConfig::new().with_threads(0),
        ) {
            Ok(_) => panic!("invalid preparation settings should fail"),
            Err(error) => error,
        };

        assert!(matches!(error, SolverError::InvalidSettings(_)));
        assert_eq!(model.inner.num_variables(), 1);
        assert_eq!(model.inner.num_constraints(), 1);
        assert!(model.last_solution.is_some());
    }

    #[cfg(feature = "xpress")]
    #[test]
    #[ignore = "requires local Xpress runtime and license"]
    fn consuming_prepare_clears_model_before_native_solve() {
        let mut model = model_with_objective();
        let prepared = prepare_consuming_xpress_model(
            &mut model,
            &SolverConfig::new()
                .with_log_to_console(false)
                .with_parameter("arco.extract_solution", "false")
                .with_parameter("arco.fingerprint", "false"),
        )
        .unwrap_or_else(|error| panic!("unexpected Xpress preparation failure: {error}"));

        assert_eq!(model.inner.num_variables(), 0);
        assert_eq!(model.inner.num_constraints(), 0);
        assert!(model.last_solution.is_none());
        let result = prepared
            .solve_model_view()
            .unwrap_or_else(|error| panic!("unexpected Xpress solve failure: {error}"));
        assert_eq!(result.status, arco_solver::SolverStatus::Optimal);
        assert!((result.objective_value - 2.0).abs() < f64::EPSILON);
    }
}
