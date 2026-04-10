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
}
