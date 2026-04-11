#[derive(Debug, Error, Diagnostic)]
pub enum CompileError {
    #[error("missing scenario `{name}` during compilation in {path}")]
    #[diagnostic(
        code(arco::compile::missing_scenario),
        help("ensure semantic validation selected a scenario before compilation")
    )]
    MissingScenario { name: String, path: PathBuf },
    #[error("missing asset `{name}` during compilation in {path}")]
    #[diagnostic(
        code(arco::compile::missing_asset),
        help("ensure every referenced asset is declared in the input")
    )]
    MissingAsset { name: String, path: PathBuf },
    #[error("missing declaration `{kind}` named `{name}` during compilation in {path}")]
    #[diagnostic(
        code(arco::compile::missing_declaration),
        help("add the missing declaration or update the reference")
    )]
    MissingDeclaration {
        kind: &'static str,
        name: String,
        path: PathBuf,
    },
    #[error("missing required parameter `{name}` for asset `{asset}` during compilation in {path}")]
    #[diagnostic(
        code(arco::compile::missing_parameter),
        help("provide the missing asset parameter before compilation")
    )]
    MissingParameter {
        name: String,
        asset: String,
        path: PathBuf,
    },
    #[error("missing required data `{name}` during compilation in {path}")]
    #[diagnostic(
        code(arco::compile::missing_data),
        help("bind the required scenario data before compilation")
    )]
    MissingData { name: String, path: PathBuf },
    #[error("missing required data point `{name}` for key `{key}` during compilation in {path}")]
    #[diagnostic(
        code(arco::compile::missing_data_point),
        help("add the missing row in the data table or restrict iteration to keys that exist")
    )]
    MissingDataPoint {
        name: String,
        key: String,
        path: PathBuf,
    },
    #[error("failed to read csv {path}: {source}")]
    #[diagnostic(
        code(arco::compile::csv),
        help("verify the CSV path exists and is readable")
    )]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },
    #[error("failed to parse numeric value `{value}` for `{field}` in {path}")]
    #[diagnostic(
        code(arco::compile::invalid_number),
        help("replace the non-numeric value with a valid number")
    )]
    InvalidNumber {
        value: String,
        field: String,
        path: PathBuf,
    },
    #[error("missing required column `{column}` in {path}")]
    #[diagnostic(
        code(arco::compile::missing_column),
        help("add the missing CSV column or update the mapping")
    )]
    MissingColumn { column: String, path: PathBuf },
    #[error("constraint filter for `{constraint}` is invalid during compilation in {path}: {message}")]
    #[diagnostic(
        code(arco::compile::invalid_constraint_filter),
        help(
            "use only numeric, boolean, or string comparisons over names available in the current asset/time scope"
        )
    )]
    InvalidConstraintFilter {
        constraint: String,
        message: String,
        path: PathBuf,
    },
    #[error("invalid formulation during compilation in {path}: {message}")]
    #[diagnostic(
        code(arco::compile::invalid_formulation),
        help(
            "rewrite the algebra so every constraint, objective term, and report remains linear over supported domains"
        )
    )]
    InvalidFormulation { message: String, path: PathBuf },
}
