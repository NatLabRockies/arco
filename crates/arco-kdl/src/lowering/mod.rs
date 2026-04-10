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

include!("error.rs");

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

include!("filters.rs");
include!("data.rs");
include!("algebra.rs");
include!("constraints.rs");
include!("expressions.rs");
include!("expressions_domains.rs");
include!("expressions_lookup.rs");
include!("constraints_bindings.rs");
include!("expressions_misc.rs");
include!("data_tables.rs");
include!("data_values.rs");
