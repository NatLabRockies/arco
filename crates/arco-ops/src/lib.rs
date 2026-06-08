//! Operations facade seam for Arco interaction surfaces.

pub mod benchmark;
pub mod compile;
pub mod dto;
pub mod execution;
mod execution_backends;
pub mod inspect;

/// Stable diagnostic vocabulary exposed through the ops seam.
pub mod diagnostics {
    pub use arco_diagnostics::codes;
}

use crate::compile::compile::{LinearTerm as TargetLinearTerm, SolveTarget};
use crate::compile::pipeline::{
    CompiledProgram, PipelineError, ValidatedProgram, compile_file, validate_file,
};
use crate::compile::targets::{
    AlgebraicProblem, ConstraintSense, ObjectiveSense as TargetObjectiveSense, VariableKind,
};
pub use arco_format::ExportError;
use arco_format::{
    PortableConstraintSense, PortableLinearConstraint, PortableLinearObjective,
    PortableLinearReport, PortableLinearTerm, PortableObjectiveSense, PortableProblem,
    PortableVariableInstance, PortableVariableKind, write_lp, write_mps,
};
use arco_kdl as kdl;
use arco_kdl::PrimitiveBuildError;
use arco_kdl::source::{ParsedSource, SourceError, format_program_text, parse_program_file};
/// Stable model-facing vocabulary exposed through the ops seam.
pub mod modeling {
    pub use arco_model::{
        ElasticHandle, InspectOptions, Model, ModelPatch, ModelSnapshot, ModelView, Objective,
        PatchedModelView, Sense, SimplifyLevel, SlackBound, SlackHandle, SnapshotMemoryEstimate,
        SnapshotMetadata, Variable,
    };

    /// Model-core types intentionally surfaced through ops.
    pub mod model {
        pub use arco_model::model::SparseMatrixExport;
        pub use arco_model::{
            CscInput, ModelError, PrettyBoundGroup, PrettyPrintAdapter, PrettyPrintOptions,
            PrettySection, format_ascii_number,
        };
    }

    /// Primitive value types intentionally surfaced through ops.
    pub mod types {
        pub use arco_model::{Bounds, Constraint, Variable};
    }

    /// Slack helper types intentionally surfaced through ops.
    pub mod slack {
        pub use arco_model::SlackVariables;
    }
}

/// Stable expression vocabulary exposed through the ops seam.
pub mod expression {
    pub use arco_model::expr::{ComparisonSense, ConstraintExpr, Expr, LinearExprError};
    pub use arco_model::{ConstraintId, VariableId};
}

/// Public nonlinear-programming surface exposed for embedded callers (Python
/// bindings). Re-exports the nonlinear expression IR and the free-standing
/// IPOPT entry point that operates on a `NonlinearProblem` plus per-variable
/// bounds.
#[cfg(feature = "ipopt")]
pub mod nlp {
    pub use crate::compile::compile::{
        ConstraintSense, NonlinearConstraint, NonlinearExpr, NonlinearObjective, NonlinearProblem,
        NonlinearReport,
    };
    pub use crate::execution::{
        NlpError, NlpOptions, NlpSolution, NlpVariableSpec, solve_nonlinear_problem,
    };
    pub use arco_kdl::ObjectiveSense;
    pub use arco_kdl::algebra::{BinaryOp, UnaryOp};
}

/// Stable solver-facing operations vocabulary exposed through the ops seam.
pub mod solve {
    pub use arco_solver::{
        ModelViewBackend, ModelViewBackendRegistry, ModelViewSolveResult, ResolvedSelection,
        SelectionError, Solution, SolutionView, Solve, SolveRequest, SolverCapabilityModel,
        SolverConfig, SolverConfigDocument, SolverDiagnostic, SolverError, SolverFamily,
        SolverModelStats, SolverProfile, SolverRegistry, SolverRequirements, SolverSelection,
        SolverStatus, SolverTransport, merged_profiles, preflight_model_view, preflight_selection,
        resolve_selection,
    };
}

use arco_solver::{
    ModelViewBackendRegistry, ModelViewSolveResult, PreflightError, ResolvedSelection,
    SelectionError, Solution, SolutionView, Solve, SolveRequest, SolverConfig, SolverError,
    SolverProfile, SolverRegistry, SolverRequirements, SolverSelection,
};
use arco_validate::{ValidationIssue, ValidationReport, ValidationSeverity, validate_solve_target};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

/// Validation severity for the operations facade.
pub type OpsValidationSeverity = ValidationSeverity;

/// KDL source error type for the operations facade.
pub type OpsSourceError = SourceError;

/// Validation issue for the operations facade.
pub type OpsValidationIssue = ValidationIssue;

/// Validation report for the operations facade.
pub type OpsValidationReport = ValidationReport;

/// Compile/check error exposed by the operations facade.
pub type OpsCompileError = PipelineError;

pub use crate::dto::{
    OpsAlgebraicProblem, OpsConstraintSense, OpsLinearConstraint, OpsLinearObjective,
    OpsLinearReport, OpsLinearTerm, OpsObjectiveSense, OpsVariableInstance, OpsVariableKind,
};

/// Errors emitted when exporting a model file.
#[derive(Debug, Error, miette::Diagnostic)]
pub enum OpsExportFileError {
    #[error(transparent)]
    PrimitiveBuild(#[from] PrimitiveBuildError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error(transparent)]
    Compile(Box<PipelineError>),
}

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
    pub fn load_file(path: &Path) -> Result<ParsedSource, OpsSourceError> {
        parse_program_file(path)
    }

    /// Format KDL source text through the canonical formatter.
    pub fn format_kdl_text(text: &str) -> Result<String, String> {
        format_program_text(text).map_err(|error| error.to_string())
    }

    /// Check a KDL model file through semantic validation.
    pub fn check_file(path: &Path) -> Result<ValidatedProgram, OpsCompileError> {
        validate_file(path)
    }

    /// Compile a KDL model file to Arco's algebraic problem representation.
    pub fn compile_file(path: &Path) -> Result<CompiledProgram, OpsCompileError> {
        compile_file(path)
    }

    /// Export an algebraic problem to a text interchange format.
    pub fn export_problem(
        problem: &OpsAlgebraicProblem,
        format: OpsExportFormat,
    ) -> Result<Vec<u8>, ExportError> {
        let mut buffer = Vec::new();
        match format {
            OpsExportFormat::Lp => {
                write_lp(&portable_problem_from_ops(problem), &mut buffer)?;
            }
            OpsExportFormat::Mps => {
                write_mps(&portable_problem_from_ops(problem), &mut buffer)?;
            }
        }
        Ok(buffer)
    }

    /// Export a KDL model file through the legacy algebraic export path.
    pub fn export_model_file(
        path: &Path,
        format: OpsExportFormat,
    ) -> Result<Vec<u8>, OpsExportFileError> {
        let compiled = Self::compile_file(path)
            .map_err(|error| OpsExportFileError::Compile(Box::new(error)))?;
        let problem = ops_problem_from_algebraic(&compiled.compiled_problem.algebra);
        Self::export_problem(&problem, format).map_err(OpsExportFileError::Export)
    }

    /// Solve through a solver implementation using the shared solver contract.
    pub fn solve<S>(solver: &mut S, config: &SolverConfig) -> Result<S::Solution, SolverError>
    where
        S: Solve,
        S::Solution: SolutionView,
    {
        solver.solve(config)
    }

    /// Build solver registry with builtin abstract families from solver contracts.
    pub fn solver_registry_with_builtin_families() -> SolverRegistry {
        crate::execution_backends::solver_registry_with_builtin_families()
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

    /// Construct a builtin execution adapter for a resolved solver selection.
    pub fn builtin_adapter_for_selection(
        selection: &ResolvedSelection,
        log_to_console: bool,
        profile: Option<&SolverProfile>,
    ) -> Result<Box<dyn execution::OptimizationAdapter>, String> {
        execution::builtin_adapter_for_selection(selection, log_to_console, profile)
    }

    /// Compatibility shim: arco-ops no longer embeds concrete model-view backends.
    pub fn solve_model_view_with_builtin_backend(
        family: &str,
        model: &dyn arco_model::ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        crate::execution_backends::solve_model_view_with_builtin_backend(family, model, config)
    }

    /// Solve an owned in-memory model through a builtin backend.
    ///
    /// This is intended for memory-sensitive callers that will not use the
    /// model after solver handoff.
    pub fn solve_owned_model_with_builtin_backend(
        family: &str,
        model: arco_model::Model,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        crate::execution_backends::solve_owned_model_with_builtin_backend(family, model, config)
    }

    /// Return the version for a builtin backend family when available.
    pub fn builtin_solver_version(family: &str) -> Option<String> {
        crate::execution_backends::builtin_solver_version(family)
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

    /// Execute a compiled problem using a caller-supplied adapter.
    pub fn execute_compiled_problem_with_adapter(
        problem: &compile::compile::CompiledProblem,
        adapter: &dyn execution::OptimizationAdapter,
        include_variable_values: bool,
    ) -> Result<execution::ExecutionResult, execution::ExecutionError> {
        execution::execute_problem_with_options(problem, adapter, include_variable_values)
    }
}

/// Extract a source span from KDL source errors that carry declaration locations.
pub fn source_error_span(error: &OpsSourceError) -> Option<miette::SourceSpan> {
    match error {
        SourceError::MissingNode { span, .. }
        | SourceError::MissingArgument { span, .. }
        | SourceError::MissingProperty { span, .. }
        | SourceError::InvalidValue { span, .. }
        | SourceError::UnsupportedDeclaration { span, .. }
        | SourceError::InvalidInclude { span, .. }
        | SourceError::InvalidAlgebra { span, .. } => Some(*span),
        SourceError::Io { .. } | SourceError::Kdl { .. } => None,
    }
}

/// Copy a compile-internal algebraic problem into the stable ops DTO.
pub fn ops_problem_from_algebraic(problem: &AlgebraicProblem) -> OpsAlgebraicProblem {
    OpsAlgebraicProblem {
        variable_instances: problem
            .variable_instances
            .iter()
            .map(|variable| OpsVariableInstance {
                name: variable.name.clone(),
                family: variable.family.clone(),
                lower: variable.lower,
                upper: variable.upper,
                kind: match variable.kind {
                    VariableKind::Continuous => OpsVariableKind::Continuous,
                    VariableKind::Integer => OpsVariableKind::Integer,
                    VariableKind::Binary => OpsVariableKind::Binary,
                },
            })
            .collect(),
        constraints: problem
            .constraints
            .iter()
            .map(|constraint| OpsLinearConstraint {
                name: constraint.name.clone(),
                sense: match constraint.sense {
                    ConstraintSense::GreaterEqual => OpsConstraintSense::GreaterEqual,
                    ConstraintSense::LessEqual => OpsConstraintSense::LessEqual,
                    ConstraintSense::Equal => OpsConstraintSense::Equal,
                },
                rhs: constraint.rhs,
                terms: ops_terms(&constraint.terms),
            })
            .collect(),
        objective: OpsLinearObjective {
            name: problem.objective.name.clone(),
            sense: match problem.objective.sense {
                TargetObjectiveSense::Minimize => OpsObjectiveSense::Minimize,
                TargetObjectiveSense::Maximize => OpsObjectiveSense::Maximize,
            },
            constant: problem.objective.constant,
            terms: ops_terms(&problem.objective.terms),
        },
        reports: problem
            .reports
            .iter()
            .map(|report| OpsLinearReport {
                name: report.name.clone(),
                constant: report.constant,
                terms: ops_terms(&report.terms),
            })
            .collect(),
    }
}

pub fn portable_problem_from_ops(problem: &OpsAlgebraicProblem) -> PortableProblem {
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
                    OpsVariableKind::Continuous => PortableVariableKind::Continuous,
                    OpsVariableKind::Integer => PortableVariableKind::Integer,
                    OpsVariableKind::Binary => PortableVariableKind::Binary,
                },
            })
            .collect(),
        constraints: problem
            .constraints
            .iter()
            .map(|constraint| PortableLinearConstraint {
                name: constraint.name.clone(),
                sense: match constraint.sense {
                    OpsConstraintSense::GreaterEqual => PortableConstraintSense::GreaterEqual,
                    OpsConstraintSense::LessEqual => PortableConstraintSense::LessEqual,
                    OpsConstraintSense::Equal => PortableConstraintSense::Equal,
                },
                rhs: constraint.rhs,
                terms: portable_terms(&constraint.terms),
            })
            .collect(),
        objective: PortableLinearObjective {
            name: problem.objective.name.clone(),
            sense: match problem.objective.sense {
                OpsObjectiveSense::Minimize => PortableObjectiveSense::Minimize,
                OpsObjectiveSense::Maximize => PortableObjectiveSense::Maximize,
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

fn ops_terms(terms: &[TargetLinearTerm]) -> Vec<OpsLinearTerm> {
    terms
        .iter()
        .map(|term| OpsLinearTerm {
            variable_name: term.variable_name.clone(),
            coefficient: term.coefficient,
        })
        .collect()
}

fn portable_terms(terms: &[OpsLinearTerm]) -> Vec<PortableLinearTerm> {
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
    use super::{ArcoOps, OpsExportFormat, source_error_span};
    use crate::compile::compile::SolveTarget;
    use arco_model::ModelView;
    use arco_solver::{
        ModelViewBackend, ModelViewBackendRegistry, ModelViewSolveResult, SolutionView, Solve,
        SolverConfig, SolverError, SolverRegistry, SolverRequirements, SolverSelection,
        SolverStatus, SolverTransport,
    };
    use miette::{NamedSource, SourceOffset};
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
                variable_duals: Vec::new(),
                row_values: Vec::new(),
                constraint_duals: Vec::new(),
                metadata: Default::default(),
            })
        }
    }

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/dense-lp/input.kdl")
    }

    fn composed_fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/composition/input.kdl")
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
    fn compile_file_lowers_composed_kdl_source() {
        let compiled =
            ArcoOps::compile_file(&composed_fixture_path()).expect("fixture should compile");

        assert_eq!(compiled.semantic_program.active_scenario, "Base");
        assert_eq!(
            compiled.compiled_problem.algebra.variable_instances.len(),
            2
        );
        assert_eq!(compiled.compiled_problem.algebra.constraints.len(), 2);
    }

    #[test]
    fn export_problem_writes_lp_bytes() {
        let compiled = ArcoOps::compile_file(&fixture_path()).expect("fixture should compile");
        let problem = crate::ops_problem_from_algebraic(&compiled.compiled_problem.algebra);
        let exported =
            ArcoOps::export_problem(&problem, OpsExportFormat::Lp).expect("fixture should export");

        assert!(String::from_utf8_lossy(&exported).starts_with("\\ Problem name: MODEL"));
    }

    #[test]
    fn export_model_file_uses_legacy_algebraic_path() {
        let exported = ArcoOps::export_model_file(&fixture_path(), OpsExportFormat::Lp)
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
        assert_eq!(report.issues[0].code, "arco::target::empty_variable_set");
    }

    #[test]
    fn validate_target_accepts_targets_with_variables() {
        let report = ArcoOps::validate_target(&SolveTarget::new("ok", 2, 1));

        assert!(report.is_valid());
        assert!(report.issues.is_empty());
    }

    #[test]
    fn source_error_span_returns_span_for_declaration_errors() {
        let source = NamedSource::new("test.kdl", "model demo {}".to_string());
        let error = crate::kdl::source::SourceError::UnsupportedDeclaration {
            name: "legacy_decl".to_string(),
            path: PathBuf::from("test.kdl"),
            source_text: Box::new(source),
            span: (SourceOffset::from(0), 5).into(),
        };

        let span = source_error_span(&error).expect("unsupported declaration should include span");
        assert_eq!(span.offset(), 0);
        assert_eq!(span.len(), 5);
    }

    #[test]
    fn source_error_span_returns_none_for_io_errors() {
        let error = crate::kdl::source::SourceError::Io {
            path: PathBuf::from("missing.kdl"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        };

        assert!(source_error_span(&error).is_none());
    }
}
