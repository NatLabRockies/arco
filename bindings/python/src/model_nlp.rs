//! Nonlinear (IPOPT) integration on `PyModel`. Only compiled with the `ipopt`
//! feature.

use arco_ops::expression::{ConstraintId, Expr, VariableId};
use arco_ops::modeling::{Model, Sense};
use arco_ops::nlp::{
    BinaryOp, ConstraintSense, NlpOptions, NlpVariableSpec, NonlinearConstraint, NonlinearExpr,
    NonlinearObjective, NonlinearProblem, ObjectiveSense, solve_nonlinear_problem,
};
use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::PyModel;
use crate::py_modules::errors::{
    ConstraintTypeError, SolverInternalError, SolverInvalidSettingError,
};
use crate::py_modules::expr::PyConstraintExpr;
use crate::py_modules::nonlinear::{
    NlSense, PyNonlinearConstraintExpr, linear_constraint_to_nl, linear_expr_to_nl, nl_var_name,
};
use crate::py_modules::nonlinear_state::{NonlinearConstraintEntry, NonlinearObjectiveEntry};
use crate::py_modules::solution::PySolveResult;
use crate::py_modules::solver::PySolver;

// ───── add_nonlinear_constraint / objective helpers ─────────────────────────

pub(crate) fn add_nonlinear_constraint(
    model: &mut PyModel,
    expr: &Bound<'_, PyAny>,
    name: Option<String>,
) -> PyResult<()> {
    let constraint = if let Ok(nl) = expr.extract::<PyNonlinearConstraintExpr>() {
        nl
    } else if let Ok(linear) = expr.extract::<PyConstraintExpr>() {
        linear_constraint_to_nl(&linear)
    } else {
        return Err(ConstraintTypeError::new_err(
            "expected a NonlinearConstraintExpr or linear ConstraintExpr",
        ));
    };

    let entry = NonlinearConstraintEntry {
        expr: constraint.nl().clone(),
        sense: constraint.nl_sense(),
        name,
    };
    model.nonlinear_state.constraints.push(entry);
    Ok(())
}

pub(crate) fn try_set_nonlinear_objective(
    model: &mut PyModel,
    expr: &Bound<'_, PyAny>,
    sense: Sense,
    name: Option<String>,
) -> PyResult<bool> {
    use crate::py_modules::nonlinear::PyNonlinearExpr;

    let Ok(nl) = expr.extract::<PyNonlinearExpr>() else {
        return Ok(false);
    };
    model.nonlinear_state.objective = Some(NonlinearObjectiveEntry {
        expr: nl.into_inner(),
        minimize: matches!(sense, Sense::Minimize),
        name,
    });
    Ok(true)
}

// ───── IPOPT solve dispatch ─────────────────────────────────────────────────

fn collect_row_expressions(inner: &Model) -> Vec<Expr> {
    let n_constraints = inner.num_constraints();
    let mut rows: Vec<Expr> = vec![Expr::default(); n_constraints];
    for (var_id, column) in inner.columns() {
        for &(con_id, coeff) in column {
            let idx = con_id.inner() as usize;
            if idx < rows.len() {
                rows[idx].add_assign_owned(Expr::term(var_id, coeff));
            }
        }
    }
    rows
}

pub(crate) fn solve_with_ipopt(
    model: &PyModel,
    py: Python<'_>,
    solver: &PySolver,
) -> PyResult<Py<PySolveResult>> {
    let inner = &model.inner;

    // ── Variables ──
    let n_vars = inner.num_variables();
    let mut variable_specs: Vec<NlpVariableSpec> = Vec::with_capacity(n_vars);
    for i in 0..n_vars {
        let id = VariableId::new(i as u32);
        let var = inner
            .get_variable(id)
            .map_err(|e| SolverInternalError::new_err(format!("variable {i}: {e}")))?;
        if var.is_integer {
            return Err(SolverInvalidSettingError::new_err(format!(
                "IPOPT does not support integer variables (variable index {i})"
            )));
        }
        variable_specs.push(NlpVariableSpec {
            name: nl_var_name(i as u32),
            lower: var.bounds.lower,
            upper: var.bounds.upper,
            initial: None,
        });
    }

    // ── Linear constraints ──
    let row_exprs = collect_row_expressions(inner);
    let mut nl_constraints: Vec<NonlinearConstraint> = Vec::new();
    let n_constraints = inner.num_constraints();
    for ci in 0..n_constraints {
        let con_id = ConstraintId::new(ci as u32);
        let constraint = inner
            .get_constraint(con_id)
            .map_err(|e| SolverInternalError::new_err(format!("constraint {ci}: {e}")))?;
        let row_expr = &row_exprs[ci];
        let lower = constraint.bounds.lower;
        let upper = constraint.bounds.upper;
        let name = inner
            .get_constraint_name(con_id)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("__c{ci}"));

        let nl_expr = linear_expr_to_nl(row_expr);

        if (lower - upper).abs() <= f64::EPSILON && lower.is_finite() {
            nl_constraints.push(NonlinearConstraint {
                name,
                sense: ConstraintSense::Equal,
                rhs: lower,
                expression: nl_expr,
            });
        } else {
            if lower.is_finite() {
                nl_constraints.push(NonlinearConstraint {
                    name: format!("{name}__lb"),
                    sense: ConstraintSense::GreaterEqual,
                    rhs: lower,
                    expression: nl_expr.clone(),
                });
            }
            if upper.is_finite() {
                nl_constraints.push(NonlinearConstraint {
                    name: format!("{name}__ub"),
                    sense: ConstraintSense::LessEqual,
                    rhs: upper,
                    expression: nl_expr,
                });
            }
        }
    }

    // ── Nonlinear constraints ──
    for (idx, entry) in model.nonlinear_state.constraints.iter().enumerate() {
        let name = entry.name.clone().unwrap_or_else(|| format!("__nlc{idx}"));
        let sense = match entry.sense {
            NlSense::Ge => ConstraintSense::GreaterEqual,
            NlSense::Le => ConstraintSense::LessEqual,
            NlSense::Eq => ConstraintSense::Equal,
        };
        nl_constraints.push(NonlinearConstraint {
            name,
            sense,
            rhs: 0.0,
            expression: entry.expr.clone(),
        });
    }

    // ── Objective ──
    let objective = if let Some(nl_obj) = &model.nonlinear_state.objective {
        NonlinearObjective {
            name: nl_obj.name.clone().unwrap_or_else(|| "__obj".to_string()),
            sense: if nl_obj.minimize {
                ObjectiveSense::Minimize
            } else {
                ObjectiveSense::Maximize
            },
            expression: nl_obj.expr.clone(),
        }
    } else {
        let linear_obj = inner.objective();
        let sense = match linear_obj.sense {
            Some(Sense::Minimize) | None => ObjectiveSense::Minimize,
            Some(Sense::Maximize) => ObjectiveSense::Maximize,
        };
        let mut nl_expr = NonlinearExpr::Constant(0.0);
        for &(var_id, coeff) in &linear_obj.terms {
            let var_node = NonlinearExpr::Variable(nl_var_name(var_id.inner()));
            let term = if coeff == 1.0 {
                var_node
            } else {
                NonlinearExpr::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(NonlinearExpr::Constant(coeff)),
                    right: Box::new(var_node),
                }
            };
            nl_expr = NonlinearExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(nl_expr),
                right: Box::new(term),
            };
        }
        NonlinearObjective {
            name: inner
                .get_objective_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "__obj".to_string()),
            sense,
            expression: nl_expr,
        }
    };

    let problem = NonlinearProblem {
        objective,
        constraints: nl_constraints,
        reports: Vec::new(),
    };

    let options = NlpOptions {
        log_to_console: solver.settings.log_to_console.unwrap_or(false),
        max_iter: 300,
        tol: solver.settings.tolerance.unwrap_or(1e-6),
        acceptable_tol: 1e-4,
        acceptable_iter: 8,
    };

    let solution = solve_nonlinear_problem(&problem, &variable_specs, &options)
        .map_err(|e| SolverInternalError::new_err(format!("IPOPT solve failed: {e}")))?;

    let mut primal_values: Vec<f64> = vec![0.0; n_vars];
    for i in 0..n_vars {
        if let Some(&v) = solution.primal_values.get(&nl_var_name(i as u32)) {
            primal_values[i] = v;
        }
    }

    let status = match solution.status {
        arco_ops::execution::SolveStatus::Optimal => arco_ops::solve::SolverStatus::Optimal,
        arco_ops::execution::SolveStatus::Infeasible => arco_ops::solve::SolverStatus::Infeasible,
        arco_ops::execution::SolveStatus::TimeLimit => arco_ops::solve::SolverStatus::TimeLimit,
        arco_ops::execution::SolveStatus::Failed => arco_ops::solve::SolverStatus::Unknown,
    };

    let inner_solution = arco_ops::solve::Solution {
        primal_values,
        variable_duals: Vec::new(),
        constraint_duals: Vec::new(),
        row_values: Vec::new(),
        objective_value: solution.objective_value,
        status,
        solve_time_seconds: 0.0,
        metadata: std::collections::BTreeMap::new(),
    };

    Py::new(py, PySolveResult::new(inner_solution)).map_err(Into::into)
}
