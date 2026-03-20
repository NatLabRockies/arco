use crate::config::SolverBackend;
#[cfg(feature = "xpress")]
use crate::execution::XpressArcoAdapter;
use crate::execution::{
    ExecutionError, RustArcoAdapter, SolveStatus, execute_problem_with_options,
    render_problem_model,
};
use arco_kdl::pipeline::{PipelineError, compile_file, validate_file};
use miette::Diagnostic;
use serde::Serialize;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;

const DEFAULT_BACKEND: SolverBackend = SolverBackend::Highs;

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub compact: bool,
    pub filter_variable: Option<String>,
    pub filter_asset: Option<String>,
    pub solver_log: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RunSummary {
    pub entrypoint: String,
    pub backend: &'static str,
    pub solve_status: &'static str,
    pub active_scenario: String,
    pub objective: ObjectiveSummary,
    pub reports: Vec<ReportSummary>,
    pub variables: Vec<VariableSummary>,
    pub counts: ProblemCounts,
    pub timing: TimingSummary,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ObjectiveSummary {
    pub name: String,
    pub sense: String,
    pub value: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ReportSummary {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct VariableSummary {
    pub name: String,
    pub representative_value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<VariableValueSummary>>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct VariableValueSummary {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ProblemCounts {
    pub parameters: usize,
    pub variables: usize,
    pub constraints: usize,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TimingSummary {
    pub parse_ms: f64,
    pub validate_ms: f64,
    pub lower_ms: f64,
    pub solve_ms: f64,
    pub total_ms: f64,
    pub peak_memory_bytes: Option<u64>,
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error(transparent)]
    Pipeline(#[from] PipelineError),
    #[error(transparent)]
    Execution(#[from] ExecutionError),
    #[error("failed to serialize run summary for {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("{message}")]
    BackendNotAvailable { message: String },
}

impl Diagnostic for DriverError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::Json { .. } => Some(Box::new("arco::driver::json")),
            Self::BackendNotAvailable { .. } => {
                Some(Box::new("arco::driver::backend_not_available"))
            }
            Self::Pipeline { .. } | Self::Execution { .. } => None,
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::Json { .. } => Some(Box::new(
                "inspect the summary payload for non-finite values that cannot be serialized",
            )),
            Self::BackendNotAvailable { .. } => Some(Box::new(
                "To enable Xpress support, rebuild arco with the xpress feature:\n\n\
                 \x20   cargo install --path . --features xpress\n\n\
                 This requires the FICO Xpress SDK installed and XPRESSDIR set.\n\n\
                 On macOS (DMG install):\n\
                 \x20   export XPRESSDIR=\"/Applications/FICO Xpress/xpressmp\"\n\n\
                 On Linux:\n\
                 \x20   export XPRESSDIR=\"/opt/xpressmp\"\n\n\
                 To switch back to HiGHS (no extra dependencies):\n\
                 \x20   arco solver set highs",
            )),
            Self::Pipeline { .. } | Self::Execution { .. } => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::Pipeline(error) => Some(error),
            Self::Execution(_) | Self::Json { .. } | Self::BackendNotAvailable { .. } => None,
        }
    }
}

pub fn run_file(path: &Path) -> Result<RunSummary, DriverError> {
    run_file_with_options_and_backend(path, &RunOptions::default(), DEFAULT_BACKEND)
}

pub fn run_file_with_options(path: &Path, options: &RunOptions) -> Result<RunSummary, DriverError> {
    run_file_with_options_and_backend(path, options, DEFAULT_BACKEND)
}

pub fn run_file_with_options_and_backend(
    path: &Path,
    options: &RunOptions,
    backend: SolverBackend,
) -> Result<RunSummary, DriverError> {
    let total_start = Instant::now();
    let compiled = compile_file(path)?;

    let solve_start = Instant::now();
    let include_variable_values = !(options.compact && options.filter_asset.is_none());
    let execution_result = match backend {
        SolverBackend::Highs => execute_problem_with_options(
            &compiled.lowered_problem,
            &RustArcoAdapter::with_console_log(options.solver_log),
            include_variable_values,
        )?,
        #[cfg(feature = "xpress")]
        SolverBackend::Xpress => execute_problem_with_options(
            &compiled.lowered_problem,
            &XpressArcoAdapter::with_console_log(options.solver_log),
            include_variable_values,
        )?,
        #[cfg(not(feature = "xpress"))]
        SolverBackend::Xpress => {
            return Err(DriverError::BackendNotAvailable {
                message: "Xpress solver backend is not available in this build".to_string(),
            });
        }
    };
    let solve = solve_start.elapsed();
    let total = total_start.elapsed();

    let variables = summarize_variables(&execution_result.variables, options);

    Ok(RunSummary {
        entrypoint: compiled.entrypoint.display().to_string(),
        backend: execution_result.backend,
        solve_status: solve_status_name(execution_result.status),
        active_scenario: compiled.semantic_program.active_scenario,
        objective: ObjectiveSummary {
            name: execution_result.objective.dsl_name,
            sense: execution_result.objective_sense,
            value: execution_result.objective.value,
        },
        reports: execution_result
            .reports
            .into_iter()
            .map(|report| ReportSummary {
                name: report.dsl_name,
                value: report.value,
            })
            .collect(),
        variables,
        counts: ProblemCounts {
            parameters: compiled.lowered_problem.parameters.len(),
            variables: compiled.lowered_problem.variables.len(),
            constraints: compiled.lowered_problem.constraints.len(),
        },
        timing: TimingSummary {
            parse_ms: compiled.timing.parse.as_secs_f64() * 1000.0,
            validate_ms: compiled.timing.validate.as_secs_f64() * 1000.0,
            lower_ms: compiled.timing.lower.as_secs_f64() * 1000.0,
            solve_ms: solve.as_secs_f64() * 1000.0,
            total_ms: total.as_secs_f64() * 1000.0,
            peak_memory_bytes: peak_rss_bytes(),
        },
    })
}

pub fn run_file_json(path: &Path) -> Result<String, DriverError> {
    run_file_json_with_options_and_backend(path, &RunOptions::default(), DEFAULT_BACKEND)
}

pub fn run_file_json_with_options(
    path: &Path,
    options: &RunOptions,
) -> Result<String, DriverError> {
    run_file_json_with_options_and_backend(path, options, DEFAULT_BACKEND)
}

pub fn run_file_json_with_options_and_backend(
    path: &Path,
    options: &RunOptions,
    backend: SolverBackend,
) -> Result<String, DriverError> {
    let summary = run_file_with_options_and_backend(path, options, backend)?;
    serde_json::to_string(&summary).map_err(|source| DriverError::Json {
        path: path.to_path_buf(),
        source,
    })
}

pub fn print_file_model(path: &Path) -> Result<String, DriverError> {
    let compiled = compile_file(path)?;
    render_problem_model(&compiled.lowered_problem).map_err(DriverError::from)
}

pub fn validate_file_report(path: &Path) -> Result<String, DriverError> {
    let validated = validate_file(path)?;
    Ok(format!(
        "Validation succeeded\nentrypoint: {}\nscenario: {}\nassets: {}\nconstraints: {}",
        validated.entrypoint.display(),
        validated.semantic_program.active_scenario,
        validated.semantic_program.sets.assets.len(),
        validated.semantic_program.active_constraints.len()
    ))
}

fn summarize_variables(
    variables: &[crate::execution::MappedVariableResult],
    options: &RunOptions,
) -> Vec<VariableSummary> {
    variables
        .iter()
        .filter(|variable| {
            options
                .filter_variable
                .as_deref()
                .is_none_or(|pattern| wildcard_match(pattern, &variable.dsl_name))
        })
        .filter_map(|variable| {
            let filtered_values = variable
                .values
                .iter()
                .filter(|value| {
                    options.filter_asset.as_deref().is_none_or(|pattern| {
                        extract_asset_name(&value.lowered_name)
                            .is_some_and(|asset| wildcard_match(pattern, asset))
                    })
                })
                .map(|value| VariableValueSummary {
                    name: trim_family_prefix(&variable.dsl_name, &value.lowered_name),
                    value: value.value,
                })
                .collect::<Vec<_>>();

            if options.filter_asset.is_some() && filtered_values.is_empty() {
                return None;
            }

            let representative_value = filtered_values
                .first()
                .map_or(variable.representative_value, |value| value.value);
            let values = if options.compact || values_are_redundant(&filtered_values) {
                None
            } else {
                Some(filtered_values)
            };

            Some(VariableSummary {
                name: variable.dsl_name.clone(),
                representative_value,
                values,
            })
        })
        .collect()
}

fn values_are_redundant(values: &[VariableValueSummary]) -> bool {
    match values.first() {
        Some(first) => values
            .iter()
            .all(|value| (value.value - first.value).abs() < f64::EPSILON),
        None => true,
    }
}

fn trim_family_prefix(family_name: &str, value_name: &str) -> String {
    let prefix = family_name.split('[').next().unwrap_or(family_name);
    value_name
        .strip_prefix(prefix)
        .map_or_else(|| value_name.to_string(), ToString::to_string)
}

fn extract_asset_name(value_name: &str) -> Option<&str> {
    let start = value_name.find('[')? + 1;
    let remainder = &value_name[start..];
    let end = remainder.find([',', ']'])?;
    let asset = &remainder[..end];
    if asset.is_empty() || asset.chars().all(|character| character.is_ascii_digit()) {
        None
    } else {
        Some(asset)
    }
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut pattern_index = 0usize;
    let mut value_index = 0usize;
    let mut star_index = None;
    let mut match_index = 0usize;

    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star_index = Some(pattern_index);
            match_index = value_index;
            pattern_index += 1;
        } else if let Some(star) = star_index {
            pattern_index = star + 1;
            match_index += 1;
            value_index = match_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}

fn solve_status_name(status: SolveStatus) -> &'static str {
    match status {
        SolveStatus::Optimal => "optimal",
        SolveStatus::Infeasible => "infeasible",
        SolveStatus::Failed => "failed",
    }
}

fn peak_rss_bytes() -> Option<u64> {
    use sysinfo::{Pid, System};
    let pid = Pid::from_u32(std::process::id());
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).map(|p| p.memory())
}
