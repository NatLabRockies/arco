//! HiGHS solver implementation over solver-facing targets.

use crate::ffi::{HighsModel, HighsModelError, HighsOption, ObjectiveSense};
use crate::status::{highs_has_solution, highs_to_core_status};
use arco_model::{ConstraintId, ModelView, Sense, VariableId};
use arco_solver::{ModelViewBackend, ModelViewSolveResult, SolverConfig};
use std::collections::BTreeMap;

/// Re-export of contract solver error for backward compatibility.
pub type SolverError = arco_solver::SolverError;

fn highs_model_error_to_solver_error(err: HighsModelError) -> SolverError {
    SolverError::SolverSpecific(err.to_string())
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
        highs_model.set_option(
            "presolve",
            HighsOption::Str(if presolve { "on" } else { "off" }.to_string()),
        );
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
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }

    let mut highs_model = HighsModel::new();
    apply_solver_config(&mut highs_model, config)?;
    highs_model.set_objective_sense(match model.objective().sense.unwrap_or(Sense::Minimize) {
        Sense::Minimize => ObjectiveSense::Minimize,
        Sense::Maximize => ObjectiveSense::Maximize,
    });

    let objective_terms = model
        .objective()
        .terms
        .iter()
        .copied()
        .collect::<BTreeMap<VariableId, f64>>();
    for index in 0..model.num_variables() {
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(index as u32))?;
        let objective = objective_terms.get(&variable_id).copied().unwrap_or(0.0);
        if variable.is_integer {
            highs_model.add_integer_col(variable.bounds.lower, variable.bounds.upper, objective);
        } else {
            highs_model.add_col(variable.bounds.lower, variable.bounds.upper, objective);
        }
    }

    let mut rows: Vec<(Vec<usize>, Vec<f64>)> =
        vec![(Vec::new(), Vec::new()); model.num_constraints()];
    for index in 0..model.num_variables() {
        let variable_id = VariableId::new(index as u32);
        let Some(column) = model.column(variable_id) else {
            continue;
        };
        for (constraint_id, coefficient) in column {
            let row_index = constraint_id.inner() as usize;
            if row_index >= rows.len() {
                return Err(SolverError::SolverSpecific(format!(
                    "constraint ID {row_index} does not exist"
                )));
            }
            rows[row_index].0.push(index);
            rows[row_index].1.push(*coefficient);
        }
    }

    for (index, (columns, coefficients)) in rows.iter().enumerate() {
        let constraint = model
            .constraint(ConstraintId::new(index as u32))
            .ok_or_else(|| {
                SolverError::SolverSpecific(format!("constraint ID {index} does not exist"))
            })?;
        highs_model
            .add_row(
                constraint.bounds.lower,
                constraint.bounds.upper,
                columns,
                coefficients,
            )
            .map_err(highs_model_error_to_solver_error)?;
    }

    let status = highs_model.solve();
    if !highs_has_solution(status) {
        return Err(SolverError::SolveFailure {
            status: highs_to_core_status(status),
        });
    }
    let snapshot = highs_model
        .solution_snapshot()
        .map_err(highs_model_error_to_solver_error)?;
    let objective_value = highs_model
        .objective_value()
        .map_err(highs_model_error_to_solver_error)?;
    let (primal_values, variable_duals, row_values, constraint_duals) = snapshot.into_vecs();

    Ok(ModelViewSolveResult {
        fingerprint: model.fingerprint(),
        status: highs_to_core_status(status),
        objective_value,
        primal_values,
        variable_duals,
        row_values,
        constraint_duals,
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_model::{Bounds, Constraint, Model, ModelView, Objective, Sense, Variable};

    #[test]
    fn model_view_solver_rejects_empty_problem() {
        let model = Model::new();
        assert!(matches!(
            solve_model_view(&model, &SolverConfig::new()),
            Err(SolverError::EmptyModel)
        ));
    }

    #[test]
    fn model_view_problem_solves_directly() {
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
    }
}
