// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use crate::algebra::{BinaryOp, ComparisonOp, ConstraintBody, Expr, UnaryOp};
use crate::semantic::{
    FamilySignature, ResolvedConstraint, ResolvedObjective, ResolvedReport, SemanticProgram,
    VariableDeclOverrides,
};
use crate::source::{
    BoundExpr, DataDecl, FilterComparators, GenerationBinding, LiteralValue, ParamDecl,
    ScenarioDecl, SourceProgram, VariableKindDecl,
};
use csv::StringRecord;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoweredProblem {
    pub parameters: Vec<LoweredParameter>,
    pub variables: Vec<LoweredVariable>,
    pub constraints: Vec<LoweredConstraint>,
    pub objective: LoweredObjective,
    pub reports: Vec<LoweredReport>,
    pub dual_reports: Vec<LoweredDualReport>,
    pub traceability: Vec<TraceabilityRecord>,
    pub algebra: AlgebraicProblem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredParameter {
    pub name: String,
    pub binding_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredVariable {
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredConstraint {
    pub name: String,
    pub source_kind: String,
    pub source_name: String,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredObjective {
    pub name: String,
    pub sense: String,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredReport {
    pub name: String,
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredDualReport {
    pub constraint_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceabilityRecord {
    pub dsl_name: String,
    pub artifact_kind: String,
    pub lowered_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlgebraicProblem {
    pub variable_instances: Vec<VariableInstance>,
    pub constraints: Vec<LinearConstraint>,
    pub objective: LinearObjective,
    pub reports: Vec<LinearReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableInstance {
    pub name: String,
    pub family: String,
    pub lower: f64,
    pub upper: Option<f64>,
    pub kind: VariableKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableKind {
    Continuous,
    Integer,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearConstraint {
    pub name: String,
    pub sense: ConstraintSense,
    pub rhs: f64,
    pub terms: Vec<LinearTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintSense {
    GreaterEqual,
    LessEqual,
    Equal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearObjective {
    pub name: String,
    pub sense: ObjectiveSense,
    pub constant: f64,
    pub terms: Vec<LinearTerm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearReport {
    pub name: String,
    pub constant: f64,
    pub terms: Vec<LinearTerm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearTerm {
    pub variable_name: String,
    pub coefficient: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}

#[derive(Debug)]
struct ScenarioInputs {
    assets: Vec<AssetInputs>,
    series: BTreeMap<String, BTreeMap<usize, f64>>,
    indexed: BTreeMap<String, BTreeMap<(String, usize), f64>>,
    asset_data: BTreeMap<String, BTreeMap<String, f64>>,
    generic_data: BTreeMap<String, GenericDataTable>,
    set_params: BTreeMap<String, BTreeMap<String, f64>>,
}

#[derive(Debug)]
struct AssetInputs {
    name: String,
    operation: Option<String>,
    families: BTreeSet<String>,
    parameters: BTreeMap<String, f64>,
    candidate: bool,
}

#[derive(Debug, Default)]
struct GenericDataTable {
    values: BTreeMap<Vec<String>, f64>,
    default_missing: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct FilterScope<'a> {
    asset: Option<&'a AssetInputs>,
    time: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum FilterValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Error, Diagnostic)]
pub enum LoweringError {
    #[error("missing scenario `{name}` during lowering in {path}")]
    #[diagnostic(
        code(arco::lowering::missing_scenario),
        help("ensure semantic validation selected a scenario before lowering")
    )]
    MissingScenario { name: String, path: PathBuf },
    #[error("missing asset `{name}` during lowering in {path}")]
    #[diagnostic(
        code(arco::lowering::missing_asset),
        help("ensure every referenced asset is declared in the input")
    )]
    MissingAsset { name: String, path: PathBuf },
    #[error("missing declaration `{kind}` named `{name}` during lowering in {path}")]
    #[diagnostic(
        code(arco::lowering::missing_declaration),
        help("add the missing declaration or update the lowering reference")
    )]
    MissingDeclaration {
        kind: &'static str,
        name: String,
        path: PathBuf,
    },
    #[error("missing required parameter `{name}` for asset `{asset}` during lowering in {path}")]
    #[diagnostic(
        code(arco::lowering::missing_parameter),
        help("provide the missing asset parameter before lowering")
    )]
    MissingParameter {
        name: String,
        asset: String,
        path: PathBuf,
    },
    #[error("missing required data `{name}` during lowering in {path}")]
    #[diagnostic(
        code(arco::lowering::missing_data),
        help("bind the required scenario data before lowering")
    )]
    MissingData { name: String, path: PathBuf },
    #[error("missing required data point `{name}` for key `{key}` during lowering in {path}")]
    #[diagnostic(
        code(arco::lowering::missing_data_point),
        help("fill in the missing data point in the input tables")
    )]
    MissingDataPoint {
        name: String,
        key: String,
        path: PathBuf,
    },
    #[error("failed to read csv {path}: {source}")]
    #[diagnostic(
        code(arco::lowering::csv),
        help("verify the CSV path exists and is readable")
    )]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },
    #[error("failed to parse numeric value `{value}` for `{field}` in {path}")]
    #[diagnostic(
        code(arco::lowering::invalid_number),
        help("replace the non-numeric value with a valid number")
    )]
    InvalidNumber {
        value: String,
        field: String,
        path: PathBuf,
    },
    #[error("missing required column `{column}` in {path}")]
    #[diagnostic(
        code(arco::lowering::missing_column),
        help("add the missing CSV column or update the mapping")
    )]
    MissingColumn { column: String, path: PathBuf },
    #[error("constraint filter for `{constraint}` is invalid during lowering in {path}: {message}")]
    #[diagnostic(
        code(arco::lowering::invalid_constraint_filter),
        help(
            "use only numeric, boolean, or string comparisons over names available in the current asset/time scope"
        )
    )]
    InvalidConstraintFilter {
        constraint: String,
        message: String,
        path: PathBuf,
    },
    #[error("invalid formulation during lowering in {path}: {message}")]
    #[diagnostic(
        code(arco::lowering::invalid_formulation),
        help(
            "rewrite the algebra so every constraint, objective term, and report remains linear over supported domains"
        )
    )]
    InvalidFormulation { message: String, path: PathBuf },
}

pub fn lower_program(
    program: &SemanticProgram,
    source_program: &SourceProgram,
    entrypoint: &Path,
) -> Result<LoweredProblem, LoweringError> {
    info!("lowering program");

    let scenario = source_program
        .scenario(&program.active_scenario)
        .ok_or_else(|| LoweringError::MissingScenario {
            name: program.active_scenario.clone(),
            path: entrypoint.to_path_buf(),
        })?;
    let mut inputs = load_inputs(program, source_program, scenario, entrypoint)?;
    inputs.set_params.extend(program.set_params.clone());
    let algebra = lower_algebra(program, &inputs, entrypoint)?;

    let parameters = [
        ("series", &program.parameters.series),
        ("indexed", &program.parameters.indexed),
        ("asset", &program.parameters.asset),
    ]
    .into_iter()
    .flat_map(|(kind, names)| {
        names.iter().map(move |name| LoweredParameter {
            name: name.clone(),
            binding_kind: kind.to_string(),
        })
    })
    .collect::<Vec<_>>();

    let variables = program
        .variable_families
        .iter()
        .map(|family| LoweredVariable {
            family: family.render(),
        })
        .collect::<Vec<_>>();

    let constraints = program
        .active_constraints
        .iter()
        .map(lower_constraint)
        .collect::<Vec<_>>();
    let objective = lower_objective(&program.active_objective);
    let reports = program
        .active_reports
        .iter()
        .map(lower_report)
        .collect::<Vec<_>>();

    let dual_reports = program
        .active_dual_reports
        .iter()
        .map(|dr| LoweredDualReport {
            constraint_name: dr.constraint_name.clone(),
        })
        .collect::<Vec<_>>();

    let mut traceability = Vec::new();
    traceability.extend(variables.iter().map(|variable| TraceabilityRecord {
        dsl_name: variable.family.clone(),
        artifact_kind: "variable".to_string(),
        lowered_name: variable.family.clone(),
    }));
    traceability.push(TraceabilityRecord {
        dsl_name: objective.name.clone(),
        artifact_kind: "objective".to_string(),
        lowered_name: objective.name.clone(),
    });
    traceability.extend(reports.iter().map(|report| TraceabilityRecord {
        dsl_name: report.name.clone(),
        artifact_kind: "report".to_string(),
        lowered_name: report.name.clone(),
    }));

    let lowered = LoweredProblem {
        parameters,
        variables,
        constraints,
        objective,
        reports,
        dual_reports,
        traceability,
        algebra,
    };
    debug!(
        "generated {} variables, {} constraints, {} reports",
        lowered.algebra.variable_instances.len(),
        lowered.algebra.constraints.len(),
        lowered.reports.len()
    );

    Ok(lowered)
}

fn lower_constraint(constraint: &ResolvedConstraint) -> LoweredConstraint {
    LoweredConstraint {
        name: constraint.name.clone(),
        source_kind: constraint.source_kind.clone(),
        source_name: constraint.source_name.clone(),
        expression: constraint.expression_text.clone(),
    }
}

fn lower_objective(objective: &ResolvedObjective) -> LoweredObjective {
    LoweredObjective {
        name: objective.name.clone(),
        sense: objective.sense.clone(),
        expression: objective.expression_text.clone(),
    }
}

fn lower_report(report: &ResolvedReport) -> LoweredReport {
    LoweredReport {
        name: report.name.clone(),
        formula: report.formula_text.clone(),
    }
}

fn evaluate_constraint_filter(
    expr: &Expr,
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    inputs: &ScenarioInputs,
    path: &Path,
) -> Result<bool, LoweringError> {
    let value = evaluate_filter_expr(expr, constraint, scope, inputs, path)?;
    truthy_filter_value(&value, constraint, path)
}

fn evaluate_filter_expr(
    expr: &Expr,
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    inputs: &ScenarioInputs,
    path: &Path,
) -> Result<FilterValue, LoweringError> {
    match expr {
        Expr::Number(value) => value
            .parse::<f64>()
            .map(FilterValue::Number)
            .map_err(|_| invalid_constraint_filter(constraint, path, "numeric literal is invalid")),
        Expr::String(value) => Ok(FilterValue::String(value.clone())),
        Expr::Boolean(value) => Ok(FilterValue::Boolean(*value)),
        Expr::Identifier(name) => evaluate_identifier(name, constraint, scope, path),
        Expr::Indexed { target, indices } => {
            evaluate_indexed_value(target, indices, constraint, scope, inputs, path)
        }
        Expr::Unary { op, expr } => {
            let value = evaluate_filter_expr(expr, constraint, scope, inputs, path)?;
            match op {
                UnaryOp::Negate => Ok(FilterValue::Number(-numeric_filter_value(
                    &value, constraint, path,
                )?)),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = evaluate_filter_expr(left, constraint, scope, inputs, path)?;
            let right = evaluate_filter_expr(right, constraint, scope, inputs, path)?;
            let left = numeric_filter_value(&left, constraint, path)?;
            let right = numeric_filter_value(&right, constraint, path)?;
            let value = match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => left / right,
            };
            Ok(FilterValue::Number(value))
        }
        Expr::Comparison { op, left, right } => {
            let left = evaluate_filter_expr(left, constraint, scope, inputs, path)?;
            let right = evaluate_filter_expr(right, constraint, scope, inputs, path)?;
            Ok(FilterValue::Boolean(compare_filter_values(
                *op, &left, &right, constraint, path,
            )?))
        }
        Expr::FunctionCall { .. } => Err(invalid_constraint_filter(
            constraint,
            path,
            "function calls are not supported in constraint filters",
        )),
        Expr::Reduction(_) => Err(invalid_constraint_filter(
            constraint,
            path,
            "reductions are not supported in constraint filters",
        )),
    }
}

fn evaluate_identifier(
    name: &str,
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    path: &Path,
) -> Result<FilterValue, LoweringError> {
    match name {
        "a" => scope
            .asset
            .map(|asset| FilterValue::String(asset.name.clone()))
            .ok_or_else(|| invalid_constraint_filter(constraint, path, "`a` is not in scope")),
        "t" => scope
            .time
            .map(|time| FilterValue::Number(time as f64))
            .ok_or_else(|| invalid_constraint_filter(constraint, path, "`t` is not in scope")),
        "candidate" => scope
            .asset
            .map(|asset| FilterValue::Boolean(asset.candidate))
            .ok_or_else(|| {
                invalid_constraint_filter(constraint, path, "`candidate` is not in scope")
            }),
        other => scope
            .asset
            .and_then(|asset| asset.parameters.get(other).copied())
            .map(FilterValue::Number)
            .ok_or_else(|| {
                invalid_constraint_filter(
                    constraint,
                    path,
                    format!("`{other}` is not available in the current filter scope"),
                )
            }),
    }
}

fn evaluate_indexed_value(
    target: &str,
    indices: &[Expr],
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    inputs: &ScenarioInputs,
    path: &Path,
) -> Result<FilterValue, LoweringError> {
    let values = indices
        .iter()
        .map(|index| evaluate_filter_expr(index, constraint, scope, inputs, path))
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [index] => {
            if let FilterValue::String(asset_name) = index {
                if target == "candidate" {
                    return find_asset(inputs, asset_name)
                        .map(|asset| FilterValue::Boolean(asset.candidate))
                        .ok_or_else(|| {
                            invalid_constraint_filter(
                                constraint,
                                path,
                                format!("asset `{asset_name}` is not available"),
                            )
                        });
                }
                if let Some(value) = asset_parameter_value(inputs, target, asset_name) {
                    return Ok(FilterValue::Number(value));
                }
                if let Some(value) = inputs
                    .asset_data
                    .get(target)
                    .and_then(|values| values.get(asset_name))
                    .copied()
                {
                    return Ok(FilterValue::Number(value));
                }
                Err(invalid_constraint_filter(
                    constraint,
                    path,
                    format!("`{target}[{asset_name}]` is not available"),
                ))
            } else {
                let time = usize_filter_value(index, constraint, path)?;
                inputs
                    .series
                    .get(target)
                    .and_then(|values| values.get(&time))
                    .copied()
                    .map(FilterValue::Number)
                    .ok_or_else(|| {
                        invalid_constraint_filter(
                            constraint,
                            path,
                            format!("`{target}[{time}]` is not available"),
                        )
                    })
            }
        }
        [asset_name, time] => {
            let asset_name = string_filter_value(asset_name, constraint, path)?;
            let time = usize_filter_value(time, constraint, path)?;
            inputs
                .indexed
                .get(target)
                .and_then(|values| values.get(&(asset_name.clone(), time)))
                .copied()
                .map(FilterValue::Number)
                .ok_or_else(|| {
                    invalid_constraint_filter(
                        constraint,
                        path,
                        format!("`{target}[{asset_name},{time}]` is not available"),
                    )
                })
        }
        _ => Err(invalid_constraint_filter(
            constraint,
            path,
            "constraint filters support only one-dimensional asset/time lookups and two-dimensional asset-time lookups",
        )),
    }
}

fn compare_filter_values(
    op: ComparisonOp,
    left: &FilterValue,
    right: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<bool, LoweringError> {
    match op {
        ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
            compare_for_equality(left, right, constraint, path)
        }
        ComparisonOp::NotEqual => {
            compare_for_equality(left, right, constraint, path).map(|value| !value)
        }
        ComparisonOp::Less
        | ComparisonOp::LessEqual
        | ComparisonOp::Greater
        | ComparisonOp::GreaterEqual => {
            let left = numeric_filter_value(left, constraint, path)?;
            let right = numeric_filter_value(right, constraint, path)?;
            Ok(match op {
                ComparisonOp::Less => left < right,
                ComparisonOp::LessEqual => left <= right,
                ComparisonOp::Greater => left > right,
                ComparisonOp::GreaterEqual => left >= right,
                ComparisonOp::Equal | ComparisonOp::DoubleEqual | ComparisonOp::NotEqual => {
                    unreachable!()
                }
            })
        }
    }
}

fn compare_for_equality(
    left: &FilterValue,
    right: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<bool, LoweringError> {
    match (left, right) {
        (FilterValue::String(left), FilterValue::String(right)) => Ok(left == right),
        (FilterValue::Boolean(left), FilterValue::Boolean(right)) => Ok(left == right),
        _ => {
            let left = numeric_filter_value(left, constraint, path)?;
            let right = numeric_filter_value(right, constraint, path)?;
            Ok((left - right).abs() < f64::EPSILON)
        }
    }
}

fn truthy_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<bool, LoweringError> {
    match value {
        FilterValue::Boolean(value) => Ok(*value),
        FilterValue::Number(value) => Ok(*value != 0.0),
        FilterValue::String(_) => Err(invalid_constraint_filter(
            constraint,
            path,
            "string-valued filters must be used inside an explicit comparison",
        )),
    }
}

fn numeric_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<f64, LoweringError> {
    match value {
        FilterValue::Number(value) => Ok(*value),
        FilterValue::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),
        FilterValue::String(_) => Err(invalid_constraint_filter(
            constraint,
            path,
            "numeric operations in constraint filters require numeric or boolean values",
        )),
    }
}

fn string_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<String, LoweringError> {
    match value {
        FilterValue::String(value) => Ok(value.clone()),
        _ => Err(invalid_constraint_filter(
            constraint,
            path,
            "asset-indexed lookups require a string asset name",
        )),
    }
}

fn usize_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<usize, LoweringError> {
    let number = numeric_filter_value(value, constraint, path)?;
    if number.fract() == 0.0 && number >= 0.0 {
        Ok(number as usize)
    } else {
        Err(invalid_constraint_filter(
            constraint,
            path,
            "time-indexed lookups require a non-negative integer time index",
        ))
    }
}

fn find_asset<'a>(inputs: &'a ScenarioInputs, name: &str) -> Option<&'a AssetInputs> {
    inputs.assets.iter().find(|asset| asset.name == name)
}

fn asset_parameter_value(
    inputs: &ScenarioInputs,
    parameter: &str,
    asset_name: &str,
) -> Option<f64> {
    find_asset(inputs, asset_name).and_then(|asset| asset.parameters.get(parameter).copied())
}

fn invalid_constraint_filter(
    constraint: &ResolvedConstraint,
    path: &Path,
    message: impl Into<String>,
) -> LoweringError {
    LoweringError::InvalidConstraintFilter {
        constraint: format!(
            "{}:{}:{}",
            constraint.source_kind, constraint.source_name, constraint.name
        ),
        message: message.into(),
        path: path.to_path_buf(),
    }
}

fn load_inputs(
    program: &SemanticProgram,
    source_program: &SourceProgram,
    scenario: &ScenarioDecl,
    entrypoint: &Path,
) -> Result<ScenarioInputs, LoweringError> {
    let entry_dir = entrypoint
        .parent()
        .ok_or_else(|| LoweringError::MissingScenario {
            name: program.active_scenario.clone(),
            path: entrypoint.to_path_buf(),
        })?;

    let mut assets = Vec::new();
    let has_model = scenario.model_use.is_some();
    for asset_name in &scenario.assets {
        match source_program.asset(asset_name) {
            Some(asset) => {
                let parameters = asset
                    .parameters
                    .iter()
                    .map(|(name, value)| {
                        literal_to_f64(name, value, entrypoint).map(|parsed| (name.clone(), parsed))
                    })
                    .collect::<Result<BTreeMap<_, _>, _>>()?;
                let families = technology_families(source_program, &asset.technology, entrypoint)?;
                assets.push(AssetInputs {
                    name: asset.name.clone(),
                    operation: asset.operation.clone(),
                    families,
                    parameters,
                    candidate: false,
                });
            }
            None if has_model => {
                // Model-path assets declared inline in the scenario don't have
                // top-level asset declarations. Create a bare entry; the model
                // block below will assign control families.
                assets.push(AssetInputs {
                    name: asset_name.clone(),
                    operation: None,
                    families: BTreeSet::new(),
                    parameters: BTreeMap::new(),
                    candidate: false,
                });
            }
            None => {
                return Err(LoweringError::MissingDeclaration {
                    kind: "asset",
                    name: asset_name.clone(),
                    path: entrypoint.to_path_buf(),
                });
            }
        }
    }

    for instances_name in &scenario.instances {
        let instances = source_program.instances(instances_name).ok_or_else(|| {
            LoweringError::MissingDeclaration {
                kind: "instances",
                name: instances_name.clone(),
                path: entrypoint.to_path_buf(),
            }
        })?;
        let csv_path = entry_dir.join(&instances.source);
        let rows = read_csv_rows(&csv_path)?;
        let name_column = if instances.columns.is_empty() {
            "asset_name".to_string()
        } else {
            instances
                .columns
                .iter()
                .find(|column| column.target == "name")
                .map(|column| column.source.clone())
                .ok_or_else(|| LoweringError::MissingColumn {
                    column: "name".to_string(),
                    path: csv_path.clone(),
                })?
        };

        for row in &rows {
            let asset_name =
                row.get(&name_column)
                    .cloned()
                    .ok_or_else(|| LoweringError::MissingColumn {
                        column: name_column.clone(),
                        path: csv_path.clone(),
                    })?;
            let mut parameters = BTreeMap::new();
            if instances.columns.is_empty() {
                // Auto-map: every column except the identity becomes a parameter.
                for (col_name, raw_value) in row {
                    if col_name == &name_column {
                        continue;
                    }
                    let parsed =
                        raw_value
                            .parse::<f64>()
                            .map_err(|_| LoweringError::InvalidNumber {
                                value: raw_value.clone(),
                                field: col_name.clone(),
                                path: csv_path.clone(),
                            })?;
                    parameters.insert(col_name.clone(), parsed);
                }
            } else {
                for column in &instances.columns {
                    if column.target == "name" {
                        continue;
                    }
                    let raw_value = row.get(&column.source).cloned().ok_or_else(|| {
                        LoweringError::MissingColumn {
                            column: column.source.clone(),
                            path: csv_path.clone(),
                        }
                    })?;
                    let parsed =
                        raw_value
                            .parse::<f64>()
                            .map_err(|_| LoweringError::InvalidNumber {
                                value: raw_value,
                                field: column.target.clone(),
                                path: csv_path.clone(),
                            })?;
                    parameters.insert(column.target.clone(), parsed);
                }
            }
            let families = technology_families(source_program, &instances.technology, entrypoint)?;
            assets.push(AssetInputs {
                name: asset_name,
                operation: instances.operation.clone(),
                families,
                parameters,
                candidate: instances.name.starts_with("Candidate"),
            });
        }
    }

    if let Some(model_name) = &scenario.model_use {
        let model =
            source_program
                .model(model_name)
                .ok_or_else(|| LoweringError::MissingDeclaration {
                    kind: "model",
                    name: model_name.clone(),
                    path: entrypoint.to_path_buf(),
                })?;

        let model_families: BTreeSet<String> = model
            .controls
            .iter()
            .map(|control| control.name.clone())
            .collect();

        if assets.is_empty() {
            // No explicit assets: create a synthetic default asset.
            if !model_families.is_empty() {
                assets.push(AssetInputs {
                    name: "default".to_string(),
                    operation: None,
                    families: model_families,
                    parameters: BTreeMap::new(),
                    candidate: false,
                });
            }
        } else {
            // Explicit assets declared: assign model control families to each.
            for asset in &mut assets {
                asset.families.clone_from(&model_families);
            }
        }
    }

    assets.sort_by(|a, b| a.name.cmp(&b.name));

    let mut series = BTreeMap::new();
    let mut indexed = BTreeMap::new();
    let mut asset_data = BTreeMap::new();
    let mut generic_data = BTreeMap::new();
    let mut set_params = BTreeMap::new();

    for data_decl in &source_program.data {
        load_data_decl_params(data_decl, entry_dir, &mut generic_data, &mut set_params)?;
    }

    for binding in &scenario.data {
        generic_data.remove(&binding.name);

        let csv_path = entry_dir.join(&binding.source);
        let rows = read_csv_rows(&csv_path)?;
        let Some(first_row) = rows.first() else {
            return Err(LoweringError::MissingData {
                name: binding.name.clone(),
                path: csv_path,
            });
        };
        let headers = first_row.keys().cloned().collect::<BTreeSet<_>>();
        if headers.contains("asset_name") && headers.contains("t") {
            let mut values = BTreeMap::new();
            for row in &rows {
                let asset_name =
                    row.get("asset_name")
                        .cloned()
                        .ok_or_else(|| LoweringError::MissingColumn {
                            column: "asset_name".to_string(),
                            path: csv_path.clone(),
                        })?;
                let time = parse_usize_field(row, "t", &csv_path)?;
                let value = parse_data_value(row, &binding.name, &csv_path)?;
                values.insert((asset_name, time), value);
            }
            indexed.insert(binding.name.clone(), values);
        } else if headers.contains("asset_name") {
            let mut values = BTreeMap::new();
            for row in &rows {
                let asset_name =
                    row.get("asset_name")
                        .cloned()
                        .ok_or_else(|| LoweringError::MissingColumn {
                            column: "asset_name".to_string(),
                            path: csv_path.clone(),
                        })?;
                let value = parse_data_value(row, &binding.name, &csv_path)?;
                values.insert(asset_name, value);
            }
            asset_data.insert(binding.name.clone(), values);
        } else if headers.contains("t") {
            let mut values = BTreeMap::new();
            for row in &rows {
                let time = parse_usize_field(row, "t", &csv_path)?;
                let value = parse_data_value(row, &binding.name, &csv_path)?;
                values.insert(time, value);
            }
            series.insert(binding.name.clone(), values);
        } else {
            let table = load_generic_data_table(program, &binding.name, &rows, &csv_path)?;
            generic_data.insert(binding.name.clone(), table);
        }
    }

    Ok(ScenarioInputs {
        assets,
        series,
        indexed,
        asset_data,
        generic_data,
        set_params,
    })
}

fn technology_families(
    source_program: &SourceProgram,
    technology_name: &str,
    path: &Path,
) -> Result<BTreeSet<String>, LoweringError> {
    let technology = source_program.technology(technology_name).ok_or_else(|| {
        LoweringError::MissingDeclaration {
            kind: "technology",
            name: technology_name.to_string(),
            path: path.to_path_buf(),
        }
    })?;
    Ok(technology
        .investments
        .iter()
        .map(|i| i.name.clone())
        .chain(technology.controls.iter().map(|c| c.name.clone()))
        .chain(technology.states.iter().cloned())
        .collect())
}

#[derive(Debug, Clone, Default)]
struct AffineExpr {
    constant: f64,
    terms: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Default)]
struct LinearizationBindings {
    values: BTreeMap<String, FilterValue>,
}

fn lower_algebra(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    entrypoint: &Path,
) -> Result<AlgebraicProblem, LoweringError> {
    let named_expressions = program
        .active_expressions
        .iter()
        .map(|expression| (expression.name.clone(), expression.formula.clone()))
        .collect::<BTreeMap<_, _>>();
    let variable_signatures = program
        .variable_families
        .iter()
        .map(|family| (family.render(), family.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut variable_instances =
        instantiate_variable_instances(program, inputs, &variable_signatures, entrypoint)?;
    let instantiated_names: BTreeSet<String> =
        variable_instances.iter().map(|i| i.name.clone()).collect();
    let mut constraints = lower_constraint_instances(
        program,
        inputs,
        &named_expressions,
        &variable_signatures,
        &instantiated_names,
        entrypoint,
    )?;
    constraints.extend(emit_terminal_boundary_constraints(
        program,
        inputs,
        &variable_signatures,
        entrypoint,
    )?);

    let objective = linearize_value_expr(
        &program.active_objective.expression,
        &LinearizationBindings::default(),
        program,
        inputs,
        &named_expressions,
        &variable_signatures,
        &instantiated_names,
        entrypoint,
    )?;
    let reports = program
        .active_reports
        .iter()
        .map(|report| {
            linearize_value_expr(
                &report.formula,
                &LinearizationBindings::default(),
                program,
                inputs,
                &named_expressions,
                &variable_signatures,
                &instantiated_names,
                entrypoint,
            )
            .map(|linearized| LinearReport {
                name: report.name.clone(),
                constant: linearized.constant,
                terms: linearized.into_terms(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    variable_instances.sort_by(|a, b| a.name.cmp(&b.name));
    constraints.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(AlgebraicProblem {
        variable_instances,
        constraints,
        objective: LinearObjective {
            name: program.active_objective.name.clone(),
            sense: objective_sense(&program.active_objective.sense),
            constant: objective.constant,
            terms: objective.into_terms(),
        },
        reports,
    })
}

impl AffineExpr {
    fn constant(value: f64) -> Self {
        Self {
            constant: value,
            terms: BTreeMap::new(),
        }
    }

    fn variable(name: String, coefficient: f64) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(name, coefficient);
        Self {
            constant: 0.0,
            terms,
        }
    }

    fn add_assign(&mut self, other: Self) {
        self.constant += other.constant;
        for (name, coefficient) in other.terms {
            let entry = self.terms.entry(name).or_default();
            *entry += coefficient;
        }
        self.terms
            .retain(|_, coefficient| coefficient.abs() >= 1e-12);
    }

    fn subtract(self, other: Self) -> Self {
        let mut value = self;
        value.add_assign(other.scale(-1.0));
        value
    }

    fn scale(mut self, factor: f64) -> Self {
        self.constant *= factor;
        for coefficient in self.terms.values_mut() {
            *coefficient *= factor;
        }
        self.terms
            .retain(|_, coefficient| coefficient.abs() >= 1e-12);
        self
    }

    fn as_scalar(&self, path: &Path, context: &str) -> Result<f64, LoweringError> {
        if self.terms.is_empty() {
            Ok(self.constant)
        } else {
            Err(LoweringError::InvalidFormulation {
                message: format!("{context} must remain scalar"),
                path: path.to_path_buf(),
            })
        }
    }

    fn into_terms(self) -> Vec<LinearTerm> {
        self.terms
            .into_iter()
            .filter(|(_, coefficient)| coefficient.abs() >= 1e-12)
            .map(|(variable_name, coefficient)| LinearTerm {
                variable_name,
                coefficient,
            })
            .collect()
    }
}

fn instantiate_variable_instances(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    entrypoint: &Path,
) -> Result<Vec<VariableInstance>, LoweringError> {
    let mut instances = Vec::new();
    for (family, signature) in variable_signatures {
        let overrides = program.variable_overrides.get(&signature.target);
        match signature.indices.as_slice() {
            [asset_index, time_index] if asset_index == "a" && time_index == "t" => {
                for asset in &inputs.assets {
                    if !variable_instance_is_active(&signature.target, Some(asset)) {
                        continue;
                    }
                    for time in 1..=program.sets.time.steps {
                        instances.push(variable_instance_from_signature(
                            family,
                            signature,
                            Some(asset),
                            Some(time),
                            overrides,
                            entrypoint,
                        )?);
                    }
                }
            }
            [asset_index] if asset_index == "a" => {
                for asset in &inputs.assets {
                    if !variable_instance_is_active(&signature.target, Some(asset)) {
                        continue;
                    }
                    instances.push(variable_instance_from_signature(
                        family,
                        signature,
                        Some(asset),
                        None,
                        overrides,
                        entrypoint,
                    )?);
                }
            }
            [time_index] if time_index == "t" => {
                for time in 1..=program.sets.time.steps {
                    instances.push(variable_instance_from_signature(
                        family,
                        signature,
                        None,
                        Some(time),
                        overrides,
                        entrypoint,
                    )?);
                }
            }
            _ => {
                // Try to resolve custom index domains via the set registry.
                let resolved = resolve_custom_index_domains(
                    signature, program, inputs, family, overrides, entrypoint,
                )?;
                instances.extend(resolved);
            }
        }
    }
    Ok(instances)
}

/// Expand variable instances for families with custom index domains that
/// don't match the built-in "a"/"t" patterns. Each index is resolved by
/// checking its explicit domain binding (from `IndexDecl`) or falling back
/// to the set_registry. Indices bound to "time" produce numeric time steps;
/// all others produce string-named instances.
fn resolve_custom_index_domains(
    signature: &FamilySignature,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    family: &str,
    overrides: Option<&VariableDeclOverrides>,
    entrypoint: &Path,
) -> Result<Vec<VariableInstance>, LoweringError> {
    // Resolve each index to its domain values.
    let mut domain_values: Vec<Vec<String>> = Vec::new();
    for index_name in &signature.indices {
        let values = resolve_single_index_domain(
            index_name, signature, program, inputs, family, entrypoint,
        )?;
        domain_values.push(values);
    }

    // Cartesian product of all domain values.
    let mut combos: Vec<Vec<String>> = vec![vec![]];
    for values in &domain_values {
        let mut next = Vec::new();
        for combo in &combos {
            for value in values {
                let mut extended = combo.clone();
                extended.push(value.clone());
                next.push(extended);
            }
        }
        combos = next;
    }

    let mut instances = Vec::new();
    for combo in &combos {
        let name = format!("{}[{}]", signature.target, combo.join(","));
        let (lower, upper, kind) =
            variable_domain_policy(&signature.target, None, overrides, entrypoint)?;
        instances.push(VariableInstance {
            name,
            family: family.to_string(),
            lower,
            upper,
            kind,
        });
    }
    Ok(instances)
}

/// Resolve values for a single index. Checks the signature's explicit domain
/// binding first, then falls back to known index names ("a" -> assets,
/// "t" -> time), and finally looks up any set matching the index name in
/// the registry.
fn resolve_single_index_domain(
    index_name: &str,
    signature: &FamilySignature,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    family: &str,
    entrypoint: &Path,
) -> Result<Vec<String>, LoweringError> {
    // Check explicit domain binding from IndexDecl.
    if let Some(domain) = signature.index_domains.get(index_name) {
        if domain == "time" {
            return Ok((1..=program.sets.time.steps)
                .map(|t| t.to_string())
                .collect());
        }
        if domain == "assets" {
            return Ok(inputs.assets.iter().map(|a| a.name.clone()).collect());
        }
        if let Some(set) = program.set_registry.get(domain.as_str()) {
            return Ok(set.values.clone());
        }
        return Err(LoweringError::InvalidFormulation {
            message: format!(
                "index `{index_name}` in `{family}` references unknown set `{domain}`"
            ),
            path: entrypoint.to_path_buf(),
        });
    }

    // Fallback: infer from conventional index names.
    match index_name {
        "a" => Ok(inputs.assets.iter().map(|a| a.name.clone()).collect()),
        "t" => Ok((1..=program.sets.time.steps)
            .map(|t| t.to_string())
            .collect()),
        _ => {
            // Last resort: check if the index name itself is a set in the registry.
            if let Some(set) = program.set_registry.get(index_name) {
                return Ok(set.values.clone());
            }
            Err(LoweringError::InvalidFormulation {
                message: format!("unsupported variable family domain `{family}`"),
                path: entrypoint.to_path_buf(),
            })
        }
    }
}

fn variable_instance_from_signature(
    family: &str,
    signature: &FamilySignature,
    asset: Option<&AssetInputs>,
    time: Option<usize>,
    overrides: Option<&VariableDeclOverrides>,
    entrypoint: &Path,
) -> Result<VariableInstance, LoweringError> {
    let (lower, upper, kind) =
        variable_domain_policy(&signature.target, asset, overrides, entrypoint)?;
    let name = match (asset, time) {
        (Some(asset), Some(time)) => indexed_name(&signature.target, &asset.name, time),
        (Some(asset), None) => asset_indexed_name(&signature.target, &asset.name),
        (None, Some(time)) => time_name(&signature.target, time),
        (None, None) => signature.target.clone(),
    };
    Ok(VariableInstance {
        name,
        family: family.to_string(),
        lower,
        upper,
        kind,
    })
}

fn variable_domain_policy(
    target: &str,
    asset: Option<&AssetInputs>,
    overrides: Option<&VariableDeclOverrides>,
    path: &Path,
) -> Result<(f64, Option<f64>, VariableKind), LoweringError> {
    let (mut lower, mut upper, mut kind) = match target {
        "build" => (
            0.0,
            Some(asset_parameter(
                asset.ok_or_else(|| LoweringError::InvalidFormulation {
                    message: "`build[a]` requires an asset scope".to_string(),
                    path: path.to_path_buf(),
                })?,
                "max_build",
                path,
            )?),
            VariableKind::Continuous,
        ),
        "unserved_energy" => (0.0, None, VariableKind::Continuous),
        "charge" | "discharge" | "generation" => (0.0, None, VariableKind::Continuous),
        "dispatch" => (
            if asset.is_some_and(|asset| has_asset_parameter(asset, "energy_mwh")) {
                f64::NEG_INFINITY
            } else {
                0.0
            },
            None,
            VariableKind::Continuous,
        ),
        "commit" | "start" | "shutdown" => (0.0, Some(1.0), VariableKind::Binary),
        _ => (f64::NEG_INFINITY, None, VariableKind::Continuous),
    };

    if let Some(overrides) = overrides {
        if let Some(decl_kind) = &overrides.kind {
            kind = match decl_kind {
                VariableKindDecl::Continuous => VariableKind::Continuous,
                VariableKindDecl::Integer => VariableKind::Integer,
                VariableKindDecl::Binary => VariableKind::Binary,
            };
        }
        if let Some(bound) = &overrides.lower {
            if let Some(value) = literal_bound_to_f64(bound, path)? {
                lower = value;
            }
        }
        if let Some(bound) = &overrides.upper {
            if let Some(value) = literal_bound_to_f64(bound, path)? {
                upper = Some(value);
            }
        }
    }

    Ok((lower, upper, kind))
}

fn variable_instance_is_active(target: &str, asset: Option<&AssetInputs>) -> bool {
    match target {
        "build" => asset.is_some_and(|asset| asset.candidate),
        "unserved_energy" => true,
        _ => asset.is_some_and(|asset| asset.families.contains(target)),
    }
}

fn lower_constraint_instances(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<Vec<LinearConstraint>, LoweringError> {
    let mut constraints = Vec::new();
    for constraint in &program.active_constraints {
        if constraint.generation_bindings.is_empty() {
            for bindings in
                constraint_instance_bindings(constraint, inputs, program.sets.time.steps)
            {
                let asset = bindings_asset(&bindings, inputs);
                let time = bindings_time(&bindings, entrypoint)?;
                if let Some(filter) = &constraint.generation_filter {
                    if !evaluate_constraint_filter(
                        filter,
                        constraint,
                        FilterScope { asset, time },
                        inputs,
                        entrypoint,
                    )? {
                        continue;
                    }
                }
                constraints.extend(linearize_constraint_body(
                    constraint,
                    &bindings,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                )?);
            }
        } else {
            let generation_scopes = expand_generation_bindings(
                &constraint.generation_bindings,
                inputs,
                program,
                entrypoint,
            )?;
            for scope in generation_scopes {
                if let Some(filter) = &constraint.generation_filter {
                    if !evaluate_reduction_filter(
                        filter,
                        &scope,
                        program,
                        inputs,
                        named_expressions,
                        variable_signatures,
                        instantiated_names,
                        entrypoint,
                    )? {
                        continue;
                    }
                }
                constraints.extend(linearize_constraint_body(
                    constraint,
                    &scope,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                )?);
            }
        }
    }
    Ok(constraints)
}

#[allow(clippy::too_many_arguments)]
fn linearize_constraint_body(
    constraint: &ResolvedConstraint,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<Vec<LinearConstraint>, LoweringError> {
    let suffix = constraint_binding_suffix(bindings, entrypoint)?;
    match &constraint.expression {
        ConstraintBody::Comparison { op, left, right } => Ok(vec![linearize_comparison(
            format!("{}{}", constraint.name, suffix),
            *op,
            left,
            right,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?]),
        ConstraintBody::Range {
            lower,
            lower_op,
            middle,
            upper_op,
            upper,
        } => Ok(vec![
            linearize_comparison(
                format!("{}{}_lower", constraint.name, suffix),
                *lower_op,
                lower,
                middle,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?,
            linearize_comparison(
                format!("{}{}_upper", constraint.name, suffix),
                *upper_op,
                middle,
                upper,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?,
        ]),
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_comparison(
    name: String,
    op: ComparisonOp,
    left: &Expr,
    right: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<LinearConstraint, LoweringError> {
    let left = linearize_value_expr(
        left,
        bindings,
        program,
        inputs,
        named_expressions,
        variable_signatures,
        instantiated_names,
        entrypoint,
    )?;
    let right = linearize_value_expr(
        right,
        bindings,
        program,
        inputs,
        named_expressions,
        variable_signatures,
        instantiated_names,
        entrypoint,
    )?;
    let expression = left.subtract(right);
    let sense = comparison_to_constraint_sense(op, entrypoint)?;
    Ok(LinearConstraint {
        name,
        sense,
        rhs: -expression.constant,
        terms: expression.into_terms(),
    })
}

fn comparison_to_constraint_sense(
    op: ComparisonOp,
    path: &Path,
) -> Result<ConstraintSense, LoweringError> {
    match op {
        ComparisonOp::Equal | ComparisonOp::DoubleEqual => Ok(ConstraintSense::Equal),
        ComparisonOp::LessEqual => Ok(ConstraintSense::LessEqual),
        ComparisonOp::GreaterEqual => Ok(ConstraintSense::GreaterEqual),
        ComparisonOp::Less | ComparisonOp::Greater | ComparisonOp::NotEqual => {
            Err(LoweringError::InvalidFormulation {
                message: format!(
                    "strict or not-equal comparison `{op}` is not supported in linear constraints"
                ),
                path: path.to_path_buf(),
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_value_expr(
    expr: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, LoweringError> {
    match expr {
        Expr::Number(value) => value.parse::<f64>().map(AffineExpr::constant).map_err(|_| {
            LoweringError::InvalidFormulation {
                message: format!("invalid numeric literal `{value}`"),
                path: entrypoint.to_path_buf(),
            }
        }),
        Expr::Identifier(name) => {
            if let Some(binding) = bindings.values.get(name) {
                return Ok(AffineExpr::constant(numeric_filter_value(
                    binding,
                    &synthetic_constraint(name),
                    entrypoint,
                )?));
            }
            if let Some(expression) = named_expressions.get(name) {
                return linearize_value_expr(
                    expression,
                    bindings,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                );
            }
            Err(LoweringError::InvalidFormulation {
                message: format!("unresolved symbol `{name}` in linear expression"),
                path: entrypoint.to_path_buf(),
            })
        }
        Expr::Indexed { target, indices } => linearize_indexed_expr(
            target,
            indices,
            bindings,
            program,
            inputs,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::Unary { op, expr } => {
            let value = linearize_value_expr(
                expr,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            match op {
                UnaryOp::Negate => Ok(value.scale(-1.0)),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = linearize_value_expr(
                left,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            let right = linearize_value_expr(
                right,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            match op {
                BinaryOp::Add => {
                    let mut value = left;
                    value.add_assign(right);
                    Ok(value)
                }
                BinaryOp::Subtract => Ok(left.subtract(right)),
                BinaryOp::Multiply => {
                    if left.terms.is_empty() {
                        Ok(right.scale(left.constant))
                    } else if right.terms.is_empty() {
                        Ok(left.scale(right.constant))
                    } else {
                        Err(LoweringError::InvalidFormulation {
                            message: "non-linear multiplication is not supported".to_string(),
                            path: entrypoint.to_path_buf(),
                        })
                    }
                }
                BinaryOp::Divide => {
                    let denominator = right.as_scalar(entrypoint, "division denominator")?;
                    Ok(left.scale(1.0 / denominator))
                }
            }
        }
        Expr::FunctionCall { name, args } => {
            let evaluated_args = args
                .iter()
                .map(|arg| {
                    let result = linearize_value_expr(
                        arg,
                        bindings,
                        program,
                        inputs,
                        named_expressions,
                        variable_signatures,
                        instantiated_names,
                        entrypoint,
                    )?;
                    result.as_scalar(entrypoint, &format!("{name}() argument"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let value = evaluate_builtin_function(name, &evaluated_args, entrypoint)?;
            Ok(AffineExpr::constant(value))
        }
        Expr::Reduction(reduction) => linearize_reduction(
            reduction,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::String(_) | Expr::Boolean(_) | Expr::Comparison { .. } => {
            Err(LoweringError::InvalidFormulation {
                message: "boolean and string expressions cannot appear in linear algebra"
                    .to_string(),
                path: entrypoint.to_path_buf(),
            })
        }
    }
}

fn evaluate_builtin_function(
    name: &str,
    args: &[f64],
    entrypoint: &Path,
) -> Result<f64, LoweringError> {
    match (name, args.len()) {
        ("sqrt", 1) => Ok(args[0].sqrt()),
        ("abs", 1) => Ok(args[0].abs()),
        ("exp", 1) => Ok(args[0].exp()),
        ("ln", 1) => Ok(args[0].ln()),
        ("pow", 2) => Ok(args[0].powf(args[1])),
        (name, n) => Err(LoweringError::InvalidFormulation {
            message: format!(
                "{name}() received {n} argument(s), expected {}",
                if name == "pow" { 2 } else { 1 }
            ),
            path: entrypoint.to_path_buf(),
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_reduction(
    reduction: &crate::algebra::ReductionExpr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, LoweringError> {
    let expanded =
        expand_reduction_bindings(&reduction.bindings, bindings, inputs, program, entrypoint)?;
    let mut total = AffineExpr::default();
    'outer: for scope in expanded {
        for filter in &reduction.filters {
            if !evaluate_reduction_filter(
                filter,
                &scope,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )? {
                continue 'outer;
            }
        }
        total.add_assign(linearize_value_expr(
            &reduction.body,
            &scope,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?);
    }
    Ok(total)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_reduction_filter(
    filter: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<bool, LoweringError> {
    if let Expr::Comparison { op, left, right } = filter {
        let left_affine = linearize_value_expr(
            left,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?;
        let right_affine = linearize_value_expr(
            right,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?;
        let left_value = left_affine.as_scalar(entrypoint, "reduction filter operand")?;
        let right_value = right_affine.as_scalar(entrypoint, "reduction filter operand")?;
        Ok(match op {
            ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
                (left_value - right_value).abs() < 1e-12
            }
            ComparisonOp::NotEqual => (left_value - right_value).abs() >= 1e-12,
            ComparisonOp::Less => left_value < right_value,
            ComparisonOp::LessEqual => left_value <= right_value,
            ComparisonOp::Greater => left_value > right_value,
            ComparisonOp::GreaterEqual => left_value >= right_value,
        })
    } else {
        let value = linearize_value_expr(
            filter,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?
        .as_scalar(entrypoint, "reduction filter expression")?;
        Ok(value.abs() >= 1e-12)
    }
}

fn expand_reduction_bindings(
    bindings: &[crate::algebra::Binding],
    current: &LinearizationBindings,
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<LinearizationBindings>, LoweringError> {
    let mut scopes = vec![current.clone()];
    for binding in bindings {
        let values = reduction_domain_values(&binding.domain, inputs, program, entrypoint)?;
        let mut next = Vec::new();
        for scope in &scopes {
            match &binding.pattern {
                crate::algebra::BindingPattern::Name(name) => {
                    for value in &values {
                        let mut scope = scope.clone();
                        scope.values.insert(name.clone(), value.clone());
                        next.push(scope);
                    }
                }
                crate::algebra::BindingPattern::Tuple(_) => {
                    return Err(LoweringError::InvalidFormulation {
                        message: "tuple reduction bindings are not lowered yet".to_string(),
                        path: entrypoint.to_path_buf(),
                    });
                }
            }
        }
        scopes = next;
    }
    Ok(scopes)
}

fn expand_generation_bindings(
    bindings: &[GenerationBinding],
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<LinearizationBindings>, LoweringError> {
    let mut scopes = vec![LinearizationBindings::default()];
    for binding in bindings {
        let values = reduction_domain_values(&binding.domain, inputs, program, entrypoint)?;
        let mut next = Vec::new();
        for scope in &scopes {
            for value in &values {
                let mut scope = scope.clone();
                scope.values.insert(binding.variable.clone(), value.clone());
                next.push(scope);
            }
        }
        scopes = next;
    }
    Ok(scopes)
}

fn reduction_domain_values(
    domain: &str,
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<FilterValue>, LoweringError> {
    if let Some((base, selectors)) = parse_inline_selector_domain(domain, entrypoint)? {
        let base_values = reduction_domain_values(base.as_str(), inputs, program, entrypoint)?;
        let filtered = base_values
            .into_iter()
            .filter(|value| domain_value_matches_selectors(value, &selectors, inputs))
            .collect();
        return Ok(filtered);
    }

    match domain {
        "assets" => Ok(inputs
            .assets
            .iter()
            .map(|asset| FilterValue::String(asset.name.clone()))
            .collect()),
        "candidate_assets" => Ok(inputs
            .assets
            .iter()
            .filter(|asset| asset.candidate)
            .map(|asset| FilterValue::String(asset.name.clone()))
            .collect()),
        "time" => Ok((1..=program.sets.time.steps)
            .map(|time| FilterValue::Number(time as f64))
            .collect()),
        _ => {
            if let Some(set) = program.set_registry.get(domain) {
                Ok(set
                    .values
                    .iter()
                    .map(|v| FilterValue::String(v.clone()))
                    .collect())
            } else {
                Err(LoweringError::InvalidFormulation {
                    message: format!("unsupported reduction domain `{domain}`"),
                    path: entrypoint.to_path_buf(),
                })
            }
        }
    }
}

type InlineSelector = (String, String);
type InlineSelectorDomain = (String, Vec<InlineSelector>);

fn parse_inline_selector_domain(
    domain: &str,
    entrypoint: &Path,
) -> Result<Option<InlineSelectorDomain>, LoweringError> {
    let Some(start) = domain.find('[') else {
        return Ok(None);
    };
    let Some(end) = domain.rfind(']') else {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    };
    if end <= start || !domain[end + 1..].trim().is_empty() {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    }

    let base = domain[..start].trim().to_string();
    if base.is_empty() {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    }

    let body = &domain[start + 1..end];
    let bytes = body.as_bytes();
    let mut index = 0;
    let mut selectors = Vec::new();

    while index < bytes.len() {
        while index < bytes.len() && matches!(bytes[index] as char, ' ' | '\t' | ',') {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }

        let key_start = index;
        while index < bytes.len() {
            let character = bytes[index] as char;
            if character == '=' || character == ' ' || character == '\t' || character == ',' {
                break;
            }
            index += 1;
        }
        let key = body[key_start..index].trim();
        if key.is_empty() {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }

        while index < bytes.len() && matches!(bytes[index] as char, ' ' | '\t') {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] as char != '=' {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }
        index += 1;
        while index < bytes.len() && matches!(bytes[index] as char, ' ' | '\t') {
            index += 1;
        }

        if index >= bytes.len() {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }

        let value = if bytes[index] as char == '"' {
            index += 1;
            let mut escaped = false;
            let mut terminated = false;
            let mut literal = String::new();
            while index < bytes.len() {
                let character = bytes[index] as char;
                index += 1;
                if escaped {
                    literal.push(character);
                    escaped = false;
                    continue;
                }
                match character {
                    '\\' => escaped = true,
                    '"' => {
                        terminated = true;
                        break;
                    }
                    _ => literal.push(character),
                }
            }
            if escaped || !terminated {
                return Err(LoweringError::InvalidFormulation {
                    message: format!("invalid inline selector domain `{domain}`"),
                    path: entrypoint.to_path_buf(),
                });
            }
            literal
        } else {
            let value_start = index;
            while index < bytes.len() {
                let character = bytes[index] as char;
                if matches!(character, ' ' | '\t' | ',') {
                    break;
                }
                index += 1;
            }
            body[value_start..index].trim().to_string()
        };

        if value.is_empty() {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }

        selectors.push((key.to_string(), value));
    }

    if selectors.is_empty() {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    }

    Ok(Some((base, selectors)))
}

fn domain_value_matches_selectors(
    value: &FilterValue,
    selectors: &[(String, String)],
    inputs: &ScenarioInputs,
) -> bool {
    let FilterValue::String(member) = value else {
        return false;
    };
    let key = vec![member.clone()];

    selectors.iter().all(|(field, expected)| {
        if let Some(param_values) = inputs.set_params.get(field)
            && let Some(actual) = param_values.get(member)
        {
            return numeric_or_string_match(*actual, expected);
        }

        if let Some(table) = inputs.generic_data.get(field)
            && let Some(actual) = table.values.get(&key)
        {
            return numeric_or_string_match(*actual, expected);
        }

        false
    })
}

fn numeric_or_string_match(actual: f64, expected: &str) -> bool {
    if let Ok(expected_number) = expected.parse::<f64>() {
        return (actual - expected_number).abs() < 1e-9;
    }

    if expected.eq_ignore_ascii_case("true") {
        return (actual - 1.0).abs() < 1e-9;
    }
    if expected.eq_ignore_ascii_case("false") {
        return actual.abs() < 1e-9;
    }

    let actual_string = if actual.fract().abs() < 1e-9 {
        (actual as i64).to_string()
    } else {
        actual.to_string()
    };
    actual_string == expected
}

#[allow(clippy::too_many_arguments)]
fn linearize_indexed_expr(
    target: &str,
    indices: &[Expr],
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, LoweringError> {
    let resolved = indices
        .iter()
        .map(|index| resolve_index_expr(index, bindings, entrypoint))
        .collect::<Result<Vec<_>, _>>()?;

    // Compute the candidate instance name using the same conventions as
    // instantiate_variable_instances / resolve_custom_index_domains.
    let candidate = candidate_instance_name(target, &resolved, entrypoint)?;

    if instantiated_names.contains(&candidate) {
        return Ok(AffineExpr::variable(candidate, 1.0));
    }

    // The candidate was not found in the instantiated set. Before falling
    // through to parameter lookup, handle chronology boundary cases for
    // [String, Number] references where the time index is out of range.
    if let [FilterValue::String(_), FilterValue::Number(_)] = resolved.as_slice() {
        let synthetic = synthetic_constraint(target);
        let asset_name = string_filter_value(&resolved[0], &synthetic, entrypoint)?;
        let time = integer_time_index(&resolved[1], entrypoint)?;

        // Only attempt chronology handling when the time is out of the
        // normal 1..=steps range AND a variable family with matching
        // arity exists (so we know this target is a variable, not a
        // parameter that happens to be missing).
        if !(1..=program.sets.time.steps as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            if let Some(value) =
                chronology_boundary_value(target, &asset_name, time, program, inputs, entrypoint)?
            {
                return Ok(AffineExpr::constant(value));
            }
            return Err(LoweringError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    if let [FilterValue::Number(_)] = resolved.as_slice() {
        let time = integer_time_index(&resolved[0], entrypoint)?;
        if !(1..=program.sets.time.steps as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            return Err(LoweringError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    parameter_reference_expr(target, &resolved, inputs, entrypoint)
}

/// Compute the instance name that `instantiate_variable_instances` would
/// produce for a given target and resolved index values.
fn candidate_instance_name(
    target: &str,
    resolved: &[FilterValue],
    entrypoint: &Path,
) -> Result<String, LoweringError> {
    match resolved {
        [FilterValue::String(a), FilterValue::Number(_)] => {
            let time = integer_time_index(&resolved[1], entrypoint)?;
            Ok(indexed_name(target, a, time as usize))
        }
        [FilterValue::String(a)] => Ok(asset_indexed_name(target, a)),
        [FilterValue::Number(_)] => {
            let time = integer_time_index(&resolved[0], entrypoint)?;
            Ok(time_name(target, time as usize))
        }
        _ => {
            // General case for custom index domains: join all values as
            // strings, matching the format in resolve_custom_index_domains.
            let parts: Vec<String> = resolved
                .iter()
                .map(|v| match v {
                    FilterValue::String(s) => Ok(s.clone()),
                    FilterValue::Number(n) => {
                        if n.fract() == 0.0 {
                            Ok((*n as i64).to_string())
                        } else {
                            Ok(n.to_string())
                        }
                    }
                    FilterValue::Boolean(_) => Err(LoweringError::InvalidFormulation {
                        message: format!("unsupported boolean index in reference to `{target}`"),
                        path: entrypoint.to_path_buf(),
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("{}[{}]", target, parts.join(",")))
        }
    }
}

/// Find the variable family key in the signatures map by matching target
/// name and index arity. Used for traceability and to detect whether a
/// target is a known variable family.
fn find_variable_family<'a>(
    target: &str,
    arity: usize,
    variable_signatures: &'a BTreeMap<String, FamilySignature>,
) -> Option<&'a str> {
    variable_signatures
        .iter()
        .find(|(_key, sig)| sig.target == target && sig.indices.len() == arity)
        .map(|(key, _)| key.as_str())
}

fn parameter_reference_expr(
    target: &str,
    resolved: &[FilterValue],
    inputs: &ScenarioInputs,
    entrypoint: &Path,
) -> Result<AffineExpr, LoweringError> {
    if let Some(value) = generic_data_value(&inputs.generic_data, target, resolved, entrypoint)? {
        return Ok(AffineExpr::constant(value));
    }

    let value = match resolved {
        [index] => {
            if let FilterValue::String(name) = index {
                if find_asset(inputs, name).is_some() {
                    asset_parameter_value(inputs, target, name)
                        .or_else(|| {
                            inputs
                                .asset_data
                                .get(target)
                                .and_then(|values| values.get(name))
                                .copied()
                        })
                        .unwrap_or(0.0)
                } else if let Some(member_params) = inputs.set_params.get(name) {
                    member_params.get(target).copied().unwrap_or(0.0)
                } else {
                    return Err(LoweringError::MissingAsset {
                        name: name.clone(),
                        path: entrypoint.to_path_buf(),
                    });
                }
            } else {
                let time = integer_time_index(index, entrypoint)? as usize;
                series_value(&inputs.series, target, time, entrypoint)?
            }
        }
        [FilterValue::String(asset_name), FilterValue::Number(time)] => {
            if time.fract() != 0.0 || *time < 0.0 {
                return Err(LoweringError::InvalidFormulation {
                    message: format!("time index `{time}` must be a non-negative integer"),
                    path: entrypoint.to_path_buf(),
                });
            }
            indexed_value(
                &inputs.indexed,
                target,
                asset_name,
                *time as usize,
                entrypoint,
            )?
        }
        _ => {
            return Err(LoweringError::InvalidFormulation {
                message: format!("unsupported parameter reference `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    };
    Ok(AffineExpr::constant(value))
}

fn chronology_boundary_value(
    target: &str,
    asset_name: &str,
    time: i64,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    entrypoint: &Path,
) -> Result<Option<f64>, LoweringError> {
    if time == 0 && target == "soc" && program.chronology.initial_boundary.is_some() {
        let asset = find_asset(inputs, asset_name).ok_or_else(|| LoweringError::MissingAsset {
            name: asset_name.to_string(),
            path: entrypoint.to_path_buf(),
        })?;
        return asset_parameter(asset, "initial_soc_mwh", entrypoint).map(Some);
    }
    if time == 0 && target == "commit" && program.chronology.initial_commitment_boundary.is_some() {
        return asset_data_value(
            &inputs.asset_data,
            "initial_commitment",
            asset_name,
            entrypoint,
        )
        .map(Some);
    }
    if time == 0
        && target == "generation"
        && program.chronology.initial_commitment_boundary.is_some()
    {
        let asset = find_asset(inputs, asset_name).ok_or_else(|| LoweringError::MissingAsset {
            name: asset_name.to_string(),
            path: entrypoint.to_path_buf(),
        })?;
        let p_min = asset_parameter(asset, "p_min", entrypoint)?;
        let initial_commitment = asset_data_value(
            &inputs.asset_data,
            "initial_commitment",
            asset_name,
            entrypoint,
        )?;
        return Ok(Some(p_min * initial_commitment));
    }
    Ok(None)
}

fn emit_terminal_boundary_constraints(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    entrypoint: &Path,
) -> Result<Vec<LinearConstraint>, LoweringError> {
    if program.chronology.terminal_boundary.is_none()
        || !variable_signatures.contains_key("soc[a,t]")
    {
        return Ok(Vec::new());
    }

    let mut constraints = Vec::new();
    for asset in &inputs.assets {
        if !asset.families.contains("soc") {
            continue;
        }
        constraints.push(LinearConstraint {
            name: format!("terminal_soc[{}]", asset.name),
            sense: ConstraintSense::Equal,
            rhs: asset_parameter(asset, "terminal_soc_mwh", entrypoint)?,
            terms: vec![term(
                &indexed_name("soc", &asset.name, program.sets.time.steps),
                1.0,
            )],
        });
    }
    Ok(constraints)
}

fn constraint_instance_bindings(
    constraint: &ResolvedConstraint,
    inputs: &ScenarioInputs,
    steps: usize,
) -> Vec<LinearizationBindings> {
    let binds_asset = constraint_uses_free_index(&constraint.expression, "a");
    let binds_time = constraint_uses_free_index(&constraint.expression, "t");
    let assets = if binds_asset {
        relevant_constraint_assets(constraint, inputs)
            .into_iter()
            .map(|asset| asset.name.clone())
            .collect::<Vec<_>>()
    } else {
        vec![String::new()]
    };
    let times = if binds_time {
        (1..=steps).collect::<Vec<_>>()
    } else {
        vec![0]
    };

    let mut bindings = Vec::new();
    for asset in &assets {
        for time in &times {
            let mut scope = LinearizationBindings::default();
            if binds_asset {
                scope
                    .values
                    .insert("a".to_string(), FilterValue::String(asset.clone()));
            }
            if binds_time {
                scope
                    .values
                    .insert("t".to_string(), FilterValue::Number(*time as f64));
            }
            bindings.push(scope);
        }
    }
    bindings
}

fn relevant_constraint_assets<'a>(
    constraint: &ResolvedConstraint,
    inputs: &'a ScenarioInputs,
) -> Vec<&'a AssetInputs> {
    if constraint.source_kind == "operation" {
        return inputs
            .assets
            .iter()
            .filter(|asset| asset.operation.as_deref() == Some(constraint.source_name.as_str()))
            .collect();
    }
    inputs.assets.iter().collect()
}

fn constraint_uses_free_index(body: &ConstraintBody, name: &str) -> bool {
    match body {
        ConstraintBody::Comparison { left, right, .. } => {
            expr_uses_free_index(left, name, &mut BTreeSet::new())
                || expr_uses_free_index(right, name, &mut BTreeSet::new())
        }
        ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            expr_uses_free_index(lower, name, &mut BTreeSet::new())
                || expr_uses_free_index(middle, name, &mut BTreeSet::new())
                || expr_uses_free_index(upper, name, &mut BTreeSet::new())
        }
    }
}

fn expr_uses_free_index(expr: &Expr, name: &str, bound: &mut BTreeSet<String>) -> bool {
    match expr {
        Expr::Identifier(identifier) => identifier == name && !bound.contains(identifier),
        Expr::Indexed { indices, .. } => indices
            .iter()
            .any(|index| expr_uses_free_index(index, name, bound)),
        Expr::Unary { expr, .. } => expr_uses_free_index(expr, name, bound),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            expr_uses_free_index(left, name, bound) || expr_uses_free_index(right, name, bound)
        }
        Expr::Reduction(reduction) => {
            let mut local_bound = bound.clone();
            for binding in &reduction.bindings {
                match &binding.pattern {
                    crate::algebra::BindingPattern::Name(identifier) => {
                        local_bound.insert(identifier.clone());
                    }
                    crate::algebra::BindingPattern::Tuple(identifiers) => {
                        local_bound.extend(identifiers.iter().cloned());
                    }
                }
            }
            expr_uses_free_index(&reduction.body, name, &mut local_bound)
                || reduction
                    .filters
                    .iter()
                    .any(|filter| expr_uses_free_index(filter, name, &mut local_bound))
        }
        Expr::FunctionCall { args, .. } => args
            .iter()
            .any(|arg| expr_uses_free_index(arg, name, bound)),
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => false,
    }
}

fn bindings_asset<'a>(
    bindings: &LinearizationBindings,
    inputs: &'a ScenarioInputs,
) -> Option<&'a AssetInputs> {
    bindings.values.get("a").and_then(|value| match value {
        FilterValue::String(name) => find_asset(inputs, name),
        _ => None,
    })
}

fn bindings_time(
    bindings: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<Option<usize>, LoweringError> {
    bindings
        .values
        .get("t")
        .map(|value| integer_time_index(value, entrypoint).map(|time| time as usize))
        .transpose()
}

fn constraint_binding_suffix(
    bindings: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<String, LoweringError> {
    let mut indices = Vec::new();
    if let Some(value) = bindings.values.get("a") {
        indices.push(string_filter_value(
            value,
            &synthetic_constraint("constraint"),
            entrypoint,
        )?);
    }
    if let Some(value) = bindings.values.get("t") {
        indices.push(integer_time_index(value, entrypoint)?.to_string());
    }
    // Include any custom binding variables (e.g. "b" from `over "b" in="periods"`)
    for (name, value) in &bindings.values {
        if name == "a" || name == "t" {
            continue;
        }
        match value {
            FilterValue::String(s) => indices.push(s.clone()),
            FilterValue::Number(n) => {
                if n.fract() == 0.0 {
                    indices.push((*n as i64).to_string());
                } else {
                    indices.push(n.to_string());
                }
            }
            FilterValue::Boolean(b) => indices.push(b.to_string()),
        }
    }
    if indices.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("[{}]", indices.join(",")))
    }
}

fn integer_time_index(value: &FilterValue, entrypoint: &Path) -> Result<i64, LoweringError> {
    match value {
        FilterValue::Number(number) => {
            if number.fract() == 0.0 {
                Ok(*number as i64)
            } else {
                Err(LoweringError::InvalidFormulation {
                    message: format!("time index `{number}` must be integral"),
                    path: entrypoint.to_path_buf(),
                })
            }
        }
        FilterValue::String(value) => {
            value
                .parse::<i64>()
                .map_err(|_| LoweringError::InvalidFormulation {
                    message: format!("time index `{value}` must be integral"),
                    path: entrypoint.to_path_buf(),
                })
        }
        FilterValue::Boolean(value) => Err(LoweringError::InvalidFormulation {
            message: format!("time index `{value}` must be numeric"),
            path: entrypoint.to_path_buf(),
        }),
    }
}

fn resolve_index_expr(
    expr: &Expr,
    bindings: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<FilterValue, LoweringError> {
    match expr {
        Expr::Identifier(name) => {
            bindings
                .values
                .get(name)
                .cloned()
                .ok_or_else(|| LoweringError::InvalidFormulation {
                    message: format!("unbound index identifier `{name}`"),
                    path: entrypoint.to_path_buf(),
                })
        }
        Expr::Number(value) => value.parse::<f64>().map(FilterValue::Number).map_err(|_| {
            LoweringError::InvalidFormulation {
                message: format!("invalid numeric index `{value}`"),
                path: entrypoint.to_path_buf(),
            }
        }),
        Expr::String(value) => Ok(FilterValue::String(value.clone())),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Negate => Ok(FilterValue::Number(-numeric_filter_value(
                &resolve_index_expr(expr, bindings, entrypoint)?,
                &synthetic_constraint("index"),
                entrypoint,
            )?)),
        },
        Expr::Binary { op, left, right } => {
            let left = resolve_index_expr(left, bindings, entrypoint)?;
            let right = resolve_index_expr(right, bindings, entrypoint)?;
            let left = numeric_filter_value(&left, &synthetic_constraint("index"), entrypoint)?;
            let right = numeric_filter_value(&right, &synthetic_constraint("index"), entrypoint)?;
            Ok(FilterValue::Number(match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => left / right,
            }))
        }
        _ => Err(LoweringError::InvalidFormulation {
            message: "unsupported index expression during lowering".to_string(),
            path: entrypoint.to_path_buf(),
        }),
    }
}

fn synthetic_constraint(name: &str) -> ResolvedConstraint {
    ResolvedConstraint {
        name: name.to_string(),
        source_kind: "lowering".to_string(),
        source_name: "synthetic".to_string(),
        expression_text: String::new(),
        expression: ConstraintBody::Comparison {
            op: ComparisonOp::Equal,
            left: Expr::Number("0".to_string()),
            right: Expr::Number("0".to_string()),
        },
        generation_bindings: Vec::new(),
        generation_filter_text: None,
        generation_filter: None,
    }
}

fn read_csv_rows(path: &Path) -> Result<Vec<HashMap<String, String>>, LoweringError> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| LoweringError::Csv {
        path: path.to_path_buf(),
        source,
    })?;
    let headers = reader
        .headers()
        .map_err(|source| LoweringError::Csv {
            path: path.to_path_buf(),
            source,
        })?
        .clone();

    reader
        .records()
        .map(|record| {
            let record = record.map_err(|source| LoweringError::Csv {
                path: path.to_path_buf(),
                source,
            })?;
            record_to_map(path, &headers, record)
        })
        .collect()
}

fn record_to_map(
    path: &Path,
    headers: &StringRecord,
    record: StringRecord,
) -> Result<HashMap<String, String>, LoweringError> {
    let mut row = HashMap::with_capacity(headers.len());
    for i in 0..headers.len() {
        if let Some(value) = record.get(i) {
            row.insert(headers[i].to_string(), value.to_string());
        }
    }
    if row.is_empty() {
        return Err(LoweringError::MissingData {
            name: "csv".to_string(),
            path: path.to_path_buf(),
        });
    }
    Ok(row)
}

fn parse_usize_field(
    row: &HashMap<String, String>,
    field: &str,
    path: &Path,
) -> Result<usize, LoweringError> {
    let raw = row
        .get(field)
        .cloned()
        .ok_or_else(|| LoweringError::MissingColumn {
            column: field.to_string(),
            path: path.to_path_buf(),
        })?;
    raw.parse::<usize>()
        .map_err(|_| LoweringError::InvalidNumber {
            value: raw,
            field: field.to_string(),
            path: path.to_path_buf(),
        })
}

fn parse_data_value(
    row: &HashMap<String, String>,
    name: &str,
    path: &Path,
) -> Result<f64, LoweringError> {
    let raw = row
        .get(name)
        .cloned()
        .or_else(|| row.get("value").cloned())
        .ok_or_else(|| LoweringError::MissingColumn {
            column: name.to_string(),
            path: path.to_path_buf(),
        })?;
    raw.parse::<f64>()
        .map_err(|_| LoweringError::InvalidNumber {
            value: raw,
            field: name.to_string(),
            path: path.to_path_buf(),
        })
}

fn load_data_decl_params(
    data_decl: &DataDecl,
    entry_dir: &Path,
    generic_data: &mut BTreeMap<String, GenericDataTable>,
    set_params: &mut BTreeMap<String, BTreeMap<String, f64>>,
) -> Result<(), LoweringError> {
    let csv_path = entry_dir.join(&data_decl.source);
    let rows = read_csv_rows(&csv_path)?;
    if rows.is_empty() {
        return Ok(());
    }

    for parameter in &data_decl.parameters {
        let value_column = parameter
            .from
            .clone()
            .unwrap_or_else(|| parameter.name.clone());
        let key_columns = resolve_param_key_columns(data_decl, parameter);
        let mut values: BTreeMap<Vec<String>, f64> = BTreeMap::new();
        let mut counts: BTreeMap<Vec<String>, usize> = BTreeMap::new();

        for row in &rows {
            if !matches_data_param_filter(row, data_decl, parameter) {
                continue;
            }

            let value_raw = row
                .get(&value_column)
                .cloned()
                .or_else(|| row.get("value").cloned())
                .ok_or_else(|| LoweringError::MissingColumn {
                    column: value_column.clone(),
                    path: csv_path.clone(),
                })?;
            let value = value_raw
                .parse::<f64>()
                .map_err(|_| LoweringError::InvalidNumber {
                    value: value_raw,
                    field: value_column.clone(),
                    path: csv_path.clone(),
                })?;

            let key = key_columns
                .iter()
                .map(|column| {
                    row.get(column)
                        .cloned()
                        .ok_or_else(|| LoweringError::MissingColumn {
                            column: column.clone(),
                            path: csv_path.clone(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let reducer = parameter.reduce.as_deref().unwrap_or("replace");
            match reducer {
                "sum" => {
                    *values.entry(key.clone()).or_insert(0.0) += value;
                }
                "min" => {
                    values
                        .entry(key.clone())
                        .and_modify(|existing| *existing = existing.min(value))
                        .or_insert(value);
                }
                "max" => {
                    values
                        .entry(key.clone())
                        .and_modify(|existing| *existing = existing.max(value))
                        .or_insert(value);
                }
                "avg" | "mean" => {
                    *values.entry(key.clone()).or_insert(0.0) += value;
                    *counts.entry(key).or_insert(0) += 1;
                }
                _ => {
                    values.insert(key, value);
                }
            }
        }

        if matches!(parameter.reduce.as_deref(), Some("avg" | "mean")) {
            for (key, sum) in &mut values {
                if let Some(count) = counts.get(key) {
                    *sum /= *count as f64;
                }
            }
        }

        if key_columns.len() == 1 {
            let members = set_params.entry(parameter.name.clone()).or_default();
            for (key, value) in &values {
                if let Some(member) = key.first() {
                    members.insert(member.clone(), *value);
                }
            }
        }

        generic_data.insert(
            parameter.name.clone(),
            GenericDataTable {
                values,
                default_missing: None,
            },
        );
    }

    Ok(())
}

fn resolve_param_key_columns(data_decl: &DataDecl, parameter: &ParamDecl) -> Vec<String> {
    if !parameter.indices.is_empty() {
        return parameter
            .indices
            .iter()
            .map(|index| resolve_data_column(data_decl, index))
            .collect();
    }

    if let Some(index_by) = &parameter.index_by {
        return vec![resolve_data_column(data_decl, index_by)];
    }

    if let Some(index_decl) = data_decl.indices.first() {
        return index_decl
            .columns
            .iter()
            .map(|column| resolve_data_column(data_decl, column))
            .collect();
    }

    Vec::new()
}

fn resolve_data_column(data_decl: &DataDecl, logical_name: &str) -> String {
    data_decl
        .maps
        .iter()
        .find(|mapping| mapping.name == logical_name)
        .map_or_else(
            || logical_name.to_string(),
            |mapping| {
                mapping
                    .source
                    .clone()
                    .unwrap_or_else(|| mapping.name.clone())
            },
        )
}

fn matches_data_param_filter(
    row: &HashMap<String, String>,
    data_decl: &DataDecl,
    parameter: &ParamDecl,
) -> bool {
    if parameter.comparators.eq.is_none()
        && parameter.comparators.ge.is_none()
        && parameter.comparators.geq.is_none()
        && parameter.comparators.le.is_none()
        && parameter.comparators.leq.is_none()
    {
        return true;
    }

    let filter_column = parameter
        .filter_by
        .as_ref()
        .or(parameter.from.as_ref())
        .map_or_else(
            || resolve_data_column(data_decl, &parameter.name),
            |logical_name| resolve_data_column(data_decl, logical_name),
        );
    let Some(raw_value) = row.get(&filter_column) else {
        return false;
    };

    compare_data_param_comparators(raw_value, &parameter.comparators)
}

fn compare_data_param_comparators(raw_value: &str, comparators: &FilterComparators) -> bool {
    if let Some(expected) = &comparators.eq {
        if !literal_filter_match(raw_value, expected) {
            return false;
        }
    }
    if let Some(expected) = &comparators.ge {
        if !numeric_filter_compare(raw_value, expected, |left, right| left > right) {
            return false;
        }
    }
    if let Some(expected) = &comparators.geq {
        if !numeric_filter_compare(raw_value, expected, |left, right| left >= right) {
            return false;
        }
    }
    if let Some(expected) = &comparators.le {
        if !numeric_filter_compare(raw_value, expected, |left, right| left < right) {
            return false;
        }
    }
    if let Some(expected) = &comparators.leq {
        if !numeric_filter_compare(raw_value, expected, |left, right| left <= right) {
            return false;
        }
    }
    true
}

fn literal_filter_match(raw_value: &str, literal: &LiteralValue) -> bool {
    match literal {
        LiteralValue::String(value) => raw_value == value,
        LiteralValue::Integer(value) => raw_value.parse::<i128>() == Ok(*value),
        LiteralValue::Decimal(value) => {
            let Ok(expected) = value.parse::<f64>() else {
                return false;
            };
            raw_value
                .parse::<f64>()
                .map(|actual| (actual - expected).abs() < 1e-9)
                .unwrap_or(false)
        }
        LiteralValue::Boolean(value) => {
            let normalized = raw_value.trim().to_ascii_lowercase();
            (*value && (normalized == "true" || normalized == "1"))
                || (!*value && (normalized == "false" || normalized == "0"))
        }
    }
}

fn numeric_filter_compare(
    raw_value: &str,
    literal: &LiteralValue,
    comparator: impl Fn(f64, f64) -> bool,
) -> bool {
    let Ok(actual) = raw_value.parse::<f64>() else {
        return false;
    };
    let expected = match literal {
        LiteralValue::Integer(value) => *value as f64,
        LiteralValue::Decimal(value) | LiteralValue::String(value) => {
            let Ok(parsed) = value.parse::<f64>() else {
                return false;
            };
            parsed
        }
        LiteralValue::Boolean(value) => {
            if *value {
                1.0
            } else {
                0.0
            }
        }
    };
    comparator(actual, expected)
}

fn load_generic_data_table(
    program: &SemanticProgram,
    binding_name: &str,
    rows: &[HashMap<String, String>],
    path: &Path,
) -> Result<GenericDataTable, LoweringError> {
    let Some(first_row) = rows.first() else {
        return Err(LoweringError::MissingData {
            name: binding_name.to_string(),
            path: path.to_path_buf(),
        });
    };
    let mut headers = first_row.keys().cloned().collect::<Vec<_>>();
    headers.sort();
    let value_column = if headers.iter().any(|h| h == binding_name) {
        Some(binding_name.to_string())
    } else if headers.iter().any(|h| h == "value") {
        Some("value".to_string())
    } else {
        None
    };

    let key_columns = infer_data_key_columns(program, rows, &headers, value_column.as_deref());
    let mut values = BTreeMap::new();
    for row in rows {
        let key = key_columns
            .iter()
            .map(|column| {
                row.get(column)
                    .cloned()
                    .ok_or_else(|| LoweringError::MissingColumn {
                        column: column.clone(),
                        path: path.to_path_buf(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let value = if let Some(column) = &value_column {
            let raw = row
                .get(column)
                .cloned()
                .ok_or_else(|| LoweringError::MissingColumn {
                    column: column.clone(),
                    path: path.to_path_buf(),
                })?;
            raw.parse::<f64>()
                .map_err(|_| LoweringError::InvalidNumber {
                    value: raw,
                    field: column.clone(),
                    path: path.to_path_buf(),
                })?
        } else {
            1.0
        };
        values.insert(key, value);
    }

    Ok(GenericDataTable {
        values,
        default_missing: value_column.is_none().then_some(0.0),
    })
}

fn infer_data_key_columns(
    program: &SemanticProgram,
    rows: &[HashMap<String, String>],
    headers: &[String],
    value_column: Option<&str>,
) -> Vec<String> {
    let candidate_columns: Vec<String> = headers
        .iter()
        .filter(|column| value_column != Some(column.as_str()))
        .cloned()
        .collect();

    // Use columns whose values match a declared set as key columns.
    let membership_columns: Vec<String> = candidate_columns
        .iter()
        .filter(|column| data_column_matches_any_set(program, rows, column))
        .cloned()
        .collect();
    if !membership_columns.is_empty() {
        return membership_columns;
    }

    // Fallback: treat every non-value column as a key column.
    candidate_columns
}

fn data_column_matches_any_set(
    program: &SemanticProgram,
    rows: &[HashMap<String, String>],
    column: &str,
) -> bool {
    let mut column_values = BTreeSet::new();
    for row in rows {
        let Some(value) = row.get(column) else {
            return false;
        };
        column_values.insert(value.as_str());
    }
    if column_values.is_empty() {
        return false;
    }

    program.set_registry.values().any(|set| {
        column_values
            .iter()
            .all(|v| set.values.iter().any(|s| s == v))
    })
}

fn generic_data_value(
    generic_data: &BTreeMap<String, GenericDataTable>,
    target: &str,
    resolved: &[FilterValue],
    entrypoint: &Path,
) -> Result<Option<f64>, LoweringError> {
    let Some(table) = generic_data.get(target) else {
        return Ok(None);
    };

    let key = resolved
        .iter()
        .map(|value| filter_value_to_key_component(value, entrypoint))
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(value) = table.values.get(&key).copied() {
        return Ok(Some(value));
    }

    for prefix_len in (1..key.len()).rev() {
        if let Some(value) = table.values.get(&key[..prefix_len]).copied() {
            return Ok(Some(value));
        }
    }

    Ok(table.default_missing)
}

fn filter_value_to_key_component(
    value: &FilterValue,
    entrypoint: &Path,
) -> Result<String, LoweringError> {
    match value {
        FilterValue::String(value) => Ok(value.clone()),
        FilterValue::Number(value) => {
            if value.fract() == 0.0 {
                Ok((*value as i64).to_string())
            } else {
                Ok(value.to_string())
            }
        }
        FilterValue::Boolean(_) => Err(LoweringError::InvalidFormulation {
            message: "boolean indices are not supported for data lookups".to_string(),
            path: entrypoint.to_path_buf(),
        }),
    }
}

/// Convert a `BoundExpr::Literal` to `f64`. Returns `None` for
/// `BoundExpr::Formula` (parameter-based bounds need a linearization context
/// that isn't available yet).
fn literal_bound_to_f64(bound: &BoundExpr, path: &Path) -> Result<Option<f64>, LoweringError> {
    match bound {
        BoundExpr::Literal(literal) => literal_to_f64("bound", literal, path).map(Some),
        BoundExpr::Formula(_) => Ok(None),
    }
}

fn literal_to_f64(name: &str, value: &LiteralValue, path: &Path) -> Result<f64, LoweringError> {
    match value {
        LiteralValue::Integer(value) => Ok(*value as f64),
        LiteralValue::Decimal(value) | LiteralValue::String(value) => {
            value
                .parse::<f64>()
                .map_err(|_| LoweringError::InvalidNumber {
                    value: value.clone(),
                    field: name.to_string(),
                    path: path.to_path_buf(),
                })
        }
        LiteralValue::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),
    }
}

fn asset_parameter(asset: &AssetInputs, name: &str, path: &Path) -> Result<f64, LoweringError> {
    asset
        .parameters
        .get(name)
        .copied()
        .ok_or_else(|| LoweringError::MissingParameter {
            name: name.to_string(),
            asset: asset.name.clone(),
            path: path.to_path_buf(),
        })
}

fn has_asset_parameter(asset: &AssetInputs, name: &str) -> bool {
    asset.parameters.contains_key(name)
}

fn series_value(
    series: &BTreeMap<String, BTreeMap<usize, f64>>,
    name: &str,
    time: usize,
    path: &Path,
) -> Result<f64, LoweringError> {
    series
        .get(name)
        .ok_or_else(|| LoweringError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(&time)
        .copied()
        .ok_or_else(|| LoweringError::MissingDataPoint {
            name: name.to_string(),
            key: time.to_string(),
            path: path.to_path_buf(),
        })
}

fn indexed_value(
    indexed: &BTreeMap<String, BTreeMap<(String, usize), f64>>,
    name: &str,
    asset_name: &str,
    time: usize,
    path: &Path,
) -> Result<f64, LoweringError> {
    indexed
        .get(name)
        .ok_or_else(|| LoweringError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(&(asset_name.to_string(), time))
        .copied()
        .ok_or_else(|| LoweringError::MissingDataPoint {
            name: name.to_string(),
            key: format!("{asset_name},{time}"),
            path: path.to_path_buf(),
        })
}

fn asset_data_value(
    asset_data: &BTreeMap<String, BTreeMap<String, f64>>,
    name: &str,
    asset_name: &str,
    path: &Path,
) -> Result<f64, LoweringError> {
    asset_data
        .get(name)
        .ok_or_else(|| LoweringError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(asset_name)
        .copied()
        .ok_or_else(|| LoweringError::MissingDataPoint {
            name: name.to_string(),
            key: asset_name.to_string(),
            path: path.to_path_buf(),
        })
}

fn objective_sense(value: &str) -> ObjectiveSense {
    match value {
        "maximize" => ObjectiveSense::Maximize,
        _ => ObjectiveSense::Minimize,
    }
}

fn term(variable_name: &str, coefficient: f64) -> LinearTerm {
    LinearTerm {
        variable_name: variable_name.to_string(),
        coefficient,
    }
}

fn indexed_name(family: &str, asset_name: &str, time: usize) -> String {
    format!("{family}[{asset_name},{time}]")
}

fn time_name(family: &str, time: usize) -> String {
    format!("{family}[{time}]")
}

fn asset_indexed_name(target: &str, asset: &str) -> String {
    format!("{target}[{asset}]")
}
