use crate::ArcoOps;
use crate::execution::{ExecutionError, RustArcoAdapter, SolveStatus, execute_problem};
use crate::kdl::ObjectiveSense;
use crate::kdl::pipeline::PipelineError;
use crate::kdl::semantic::{ResolvedChronology, ResolvedParameters, SemanticProgram};
use crate::targets::ObjectiveSense as TargetObjectiveSense;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse json {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Pipeline(#[from] PipelineError),
    #[error(transparent)]
    Execution(#[from] ExecutionError),
    #[error("missing required node `{name}` in {path}")]
    MissingNode { name: &'static str, path: PathBuf },
    #[error("semantic program mismatch for case `{case_id}`")]
    SemanticMismatch { case_id: String },
    #[error("e2e summary mismatch for case `{case_id}`")]
    E2eSummaryMismatch { case_id: String },
}

#[derive(Debug, Deserialize)]
pub struct BenchmarkManifest {
    pub version: u32,
    pub cases: Vec<BenchmarkCaseDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct BenchmarkCaseDefinition {
    pub id: String,
    pub description: String,
    pub entrypoint: String,
    pub expected_semantic_program: String,
    pub expected_e2e_summary: String,
    pub solvable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SemanticProgramExpectation {
    pub case_id: String,
    pub active_scenario: String,
    pub sets: ExpectedSets,
    #[serde(default)]
    pub parameters: ExpectedParameters,
    #[serde(default)]
    pub variable_families: Vec<String>,
    #[serde(default)]
    pub chronology: ExpectedChronology,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExpectedSets {
    pub assets: Vec<String>,
    #[serde(default)]
    pub candidate_assets: Vec<String>,
    pub time: ExpectedTimeSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExpectedTimeSet {
    pub steps: usize,
    pub resolution: String,
}

pub type ExpectedParameters = ResolvedParameters;

pub type ExpectedChronology = ResolvedChronology;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExpectedE2eSummary {
    pub case_id: String,
    pub expect_parse_success: bool,
    pub expect_semantic_validation_success: bool,
    pub expect_compile_success: bool,
    pub expect_solve_success: bool,
    #[serde(default)]
    pub objective: Option<ExpectedObjective>,
    #[serde(default)]
    pub reports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExpectedObjective {
    pub name: String,
    pub sense: ObjectiveSense,
}

#[derive(Debug)]
pub struct CaseOutcome {
    pub case_id: String,
    pub actual_semantic_program: SemanticProgramExpectation,
    pub actual_e2e_summary: ExpectedE2eSummary,
}

pub fn load_manifest(path: &Path) -> Result<BenchmarkManifest, BenchmarkError> {
    read_json(path)
}

pub fn evaluate_manifest(path: &Path) -> Result<Vec<CaseOutcome>, BenchmarkError> {
    let manifest = load_manifest(path)?;
    let repo_root = path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| BenchmarkError::MissingNode {
            name: "manifest parent",
            path: path.to_path_buf(),
        })?;

    manifest
        .cases
        .iter()
        .map(|case| evaluate_case(repo_root, case))
        .collect()
}

fn evaluate_case(
    repo_root: &Path,
    case: &BenchmarkCaseDefinition,
) -> Result<CaseOutcome, BenchmarkError> {
    let entrypoint = repo_root.join(&case.entrypoint);
    let compiled = ArcoOps::compile_file(&entrypoint)?;
    let execution_result = execute_problem(&compiled.compiled_problem, &RustArcoAdapter::new())?;

    let actual_semantic_program =
        to_semantic_expectation(case, &compiled.semantic_program, &entrypoint)?;
    let actual_e2e_summary = to_e2e_summary(case, &execution_result);
    let expected_semantic_program = read_json(&repo_root.join(&case.expected_semantic_program))?;
    let expected_e2e_summary = read_json(&repo_root.join(&case.expected_e2e_summary))?;

    if actual_semantic_program != expected_semantic_program {
        return Err(BenchmarkError::SemanticMismatch {
            case_id: case.id.clone(),
        });
    }

    if actual_e2e_summary != expected_e2e_summary {
        return Err(BenchmarkError::E2eSummaryMismatch {
            case_id: case.id.clone(),
        });
    }

    Ok(CaseOutcome {
        case_id: case.id.clone(),
        actual_semantic_program,
        actual_e2e_summary,
    })
}

fn to_semantic_expectation(
    case: &BenchmarkCaseDefinition,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<SemanticProgramExpectation, BenchmarkError> {
    Ok(SemanticProgramExpectation {
        case_id: case.id.clone(),
        active_scenario: program.active_scenario.clone(),
        sets: ExpectedSets {
            assets: required_set_values(program, "assets", entrypoint)?,
            candidate_assets: program
                .set_registry
                .get("candidate_assets")
                .map(|set| set.values.clone())
                .unwrap_or_default(),
            time: ExpectedTimeSet {
                steps: program.sets.time.steps,
                resolution: program.sets.time.resolution.to_string(),
            },
        },
        parameters: ExpectedParameters {
            series: program.parameters.series.clone(),
            indexed: program.parameters.indexed.clone(),
            asset: program.parameters.asset.clone(),
        },
        variable_families: program
            .variable_families
            .iter()
            .map(|f| f.render())
            .collect(),
        chronology: ExpectedChronology {
            initial_boundary: program.chronology.initial_boundary.clone(),
            terminal_boundary: program.chronology.terminal_boundary.clone(),
            initial_commitment_boundary: program.chronology.initial_commitment_boundary.clone(),
        },
    })
}

fn required_set_values(
    program: &SemanticProgram,
    set_name: &'static str,
    entrypoint: &Path,
) -> Result<Vec<String>, BenchmarkError> {
    program
        .set_registry
        .get(set_name)
        .map(|set| set.values.clone())
        .ok_or_else(|| BenchmarkError::MissingNode {
            name: set_name,
            path: entrypoint.to_path_buf(),
        })
}

fn to_e2e_summary(
    case: &BenchmarkCaseDefinition,
    execution_result: &crate::execution::ExecutionResult,
) -> ExpectedE2eSummary {
    ExpectedE2eSummary {
        case_id: case.id.clone(),
        expect_parse_success: true,
        expect_semantic_validation_success: true,
        expect_compile_success: true,
        expect_solve_success: case.solvable && execution_result.status == SolveStatus::Optimal,
        objective: Some(ExpectedObjective {
            name: execution_result.objective.dsl_name.clone(),
            sense: kdl_objective_sense(execution_result.objective_sense),
        }),
        reports: execution_result
            .reports
            .iter()
            .map(|report| report.name.clone())
            .collect(),
    }
}

fn kdl_objective_sense(sense: TargetObjectiveSense) -> ObjectiveSense {
    match sense {
        TargetObjectiveSense::Minimize => ObjectiveSense::Minimize,
        TargetObjectiveSense::Maximize => ObjectiveSense::Maximize,
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, BenchmarkError> {
    let text = fs::read_to_string(path).map_err(|source| BenchmarkError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| BenchmarkError::Json {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdl::algebra::Expr;
    use crate::kdl::semantic::{
        ResolvedChronology, ResolvedObjective, ResolvedParameters, ResolvedSet, ResolvedSets,
        ResolvedTimeSet, SemanticProgram, TimeResolution,
    };
    use std::collections::BTreeMap;

    fn base_program() -> SemanticProgram {
        SemanticProgram {
            active_scenario: "Base".to_string(),
            sets: ResolvedSets {
                time: ResolvedTimeSet {
                    steps: 24,
                    resolution: TimeResolution::Hourly,
                },
            },
            set_registry: BTreeMap::new(),
            set_aliases: BTreeMap::new(),
            set_params: BTreeMap::new(),
            parameters: ResolvedParameters::default(),
            variable_families: Vec::new(),
            variable_overrides: BTreeMap::new(),
            chronology: ResolvedChronology::default(),
            active_constraints: Vec::new(),
            active_expressions: Vec::new(),
            active_objective: ResolvedObjective {
                name: "Obj".to_string(),
                sense: ObjectiveSense::Minimize,
                expression_text: "0".to_string(),
                expression: Expr::Number("0".to_string()),
            },
            active_reports: Vec::new(),
            active_variable_reports: Vec::new(),
            active_dual_reports: Vec::new(),
        }
    }

    #[test]
    fn semantic_expectation_requires_assets_set() {
        let case = BenchmarkCaseDefinition {
            id: "case-1".to_string(),
            description: "desc".to_string(),
            entrypoint: "examples/example.kdl".to_string(),
            expected_semantic_program: "expected/semantic.json".to_string(),
            expected_e2e_summary: "expected/e2e.json".to_string(),
            solvable: true,
        };

        let error = to_semantic_expectation(&case, &base_program(), Path::new("/tmp/in.kdl"))
            .expect_err("missing assets set should fail loudly");

        match error {
            BenchmarkError::MissingNode { name, .. } => assert_eq!(name, "assets"),
            other => panic!("expected MissingNode, got {other:?}"),
        }
    }

    #[test]
    fn semantic_expectation_reads_assets_from_registry() {
        let case = BenchmarkCaseDefinition {
            id: "case-1".to_string(),
            description: "desc".to_string(),
            entrypoint: "examples/example.kdl".to_string(),
            expected_semantic_program: "expected/semantic.json".to_string(),
            expected_e2e_summary: "expected/e2e.json".to_string(),
            solvable: true,
        };

        let mut program = base_program();
        program.set_registry.insert(
            "assets".to_string(),
            ResolvedSet {
                values: vec!["g1".to_string(), "g2".to_string()],
                tuple_components: None,
                tuple_component_domains: None,
                tuple_rows: None,
            },
        );

        let expectation =
            to_semantic_expectation(&case, &program, Path::new("/tmp/in.kdl")).unwrap();

        assert_eq!(expectation.sets.assets, vec!["g1", "g2"]);
    }
}
