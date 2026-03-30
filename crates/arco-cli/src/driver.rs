use crate::config::SolverBackend;
#[cfg(feature = "xpress")]
use crate::execution::XpressArcoAdapter;
use crate::execution::{
    ExecutionError, RustArcoAdapter, SolveStatus, execute_problem_with_options,
    render_problem_model,
};
use arco_kdl::pipeline::{PipelineError, compile_file, validate_file};
use clap::ValueEnum;
use miette::Diagnostic;
use serde::Serialize;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info};

const DEFAULT_BACKEND: SolverBackend = SolverBackend::Highs;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InspectCategory {
    Sets,
    Constraints,
    Variables,
    Parameters,
    Expressions,
    Objective,
    Reports,
    Chronology,
}

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
    #[error("{message}")]
    InspectLookup { message: String },
}

impl Diagnostic for DriverError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::Json { .. } => Some(Box::new("arco::driver::json")),
            Self::BackendNotAvailable { .. } => {
                Some(Box::new("arco::driver::backend_not_available"))
            }
            Self::InspectLookup { .. } => Some(Box::new("arco::driver::inspect_lookup")),
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
            Self::InspectLookup { .. } | Self::Pipeline { .. } | Self::Execution { .. } => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::Pipeline(error) => Some(error),
            Self::Execution(_)
            | Self::Json { .. }
            | Self::BackendNotAvailable { .. }
            | Self::InspectLookup { .. } => None,
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
        compiled.timing.lower.as_secs_f64() * 1000.0
    );
    debug!(
        "lowered problem size: {} variable instances, {} constraint rows",
        compiled.lowered_problem.algebra.variable_instances.len(),
        compiled.lowered_problem.algebra.constraints.len()
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

pub fn validate_file_report(
    path: &Path,
    inspect: Option<InspectCategory>,
    name: Option<&str>,
) -> Result<String, DriverError> {
    let validated = validate_file(path)?;
    let program = &validated.semantic_program;

    match inspect {
        None => Ok(format!(
            "Validation succeeded\nentrypoint: {}\nscenario: {}\nassets: {}\nconstraints: {}",
            validated.entrypoint.display(),
            program.active_scenario,
            program.sets.assets.len(),
            program.active_constraints.len()
        )),
        Some(category) => match category {
            InspectCategory::Sets => format_inspect_sets(program, name),
            InspectCategory::Constraints => format_inspect_constraints(program, name),
            InspectCategory::Variables => format_inspect_variables(program, name),
            InspectCategory::Parameters => format_inspect_parameters(program, name),
            InspectCategory::Expressions => format_inspect_expressions(program, name),
            InspectCategory::Objective => format_inspect_objective(program),
            InspectCategory::Reports => format_inspect_reports(program, name),
            InspectCategory::Chronology => format_inspect_chronology(program),
        },
    }
}

fn format_inspect_sets(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<String, DriverError> {
    let sets = &program.sets;
    let set_registry = &program.set_registry;

    if let Some(target_name) = name {
        // Detail mode: look for a specific set
        match target_name {
            "assets" => Ok(format!("set \"assets\": {:?}", sets.assets)),
            "candidate_assets" => Ok(format!(
                "set \"candidate_assets\": {:?}",
                sets.candidate_assets
            )),
            "time" => Ok(format!(
                "set \"time\": {} steps @ {}",
                sets.time.steps, sets.time.resolution
            )),
            _ => {
                // Check user-declared sets
                if let Some(set) = set_registry.get(target_name) {
                    Ok(format!("set \"{}\": {:?}", target_name, set.values))
                } else {
                    // Build list of available names
                    let mut available = vec!["assets", "candidate_assets", "time"];
                    let mut user_sets: Vec<_> = set_registry
                        .keys()
                        .map(|s| s.as_str())
                        .filter(|s| !available.contains(s))
                        .collect();
                    user_sets.sort();
                    available.extend(user_sets);
                    Err(DriverError::InspectLookup {
                        message: format!(
                            "set '{}' not found. Available sets: {}",
                            target_name,
                            available.join(", ")
                        ),
                    })
                }
            }
        }
    } else {
        // List mode: show all sets with counts
        let mut lines = vec!["sets:".to_string()];
        lines.push(format!("  assets: {}", sets.assets.len()));
        lines.push(format!(
            "  candidate_assets: {}",
            sets.candidate_assets.len()
        ));
        lines.push(format!(
            "  time: {} steps @ {}",
            sets.time.steps, sets.time.resolution
        ));

        // Add user-declared sets (excluding built-in sets)
        let builtin = ["assets", "candidate_assets", "time"];
        let mut user_set_names: Vec<_> = set_registry
            .keys()
            .filter(|s| !builtin.contains(&s.as_str()))
            .collect();
        user_set_names.sort();
        for set_name in user_set_names {
            let count = set_registry
                .get(set_name)
                .map(|v| v.values.len())
                .unwrap_or(0);
            lines.push(format!("  {}: {}", set_name, count));
        }

        Ok(lines.join("\n"))
    }
}

fn format_inspect_constraints(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<String, DriverError> {
    let constraints = &program.active_constraints;

    if let Some(target_name) = name {
        // Detail mode: look for a specific constraint
        let target = constraints.iter().find(|c| c.name == target_name);
        match target {
            Some(constraint) => {
                let mut lines = vec![];
                lines.push(format!("constraint \"{}\":", target_name));
                lines.push(format!("  source_kind: {}", constraint.source_kind));
                lines.push(format!("  source_name: {}", constraint.source_name));
                lines.push(format!("  expression: {}", constraint.expression_text));
                if !constraint.generation_bindings.is_empty() {
                    lines.push("  generation_bindings:".to_string());
                    for binding in &constraint.generation_bindings {
                        lines.push(format!(
                            "    - variable: {}, domain: {}",
                            binding.variable, binding.domain
                        ));
                    }
                }
                Ok(lines.join("\n"))
            }
            None => {
                let available: Vec<_> = constraints.iter().map(|c| c.name.as_str()).collect();
                Err(DriverError::InspectLookup {
                    message: format!(
                        "constraint '{}' not found. Available constraints: {}",
                        target_name,
                        available.join(", ")
                    ),
                })
            }
        }
    } else {
        // List mode: show all constraint names
        let mut lines = vec!["constraints:".to_string()];
        for constraint in constraints {
            lines.push(format!(
                "  {} ({})",
                constraint.name, constraint.source_kind
            ));
        }
        Ok(lines.join("\n"))
    }
}

fn format_inspect_variables(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<String, DriverError> {
    let families = &program.variable_families;
    let overrides = &program.variable_overrides;

    if let Some(target_name) = name {
        // Detail mode: look for a specific variable family
        let target = families.iter().find(|f| f.target == target_name);
        match target {
            Some(family) => {
                let mut lines = vec![];
                lines.push(format!("variable \"{}\":", target_name));
                lines.push(format!("  signature: {}", family.render()));
                lines.push(format!("  index_domains: {:?}", family.index_domains));

                // Look for overrides (overrides is a BTreeMap<String, VariableDeclOverrides>)
                if let Some(override_def) = overrides.get(target_name) {
                    lines.push("  overrides:".to_string());
                    if let Some(kind) = &override_def.kind {
                        lines.push(format!("    kind: {:?}", kind));
                    }
                    if let Some(lower) = &override_def.lower {
                        lines.push(format!("    lower: {:?}", lower));
                    }
                    if let Some(upper) = &override_def.upper {
                        lines.push(format!("    upper: {:?}", upper));
                    }
                }
                Ok(lines.join("\n"))
            }
            None => {
                let available: Vec<_> = families.iter().map(|f| f.target.as_str()).collect();
                Err(DriverError::InspectLookup {
                    message: format!(
                        "variable '{}' not found. Available variables: {}",
                        target_name,
                        available.join(", ")
                    ),
                })
            }
        }
    } else {
        // List mode: show all variable family signatures
        let mut lines = vec!["variables:".to_string()];
        for family in families {
            lines.push(format!("  {}", family.render()));
        }
        Ok(lines.join("\n"))
    }
}

fn format_inspect_parameters(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<String, DriverError> {
    let params = &program.parameters;

    if let Some(target_name) = name {
        // Detail mode: find parameter by name
        let mut found_type = None;

        if params.series.iter().any(|p| p == target_name) {
            found_type = Some("series");
        } else if params.indexed.iter().any(|p| p == target_name) {
            found_type = Some("indexed");
        } else if params.asset.iter().any(|p| p == target_name) {
            found_type = Some("asset");
        }

        match found_type {
            Some(param_type) => Ok(format!(
                "parameter \"{}\": type = {}",
                target_name, param_type
            )),
            None => {
                let mut available = vec![];
                available.extend(params.series.iter().map(|p| p.as_str()));
                available.extend(params.indexed.iter().map(|p| p.as_str()));
                available.extend(params.asset.iter().map(|p| p.as_str()));
                Err(DriverError::InspectLookup {
                    message: format!(
                        "parameter '{}' not found. Available parameters: {}",
                        target_name,
                        available.join(", ")
                    ),
                })
            }
        }
    } else {
        // List mode: group by type
        let mut lines = vec!["parameters:".to_string()];

        if !params.series.is_empty() {
            lines.push("  series:".to_string());
            for param in &params.series {
                lines.push(format!("    - {}", param));
            }
        }

        if !params.indexed.is_empty() {
            lines.push("  indexed:".to_string());
            for param in &params.indexed {
                lines.push(format!("    - {}", param));
            }
        }

        if !params.asset.is_empty() {
            lines.push("  asset:".to_string());
            for param in &params.asset {
                lines.push(format!("    - {}", param));
            }
        }

        Ok(lines.join("\n"))
    }
}

fn format_inspect_expressions(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<String, DriverError> {
    let expressions = &program.active_expressions;

    if let Some(target_name) = name {
        // Detail mode: look for a specific expression
        let target = expressions.iter().find(|e| e.name == target_name);
        match target {
            Some(expr) => {
                let mut lines = vec![];
                lines.push(format!("expression \"{}\":", target_name));
                lines.push(format!("  formula: {}", expr.formula_text));
                Ok(lines.join("\n"))
            }
            None => {
                let available: Vec<_> = expressions.iter().map(|e| e.name.as_str()).collect();
                Err(DriverError::InspectLookup {
                    message: format!(
                        "expression '{}' not found. Available expressions: {}",
                        target_name,
                        available.join(", ")
                    ),
                })
            }
        }
    } else {
        // List mode: show all expression names
        let mut lines = vec!["expressions:".to_string()];
        for expr in expressions {
            lines.push(format!("  {}", expr.name));
        }
        Ok(lines.join("\n"))
    }
}

fn format_inspect_objective(
    program: &arco_kdl::semantic::SemanticProgram,
) -> Result<String, DriverError> {
    let objective = &program.active_objective;
    let mut lines = vec!["objective:".to_string()];
    lines.push(format!("  name: {}", objective.name));
    lines.push(format!("  sense: {}", objective.sense));
    lines.push(format!("  expression: {}", objective.expression_text));
    Ok(lines.join("\n"))
}

fn format_inspect_reports(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<String, DriverError> {
    let reports = &program.active_reports;

    if let Some(target_name) = name {
        // Detail mode: look for a specific report
        let target = reports.iter().find(|r| r.name == target_name);
        match target {
            Some(report) => {
                let mut lines = vec![];
                lines.push(format!("report \"{}\":", target_name));
                lines.push(format!("  formula: {}", report.formula_text));
                Ok(lines.join("\n"))
            }
            None => {
                let available: Vec<_> = reports.iter().map(|r| r.name.as_str()).collect();
                Err(DriverError::InspectLookup {
                    message: format!(
                        "report '{}' not found. Available reports: {}",
                        target_name,
                        available.join(", ")
                    ),
                })
            }
        }
    } else {
        // List mode: show all report names
        let mut lines = vec!["reports:".to_string()];
        for report in reports {
            lines.push(format!("  {}", report.name));
        }
        Ok(lines.join("\n"))
    }
}

fn format_inspect_chronology(
    program: &arco_kdl::semantic::SemanticProgram,
) -> Result<String, DriverError> {
    let chronology = &program.chronology;
    let mut lines = vec!["chronology:".to_string()];
    lines.push(format!(
        "  initial_boundary: {}",
        chronology.initial_boundary.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "  terminal_boundary: {}",
        chronology.terminal_boundary.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "  initial_commitment_boundary: {}",
        chronology
            .initial_commitment_boundary
            .as_deref()
            .unwrap_or("none")
    ));
    Ok(lines.join("\n"))
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
