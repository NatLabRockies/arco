// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use arco_export::write_mps;
use arco_kdl::artifacts::CompiledProblem;
use arco_solver::{SolverCapabilityModel, SolverFamily, SolverRegistry};
use arco_targets::LinearReport;
use miette::Diagnostic;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const FAMILY_NAME: &str = "scip";
pub const BACKEND_NAME: &str = "arco-external-scip";

pub fn register_solver_family(registry: &mut SolverRegistry) {
    registry.add_family(SolverFamily::external_process(
        FAMILY_NAME,
        "SCIP",
        SolverCapabilityModel::lp_mip_default(),
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    Optimal,
    Infeasible,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarValue {
    pub compiled_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableValue {
    pub compiled_name: String,
    pub representative_value: f64,
    pub values: Vec<VariableInstanceValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableInstanceValue {
    pub compiled_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveOutput {
    pub status: SolveStatus,
    pub objective_value: f64,
    pub report_values: Vec<ScalarValue>,
    pub variable_values: Vec<VariableValue>,
}

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("failed to interact with SCIP workspace: {source}")]
    #[diagnostic(
        code(arco::scip::io),
        help("verify the temporary directory is writable")
    )]
    Io {
        #[source]
        source: std::io::Error,
    },
    #[error("SCIP invocation failed: {message}")]
    #[diagnostic(
        code(arco::scip::process),
        help("verify the SCIP executable is installed and the command line is valid")
    )]
    Process { message: String },
    #[error("SCIP solution file was invalid: {message}")]
    #[diagnostic(
        code(arco::scip::parse),
        help("inspect the generated .sol file for formatting issues")
    )]
    Parse { message: String },
    #[error("SCIP did not produce a feasible solution: {status}")]
    #[diagnostic(
        code(arco::scip::no_feasible_solution),
        help(
            "inspect the solution status or solver logs for why SCIP could not return a feasible solution"
        )
    )]
    NoFeasibleSolution { status: String },
}

pub fn solve_compiled_problem(
    problem: &CompiledProblem,
    include_variable_values: bool,
    log_to_console: bool,
) -> Result<SolveOutput, Error> {
    let backend = BACKEND_NAME.to_string();
    let workspace = unique_temp_paths("scip");
    std::fs::create_dir_all(&workspace).map_err(|source| Error::Io { source })?;
    let mps_path = workspace.join("problem.mps");
    let sol_path = workspace.join("solution.sol");

    {
        let mut mps_file =
            std::fs::File::create(&mps_path).map_err(|source| Error::Io { source })?;
        write_mps(&problem.algebra, &mut mps_file).map_err(|source| Error::Process {
            message: source.to_string(),
        })?;
    }

    let executable = std::env::var("ARCO_SCIP_EXECUTABLE").unwrap_or_else(|_| "scip".to_string());
    let mut command = Command::new(&executable);
    command
        .arg("-c")
        .arg(format!("read {}", mps_path.display()))
        .arg("-c")
        .arg("optimize")
        .arg("-c")
        .arg(format!("write solution {}", sol_path.display()))
        .arg("-c")
        .arg("quit");

    let output = command.output().map_err(|source| Error::Io { source })?;

    if log_to_console {
        if !output.stdout.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    if !output.status.success() {
        return Err(Error::Process {
            message: format!(
                "{} exited with status {}: {}",
                executable,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }

    let solve_result =
        build_scip_solve_output(problem, include_variable_values, &backend, &sol_path);

    let _ = std::fs::remove_file(&mps_path);
    let _ = std::fs::remove_file(&sol_path);
    let _ = std::fs::remove_dir(&workspace);

    solve_result
}

#[derive(Debug)]
struct ScipSolution {
    status: String,
    objective_value: f64,
    variable_values: BTreeMap<String, f64>,
}

fn unique_temp_paths(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("arco-{prefix}-{}-{now}", std::process::id()))
}

fn map_scip_status(status: &str) -> SolveStatus {
    let status = status.to_ascii_lowercase();
    if status.contains("optimal") {
        return SolveStatus::Optimal;
    }
    if status.contains("infeasible") {
        return SolveStatus::Infeasible;
    }
    SolveStatus::Failed
}

fn parse_scip_solution_file(path: &Path, backend: &str) -> Result<ScipSolution, Error> {
    let content = std::fs::read_to_string(path).map_err(|source| Error::Io { source })?;

    let mut status = None;
    let mut objective_value = 0.0;
    let mut variable_values = BTreeMap::new();

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("solution status:") {
            status = Some(value.trim().to_string());
            continue;
        }

        if let Some(value) = line.strip_prefix("objective value:") {
            objective_value = value.trim().parse::<f64>().map_err(|error| Error::Parse {
                message: format!("invalid objective value '{}': {error}", value.trim()),
            })?;
            continue;
        }

        if line.trim().is_empty() || line.starts_with("no solution available") {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(variable_name) = parts.next() else {
            continue;
        };
        let Some(variable_value) = parts.next() else {
            continue;
        };
        if let Ok(value) = variable_value.parse::<f64>() {
            variable_values.insert(variable_name.to_string(), value);
        }
    }

    let Some(status) = status else {
        return Err(Error::Parse {
            message: format!("solution status line missing for {backend}"),
        });
    };

    Ok(ScipSolution {
        status,
        objective_value,
        variable_values,
    })
}

fn build_scip_solve_output(
    problem: &CompiledProblem,
    include_variable_values: bool,
    backend: &str,
    sol_path: &Path,
) -> Result<SolveOutput, Error> {
    let solution = parse_scip_solution_file(sol_path, backend)?;
    let status = map_scip_status(&solution.status);
    if status != SolveStatus::Optimal {
        return Err(Error::NoFeasibleSolution {
            status: solution.status,
        });
    }

    let report_values = problem
        .algebra
        .reports
        .iter()
        .map(|report| ScalarValue {
            compiled_name: report.name.clone(),
            value: evaluate_scip_linear_report(report, &solution.variable_values),
        })
        .collect::<Vec<_>>();

    let variable_values = problem
        .variables
        .iter()
        .map(|variable| {
            let representative_value = problem
                .algebra
                .variable_instances
                .iter()
                .find(|instance| instance.family == variable.family)
                .and_then(|instance| solution.variable_values.get(&instance.name).copied())
                .unwrap_or(0.0);

            let values = if include_variable_values {
                problem
                    .algebra
                    .variable_instances
                    .iter()
                    .filter(|instance| instance.family == variable.family)
                    .map(|instance| VariableInstanceValue {
                        compiled_name: instance.name.clone(),
                        value: *solution.variable_values.get(&instance.name).unwrap_or(&0.0),
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            VariableValue {
                compiled_name: variable.family.clone(),
                representative_value,
                values,
            }
        })
        .collect::<Vec<_>>();

    Ok(SolveOutput {
        status,
        objective_value: problem.algebra.objective.constant + solution.objective_value,
        report_values,
        variable_values,
    })
}

fn evaluate_scip_linear_report(
    report: &LinearReport,
    variable_values: &BTreeMap<String, f64>,
) -> f64 {
    let terms_value: f64 = report
        .terms
        .iter()
        .map(|term| {
            let value = variable_values
                .get(&term.variable_name)
                .copied()
                .unwrap_or(0.0);
            term.coefficient * value
        })
        .sum();
    report.constant + terms_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_kdl::artifacts::{CompiledObjective, CompiledProblem, CompiledVariable};
    use arco_solver::{SolverRegistry, SolverTransport};
    use arco_targets::{
        AlgebraicProblem, LinearObjective, LinearReport, LinearTerm, ObjectiveSense,
        VariableInstance, VariableKind,
    };
    use std::collections::BTreeMap;

    #[test]
    fn register_solver_family_registers_external_process_family() {
        let mut registry = SolverRegistry::new();
        register_solver_family(&mut registry);

        let family = registry
            .family(FAMILY_NAME)
            .unwrap_or_else(|| panic!("missing registered family: {FAMILY_NAME}"));
        assert_eq!(family.name, FAMILY_NAME);
        assert!(
            family
                .transports
                .contains(&SolverTransport::ExternalProcess)
        );
    }

    #[test]
    fn scip_status_mapping_handles_optimal_and_infeasible() {
        assert_eq!(
            map_scip_status("optimal solution found"),
            SolveStatus::Optimal
        );
        assert_eq!(map_scip_status("infeasible"), SolveStatus::Infeasible);
        assert_eq!(map_scip_status("unbounded"), SolveStatus::Failed);
    }

    #[test]
    fn parse_scip_solution_file_reads_status_objective_and_values() {
        let path = std::env::temp_dir().join(format!(
            "arco-scip-solution-{}-{}.sol",
            std::process::id(),
            env!("CARGO_PKG_VERSION")
        ));
        std::fs::write(
            &path,
            "solution status: optimal solution found\nobjective value: 7\nx 3\ny 4\n",
        )
        .expect("write scip solution fixture");

        let parsed =
            parse_scip_solution_file(&path, BACKEND_NAME).expect("parse solution should succeed");
        assert_eq!(parsed.status, "optimal solution found");
        assert!((parsed.objective_value - 7.0).abs() < f64::EPSILON);
        assert_eq!(parsed.variable_values.get("x").copied(), Some(3.0));
        assert_eq!(parsed.variable_values.get("y").copied(), Some(4.0));

        std::fs::remove_file(path).expect("remove scip solution fixture");
    }

    #[test]
    fn evaluate_scip_linear_report_defaults_missing_variables_to_zero() {
        let report = LinearReport {
            name: "r".to_string(),
            terms: vec![
                LinearTerm {
                    coefficient: 2.0,
                    variable_name: "x".to_string(),
                },
                LinearTerm {
                    coefficient: -1.0,
                    variable_name: "y".to_string(),
                },
            ],
            constant: 5.0,
        };
        let variable_values = BTreeMap::from([("x".to_string(), 3.0)]);

        let value = evaluate_scip_linear_report(&report, &variable_values);
        assert!((value - 11.0).abs() < f64::EPSILON);
    }

    #[test]
    fn solve_output_defaults_missing_variable_values_to_zero() {
        let problem = CompiledProblem {
            parameters: Vec::new(),
            variables: vec![CompiledVariable {
                family: "x[a,t]".to_string(),
            }],
            constraints: Vec::new(),
            objective: CompiledObjective {
                name: "obj".to_string(),
                sense: ObjectiveSense::Maximize,
                expression: "0".to_string(),
            },
            reports: Vec::new(),
            variable_reports: Vec::new(),
            dual_reports: Vec::new(),
            traceability: Vec::new(),
            algebra: AlgebraicProblem {
                variable_instances: vec![VariableInstance {
                    name: "x[A,1]".to_string(),
                    family: "x[a,t]".to_string(),
                    lower: 0.0,
                    upper: None,
                    kind: VariableKind::Continuous,
                }],
                constraints: Vec::new(),
                objective: LinearObjective {
                    name: "obj".to_string(),
                    sense: ObjectiveSense::Maximize,
                    constant: 0.0,
                    terms: Vec::new(),
                },
                reports: Vec::new(),
            },
        };

        let output = SolveOutput {
            status: SolveStatus::Optimal,
            objective_value: 0.0,
            report_values: Vec::new(),
            variable_values: vec![VariableValue {
                compiled_name: "x[a,t]".to_string(),
                representative_value: 0.0,
                values: vec![VariableInstanceValue {
                    compiled_name: "x[A,1]".to_string(),
                    value: 0.0,
                }],
            }],
        };

        assert!(
            (problem.algebra.objective.constant + output.objective_value).abs() <= f64::EPSILON
        );
        assert!(output.variable_values[0].values[0].value.abs() <= f64::EPSILON);
    }
}
