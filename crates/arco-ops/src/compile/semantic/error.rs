#![allow(unused_assignments)]

use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum SemanticError {
    #[error("no scenario is available for semantic validation in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_scenario),
        help("add a `scenario` declaration")
    )]
    MissingScenario { path: PathBuf },

    #[error("semantic validation currently supports exactly one scenario in {path}, found {count}")]
    #[diagnostic(
        code(arco::semantic::scenario_count),
        help("keep a single `scenario` declaration")
    )]
    ScenarioCount { count: usize, path: PathBuf },

    #[error("no `model` declaration is available in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_model),
        help("declare at least one `model` block")
    )]
    MissingModel { path: PathBuf },

    #[error("missing declaration `{kind}` named `{name}` in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_declaration),
        help("add the missing declaration or update the reference to an existing one")
    )]
    MissingDeclaration {
        kind: &'static str,
        name: String,
        path: PathBuf,
    },

    #[error("duplicate scenario data binding `{name}` in {path}")]
    #[diagnostic(
        code(arco::semantic::duplicate_data_binding),
        help("rename or remove the duplicate data binding")
    )]
    DuplicateDataBinding { name: String, path: PathBuf },

    #[error("failed to read csv {path}: {source}")]
    #[diagnostic(
        code(arco::semantic::csv),
        help("verify the CSV path exists and is readable")
    )]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },

    #[error("missing required column `{column}` in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_column),
        help("add the missing CSV column or update the data schema")
    )]
    MissingColumn { column: String, path: PathBuf },

    #[error("missing required value in column `{column}` at row {row} in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_cell),
        help("fill in the missing value in the input table")
    )]
    MissingCell {
        column: String,
        row: usize,
        path: PathBuf,
    },

    #[error("chronology-dependent expressions require an explicit initial boundary in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_initial_boundary),
        help(
            "provide an initial state or commitment boundary for chronology-dependent constraints"
        )
    )]
    MissingInitialBoundary { path: PathBuf },

    #[error("scenario `{scenario}` must declare `use` for a model in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_model_use),
        help("add `use \"ModelName\"` to the scenario")
    )]
    MissingModelUse { scenario: String, path: PathBuf },

    #[error(
        "scenario `{scenario}` binds data `{binding}` that is not a parameter in model `{model}` in {path}"
    )]
    #[diagnostic(
        code(arco::semantic::unknown_scenario_data_binding),
        help("rename the data binding or add a matching model parameter")
    )]
    UnknownScenarioDataBinding {
        scenario: String,
        binding: String,
        model: String,
        path: PathBuf,
    },

    #[error("expression cycle detected: {cycle} in {path}")]
    #[diagnostic(
        code(arco::semantic::expression_cycle),
        help("rewrite model expressions so named dependencies form a DAG")
    )]
    ExpressionCycle { cycle: String, path: PathBuf },

    #[error("inline scalar param `{name}` conflicts with existing {kind} in {path}")]
    #[diagnostic(
        code(arco::semantic::duplicate_declaration),
        help("rename the inline scalar param or the conflicting declaration")
    )]
    DuplicateDeclaration {
        kind: String,
        name: String,
        path: PathBuf,
    },

    #[error("duplicate {kind} declaration `{name}` in model `{model}` in {path}")]
    #[diagnostic(
        code(arco::semantic::duplicate_model_declaration),
        help("rename or remove one of the duplicate declarations")
    )]
    DuplicateModelDeclaration {
        kind: &'static str,
        name: String,
        model: String,
        path: PathBuf,
    },

    #[error(
        "unresolved filter identifier `{identifier}` in {declaration_kind} `{declaration}` from data `{data}` in {path}"
    )]
    #[diagnostic(
        code(arco::semantic::unresolved_filter_identifier),
        help(
            "if token is categorical value, quote it in filter, e.g. `filter {{ tech == \"wind\" }}`"
        )
    )]
    UnresolvedFilterIdentifier {
        identifier: String,
        declaration_kind: &'static str,
        declaration: String,
        data: String,
        path: PathBuf,
    },

    #[error("unresolved identifier `{identifier}` in top-level set filter for `{set}` in {path}")]
    #[diagnostic(
        code(arco::semantic::unresolved_rule_set_filter_identifier),
        help(
            "if token is a categorical value, quote it in filter, e.g. `filter {{ a == \"north\" }}`"
        )
    )]
    UnresolvedRuleSetFilterIdentifier {
        identifier: String,
        set: String,
        path: PathBuf,
    },

    #[error(
        "tuple subset index `{index}` in `{set}` resolves ambiguously to multiple parent components `{candidates}` in {path}"
    )]
    #[diagnostic(
        code(arco::semantic::ambiguous_tuple_subset_index),
        help("disambiguate by using the canonical tuple component name for the parent tuple set")
    )]
    AmbiguousTupleSubsetIndex {
        set: String,
        index: String,
        candidates: String,
        path: PathBuf,
    },

    #[error(
        "tuple subset index `{index}` in `{set}` declares domain `{received_domain}` but parent tuple domain is `{expected_domain}` in {path}"
    )]
    #[diagnostic(
        code(arco::semantic::tuple_subset_domain_mismatch),
        help("align subset index `in` domain with the matched parent tuple component domain")
    )]
    TupleSubsetDomainMismatch {
        set: String,
        index: String,
        expected_domain: String,
        received_domain: String,
        path: PathBuf,
    },

    #[error(
        "tuple component schema mismatch for merged tuple set `{set}` in {path}: existing `{existing_components}` vs incoming `{incoming_components}`"
    )]
    #[diagnostic(
        code(arco::semantic::tuple_set_schema_mismatch),
        help("ensure all tuple sources for a set use the exact same component names and order")
    )]
    TupleSetSchemaMismatch {
        set: String,
        existing_components: String,
        incoming_components: String,
        path: PathBuf,
    },

    #[error("duplicate feasible tuples detected for `{set}` in {path}: {duplicates}")]
    #[diagnostic(
        code(arco::semantic::duplicate_tuple_rows),
        help(
            "remove duplicate tuple rows or correct tuple projection/filter rules so each tuple appears once"
        )
    )]
    DuplicateTupleRows {
        set: String,
        duplicates: String,
        path: PathBuf,
    },
}
