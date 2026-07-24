//! Format-neutral diagnostics primitives for Arco.
//!
//! This crate is intentionally independent of authoring formats. KDL, CLI, and
//! Python layers can attach their own rendering while sharing stable codes,
//! severities, spans, and coarse provenance.

/// Stable diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(pub &'static str);

impl DiagnosticCode {
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Stable diagnostic codes shared by validation and user-facing surfaces.
pub mod codes {
    pub const ARRAY_DIMENSION: &str = "arco::array::dimension";
    pub const ARRAY_INDEX: &str = "arco::array::index";
    pub const ARRAY_OVERFLOW: &str = "arco::array::overflow";
    pub const ARRAY_SHAPE_MISMATCH: &str = "arco::array::shape_mismatch";
    pub const ARRAY_TYPE: &str = "arco::array::type";
    pub const ALGEBRA_PARSE_ERROR: &str = "arco::algebra::parse_error";
    pub const BLOCK_ARTIFACT_IO: &str = "arco::block::artifact_io";
    pub const BLOCK_CONTRACT: &str = "arco::block::contract";
    pub const BLOCK_RESULT: &str = "arco::block::result";
    pub const BOUNDS_INVALID: &str = "arco::bounds::invalid";
    pub const CONFIG_IO: &str = "arco::config::io";
    pub const CONFIG_MISSING_DIRECTORY: &str = "arco::config::missing_directory";
    pub const CONFIG_MISSING_PROJECT_DIRECTORY: &str = "arco::config::missing_project_directory";
    pub const CONFIG_SECRET_REFERENCE_REQUIRED: &str = "arco::config::secret_reference_required";
    pub const CONFIG_SELECTION: &str = "arco::config::selection";
    pub const CONFIG_TOML: &str = "arco::config::toml";
    pub const COMPILE_CSV: &str = "arco::compile::csv";
    pub const COMPILE_EMPTY_TUPLE_REDUCTION: &str = "arco::compile::empty_tuple_reduction";
    pub const COMPILE_INVALID_CONSTRAINT_FILTER: &str = "arco::compile::invalid_constraint_filter";
    pub const COMPILE_INVALID_FORMULATION: &str = "arco::compile::invalid_formulation";
    pub const COMPILE_INVALID_NUMBER: &str = "arco::compile::invalid_number";
    pub const COMPILE_MISSING_ASSET: &str = "arco::compile::missing_asset";
    pub const COMPILE_MISSING_COLUMN: &str = "arco::compile::missing_column";
    pub const COMPILE_MISSING_DATA: &str = "arco::compile::missing_data";
    pub const COMPILE_MISSING_DATA_POINT: &str = "arco::compile::missing_data_point";
    pub const COMPILE_MISSING_DECLARATION: &str = "arco::compile::missing_declaration";
    pub const COMPILE_MISSING_PARAMETER: &str = "arco::compile::missing_parameter";
    pub const COMPILE_MISSING_SCENARIO: &str = "arco::compile::missing_scenario";
    pub const CONSTRAINT_BOUNDS_MISSING: &str = "arco::constraint::bounds_missing";
    pub const CONSTRAINT_INVALID_BOUNDS: &str = "arco::constraint::invalid_bounds";
    pub const CONSTRAINT_INVALID_ID: &str = "arco::constraint::invalid_id";
    pub const CONSTRAINT_NOT_FOUND: &str = "arco::constraint::not_found";
    pub const CONSTRAINT_SENSE: &str = "arco::constraint::sense";
    pub const CONSTRAINT_TYPE: &str = "arco::constraint::type";
    pub const CSC_CONTIGUITY: &str = "arco::csc::contiguity";
    pub const CSC_DIMENSION: &str = "arco::csc::dimension";
    pub const CSC_DTYPE: &str = "arco::csc::dtype";
    pub const CSC_INVALID_DATA: &str = "arco::csc::invalid_data";
    pub const CSC_NEGATIVE_INDEX: &str = "arco::csc::negative_index";
    pub const DEPENDENCY_MISSING: &str = "arco::dependency::missing";
    pub const DRIVER_BACKEND_NOT_AVAILABLE: &str = "arco::driver::backend_not_available";
    pub const DRIVER_INSPECT_FORMAT: &str = "arco::driver::inspect_format";
    pub const DRIVER_JSON: &str = "arco::driver::json";
    pub const EXPR_COEFFICIENT: &str = "arco::expr::coefficient";
    pub const EXPR_CONSTANT_OFFSET: &str = "arco::expr::constant_offset";
    pub const EXPR_DIVISION_BY_ZERO: &str = "arco::expr::division_by_zero";
    pub const EXPR_NOT_SINGLE_VARIABLE: &str = "arco::expr::not_single_variable";
    pub const EXPR_TYPE: &str = "arco::expr::type";
    pub const INDEX_SET_ARGUMENT: &str = "arco::index_set::argument";
    pub const INDEX_SET_EMPTY: &str = "arco::index_set::empty";
    pub const INDEX_SET_INDEX: &str = "arco::index_set::index";
    pub const INDEX_SET_TYPE: &str = "arco::index_set::type";
    pub const LOGGING_CONFIG: &str = "arco::logging::config";
    pub const LOGGING_IO: &str = "arco::logging::io";
    pub const METADATA_CONVERSION: &str = "arco::metadata::conversion";
    pub const MODEL_BINARY_BOUNDS: &str = "arco::model::binary_bounds";
    pub const MODEL_EMPTY: &str = "arco::model::empty";
    pub const OBJECTIVE_ALREADY_SET: &str = "arco::objective::already_set";
    pub const OBJECTIVE_INDEX: &str = "arco::objective::index";
    pub const OBJECTIVE_MISSING: &str = "arco::objective::missing";
    pub const SEMANTIC_AMBIGUOUS_TUPLE_SUBSET_INDEX: &str =
        "arco::semantic::ambiguous_tuple_subset_index";
    pub const SEMANTIC_CSV: &str = "arco::semantic::csv";
    pub const SEMANTIC_DUPLICATE_DATA_BINDING: &str = "arco::semantic::duplicate_data_binding";
    pub const SEMANTIC_DUPLICATE_DECLARATION: &str = "arco::semantic::duplicate_declaration";
    pub const SEMANTIC_DUPLICATE_MODEL_DECLARATION: &str =
        "arco::semantic::duplicate_model_declaration";
    pub const SEMANTIC_DUPLICATE_TUPLE_ROWS: &str = "arco::semantic::duplicate_tuple_rows";
    pub const SEMANTIC_EXPRESSION_CYCLE: &str = "arco::semantic::expression_cycle";
    pub const SEMANTIC_MISSING_CELL: &str = "arco::semantic::missing_cell";
    pub const SEMANTIC_MISSING_COLUMN: &str = "arco::semantic::missing_column";
    pub const SEMANTIC_MISSING_DECLARATION: &str = "arco::semantic::missing_declaration";
    pub const SEMANTIC_MISSING_INITIAL_BOUNDARY: &str = "arco::semantic::missing_initial_boundary";
    pub const SEMANTIC_MISSING_MODEL: &str = "arco::semantic::missing_model";
    pub const SEMANTIC_MISSING_MODEL_USE: &str = "arco::semantic::missing_model_use";
    pub const SEMANTIC_MISSING_SCENARIO: &str = "arco::semantic::missing_scenario";
    pub const SEMANTIC_SCENARIO_COUNT: &str = "arco::semantic::scenario_count";
    pub const SEMANTIC_TUPLE_SET_SCHEMA_MISMATCH: &str =
        "arco::semantic::tuple_set_schema_mismatch";
    pub const SEMANTIC_TUPLE_SUBSET_DOMAIN_MISMATCH: &str =
        "arco::semantic::tuple_subset_domain_mismatch";
    pub const SEMANTIC_UNKNOWN_SCENARIO_DATA_BINDING: &str =
        "arco::semantic::unknown_scenario_data_binding";
    pub const SEMANTIC_UNRESOLVED_FILTER_IDENTIFIER: &str =
        "arco::semantic::unresolved_filter_identifier";
    pub const SEMANTIC_UNRESOLVED_RULE_SET_FILTER_IDENTIFIER: &str =
        "arco::semantic::unresolved_rule_set_filter_identifier";
    pub const SLACK_BOUND: &str = "arco::slack::bound";
    pub const SLACK_INVALID_PENALTY: &str = "arco::slack::invalid_penalty";
    pub const SLACK_VALUE_UNAVAILABLE: &str = "arco::slack::value_unavailable";
    pub const SOLVER_INFEASIBLE: &str = "arco::solver::infeasible";
    pub const SOLVER_INDEX: &str = "arco::solver::index";
    pub const SOLVER_INTERNAL: &str = "arco::solver::internal";
    pub const SOLVER_ITERATION_LIMIT: &str = "arco::solver::iteration_limit";
    pub const SOLVER_INVALID_SETTING: &str = "arco::solver::invalid_setting";
    pub const SOLVER_MODEL_SIZE_LIMIT: &str = "arco::solver::model_size_limit";
    pub const SOLVER_NOT_AVAILABLE: &str = "arco::solver::not_available";
    pub const SOLVER_TIME_LIMIT: &str = "arco::solver::time_limit";
    pub const SOLVER_TYPE: &str = "arco::solver::type";
    pub const SOLVER_UNBOUNDED: &str = "arco::solver::unbounded";
    pub const SOURCE_INVALID_ALGEBRA: &str = "arco::source::invalid_algebra";
    pub const SOURCE_INVALID_INCLUDE: &str = "arco::source::invalid_include";
    pub const SOURCE_INVALID_VALUE: &str = "arco::source::invalid_value";
    pub const SOURCE_IO: &str = "arco::source::io";
    pub const SOURCE_KDL: &str = "arco::source::kdl";
    pub const SOURCE_MISSING_ARGUMENT: &str = "arco::source::missing_argument";
    pub const SOURCE_MISSING_NODE: &str = "arco::source::missing_node";
    pub const SOURCE_MISSING_PROPERTY: &str = "arco::source::missing_property";
    pub const SOURCE_UNSUPPORTED_DECLARATION: &str = "arco::source::unsupported_declaration";
    pub const VARIABLE_INVALID_ID: &str = "arco::variable::invalid_id";
    pub const VARIABLE_INVALID_BOUNDS: &str = "arco::variable::invalid_bounds";
    pub const VARIABLE_NOT_FOUND: &str = "arco::variable::not_found";
    pub const TARGET_EMPTY_VARIABLE_SET: &str = "arco::target::empty_variable_set";
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// Stable identifier for an input or generated source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Byte-oriented source span. Line/column rendering belongs to authoring layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub(crate) start: u32,
    pub(crate) end: u32,
}

impl SourceSpan {
    pub(crate) fn new(start: u32, end: u32) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }
}

/// Coarse origin for a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Provenance {
    UserInput {
        source: SourceId,
        span: Option<SourceSpan>,
    },
    Generated {
        phase: &'static str,
    },
    External {
        system: &'static str,
    },
}

/// Format-neutral diagnostic item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub(crate) code: DiagnosticCode,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) provenance: Option<Provenance>,
}

impl Diagnostic {
    pub fn new(code: DiagnosticCode, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            provenance: None,
        }
    }

    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Collection of diagnostics produced by one pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticReport {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Diagnostic, DiagnosticCode, DiagnosticReport, Severity, SourceSpan};

    #[test]
    fn source_span_rejects_reversed_ranges() {
        assert!(SourceSpan::new(2, 1).is_none());
        assert_eq!(SourceSpan::new(1, 2).unwrap().start, 1);
    }

    #[test]
    fn report_tracks_error_presence() {
        let mut report = DiagnosticReport::new();
        report.push(Diagnostic::new(
            DiagnosticCode::new("ARCO_TEST"),
            Severity::Warning,
            "warn",
        ));
        assert!(!report.has_errors());
        report.push(Diagnostic::new(
            DiagnosticCode::new("ARCO_TEST_ERR"),
            Severity::Error,
            "err",
        ));
        assert!(report.has_errors());
    }
}
