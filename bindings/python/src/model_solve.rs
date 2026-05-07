use crate::py_modules::errors;
use crate::py_modules::solver::{
    SolveOverrides, detect_default_backend, extract_solver_settings, solve_failure_solution,
};
use crate::{PyModel, PySolveResult};
use arco_ops::expr::{ConstraintId, VariableId};
use arco_ops::model::{InspectOptions, Sense};
use arco_ops::solver::SolverError;
use arco_ops::targets::{
    AlgebraicProblem, ConstraintSense, LinearConstraint, LinearObjective, LinearTerm,
    ObjectiveSense, VariableInstance, VariableKind,
};
use pyo3::exceptions::PyRuntimeError;
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
    if selected_backend != "highs" && selected_backend != "arco-rust-highs" {
        return Err(PyRuntimeError::new_err(format!(
            "Python target-based solve currently supports HiGHS, got {selected_backend}"
        )));
    }

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
    let problem = build_algebraic_problem(model)?;

    let result = match arco_ops::highs::Solver::new(problem).and_then(|mut solver| {
        solver.set_config(config);
        solver.solve().map(|solution| solution.into_core_solution())
    }) {
        Ok(solution) => Ok(PySolveResult::new(solution)),
        Err(SolverError::SolveFailure { status }) => {
            Ok(PySolveResult::new(solve_failure_solution(status)))
        }
        Err(error) => Err(errors::generic_solver_error_to_py(error)),
    }?;

    Py::new(py, result)
}

fn build_algebraic_problem(model: &PyModel) -> PyResult<AlgebraicProblem> {
    let snapshot = model.inner.inspect(InspectOptions {
        include_coefficients: true,
        include_slacks: false,
        variable_filter: None,
        constraint_filter: None,
    });
    let variable_names = variable_names(model);
    let mut terms_by_constraint: BTreeMap<ConstraintId, Vec<LinearTerm>> = BTreeMap::new();
    for coefficient in snapshot.coefficients.unwrap_or_default() {
        terms_by_constraint
            .entry(coefficient.constraint_id)
            .or_default()
            .push(LinearTerm {
                variable_name: variable_names[coefficient.variable_id.inner() as usize].clone(),
                coefficient: coefficient.value,
            });
    }

    let mut constraints = Vec::new();
    for constraint in snapshot.constraints {
        let name = constraint
            .name
            .unwrap_or_else(|| format!("c{}", constraint.id.inner()));
        let terms = terms_by_constraint
            .remove(&constraint.id)
            .unwrap_or_default();
        push_bounded_constraint(&mut constraints, name, constraint.bounds, terms)?;
    }

    let objective_view = snapshot
        .objective
        .ok_or_else(|| PyRuntimeError::new_err("model has no objective"))?;
    let sense = match objective_view
        .sense
        .ok_or_else(|| PyRuntimeError::new_err("model has no objective sense"))?
    {
        Sense::Minimize => ObjectiveSense::Minimize,
        Sense::Maximize => ObjectiveSense::Maximize,
    };
    let objective_terms = objective_view
        .terms
        .into_iter()
        .map(|(variable_id, coefficient)| LinearTerm {
            variable_name: variable_names[variable_id.inner() as usize].clone(),
            coefficient,
        })
        .collect();

    Ok(AlgebraicProblem {
        variable_instances: snapshot
            .variables
            .into_iter()
            .map(|variable| {
                let index = variable.id.inner() as usize;
                let is_binary_bounds = (variable.bounds.lower - 0.0).abs() <= f64::EPSILON
                    && (variable.bounds.upper - 1.0).abs() <= f64::EPSILON;
                let kind = if variable.is_integer && is_binary_bounds {
                    VariableKind::Binary
                } else if variable.is_integer {
                    VariableKind::Integer
                } else {
                    VariableKind::Continuous
                };
                VariableInstance {
                    name: variable_names[index].clone(),
                    family: variable_names[index].clone(),
                    lower: variable.bounds.lower,
                    upper: if variable.bounds.upper.is_finite() {
                        Some(variable.bounds.upper)
                    } else {
                        None
                    },
                    kind,
                }
            })
            .collect(),
        constraints,
        objective: LinearObjective {
            name: objective_view
                .name
                .unwrap_or_else(|| "objective".to_string()),
            sense,
            constant: 0.0,
            terms: objective_terms,
        },
        reports: Vec::new(),
    })
}

fn variable_names(model: &PyModel) -> Vec<String> {
    (0..model.inner.num_variables())
        .map(|index| {
            let id = VariableId::new(index as u32);
            model
                .inner
                .get_variable_name(id)
                .map_or_else(|| format!("x{index}"), str::to_string)
        })
        .collect()
}

fn push_bounded_constraint(
    constraints: &mut Vec<LinearConstraint>,
    name: String,
    bounds: arco_ops::model::types::Bounds,
    terms: Vec<LinearTerm>,
) -> PyResult<()> {
    match (bounds.lower.is_finite(), bounds.upper.is_finite()) {
        (true, true) if (bounds.lower - bounds.upper).abs() <= f64::EPSILON => {
            constraints.push(LinearConstraint {
                name,
                sense: ConstraintSense::Equal,
                rhs: bounds.lower,
                terms,
            });
        }
        (true, true) => {
            constraints.push(LinearConstraint {
                name: format!("{name}_lower"),
                sense: ConstraintSense::GreaterEqual,
                rhs: bounds.lower,
                terms: terms.clone(),
            });
            constraints.push(LinearConstraint {
                name: format!("{name}_upper"),
                sense: ConstraintSense::LessEqual,
                rhs: bounds.upper,
                terms,
            });
        }
        (true, false) => constraints.push(LinearConstraint {
            name,
            sense: ConstraintSense::GreaterEqual,
            rhs: bounds.lower,
            terms,
        }),
        (false, true) => constraints.push(LinearConstraint {
            name,
            sense: ConstraintSense::LessEqual,
            rhs: bounds.upper,
            terms,
        }),
        (false, false) => {
            return Err(PyRuntimeError::new_err(
                "free constraints are not supported by the target-based solver seam",
            ));
        }
    }
    Ok(())
}
