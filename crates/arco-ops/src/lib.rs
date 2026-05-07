//! Operations facade seam for Arco interaction surfaces.

pub mod benchmark;
pub mod execution;
mod execution_backends;
pub mod inspect;

pub use arco_compile as compile;
use arco_compile::pipeline::{
    CompiledProgram, PipelineError, ValidatedProgram, compile_file, validate_file,
};
pub use arco_format::ExportError;
use arco_format::{
    PortableConstraintSense, PortableLinearConstraint, PortableLinearObjective,
    PortableLinearReport, PortableLinearTerm, PortableObjectiveSense, PortableProblem,
    PortableVariableInstance, PortableVariableKind, write_lp, write_mps,
};
pub use arco_highs as highs;
pub use arco_kdl as kdl;
use arco_kdl::source::{ParsedSource, SourceError, parse_program_file};
use arco_kdl::{PrimitiveBuildError, build_model};
pub use arco_model as model;
pub use arco_model::expr;
pub use arco_scip as scip;
pub use arco_solver as solver;
use arco_solver::{
    ModelViewBackendRegistry, ModelViewSolveResult, PreflightError, ResolvedSelection,
    SelectionError, Solution, SolutionView, Solve, SolveRequest, SolverConfig, SolverError,
    SolverProfile, SolverRegistry, SolverRequirements, SolverSelection,
};
pub use arco_targets as targets;
use arco_targets::{
    AlgebraicProblem, ConstraintSense, ObjectiveSense as TargetObjectiveSense, SolveTarget,
    VariableKind,
};
use arco_validate::{ValidationIssue, ValidationReport, ValidationSeverity, validate_solve_target};
#[cfg(feature = "xpress")]
pub use arco_xpress as xpress;
use std::collections::BTreeMap;
use std::path::Path;

/// Validation severity for the operations facade.
pub type OpsValidationSeverity = ValidationSeverity;

/// Validation issue for the operations facade.
pub type OpsValidationIssue = ValidationIssue;

/// Validation report for the operations facade.
pub type OpsValidationReport = ValidationReport;

/// Export format supported by the operations facade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpsExportFormat {
    /// LP text format.
    Lp,
    /// MPS fixed text format.
    Mps,
}

/// Thin operations facade used by interaction surfaces.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ArcoOps;

impl ArcoOps {
    /// Create a new operations facade.
    pub fn new() -> Self {
        Self
    }

    /// Load and parse a KDL model file.
    pub fn load_file(path: &Path) -> Result<ParsedSource, SourceError> {
        parse_program_file(path)
    }

    /// Check a KDL model file through semantic validation.
    pub fn check_file(path: &Path) -> Result<ValidatedProgram, PipelineError> {
        validate_file(path)
    }

    /// Compile a KDL model file to Arco's algebraic problem representation.
    pub fn compile_file(path: &Path) -> Result<CompiledProgram, PipelineError> {
        compile_file(path)
    }

    /// Build a primitive frozen model directly from a KDL model file.
    pub fn build_primitive_model_file(
        path: &Path,
    ) -> Result<arco_model::Model64, PrimitiveBuildError> {
        let parsed =
            Self::load_file(path).map_err(|error| PrimitiveBuildError::UnsupportedExpression {
                context: "source parse".to_string(),
                expr: error.to_string(),
            })?;
        build_model(&parsed)
    }

    /// Export an algebraic problem to a text interchange format.
    pub fn export_problem(
        problem: &AlgebraicProblem,
        format: OpsExportFormat,
    ) -> Result<Vec<u8>, ExportError> {
        let mut buffer = Vec::new();
        match format {
            OpsExportFormat::Lp => {
                write_lp(&portable_problem_from_algebraic(problem), &mut buffer)?
            }
            OpsExportFormat::Mps => {
                write_mps(&portable_problem_from_algebraic(problem), &mut buffer)?;
            }
        }
        Ok(buffer)
    }

    /// Solve through a solver implementation using the shared solver contract.
    pub fn solve<S>(solver: &mut S, config: &SolverConfig) -> Result<S::Solution, SolverError>
    where
        S: Solve,
        S::Solution: SolutionView,
    {
        solver.solve(config)
    }

    /// Resolve a solver selection against the available registry and profiles.
    pub fn resolve_solver_selection(
        registry: &SolverRegistry,
        profiles: &BTreeMap<String, SolverProfile>,
        selection: &str,
    ) -> Result<ResolvedSelection, SelectionError> {
        arco_solver::resolve_selection(registry, profiles, selection)
    }

    /// Check that a resolved solver can satisfy a model before solving.
    pub fn preflight_solver_selection(
        registry: &SolverRegistry,
        selection: &ResolvedSelection,
        model: &arco_model::Model,
        requirements: &SolverRequirements,
    ) -> Result<(), PreflightError> {
        arco_solver::preflight_selection(registry, selection, model, requirements)
    }

    /// Solve an in-memory model through a platform backend.
    pub fn solve_model_backend(
        backend: &dyn arco_solver::SolverBackend,
        model: &arco_model::Model,
        config: &SolverConfig,
        primal_start: Option<&[(arco_model::VariableId, f64)]>,
    ) -> Result<Solution, SolverError> {
        backend.solve(model, config, primal_start)
    }

    /// Solve a primitive model view through an adapter-neutral backend registry.
    pub fn solve_model_view(
        registry: &ModelViewBackendRegistry<'_>,
        family: &str,
        model: &dyn arco_model::ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        registry.solve(family, model, config)
    }

    /// Build a minimal solve request from an optional solver selection.
    pub fn build_solve_request(selection: Option<SolverSelection>) -> SolveRequest {
        selection.map_or_else(SolveRequest::new, |value| {
            SolveRequest::new().with_selection(value)
        })
    }

    /// Run canonical validation on a lowered solve target.
    pub fn validate_target(target: &SolveTarget) -> OpsValidationReport {
        validate_solve_target(target.has_variables())
    }

    /// Execute a compiled problem using a built-in solver selected by the solver platform.
    pub fn execute_compiled_problem(
        problem: &compile::compile::CompiledProblem,
        selection: &ResolvedSelection,
        log_to_console: bool,
        profile: Option<&SolverProfile>,
        include_variable_values: bool,
    ) -> Result<execution::ExecutionResult, execution::ExecutionError> {
        let adapter = execution::builtin_adapter_for_selection(selection, log_to_console, profile)
            .map_err(|message| execution::ExecutionError::BackendNotAvailable { message })?;
        execution::execute_problem_with_options(problem, adapter.as_ref(), include_variable_values)
    }
}

fn portable_problem_from_algebraic(problem: &AlgebraicProblem) -> PortableProblem {
    PortableProblem {
        variable_instances: problem
            .variable_instances
            .iter()
            .map(|variable| PortableVariableInstance {
                name: variable.name.clone(),
                family: variable.family.clone(),
                lower: variable.lower,
                upper: variable.upper,
                kind: match variable.kind {
                    VariableKind::Continuous => PortableVariableKind::Continuous,
                    VariableKind::Integer => PortableVariableKind::Integer,
                    VariableKind::Binary => PortableVariableKind::Binary,
                },
            })
            .collect(),
        constraints: problem
            .constraints
            .iter()
            .map(|constraint| PortableLinearConstraint {
                name: constraint.name.clone(),
                sense: match constraint.sense {
                    ConstraintSense::GreaterEqual => PortableConstraintSense::GreaterEqual,
                    ConstraintSense::LessEqual => PortableConstraintSense::LessEqual,
                    ConstraintSense::Equal => PortableConstraintSense::Equal,
                },
                rhs: constraint.rhs,
                terms: portable_terms(&constraint.terms),
            })
            .collect(),
        objective: PortableLinearObjective {
            name: problem.objective.name.clone(),
            sense: match problem.objective.sense {
                TargetObjectiveSense::Minimize => PortableObjectiveSense::Minimize,
                TargetObjectiveSense::Maximize => PortableObjectiveSense::Maximize,
            },
            constant: problem.objective.constant,
            terms: portable_terms(&problem.objective.terms),
        },
        reports: problem
            .reports
            .iter()
            .map(|report| PortableLinearReport {
                name: report.name.clone(),
                constant: report.constant,
                terms: portable_terms(&report.terms),
            })
            .collect(),
    }
}

fn portable_terms(terms: &[arco_targets::LinearTerm]) -> Vec<PortableLinearTerm> {
    terms
        .iter()
        .map(|term| PortableLinearTerm {
            variable_name: term.variable_name.clone(),
            coefficient: term.coefficient,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ArcoOps, OpsExportFormat};
    use arco_model::ModelView;
    use arco_solver::{
        ModelViewBackend, ModelViewBackendRegistry, ModelViewSolveResult, SolutionView, Solve,
        SolverConfig, SolverError, SolverRegistry, SolverRequirements, SolverSelection,
        SolverStatus, SolverTransport,
    };
    use arco_targets::SolveTarget;
    use std::path::PathBuf;

    struct FixtureSolver;

    struct FixtureSolution {
        objective_value: f64,
    }

    impl SolutionView for FixtureSolution {
        fn objective_value(&self) -> f64 {
            self.objective_value
        }

        fn status(&self) -> SolverStatus {
            SolverStatus::Optimal
        }

        fn get_primal(&self, _index: usize) -> Option<f64> {
            None
        }

        fn get_variable_dual(&self, _index: usize) -> Option<f64> {
            None
        }

        fn get_constraint_dual(&self, _index: usize) -> Option<f64> {
            None
        }

        fn primal_values(&self) -> &[f64] {
            &[]
        }

        fn variable_duals(&self) -> &[f64] {
            &[]
        }

        fn constraint_duals(&self) -> &[f64] {
            &[]
        }

        fn solve_time_seconds(&self) -> f64 {
            0.0
        }
    }

    impl Solve for FixtureSolver {
        type Solution = FixtureSolution;

        fn solve(&mut self, _config: &SolverConfig) -> Result<Self::Solution, SolverError> {
            Ok(FixtureSolution {
                objective_value: 42.0,
            })
        }
    }

    struct FixtureModelViewBackend;

    impl ModelViewBackend for FixtureModelViewBackend {
        fn family(&self) -> &'static str {
            "fixture"
        }

        fn solve_model_view(
            &self,
            model: &dyn arco_model::ModelView,
            _config: &SolverConfig,
        ) -> Result<ModelViewSolveResult, SolverError> {
            Ok(ModelViewSolveResult {
                fingerprint: model.fingerprint(),
                status: SolverStatus::Optimal,
                objective_value: 7.0,
                primal_values: Vec::new(),
            })
        }
    }

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/dense-lp/input.kdl")
    }

    fn primitive_fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../arco-kdl/tests/fixtures/primitives_builds_simple_model_and_docs.kdl")
    }

    #[test]
    fn load_file_parses_kdl_source() {
        let loaded = ArcoOps::load_file(&fixture_path()).expect("fixture should load");

        assert!(!loaded.program.models.is_empty());
    }

    #[test]
    fn check_file_validates_kdl_source() {
        let checked = ArcoOps::check_file(&fixture_path()).expect("fixture should validate");

        assert!(checked.entrypoint.ends_with("examples/dense-lp/input.kdl"));
    }

    #[test]
    fn compile_file_lowers_kdl_source() {
        let compiled = ArcoOps::compile_file(&fixture_path()).expect("fixture should compile");

        assert!(
            !compiled
                .compiled_problem
                .algebra
                .variable_instances
                .is_empty()
        );
    }

    #[test]
    fn build_primitive_model_file_uses_direct_kdl_builder() {
        let model = ArcoOps::build_primitive_model_file(&primitive_fixture_path())
            .expect("primitive fixture should build");

        assert_eq!(model.num_variables(), 2);
        assert_eq!(model.num_constraints(), 1);
    }

    #[test]
    fn export_problem_writes_lp_bytes() {
        let compiled = ArcoOps::compile_file(&fixture_path()).expect("fixture should compile");
        let exported =
            ArcoOps::export_problem(&compiled.compiled_problem.algebra, OpsExportFormat::Lp)
                .expect("fixture should export");

        assert!(String::from_utf8_lossy(&exported).starts_with("\\ Problem name: MODEL"));
    }

    #[test]
    fn solve_delegates_to_shared_solver_contract() {
        let mut solver = FixtureSolver;
        let solution =
            ArcoOps::solve(&mut solver, &SolverConfig::default()).expect("solve succeeds");

        assert!((solution.objective_value() - 42.0).abs() < f64::EPSILON);
        assert!(solution.is_optimal());
    }

    #[test]
    fn solve_model_view_uses_adapter_neutral_registry() {
        let backend = FixtureModelViewBackend;
        let mut registry = ModelViewBackendRegistry::new();
        registry.register(&backend);
        let model = arco_model::Model::new();

        let solution =
            ArcoOps::solve_model_view(&registry, "fixture", &model, &SolverConfig::default())
                .expect("registered backend should solve");

        assert_eq!(solution.status, SolverStatus::Optimal);
        assert_eq!(solution.fingerprint, model.fingerprint());
    }

    #[test]
    fn build_solve_request_preserves_selection() {
        let request = ArcoOps::build_solve_request(Some(SolverSelection::profile("local-highs")));

        assert_eq!(
            request.selection,
            Some(SolverSelection::profile("local-highs"))
        );
    }

    #[test]
    fn resolve_solver_selection_uses_registry_profiles() {
        let registry = SolverRegistry::with_builtin_families();
        let profiles = std::collections::BTreeMap::new();
        let resolved = ArcoOps::resolve_solver_selection(&registry, &profiles, "highs")
            .expect("selection should resolve");

        assert_eq!(resolved.family, "highs");
        assert_eq!(resolved.transport, SolverTransport::Embedded);
    }

    #[test]
    fn preflight_solver_selection_delegates_contract_checks() {
        let registry = SolverRegistry::with_builtin_families();
        let profiles = std::collections::BTreeMap::new();
        let resolved = ArcoOps::resolve_solver_selection(&registry, &profiles, "highs")
            .expect("selection should resolve");
        let requirements = SolverRequirements::default();

        ArcoOps::preflight_solver_selection(
            &registry,
            &resolved,
            &arco_model::Model::new(),
            &requirements,
        )
        .expect("preflight should pass");
    }

    #[test]
    fn validate_target_rejects_targets_without_variables() {
        let report = ArcoOps::validate_target(&SolveTarget::new("empty", 0, 0));

        assert!(!report.is_valid());
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].code, "TARGET_EMPTY_VARIABLE_SET");
    }

    #[test]
    fn validate_target_accepts_targets_with_variables() {
        let report = ArcoOps::validate_target(&SolveTarget::new("ok", 2, 1));

        assert!(report.is_valid());
        assert!(report.issues.is_empty());
    }
}
