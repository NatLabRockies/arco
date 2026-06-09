//! HiGHS solver implementation over solver-facing targets.

use crate::status::highs_model_status;
use arco_model::{ConstraintId, ModelFingerprint, ModelView, Sense, VariableId};
use arco_solver::{
    ModelViewBackend, ModelViewSolveResult, SolverConfig, SolverStatusMapping,
    validate_model_view_solve_result,
};
use highs::{ColProblem, Model as RawHighsModel, Row as HighsRow, Sense as HighsSense};
use std::collections::BTreeMap;
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
        highs_model.make_quiet();
    }
    if let Some(level) = config.verbosity {
        highs_model.set_option("output_flag", level > 0);
    }
    if config.log_to_console.unwrap_or(false) {
        highs_model.set_option("log_to_console", true);
        highs_model.set_option("output_flag", true);
    }
    if let Some(limit) = config.time_limit {
        highs_model.set_option("time_limit", limit);
    }
    if let Some(gap) = config.mip_gap {
        highs_model.set_option("mip_rel_gap", gap);
    }
    if let Some(presolve) = config.presolve {
        highs_model.set_option("presolve", if presolve { "on" } else { "off" });
    }
    if let Some(threads) = config.threads {
        highs_model.set_option("threads", threads as i32);
    }
    if let Some(tolerance) = config.tolerance {
        highs_model.set_option("primal_feasibility_tolerance", tolerance);
        highs_model.set_option("dual_feasibility_tolerance", tolerance);
    }
    for (key, value) in &config.parameters {
        if key.starts_with("arco.") {
            continue;
        }
        highs_model.set_option(key.as_str(), value.as_str());
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
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let matrix_start = Instant::now();
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

    let mut objective_coefficients = vec![0.0; model.num_variables()];
    for (variable_id, coefficient) in &model.objective().terms {
        let index = variable_id.inner() as usize;
        if index < objective_coefficients.len() {
            objective_coefficients[index] = *coefficient;
        }
    }

    for (index, objective) in objective_coefficients.into_iter().enumerate() {
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(index as u32))?;
        let Some(column) = model.column(variable_id) else {
            if variable.is_integer {
                problem.add_integer_column(
                    objective,
                    variable.bounds.lower..=variable.bounds.upper,
                    Vec::<(HighsRow, f64)>::new(),
                );
            } else {
                problem.add_column(
                    objective,
                    variable.bounds.lower..=variable.bounds.upper,
                    Vec::<(HighsRow, f64)>::new(),
                );
            }
            continue;
        };
        let factors = column
            .iter()
            .map(|(constraint_id, coefficient)| {
                let row_index = constraint_id.inner() as usize;
                rows.get(row_index)
                    .copied()
                    .map(|row| (row, *coefficient))
                    .ok_or_else(|| {
                        SolverError::SolverSpecific(format!(
                            "constraint ID {row_index} does not exist"
                        ))
                    })
            })
            .collect::<Result<Vec<_>, SolverError>>()?;
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

    let matrix_build_seconds = matrix_start.elapsed().as_secs_f64();

    let sense = match model.objective().sense.unwrap_or(Sense::Minimize) {
        Sense::Minimize => HighsSense::Minimise,
        Sense::Maximize => HighsSense::Maximise,
    };
    let mut highs_model = problem.optimise(sense);
    apply_solver_config(&mut highs_model, config)?;
    let highs_run_start = Instant::now();
    let solved = highs_model.solve();
    let highs_run_seconds = highs_run_start.elapsed().as_secs_f64();
    let status = solved.status();
    let mapped_status = highs_model_status(status);
    if !mapped_status.has_solution() {
        return Err(SolverError::SolveFailure {
            status: mapped_status.to_solver_status(),
        });
    }
    let objective_value = solved.objective_value();
    let extract_solution = config
        .parameters
        .get("arco.extract_solution")
        .is_none_or(|value| value != "false");
    let solution_extract_start = Instant::now();
    let (primal_values, variable_duals, row_values, constraint_duals) = if extract_solution {
        let solution = solved.get_solution();
        (
            solution.columns().to_vec(),
            solution.dual_columns().to_vec(),
            solution.rows().to_vec(),
            solution.dual_rows().to_vec(),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };
    let solution_extract_seconds = solution_extract_start.elapsed().as_secs_f64();

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

    let mut metadata = BTreeMap::new();
    metadata.insert("highs_matrix_build_s".to_string(), matrix_build_seconds);
    metadata.insert("highs_run_s".to_string(), highs_run_seconds);
    metadata.insert("solution_extract_s".to_string(), solution_extract_seconds);
    metadata.insert("fingerprint_s".to_string(), fingerprint_seconds);
    metadata.insert("num_variables".to_string(), model.num_variables() as f64);
    metadata.insert(
        "num_constraints".to_string(),
        model.num_constraints() as f64,
    );
    metadata.insert(
        "num_coefficients".to_string(),
        model.num_coefficients() as f64,
    );

    let result = ModelViewSolveResult {
        fingerprint,
        status: mapped_status.to_solver_status(),
        objective_value,
        primal_values,
        variable_duals,
        row_values,
        constraint_duals,
        metadata,
    };
    validate_model_view_solve_result(model, &result)?;
    Ok(result)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_model::{Bounds, Constraint, Model, ModelView, Objective, Sense, Variable};
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
}
