use crate::{ModelViewSolveResult, SolverConfig, SolverError};
use arco_model::ModelView;
use std::collections::BTreeMap;

/// Adapter-neutral backend that solves primitive model views.
pub trait ModelViewBackend: Send + Sync {
    /// Stable solver family name used for registry lookup.
    fn family(&self) -> &'static str;

    /// Solve a primitive model view with the supplied solver configuration.
    fn solve_model_view(
        &self,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError>;
}

/// Borrowed registry for primitive model-view solve backends.
#[derive(Default)]
pub struct ModelViewBackendRegistry<'a> {
    backends: BTreeMap<&'static str, &'a dyn ModelViewBackend>,
}

impl<'a> ModelViewBackendRegistry<'a> {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a concrete backend implementation by its solver family.
    pub fn register(&mut self, backend: &'a dyn ModelViewBackend) {
        self.try_register(backend)
            .expect("duplicate primitive model-view backend family");
    }

    /// Register a backend and reject duplicate solver families.
    pub fn try_register(&mut self, backend: &'a dyn ModelViewBackend) -> Result<(), SolverError> {
        let family = backend.family();
        if self.backends.contains_key(family) {
            return Err(SolverError::InvalidSettings(format!(
                "primitive model-view backend family '{family}' is already registered"
            )));
        }
        self.backends.insert(family, backend);
        Ok(())
    }

    /// Registered backend families in deterministic order.
    pub fn families(&self) -> Vec<&'static str> {
        self.backends.keys().copied().collect()
    }

    /// Register a concrete backend implementation by family, replacing any
    /// existing backend for the same family. This is intended only for tests or
    /// explicit override flows.
    pub fn replace(&mut self, backend: &'a dyn ModelViewBackend) {
        self.backends.insert(backend.family(), backend);
    }

    /// Return true when the family has a registered primitive backend.
    pub fn contains_family(&self, family: &str) -> bool {
        self.backends.contains_key(family)
    }

    /// Solve with a registered backend family.
    pub fn solve(
        &self,
        family: &str,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        let backend = self.backends.get(family).ok_or_else(|| {
            SolverError::SolverNotAvailable(format!(
                "no primitive model-view backend registered for '{family}'"
            ))
        })?;
        let result = backend.solve_model_view(model, config)?;
        validate_model_view_solve_result_with_config(model, &result, config)?;
        Ok(result)
    }
}

/// Validate that a backend result uses ModelView variable/constraint ordering.
pub fn validate_model_view_solve_result(
    model: &(impl ModelView + ?Sized),
    result: &ModelViewSolveResult,
) -> Result<(), SolverError> {
    validate_model_view_solve_result_shape(model, result, false)
}

/// Validate a backend result against the supplied solver configuration.
///
/// Solver results normally include primal values. Callers may opt into an
/// objective-only result with the private `arco.extract_solution=false` option
/// to avoid large post-solve solution vectors when only status/objective
/// metadata are needed.
pub fn validate_model_view_solve_result_with_config(
    model: &(impl ModelView + ?Sized),
    result: &ModelViewSolveResult,
    config: &SolverConfig,
) -> Result<(), SolverError> {
    validate_model_view_solve_result_shape(model, result, allows_omitted_primal_values(config))
}

fn validate_model_view_solve_result_shape(
    model: &(impl ModelView + ?Sized),
    result: &ModelViewSolveResult,
    allow_omitted_primal_values: bool,
) -> Result<(), SolverError> {
    let expected_fingerprint = model.fingerprint();
    if result.fingerprint.0 != 0 && result.fingerprint != expected_fingerprint {
        return Err(SolverError::InvalidResultShape(
            "result fingerprint does not match input model fingerprint".to_string(),
        ));
    }
    if allow_omitted_primal_values {
        validate_optional_len(
            "primal_values",
            result.primal_values.len(),
            model.num_variables(),
        )?;
    } else {
        validate_required_len(
            "primal_values",
            result.primal_values.len(),
            model.num_variables(),
        )?;
    }
    validate_optional_len(
        "variable_duals",
        result.variable_duals.len(),
        model.num_variables(),
    )?;
    validate_optional_len(
        "row_values",
        result.row_values.len(),
        model.num_constraints(),
    )?;
    validate_optional_len(
        "constraint_duals",
        result.constraint_duals.len(),
        model.num_constraints(),
    )?;
    Ok(())
}

fn allows_omitted_primal_values(config: &SolverConfig) -> bool {
    config
        .parameters
        .get("arco.extract_solution")
        .is_some_and(|value| value == "false")
}

fn validate_required_len(name: &str, actual: usize, expected: usize) -> Result<(), SolverError> {
    if actual == expected {
        return Ok(());
    }
    Err(SolverError::InvalidResultShape(format!(
        "{name} length {actual} does not match expected {expected}"
    )))
}

fn validate_optional_len(name: &str, actual: usize, expected: usize) -> Result<(), SolverError> {
    if actual == 0 || actual == expected {
        return Ok(());
    }
    Err(SolverError::InvalidResultShape(format!(
        "{name} length {actual} must be 0 or match expected {expected}"
    )))
}

#[cfg(test)]
mod tests {
    use crate::{
        ModelViewBackend, ModelViewBackendRegistry, ModelViewSolveResult, SolverConfig,
        SolverError, SolverStatus,
    };
    use arco_model::{
        Bounds, Model, ModelFingerprint, ModelView, Objective, Sense, Variable, expr::Expr,
    };
    use std::sync::Mutex;

    struct FixtureBackend;

    impl ModelViewBackend for FixtureBackend {
        fn family(&self) -> &'static str {
            "fixture"
        }

        fn solve_model_view(
            &self,
            model: &dyn ModelView,
            _config: &SolverConfig,
        ) -> Result<ModelViewSolveResult, SolverError> {
            Ok(ModelViewSolveResult {
                fingerprint: model.fingerprint(),
                status: SolverStatus::Optimal,
                objective_value: 1.0,
                primal_values: vec![0.0; model.num_variables()],
                variable_duals: Vec::new(),
                row_values: Vec::new(),
                constraint_duals: Vec::new(),
                metadata: Default::default(),
            })
        }
    }

    #[test]
    fn registry_routes_model_view_solves_by_family() {
        let backend = FixtureBackend;
        let mut registry = ModelViewBackendRegistry::new();
        registry.register(&backend);
        let model = Model::new();

        let result = registry
            .solve("fixture", &model, &SolverConfig::default())
            .expect("registered backend should solve");

        assert_eq!(result.status, SolverStatus::Optimal);
        assert_eq!(result.fingerprint, model.fingerprint());
    }

    #[test]
    fn registry_reports_missing_family() {
        let registry = ModelViewBackendRegistry::new();
        let model = Model::new();

        let error = registry
            .solve("missing", &model, &SolverConfig::default())
            .expect_err("missing backend should fail");

        assert!(matches!(error, SolverError::SolverNotAvailable(_)));
    }

    struct BadShapeBackend;

    impl ModelViewBackend for BadShapeBackend {
        fn family(&self) -> &'static str {
            "bad_shape"
        }

        fn solve_model_view(
            &self,
            model: &dyn ModelView,
            _config: &SolverConfig,
        ) -> Result<ModelViewSolveResult, SolverError> {
            Ok(ModelViewSolveResult {
                fingerprint: model.fingerprint(),
                status: SolverStatus::Optimal,
                objective_value: 0.0,
                primal_values: Vec::new(),
                variable_duals: Vec::new(),
                row_values: Vec::new(),
                constraint_duals: Vec::new(),
                metadata: Default::default(),
            })
        }
    }

    #[test]
    fn registry_rejects_backend_result_shape_mismatches() {
        let backend = BadShapeBackend;
        let mut registry = ModelViewBackendRegistry::new();
        registry.register(&backend);
        let mut model = Model::new();
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("add variable");

        let error = registry
            .solve("bad_shape", &model, &SolverConfig::default())
            .expect_err("bad result shape should fail");

        assert!(matches!(error, SolverError::InvalidResultShape(_)));
    }

    #[test]
    fn registry_accepts_objective_only_result_when_solution_extraction_is_disabled() {
        let backend = BadShapeBackend;
        let mut registry = ModelViewBackendRegistry::new();
        registry.register(&backend);
        let mut model = Model::new();
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("add variable");
        let config = SolverConfig::new().with_parameter("arco.extract_solution", "false");

        let result = registry
            .solve("bad_shape", &model, &config)
            .expect("objective-only result should be accepted when explicitly requested");

        assert!(result.primal_values.is_empty());
        assert!(result.objective_value.abs() <= f64::EPSILON);
    }

    #[test]
    fn validation_accepts_zero_fingerprint_sentinel_when_lengths_match() {
        let mut model = Model::new();
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("add variable");

        let result = ModelViewSolveResult {
            fingerprint: ModelFingerprint(0),
            status: SolverStatus::Optimal,
            objective_value: 0.0,
            primal_values: vec![0.0],
            variable_duals: Vec::new(),
            row_values: Vec::new(),
            constraint_duals: Vec::new(),
            metadata: Default::default(),
        };

        super::validate_model_view_solve_result(&model, &result)
            .expect("zero fingerprint sentinel should skip only fingerprint validation");
    }

    #[test]
    fn registry_rejects_duplicate_backend_family() {
        let first = FixtureBackend;
        let second = FixtureBackend;
        let mut registry = ModelViewBackendRegistry::new();

        registry
            .try_register(&first)
            .expect("first registration should succeed");
        let error = registry
            .try_register(&second)
            .expect_err("duplicate family should fail");

        assert!(matches!(error, SolverError::InvalidSettings(_)));
        assert_eq!(registry.families(), vec!["fixture"]);
    }

    struct RecordingBackend {
        config: Mutex<Option<SolverConfig>>,
    }

    impl RecordingBackend {
        fn new() -> Self {
            Self {
                config: Mutex::new(None),
            }
        }
    }

    impl ModelViewBackend for RecordingBackend {
        fn family(&self) -> &'static str {
            "recording"
        }

        fn solve_model_view(
            &self,
            model: &dyn ModelView,
            config: &SolverConfig,
        ) -> Result<ModelViewSolveResult, SolverError> {
            *self.config.lock().expect("record config") = Some(config.clone());
            Ok(ModelViewSolveResult {
                fingerprint: model.fingerprint(),
                status: SolverStatus::Optimal,
                objective_value: 3.0,
                primal_values: vec![1.0; model.num_variables()],
                variable_duals: vec![0.0; model.num_variables()],
                row_values: vec![2.0; model.num_constraints()],
                constraint_duals: vec![4.0; model.num_constraints()],
                metadata: Default::default(),
            })
        }
    }

    #[test]
    fn registry_preserves_config_and_model_ordered_result_shapes() {
        let backend = RecordingBackend::new();
        let mut registry = ModelViewBackendRegistry::new();
        registry.register(&backend);
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .expect("add variable");
        model
            .add_expr_constraint(Expr::var(x), Bounds::new(2.0, f64::INFINITY))
            .expect("add constraint");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 3.0)],
            })
            .expect("set objective");
        let config = SolverConfig::new()
            .with_time_limit(10.0)
            .with_threads(2)
            .with_parameter("fixture.option", "enabled");

        let result = registry
            .solve("recording", &model, &config)
            .expect("registered backend should solve");
        let captured = backend
            .config
            .lock()
            .expect("read config")
            .clone()
            .expect("config captured");

        assert_eq!(captured, config);
        assert_eq!(result.primal_values.len(), model.num_variables());
        assert_eq!(result.variable_duals.len(), model.num_variables());
        assert_eq!(result.row_values.len(), model.num_constraints());
        assert_eq!(result.constraint_duals.len(), model.num_constraints());
        assert_eq!(result.fingerprint, model.fingerprint());
    }
}
