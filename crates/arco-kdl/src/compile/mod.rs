// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use crate::algebra::{BinaryOp, ComparisonOp, ConstraintBody, Expr, UnaryOp};
use crate::semantic::{
    FamilySignature, ResolvedConstraint, ResolvedObjective, ResolvedReport, SemanticProgram,
    VariableDeclOverrides,
};
use crate::source::{
    BoundExpr, DataDecl, GenerationBinding, LiteralValue, ParamDecl, ScenarioDecl, SourceProgram,
    VariableKindDecl,
};
use csv::StringRecord;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledProblem {
    pub parameters: Vec<CompiledParameter>,
    pub variables: Vec<CompiledVariable>,
    pub constraints: Vec<CompiledConstraint>,
    pub objective: CompiledObjective,
    pub reports: Vec<CompiledReport>,
    pub dual_reports: Vec<CompiledDualReport>,
    pub traceability: Vec<TraceabilityRecord>,
    pub algebra: AlgebraicProblem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledParameter {
    pub name: String,
    pub binding_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledVariable {
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledConstraint {
    pub name: String,
    pub source_kind: String,
    pub source_name: String,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledObjective {
    pub name: String,
    pub sense: ObjectiveSense,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledReport {
    pub name: String,
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledDualReport {
    pub constraint_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceabilityRecord {
    pub dsl_name: String,
    pub artifact_kind: String,
    pub compiled_name: String,
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

pub use crate::ObjectiveSense;

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

pub fn compile_program(
    program: &SemanticProgram,
    source_program: &SourceProgram,
    entrypoint: &Path,
) -> Result<CompiledProblem, CompileError> {
    info!("compiling program");

    let scenario = source_program
        .scenario(&program.active_scenario)
        .ok_or_else(|| CompileError::MissingScenario {
            name: program.active_scenario.clone(),
            path: entrypoint.to_path_buf(),
        })?;
    let mut inputs = load_inputs(program, source_program, scenario, entrypoint)?;
    inputs.set_params.extend(program.set_params.clone());
    let algebra = compile_algebra(program, &inputs, entrypoint)?;

    let parameters = [
        ("series", &program.parameters.series),
        ("indexed", &program.parameters.indexed),
        ("asset", &program.parameters.asset),
    ]
    .into_iter()
    .flat_map(|(kind, names)| {
        names.iter().map(move |name| CompiledParameter {
            name: name.clone(),
            binding_kind: kind.to_string(),
        })
    })
    .collect::<Vec<_>>();

    let variables = program
        .variable_families
        .iter()
        .map(|family| CompiledVariable {
            family: family.render(),
        })
        .collect::<Vec<_>>();

    let constraints = program
        .active_constraints
        .iter()
        .map(compile_constraint)
        .collect::<Vec<_>>();
    let objective = compile_objective(&program.active_objective);
    let reports = program
        .active_reports
        .iter()
        .map(compile_report)
        .collect::<Vec<_>>();

    let dual_reports = program
        .active_dual_reports
        .iter()
        .map(|dr| CompiledDualReport {
            constraint_name: dr.constraint_name.clone(),
        })
        .collect::<Vec<_>>();

    let mut traceability = Vec::new();
    traceability.extend(variables.iter().map(|variable| TraceabilityRecord {
        dsl_name: variable.family.clone(),
        artifact_kind: "variable".to_string(),
        compiled_name: variable.family.clone(),
    }));
    traceability.push(TraceabilityRecord {
        dsl_name: objective.name.clone(),
        artifact_kind: "objective".to_string(),
        compiled_name: objective.name.clone(),
    });
    traceability.extend(reports.iter().map(|report| TraceabilityRecord {
        dsl_name: report.name.clone(),
        artifact_kind: "report".to_string(),
        compiled_name: report.name.clone(),
    }));

    let compiled = CompiledProblem {
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
        compiled.algebra.variable_instances.len(),
        compiled.algebra.constraints.len(),
        compiled.reports.len()
    );

    Ok(compiled)
}

fn compile_constraint(constraint: &ResolvedConstraint) -> CompiledConstraint {
    CompiledConstraint {
        name: constraint.name.clone(),
        source_kind: constraint.source_kind.clone(),
        source_name: constraint.source_name.clone(),
        expression: constraint.expression_text.clone(),
    }
}

fn compile_objective(objective: &ResolvedObjective) -> CompiledObjective {
    CompiledObjective {
        name: objective.name.clone(),
        sense: objective.sense.clone(),
        expression: objective.expression_text.clone(),
    }
}

fn compile_report(report: &ResolvedReport) -> CompiledReport {
    CompiledReport {
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
