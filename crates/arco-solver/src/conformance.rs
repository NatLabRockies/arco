//! Reusable conformance checks for primitive model-view solver backends.

use crate::{
    ModelViewBackend, ModelViewSolveResult, SolverConfig, SolverError, SolverStatus,
    validate_model_view_solve_result,
};
use arco_model::{Bounds, Constraint, Model, Objective, Sense, Variable};

/// Result of a successful small-LP backend conformance check.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendConformanceReport {
    pub family: &'static str,
    pub objective_value: f64,
    pub variables: usize,
    pub constraints: usize,
    pub coefficients: usize,
}

/// Build the canonical one-variable LP used by backend conformance checks.
///
/// The problem is `min 2x` subject to `x >= 1`, `x >= 0`.
pub fn small_lp_model() -> Model {
    let mut model = Model::new();
    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .expect("conformance LP variable should be valid");
    let demand = model
        .add_constraint(Constraint {
            bounds: Bounds::new(1.0, f64::INFINITY),
        })
        .expect("conformance LP constraint should be valid");
    model
        .set_coefficient(x, demand, 1.0)
        .expect("conformance LP coefficient should be valid");
    model
        .set_objective(Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(x, 2.0)],
        })
        .expect("conformance LP objective should be valid");
    model
}

/// Build the canonical one-variable MILP used by capability checks.
///
/// The problem is `max x` subject to `x <= 1`, `x` binary.
pub fn small_milp_model() -> Model {
    let mut model = Model::new();
    let x = model
        .add_variable(Variable::binary())
        .expect("conformance MILP variable should be valid");
    let capacity = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 1.0),
        })
        .expect("conformance MILP constraint should be valid");
    model
        .set_coefficient(x, capacity, 1.0)
        .expect("conformance MILP coefficient should be valid");
    model
        .set_objective(Objective {
            sense: Some(Sense::Maximize),
            terms: vec![(x, 1.0)],
        })
        .expect("conformance MILP objective should be valid");
    model
}

/// Check that a backend rejects an empty model before attempting a solve.
pub fn check_empty_model_rejected(backend: &dyn ModelViewBackend) -> Result<(), SolverError> {
    let model = Model::new();
    match backend.solve_model_view(&model, &SolverConfig::default()) {
        Err(SolverError::EmptyModel) => Ok(()),
        Err(error) => Err(SolverError::InvalidResultShape(format!(
            "backend '{}' rejected an empty model with the wrong error: {error}",
            backend.family()
        ))),
        Ok(_) => Err(SolverError::InvalidResultShape(format!(
            "backend '{}' accepted an empty model",
            backend.family()
        ))),
    }
}

/// Check that a backend rejects a model that has variables but no objective.
pub fn check_no_objective_rejected(backend: &dyn ModelViewBackend) -> Result<(), SolverError> {
    let mut model = Model::new();
    model
        .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
        .expect("conformance no-objective variable should be valid");

    match backend.solve_model_view(&model, &SolverConfig::default()) {
        Err(SolverError::NoObjective) => Ok(()),
        Err(error) => Err(SolverError::InvalidResultShape(format!(
            "backend '{}' rejected a no-objective model with the wrong error: {error}",
            backend.family()
        ))),
        Ok(_) => Err(SolverError::InvalidResultShape(format!(
            "backend '{}' accepted a no-objective model",
            backend.family()
        ))),
    }
}

/// Check that a backend solves the canonical small LP and returns valid result
/// vectors in `ModelView` order.
pub fn check_small_lp(
    backend: &dyn ModelViewBackend,
    config: &SolverConfig,
) -> Result<BackendConformanceReport, SolverError> {
    let model = small_lp_model();
    let result = backend.solve_model_view(&model, config)?;
    validate_small_lp_result(backend.family(), &model, &result)?;

    Ok(BackendConformanceReport {
        family: backend.family(),
        objective_value: result.objective_value,
        variables: model.num_variables(),
        constraints: model.num_constraints(),
        coefficients: model.num_coefficients(),
    })
}

/// Check that a backend solves the canonical small MILP and returns valid
/// result vectors in `ModelView` order.
pub fn check_small_milp(
    backend: &dyn ModelViewBackend,
    config: &SolverConfig,
) -> Result<BackendConformanceReport, SolverError> {
    let model = small_milp_model();
    let result = backend.solve_model_view(&model, config)?;
    validate_small_milp_result(backend.family(), &model, &result)?;

    Ok(BackendConformanceReport {
        family: backend.family(),
        objective_value: result.objective_value,
        variables: model.num_variables(),
        constraints: model.num_constraints(),
        coefficients: model.num_coefficients(),
    })
}

fn validate_small_lp_result(
    family: &'static str,
    model: &Model,
    result: &ModelViewSolveResult,
) -> Result<(), SolverError> {
    validate_model_view_solve_result(model, result)?;
    if !matches!(result.status, SolverStatus::Optimal) {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned {:?} for the conformance LP instead of Optimal",
            result.status
        )));
    }
    let Some(value) = result.primal_values.first() else {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned no primal value for the conformance LP"
        )));
    };
    if !value.is_finite() || (*value - 1.0).abs() > 1e-7 {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned primal value {value} for the conformance LP; expected 1"
        )));
    }
    if !result.objective_value.is_finite() || (result.objective_value - 2.0).abs() > 1e-7 {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned objective {} for the conformance LP; expected 2",
            result.objective_value
        )));
    }
    Ok(())
}

fn validate_small_milp_result(
    family: &'static str,
    model: &Model,
    result: &ModelViewSolveResult,
) -> Result<(), SolverError> {
    validate_model_view_solve_result(model, result)?;
    if !matches!(result.status, SolverStatus::Optimal) {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned {:?} for the conformance MILP instead of Optimal",
            result.status
        )));
    }
    let Some(value) = result.primal_values.first() else {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned no primal value for the conformance MILP"
        )));
    };
    if !value.is_finite() || (*value - 1.0).abs() > 1e-7 {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned primal value {value} for the conformance MILP; expected 1"
        )));
    }
    if !result.objective_value.is_finite() || (result.objective_value - 1.0).abs() > 1e-7 {
        return Err(SolverError::InvalidResultShape(format!(
            "backend '{family}' returned objective {} for the conformance MILP; expected 1",
            result.objective_value
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_model::ModelView;

    struct PassingBackend;

    impl ModelViewBackend for PassingBackend {
        fn family(&self) -> &'static str {
            "passing"
        }

        fn solve_model_view(
            &self,
            model: &dyn arco_model::ModelView,
            _config: &SolverConfig,
        ) -> Result<ModelViewSolveResult, SolverError> {
            if model.num_variables() == 0 {
                return Err(SolverError::EmptyModel);
            }
            if model.objective().sense.is_none() && model.objective().terms.is_empty() {
                return Err(SolverError::NoObjective);
            }
            let objective_value = if matches!(model.objective().sense, Some(Sense::Maximize)) {
                1.0
            } else {
                2.0
            };
            Ok(ModelViewSolveResult {
                fingerprint: model.fingerprint(),
                status: SolverStatus::Optimal,
                objective_value,
                primal_values: vec![1.0; model.num_variables()],
                variable_duals: Vec::new(),
                row_values: Vec::new(),
                constraint_duals: Vec::new(),
                metadata: Default::default(),
            })
        }
    }

    struct AcceptsNoObjectiveBackend;

    impl ModelViewBackend for AcceptsNoObjectiveBackend {
        fn family(&self) -> &'static str {
            "accepts_no_objective"
        }

        fn solve_model_view(
            &self,
            model: &dyn arco_model::ModelView,
            _config: &SolverConfig,
        ) -> Result<ModelViewSolveResult, SolverError> {
            Ok(ModelViewSolveResult {
                fingerprint: model.fingerprint(),
                status: SolverStatus::Optimal,
                objective_value: 0.0,
                primal_values: vec![0.0; model.num_variables()],
                variable_duals: Vec::new(),
                row_values: Vec::new(),
                constraint_duals: Vec::new(),
                metadata: Default::default(),
            })
        }
    }

    #[test]
    fn small_lp_model_has_expected_shape() {
        let model = small_lp_model();

        assert_eq!(model.num_variables(), 1);
        assert_eq!(model.num_constraints(), 1);
        assert_eq!(model.num_coefficients(), 1);
    }

    #[test]
    fn small_milp_model_has_expected_shape_and_integrality() {
        let model = small_milp_model();
        let variable = model
            .variable(arco_model::VariableId::new(0))
            .expect("MILP fixture should contain first variable");

        assert_eq!(model.num_variables(), 1);
        assert_eq!(model.num_constraints(), 1);
        assert_eq!(model.num_coefficients(), 1);
        assert!(variable.is_integer);
    }

    #[test]
    fn conformance_accepts_backend_with_required_baseline_behavior() {
        let backend = PassingBackend;

        check_empty_model_rejected(&backend).expect("empty model check");
        check_no_objective_rejected(&backend).expect("no objective check");
        let report = check_small_lp(&backend, &SolverConfig::default()).expect("small LP check");
        let milp_report =
            check_small_milp(&backend, &SolverConfig::default()).expect("small MILP check");

        assert_eq!(report.family, "passing");
        assert!((report.objective_value - 2.0).abs() <= f64::EPSILON);
        assert_eq!(report.variables, 1);
        assert_eq!(report.constraints, 1);
        assert_eq!(report.coefficients, 1);
        assert_eq!(milp_report.family, "passing");
        assert!((milp_report.objective_value - 1.0).abs() <= f64::EPSILON);
        assert_eq!(milp_report.variables, 1);
        assert_eq!(milp_report.constraints, 1);
        assert_eq!(milp_report.coefficients, 1);
    }

    #[test]
    fn conformance_rejects_backend_that_accepts_no_objective_model() {
        let backend = AcceptsNoObjectiveBackend;
        let error = check_no_objective_rejected(&backend)
            .expect_err("accepting a no-objective model should fail conformance");

        assert!(matches!(error, SolverError::InvalidResultShape(_)));
    }
}
