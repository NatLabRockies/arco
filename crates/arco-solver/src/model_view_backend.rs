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
        backend.solve_model_view(model, config)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ModelViewBackend, ModelViewBackendRegistry, ModelViewSolveResult, SolverConfig,
        SolverError, SolverStatus,
    };
    use arco_model::{Model, ModelView};

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
                primal_values: Vec::new(),
                variable_duals: Vec::new(),
                row_values: Vec::new(),
                constraint_duals: Vec::new(),
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
}
