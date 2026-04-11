use crate::cli_io::{ColorMode, format_timed_status, style_bold_in_dim};
use crate::config::SolverBackend;
#[cfg(feature = "xpress")]
use crate::execution::XpressArcoAdapter;
use crate::execution::{
    ExecutionError, RustArcoAdapter, SolveStatus, execute_problem_with_options,
    render_problem_model,
};
use arco_kdl::ObjectiveSense;
use arco_kdl::pipeline::{PipelineError, compile_file, validate_file};
use miette::Diagnostic;
use serde::Serialize;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info};

const DEFAULT_BACKEND: SolverBackend = SolverBackend::Highs;
const ARCO_VERSION_LABEL: &str = concat!("arco ", env!("CARGO_PKG_VERSION"));

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dual_reports: Vec<DualReportSummary>,
    pub variables: Vec<VariableSummary>,
    pub counts: ProblemCounts,
    pub timing: TimingSummary,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ObjectiveSummary {
    pub name: String,
    pub sense: ObjectiveSense,
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
pub struct DualReportSummary {
    pub name: String,
    pub values: Vec<DualReportValueSummary>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DualReportValueSummary {
    pub instance: String,
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
    pub compile_ms: f64,
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
    #[error("{message}")]
    InspectFormat { message: String },
}

impl Diagnostic for DriverError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::Json { .. } => Some(Box::new("arco::driver::json")),
            Self::BackendNotAvailable { .. } => {
                Some(Box::new("arco::driver::backend_not_available"))
            }
            Self::InspectFormat { .. } => Some(Box::new("arco::driver::inspect_format")),
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
            Self::InspectFormat { .. } | Self::Pipeline { .. } | Self::Execution { .. } => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::Pipeline(error) => Some(error),
            Self::Execution(_)
            | Self::Json { .. }
            | Self::BackendNotAvailable { .. }
            | Self::InspectFormat { .. } => None,
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
    debug!(
        "compile timings: parse={:.2} ms validate={:.2} ms lower={:.2} ms",
        compiled.timing.parse.as_secs_f64() * 1000.0,
        compiled.timing.validate.as_secs_f64() * 1000.0,
        compiled.timing.compile.as_secs_f64() * 1000.0
    );
    debug!(
        "lowered problem size: {} variable instances, {} constraint rows",
        compiled.compiled_problem.algebra.variable_instances.len(),
        compiled.compiled_problem.algebra.constraints.len()
    );

    let solve_start = Instant::now();
    let include_variable_values = !(options.compact && options.filter_asset.is_none());
    debug!(
        "starting backend solve phase (backend={}, include_variable_values={})",
        backend.as_str(),
        include_variable_values
    );
    let execution_result = match backend {
        SolverBackend::Highs => execute_problem_with_options(
            &compiled.compiled_problem,
            &RustArcoAdapter::with_console_log(options.solver_log),
            include_variable_values,
        )?,
        #[cfg(feature = "xpress")]
        SolverBackend::Xpress => execute_problem_with_options(
            &compiled.compiled_problem,
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
    info!(
        "backend solve phase completed in {:.2} ms",
        solve.as_secs_f64() * 1000.0
    );
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
        dual_reports: execution_result
            .dual_reports
            .into_iter()
            .map(|dr| {
                let values = dr
                    .values
                    .into_iter()
                    .map(|v| DualReportValueSummary {
                        instance: trim_family_prefix(&dr.constraint_family, &v.instance_name),
                        value: v.value,
                    })
                    .collect();
                DualReportSummary {
                    name: dr.constraint_family,
                    values,
                }
            })
            .collect(),
        variables,
        counts: ProblemCounts {
            parameters: compiled.compiled_problem.parameters.len(),
            variables: compiled.compiled_problem.variables.len(),
            constraints: compiled.compiled_problem.constraints.len(),
        },
        timing: TimingSummary {
            parse_ms: compiled.timing.parse.as_secs_f64() * 1000.0,
            validate_ms: compiled.timing.validate.as_secs_f64() * 1000.0,
            compile_ms: compiled.timing.compile.as_secs_f64() * 1000.0,
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
    render_problem_model(&compiled.compiled_problem).map_err(DriverError::from)
}

pub fn validate_file_only(path: &Path, color_mode: ColorMode) -> Result<String, DriverError> {
    let started = Instant::now();
    let validated = validate_file(path)?;
    let elapsed_ms = started.elapsed().as_millis();
    Ok(format_validate_success(
        &validated.entrypoint,
        elapsed_ms,
        color_mode,
    ))
}

fn format_validate_success(path: &Path, elapsed_ms: u128, color_mode: ColorMode) -> String {
    let path_uri = format!("file://{}", path.display());
    let subject = format!("Validated {}", style_bold_in_dim(&path_uri, color_mode));
    format_timed_status(&subject, elapsed_ms, ARCO_VERSION_LABEL, color_mode)
}

pub fn inspect_file_report(path: &Path, json_output: bool) -> Result<String, DriverError> {
    let validated = validate_file(path)?;
    let program = &validated.semantic_program;
    let payload = crate::inspect::build_inspect_payload(&validated.entrypoint, program);

    if json_output {
        return crate::inspect::render_json(&payload).map_err(|source| DriverError::Json {
            path: path.to_path_buf(),
            source,
        });
    }

    crate::inspect::render_toml(&payload).map_err(|_| DriverError::InspectFormat {
        message: "failed to serialize inspect payload as TOML".to_string(),
    })
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
                        extract_asset_name(&value.compiled_name)
                            .is_some_and(|asset| wildcard_match(pattern, asset))
                    })
                })
                .map(|value| VariableValueSummary {
                    name: trim_family_prefix(&variable.dsl_name, &value.compiled_name),
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

#[cfg(test)]
mod tests {
    use super::format_validate_success;
    use std::path::Path;

    #[test]
    fn format_validate_success_plain_output_has_no_ansi() {
        let rendered = format_validate_success(
            Path::new("/tmp/model.kdl"),
            4,
            crate::cli_io::ColorMode::Disabled,
        );
        assert_eq!(
            rendered,
            format!(
                "Validated file:///tmp/model.kdl in 4ms (arco {})",
                env!("CARGO_PKG_VERSION")
            )
        );
        assert!(!rendered.contains("\x1b["));
    }

    #[test]
    fn format_validate_success_colored_output_contains_ansi_sequences() {
        let rendered = format_validate_success(
            Path::new("/tmp/model.kdl"),
            4,
            crate::cli_io::ColorMode::Enabled,
        );
        assert!(rendered.starts_with("\x1b[38;5;245mValidated "));
        assert!(rendered.contains("\x1b[1mfile:///tmp/model.kdl\x1b[22m"));
        assert!(rendered.contains(&format!(
            "\x1b[1marco {}\x1b[22m",
            env!("CARGO_PKG_VERSION")
        )));
        assert!(rendered.ends_with(")\x1b[0m"));
    }
}
