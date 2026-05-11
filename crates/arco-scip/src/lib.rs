// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use arco_format::{
    PortableLinearReport, PortableProblem, portable_problem_from_model_view, write_mps,
};
use arco_model::{ModelView, VariableId};
use arco_solver::{
    ModelViewBackend, ModelViewSolveResult, SolverCapabilityModel, SolverConfig, SolverError,
    SolverFamily, SolverRegistry, SolverStatus,
};
use miette::Diagnostic;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const FAMILY_NAME: &str = "scip";
pub const BACKEND_NAME: &str = "arco-external-scip";

/// External-process SCIP backend for primitive model views.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScipModelViewBackend;

impl ModelViewBackend for ScipModelViewBackend {
    fn family(&self) -> &'static str {
        FAMILY_NAME
    }

    fn solve_model_view(
        &self,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        solve_model_view(model, config)
    }
}

/// Solve a primitive model view by rendering the concrete MPS interchange that SCIP requires.
pub fn solve_model_view(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    solve_model_view_with_options(model, config, &ExternalProcessOptions::default())
}

/// Solve a primitive model view with explicit external-process options.
pub fn solve_model_view_with_options(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
    options: &ExternalProcessOptions,
) -> Result<ModelViewSolveResult, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }

    let portable = portable_problem_from_model_view(model);
    let variable_families = portable
        .variable_instances
        .iter()
        .map(|variable| variable.family.clone())
        .collect::<Vec<_>>();
    let output = solve_problem_with_options(
        ScipProblem {
            portable: &portable,
            variable_families: &variable_families,
        },
        true,
        config.log_to_console.unwrap_or(false),
        options,
    )
    .map_err(|error| SolverError::SolverSpecific(error.to_string()))?;

    let solution_values = output
        .variable_values
        .iter()
        .flat_map(|variable| variable.values.iter())
        .map(|value| (value.compiled_name.as_str(), value.value))
        .collect::<BTreeMap<_, _>>();
    let primal_values = (0..model.num_variables())
        .map(|idx| {
            let variable_id = VariableId::new(idx as u32);
            let name = model
                .variable_name(variable_id)
                .map_or_else(|| format!("x{idx}"), str::to_string);
            solution_values.get(name.as_str()).copied().unwrap_or(0.0)
        })
        .collect::<Vec<_>>();
    let mut row_values = vec![0.0; model.num_constraints()];
    for (var_idx, primal_value) in primal_values.iter().copied().enumerate() {
        let variable_id = VariableId::new(var_idx as u32);
        let Some(column) = model.column(variable_id) else {
            continue;
        };
        for (constraint_id, coefficient) in column {
            if let Some(row_value) = row_values.get_mut(constraint_id.inner() as usize) {
                *row_value += coefficient * primal_value;
            }
        }
    }

    Ok(ModelViewSolveResult {
        fingerprint: model.fingerprint(),
        status: match output.status {
            SolveStatus::Optimal => SolverStatus::Optimal,
            SolveStatus::Infeasible => SolverStatus::Infeasible,
            SolveStatus::Failed => SolverStatus::Unknown,
        },
        objective_value: output.objective_value,
        primal_values,
        variable_duals: Vec::new(),
        row_values,
        constraint_duals: Vec::new(),
        metadata: Default::default(),
    })
}

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExternalProcessOptions {
    pub executable: Option<String>,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ScipProblem<'a> {
    pub portable: &'a PortableProblem,
    pub variable_families: &'a [String],
}

pub fn solve_problem(
    problem: ScipProblem<'_>,
    include_variable_values: bool,
    log_to_console: bool,
) -> Result<SolveOutput, Error> {
    solve_problem_with_options(
        problem,
        include_variable_values,
        log_to_console,
        &ExternalProcessOptions::default(),
    )
}

pub fn solve_problem_with_options(
    problem: ScipProblem<'_>,
    include_variable_values: bool,
    log_to_console: bool,
    options: &ExternalProcessOptions,
) -> Result<SolveOutput, Error> {
    let backend = BACKEND_NAME.to_string();
    let workspace = ScipWorkspace::create().map_err(|source| Error::Io { source })?;
    let mps_path = workspace.path().join("problem.mps");
    let sol_path = workspace.path().join("solution.sol");

    {
        let mut mps_file =
            std::fs::File::create(&mps_path).map_err(|source| Error::Io { source })?;
        write_mps(problem.portable, &mut mps_file).map_err(|source| Error::Process {
            message: source.to_string(),
        })?;
    }

    let executable = options.executable.clone().unwrap_or_else(|| {
        std::env::var("ARCO_SCIP_EXECUTABLE").unwrap_or_else(|_| "scip".to_string())
    });
    let mut command = Command::new(&executable);
    command.args(&options.arguments);
    command.envs(&options.environment);
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

    build_scip_solve_output(problem, include_variable_values, &backend, &sol_path)
}

#[derive(Debug)]
struct ScipWorkspace {
    path: PathBuf,
}

impl ScipWorkspace {
    fn create() -> Result<Self, std::io::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!("arco-scip-{}-{now}", std::process::id()));
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScipWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[derive(Debug)]
struct ScipSolution {
    status: String,
    objective_value: f64,
    variable_values: BTreeMap<String, f64>,
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
    problem: ScipProblem<'_>,
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
        .portable
        .reports
        .iter()
        .map(|report| ScalarValue {
            compiled_name: report.name.clone(),
            value: evaluate_scip_linear_report(report, &solution.variable_values),
        })
        .collect::<Vec<_>>();

    let variable_values = problem
        .variable_families
        .iter()
        .map(|family| {
            let representative_value = problem
                .portable
                .variable_instances
                .iter()
                .find(|instance| instance.family == *family)
                .and_then(|instance| solution.variable_values.get(&instance.name).copied())
                .unwrap_or(0.0);

            let values = if include_variable_values {
                problem
                    .portable
                    .variable_instances
                    .iter()
                    .filter(|instance| instance.family == *family)
                    .map(|instance| VariableInstanceValue {
                        compiled_name: instance.name.clone(),
                        value: *solution.variable_values.get(&instance.name).unwrap_or(&0.0),
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            VariableValue {
                compiled_name: family.clone(),
                representative_value,
                values,
            }
        })
        .collect::<Vec<_>>();

    Ok(SolveOutput {
        status,
        objective_value: problem.portable.objective.constant + solution.objective_value,
        report_values,
        variable_values,
    })
}

fn evaluate_scip_linear_report(
    report: &PortableLinearReport,
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
    use arco_format::{
        PortableLinearObjective, PortableLinearTerm, PortableObjectiveSense,
        PortableVariableInstance, PortableVariableKind,
    };
    use arco_solver::{SolverRegistry, SolverTransport};
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
        let report = PortableLinearReport {
            name: "r".to_string(),
            terms: vec![
                PortableLinearTerm {
                    coefficient: 2.0,
                    variable_name: "x".to_string(),
                },
                PortableLinearTerm {
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
    fn model_view_backend_rejects_empty_problem_before_process_spawn() {
        let backend = ScipModelViewBackend;
        let model = arco_model::Model::new();

        let error = backend
            .solve_model_view(&model, &SolverConfig::default())
            .expect_err("empty model should fail before spawning SCIP");

        assert!(matches!(error, SolverError::EmptyModel));
    }

    #[test]
    fn solve_output_defaults_missing_variable_values_to_zero() {
        let problem = PortableProblem {
            variable_instances: vec![PortableVariableInstance {
                name: "x[A,1]".to_string(),
                family: "x[a,t]".to_string(),
                lower: 0.0,
                upper: None,
                kind: PortableVariableKind::Continuous,
            }],
            constraints: Vec::new(),
            objective: PortableLinearObjective {
                name: "obj".to_string(),
                sense: PortableObjectiveSense::Maximize,
                constant: 0.0,
                terms: Vec::new(),
            },
            reports: Vec::new(),
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

        assert!((problem.objective.constant + output.objective_value).abs() <= f64::EPSILON);
        assert!(output.variable_values[0].values[0].value.abs() <= f64::EPSILON);
    }
}
