use crate::cli_io::{ColorMode, format_timed_status, style_bold_in_dim};
use crate::config::{ConfigError, SolverConfigState, load_solver_config};
pub use crate::driver_kdl::{KdlCheckOutcome, kdl_check_file_json};
pub use crate::driver_summary::{
    DualReportSummary, DualReportValueSummary, ObjectiveSummary, ProblemCounts, ReportSummary,
    RunSummary, TimingSummary, VariableSummary, VariableValueSummary,
};
use crate::driver_summary::{summarize_variables, trim_family_prefix};
use arco_ops::execution::{ExecutionError, SolveStatus, render_problem_model};
use arco_ops::solve::{ResolvedSelection, SolverProfile};
use arco_ops::{ArcoOps, OpsCompileError};
use miette::Diagnostic;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info};

const ARCO_VERSION_LABEL: &str = concat!("arco ", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub compact: bool,
    pub filter_variable: Option<String>,
    pub filter_asset: Option<String>,
    pub solver_log: bool,
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error(transparent)]
    Pipeline(#[from] OpsCompileError),
    #[error(transparent)]
    Config(#[from] ConfigError),
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
            Self::Pipeline { .. } | Self::Execution { .. } | Self::Config { .. } => None,
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
            Self::InspectFormat { .. }
            | Self::Pipeline { .. }
            | Self::Execution { .. }
            | Self::Config { .. } => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::Pipeline(error) => Some(error),
            Self::Execution(_)
            | Self::Config(_)
            | Self::Json { .. }
            | Self::BackendNotAvailable { .. }
            | Self::InspectFormat { .. } => None,
        }
    }
}

fn load_resolved_selection() -> Result<ResolvedSelection, DriverError> {
    Ok(load_solver_config()?.resolved)
}

fn selected_profile<'a>(
    state: &'a SolverConfigState,
    selection: &ResolvedSelection,
) -> Option<&'a SolverProfile> {
    selection
        .profile
        .as_ref()
        .and_then(|name| state.merged_profiles.get(name))
}

pub fn run_file(path: &Path) -> Result<RunSummary, DriverError> {
    run_file_with_options_and_selection(path, &RunOptions::default(), &load_resolved_selection()?)
}

pub fn run_file_with_options(path: &Path, options: &RunOptions) -> Result<RunSummary, DriverError> {
    run_file_with_options_and_selection(path, options, &load_resolved_selection()?)
}

pub fn run_file_with_options_and_selection(
    path: &Path,
    options: &RunOptions,
    selection: &ResolvedSelection,
) -> Result<RunSummary, DriverError> {
    run_file_with_options_and_profile(path, options, selection, None)
}

pub fn run_file_json(path: &Path) -> Result<String, DriverError> {
    run_file_json_with_options_and_selection(
        path,
        &RunOptions::default(),
        &load_resolved_selection()?,
    )
}

pub fn run_file_json_with_options(
    path: &Path,
    options: &RunOptions,
) -> Result<String, DriverError> {
    run_file_json_with_options_and_selection(path, options, &load_resolved_selection()?)
}

pub fn run_file_json_with_options_and_selection(
    path: &Path,
    options: &RunOptions,
    selection: &ResolvedSelection,
) -> Result<String, DriverError> {
    run_file_json_with_options_and_profile(path, options, selection, None)
}

pub fn run_file_json_with_options_and_config(
    path: &Path,
    options: &RunOptions,
    state: &SolverConfigState,
) -> Result<String, DriverError> {
    let profile = selected_profile(state, &state.resolved);
    run_file_json_with_options_and_profile(path, options, &state.resolved, profile)
}

fn run_file_json_with_options_and_profile(
    path: &Path,
    options: &RunOptions,
    selection: &ResolvedSelection,
    profile: Option<&SolverProfile>,
) -> Result<String, DriverError> {
    let summary = run_file_with_options_and_profile(path, options, selection, profile)?;
    serde_json::to_string(&summary).map_err(|source| DriverError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn run_file_with_options_and_profile(
    path: &Path,
    options: &RunOptions,
    selection: &ResolvedSelection,
    profile: Option<&SolverProfile>,
) -> Result<RunSummary, DriverError> {
    let total_start = Instant::now();
    let compiled = ArcoOps::compile_file(path)?;
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
        "starting backend solve phase (family={}, transport={:?}, include_variable_values={})",
        selection.family.as_str(),
        selection.transport,
        include_variable_values
    );
    let adapter = ArcoOps::builtin_adapter_for_selection(selection, options.solver_log, profile)
        .map_err(|message| DriverError::BackendNotAvailable { message })?;
    let execution_result = ArcoOps::execute_compiled_problem_with_adapter(
        &compiled.compiled_problem,
        adapter.as_ref(),
        include_variable_values,
    )?;
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
            sense: format!("{:?}", execution_result.objective_sense),
            value: execution_result.objective.value,
        },
        reports: execution_result
            .reports
            .into_iter()
            .map(|r| ReportSummary {
                name: r.name,
                index: r.index,
                values: r.values,
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

pub fn print_file_model(path: &Path) -> Result<String, DriverError> {
    let compiled = ArcoOps::compile_file(path)?;
    render_problem_model(&compiled.compiled_problem).map_err(DriverError::from)
}

pub fn validate_file_only(path: &Path, color_mode: ColorMode) -> Result<String, DriverError> {
    let started = Instant::now();
    let validated = ArcoOps::check_file(path)?;
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
    let validated = ArcoOps::check_file(path)?;
    let program = &validated.semantic_program;
    let payload = arco_ops::inspect::build_inspect_payload(&validated.entrypoint, program);

    if json_output {
        return arco_ops::inspect::render_json(&payload).map_err(|source| DriverError::Json {
            path: path.to_path_buf(),
            source,
        });
    }

    arco_ops::inspect::render_toml(&payload).map_err(|_| DriverError::InspectFormat {
        message: "failed to serialize inspect payload as TOML".to_string(),
    })
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
    use crate::driver_kdl::span_line_column;
    use miette::SourceSpan;
    use std::fs;
    use std::path::Path;

    #[test]
    fn span_line_column_counts_unicode_char_columns() {
        let path = std::env::temp_dir().join(format!(
            "arco-cli-unicode-span-{}-{}.kdl",
            std::process::id(),
            env!("CARGO_PKG_VERSION")
        ));
        let source = "set café technology\n";
        fs::write(&path, source).expect("write unicode fixture");

        let offset = source.find("technology").expect("target token");
        let location = span_line_column(&path, SourceSpan::from((offset, 1)));

        assert_eq!(location, (Some(1), Some(10)));
        fs::remove_file(path).expect("remove unicode fixture");
    }

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
