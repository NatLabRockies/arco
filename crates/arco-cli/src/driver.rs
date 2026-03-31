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
use serde_json::{Map, Value, json};
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

pub fn validate_file_only(path: &Path) -> Result<(), DriverError> {
    let _ = validate_file(path)?;
    Ok(())
}

pub fn inspect_file_report(
    path: &Path,
    section: Option<InspectCategory>,
    name: Option<&str>,
    json_output: bool,
) -> Result<String, DriverError> {
    let validated = validate_file(path)?;
    let program = &validated.semantic_program;
    let payload = inspect_payload(&validated.entrypoint, program, section, name)?;

    if json_output {
        return serialize_validation_json(&payload).map_err(|source| DriverError::Json {
            path: path.to_path_buf(),
            source,
        });
    }

    Ok(render_pretty_inspect_report(section, &payload))
}

fn inspect_payload(
    entrypoint: &Path,
    program: &arco_kdl::semantic::SemanticProgram,
    section: Option<InspectCategory>,
    name: Option<&str>,
) -> Result<Value, DriverError> {
    let parameter_targets = collect_parameter_targets(program);

    match section {
        None => {
            let summary = payload_as_array(format_validation_summary(
                entrypoint,
                &program.active_scenario,
                program.set_registry.len(),
                program.variable_families.len(),
                program.active_constraints.len(),
            ));

            let sets_array = payload_as_array(format_inspect_sets(program, None)?);
            let constraints = payload_as_array(format_inspect_constraints_with_params(
                program,
                None,
                &parameter_targets,
            )?);
            let mut variables = payload_as_array(format_inspect_variables(program, None)?);
            let expressions = payload_as_array(format_inspect_expressions(program, None)?);
            let objectives = payload_as_array(format_inspect_objective_with_params(
                program,
                &parameter_targets,
            ));
            let reports = payload_as_array(format_inspect_reports(program, None)?);
            let chronologies = payload_as_array(format_inspect_chronology(program));

            let sets = compose_set_catalog_and_ref_variables(&mut variables, &sets_array);
            let parameters = compose_parameter_catalog(program, &parameter_targets, &sets);

            Ok(json!({
                "summaries": summary,
                "sets": sets,
                "constraints": constraints,
                "variables": variables,
                "parameters": parameters,
                "expressions": expressions,
                "objectives": objectives,
                "reports": reports,
                "chronologies": chronologies,
            }))
        }
        Some(category) => match category {
            InspectCategory::Sets => format_inspect_sets(program, name),
            InspectCategory::Constraints => {
                format_inspect_constraints_with_params(program, name, &parameter_targets)
            }
            InspectCategory::Variables => format_inspect_variables(program, name),
            InspectCategory::Parameters => format_inspect_parameters(program, name),
            InspectCategory::Expressions => format_inspect_expressions(program, name),
            InspectCategory::Objective => Ok(format_inspect_objective_with_params(
                program,
                &parameter_targets,
            )),
            InspectCategory::Reports => format_inspect_reports(program, name),
            InspectCategory::Chronology => Ok(format_inspect_chronology(program)),
        },
    }
}

fn render_pretty_inspect_report(section: Option<InspectCategory>, payload: &Value) -> String {
    match section {
        Some(category) => render_pretty_section(category, payload),
        None => render_pretty_full_inspect(payload),
    }
}

fn render_pretty_full_inspect(payload: &Value) -> String {
    let Some(object) = payload.as_object() else {
        return value_to_compact_string(payload);
    };

    let mut sections = Vec::new();

    if let Some(summary) = object.get("summaries") {
        sections.push(render_pretty_card_block(
            "summary",
            summary_items(summary).as_slice(),
        ));
    }

    if let Some(sets) = object.get("sets").and_then(Value::as_object) {
        let mut lines = Vec::new();
        for (name, definition) in sets {
            let cardinality = definition
                .get("cardinality")
                .map_or_else(|| "?".to_string(), value_to_compact_string);
            let symbol = definition
                .get("symbol")
                .and_then(Value::as_str)
                .map_or(String::new(), |value| format!(" ({value})"));
            lines.push(format!("{name}{symbol}: {cardinality}"));
        }
        sections.push(lines.join("\n"));
    }

    for (category, key) in [
        (InspectCategory::Constraints, "constraints"),
        (InspectCategory::Variables, "variables"),
        (InspectCategory::Parameters, "parameters"),
        (InspectCategory::Expressions, "expressions"),
        (InspectCategory::Objective, "objectives"),
        (InspectCategory::Reports, "reports"),
        (InspectCategory::Chronology, "chronologies"),
    ] {
        if let Some(value) = object.get(key) {
            let rendered = render_pretty_section(category, value);
            if !rendered.trim().is_empty() {
                sections.push(rendered);
            }
        }
    }

    sections.join("\n\n")
}

fn render_pretty_section(category: InspectCategory, payload: &Value) -> String {
    let kind = match category {
        InspectCategory::Sets => "set",
        InspectCategory::Constraints => "constraint",
        InspectCategory::Variables => "variable",
        InspectCategory::Parameters => "parameter",
        InspectCategory::Expressions => "expression",
        InspectCategory::Objective => "objective",
        InspectCategory::Reports => "report",
        InspectCategory::Chronology => "chronology",
    };

    render_pretty_card_block(kind, summary_items(payload).as_slice())
}

fn summary_items(payload: &Value) -> Vec<Value> {
    if let Some(items) = payload.get("items").and_then(Value::as_array) {
        return items.clone();
    }
    if let Some(items) = payload.as_array() {
        return items.clone();
    }
    vec![payload.clone()]
}

fn render_pretty_card_block(kind: &str, items: &[Value]) -> String {
    let mut cards = Vec::new();

    for item in items {
        let Some(card) = item.as_object() else {
            cards.push(value_to_compact_string(item));
            continue;
        };

        let mut lines = vec![format!("[{kind}]")];
        let mut entries: Vec<(String, String)> = Vec::new();

        if let Some(name) = card.get("name") {
            entries.push(("name".to_string(), value_to_compact_string(name)));
        }
        if let Some(notation) = card.get("notation") {
            entries.push(("notation".to_string(), value_to_compact_string(notation)));
        }
        if let Some(set) = card.get("set") {
            entries.push(("domains".to_string(), format_domain_list(set)));
        }

        for (field, value) in card {
            if ["kind", "name", "notation", "set"].contains(&field.as_str()) {
                continue;
            }
            if kind == "variable" && field == "domain" {
                continue;
            }
            entries.push((field.clone(), value_to_compact_string(value)));
        }

        let label_width = entries
            .iter()
            .map(|(label, _)| label.len())
            .max()
            .unwrap_or(0);

        for (label, value) in entries {
            lines.push(format!("  {label:<label_width$} : {value}"));
        }

        cards.push(lines.join("\n"));
    }

    cards.join("\n\n")
}

fn format_domain_list(value: &Value) -> String {
    let Some(domains) = value.as_array() else {
        return value_to_compact_string(value);
    };

    let rendered = domains
        .iter()
        .map(|domain| {
            let Some(domain_object) = domain.as_object() else {
                return value_to_compact_string(domain);
            };

            if let (Some(index), Some(name)) = (
                domain_object.get("index").and_then(Value::as_str),
                domain_object.get("name").and_then(Value::as_str),
            ) {
                return format!("{index} ∈ {name}");
            }

            if let Some(reference) = domain_object.get("$ref").and_then(Value::as_str) {
                return reference
                    .rsplit('/')
                    .next()
                    .map_or_else(|| reference.to_string(), ToString::to_string);
            }

            value_to_compact_string(domain)
        })
        .collect::<Vec<_>>();

    rendered.join(", ")
}

fn value_to_compact_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => string.clone(),
        Value::Array(items) => items
            .iter()
            .map(value_to_compact_string)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                return reference
                    .rsplit('/')
                    .next()
                    .map_or_else(|| reference.to_string(), ToString::to_string);
            }

            object
                .iter()
                .map(|(key, entry)| format!("{key}={}", value_to_compact_string(entry)))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

fn serialize_validation_json(payload: &Value) -> Result<String, serde_json::Error> {
    if payload.get("kind").is_none() && payload.get("items").is_none() {
        return serde_json::to_string(payload);
    }
    serde_json::to_string(&payload_as_array(payload.clone()))
}

fn payload_as_array(payload: Value) -> Value {
    if let Some(items) = payload.get("items").and_then(Value::as_array) {
        return Value::Array(items.iter().map(strip_kind_field).collect::<Vec<_>>());
    }

    Value::Array(vec![strip_kind_field(&payload)])
}

fn compose_set_catalog_and_ref_variables(variables: &mut Value, sets: &Value) -> Value {
    let Some(variable_items) = variables.as_array_mut() else {
        return Value::Object(Map::new());
    };
    let Some(set_items) = sets.as_array() else {
        return Value::Object(Map::new());
    };

    let mut set_catalog = Map::new();
    for set_item in set_items {
        let Some(set_name) = set_item.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(cardinality) = set_item.get("cardinality") else {
            continue;
        };
        set_catalog.insert(
            set_name.to_string(),
            json!({
                "cardinality": cardinality,
            }),
        );
    }

    for variable in variable_items {
        let Some(variable_object) = variable.as_object_mut() else {
            continue;
        };
        let Some(set_memberships) = variable_object.get_mut("set").and_then(Value::as_array_mut)
        else {
            continue;
        };

        for membership in set_memberships {
            let Some(index_name) = membership.get("index").and_then(Value::as_str) else {
                continue;
            };
            let Some(set_name) = membership.get("name").and_then(Value::as_str) else {
                continue;
            };

            if !set_catalog.contains_key(set_name) {
                continue;
            }

            if let Some(set_object) = set_catalog.get_mut(set_name).and_then(Value::as_object_mut) {
                set_object
                    .entry("symbol")
                    .or_insert_with(|| Value::String(index_name.to_string()));
            }

            *membership = json!({
                "$ref": format!("#/sets/{set_name}"),
            });
        }
    }

    Value::Object(set_catalog)
}

fn strip_kind_field(value: &Value) -> Value {
    let mut stripped = value.clone();
    if let Some(object) = stripped.as_object_mut() {
        object.remove("kind");
    }
    stripped
}

fn format_validation_summary(
    entrypoint: &Path,
    scenario: &str,
    set_count: usize,
    variable_count: usize,
    constraint_count: usize,
) -> Value {
    json!({
        "kind": "summary",
        "entrypoint": entrypoint.display().to_string(),
        "scenario": scenario,
        "counts": {
            "sets": set_count,
            "variables": variable_count,
            "constraints": constraint_count,
        }
    })
}

fn render_named_card(kind: &str, fields: Vec<(&str, Value)>) -> Value {
    let mut object = Map::new();
    object.insert("kind".to_string(), Value::String(kind.to_string()));
    for (key, value) in fields {
        object.insert(key.to_string(), value);
    }
    Value::Object(object)
}

fn format_inspect_sets(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<Value, DriverError> {
    let set_registry = &program.set_registry;
    let filtered_set_names = ["assets", "candidate_assets"];
    let available_set_names: Vec<&str> = set_registry
        .keys()
        .map(String::as_str)
        .filter(|set_name| !filtered_set_names.contains(set_name))
        .collect();

    if let Some(target_name) = name {
        if filtered_set_names.contains(&target_name) {
            return Err(DriverError::InspectLookup {
                message: format!(
                    "set '{}' not found. Available sets: {}",
                    target_name,
                    available_set_names.join(", ")
                ),
            });
        }

        if let Some(set) = set_registry.get(target_name) {
            Ok(render_named_card(
                "set",
                vec![
                    ("name", Value::String(target_name.to_string())),
                    ("cardinality", json!(set.values.len())),
                    ("values", json!(set.values)),
                ],
            ))
        } else {
            Err(DriverError::InspectLookup {
                message: format!(
                    "set '{}' not found. Available sets: {}",
                    target_name,
                    available_set_names.join(", ")
                ),
            })
        }
    } else {
        let items = set_registry
            .iter()
            .filter(|(set_name, _)| !filtered_set_names.contains(&set_name.as_str()))
            .map(|(set_name, set_values)| {
                render_named_card(
                    "set",
                    vec![
                        ("name", Value::String(set_name.clone())),
                        ("cardinality", json!(set_values.values.len())),
                    ],
                )
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "kind": "sets",
            "items": items,
        }))
    }
}

fn format_inspect_constraints_with_params(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
    parameter_targets: &std::collections::BTreeSet<String>,
) -> Result<Value, DriverError> {
    let constraints = &program.active_constraints;
    let variable_targets = collect_variable_targets(program);

    if let Some(target_name) = name {
        // Detail mode: look for a specific constraint
        let target = constraints.iter().find(|c| c.name == target_name);
        if let Some(constraint) = target {
            Ok(render_constraint_details(
                constraint,
                &variable_targets,
                parameter_targets,
            ))
        } else {
            let available: Vec<_> = constraints.iter().map(|c| c.name.as_str()).collect();
            Err(DriverError::InspectLookup {
                message: format!(
                    "constraint '{}' not found. Available constraints: {}",
                    target_name,
                    available.join(", ")
                ),
            })
        }
    } else {
        let items = constraints
            .iter()
            .map(|constraint| {
                render_constraint_details(constraint, &variable_targets, parameter_targets)
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "kind": "constraints",
            "items": items,
        }))
    }
}

fn render_constraint_details(
    constraint: &arco_kdl::semantic::ResolvedConstraint,
    variable_targets: &std::collections::BTreeSet<String>,
    parameter_targets: &std::collections::BTreeSet<String>,
) -> Value {
    let mut fields = vec![
        ("name", Value::String(constraint.name.clone())),
        ("source_kind", Value::String(constraint.source_kind.clone())),
        ("source_name", Value::String(constraint.source_name.clone())),
        (
            "template",
            Value::String(constraint.expression_text.clone()),
        ),
    ];

    match &constraint.expression {
        arco_kdl::algebra::ConstraintBody::Comparison { op, left, right } => {
            fields.push((
                "relation",
                Value::String(constraint_relation_name(*op).to_string()),
            ));
            fields.push(("lhs", Value::String(left.to_string())));
            fields.push(("rhs", Value::String(right.to_string())));
            fields.push((
                "lhs_terms",
                Value::Array(
                    expr_additive_terms(left)
                        .into_iter()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ));
            fields.push((
                "rhs_terms",
                Value::Array(
                    expr_additive_terms(right)
                        .into_iter()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ));
        }
        arco_kdl::algebra::ConstraintBody::Range {
            lower,
            lower_op,
            middle,
            upper_op,
            upper,
        } => {
            fields.push(("relation", Value::String("range".to_string())));
            fields.push(("lower", Value::String(lower.to_string())));
            fields.push((
                "lower_relation",
                Value::String(constraint_relation_name(*lower_op).to_string()),
            ));
            fields.push(("middle", Value::String(middle.to_string())));
            fields.push((
                "middle_terms",
                Value::Array(
                    expr_additive_terms(middle)
                        .into_iter()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ));
            fields.push((
                "upper_relation",
                Value::String(constraint_relation_name(*upper_op).to_string()),
            ));
            fields.push(("upper", Value::String(upper.to_string())));
        }
    }

    if !constraint.generation_bindings.is_empty() {
        fields.push((
            "scope",
            Value::Array(
                constraint
                    .generation_bindings
                    .iter()
                    .map(|binding| {
                        json!({
                            "symbol": binding.variable,
                            "$ref": format!("#/sets/{}", binding.domain),
                        })
                    })
                    .collect::<Vec<_>>(),
            ),
        ));
    }

    if let Some(condition) = &constraint.generation_filter {
        fields.push(("condition", Value::String(condition.to_string())));
    }

    fields.push((
        "variable_refs",
        Value::Array(extract_constraint_variable_refs(
            constraint,
            variable_targets,
        )),
    ));
    fields.push((
        "parameter_refs",
        Value::Array(extract_constraint_parameter_refs(
            constraint,
            parameter_targets,
        )),
    ));

    render_named_card("constraint", fields)
}

fn constraint_relation_name(op: arco_kdl::algebra::ComparisonOp) -> &'static str {
    match op {
        arco_kdl::algebra::ComparisonOp::Equal | arco_kdl::algebra::ComparisonOp::DoubleEqual => {
            "equal"
        }
        arco_kdl::algebra::ComparisonOp::LessEqual => "less_or_equal",
        arco_kdl::algebra::ComparisonOp::GreaterEqual => "greater_or_equal",
        arco_kdl::algebra::ComparisonOp::Less => "less",
        arco_kdl::algebra::ComparisonOp::Greater => "greater",
        arco_kdl::algebra::ComparisonOp::NotEqual => "not_equal",
    }
}

fn expr_additive_terms(expr: &arco_kdl::algebra::Expr) -> Vec<String> {
    match expr {
        arco_kdl::algebra::Expr::Binary { op, left, right }
            if *op == arco_kdl::algebra::BinaryOp::Add =>
        {
            let mut terms = expr_additive_terms(left);
            terms.extend(expr_additive_terms(right));
            terms
        }
        arco_kdl::algebra::Expr::Binary { op, left, right }
            if *op == arco_kdl::algebra::BinaryOp::Subtract =>
        {
            let mut terms = expr_additive_terms(left);
            for term in expr_additive_terms(right) {
                terms.push(format!("-({term})"));
            }
            terms
        }
        arco_kdl::algebra::Expr::Binary { op, left, right }
            if *op == arco_kdl::algebra::BinaryOp::Multiply =>
        {
            if let Some(factors) = additive_factor_terms(left) {
                if factors.len() > 1 {
                    return factors
                        .into_iter()
                        .map(|factor| format!("{factor} * {}", right))
                        .collect::<Vec<_>>();
                }
            }

            if let Some(factors) = additive_factor_terms(right) {
                if factors.len() > 1 {
                    return factors
                        .into_iter()
                        .map(|factor| format!("{} * {factor}", left))
                        .collect::<Vec<_>>();
                }
            }

            vec![expr.to_string()]
        }
        _ => vec![expr.to_string()],
    }
}

fn additive_factor_terms(expr: &arco_kdl::algebra::Expr) -> Option<Vec<String>> {
    match expr {
        arco_kdl::algebra::Expr::Binary { op, left, right }
            if *op == arco_kdl::algebra::BinaryOp::Add =>
        {
            let mut terms = additive_factor_terms(left)?;
            terms.extend(additive_factor_terms(right)?);
            Some(terms)
        }
        arco_kdl::algebra::Expr::Binary { op, left, right }
            if *op == arco_kdl::algebra::BinaryOp::Subtract =>
        {
            let mut terms = additive_factor_terms(left)?;
            for term in additive_factor_terms(right)? {
                terms.push(format!("-({term})"));
            }
            Some(terms)
        }
        _ => Some(vec![expr.to_string()]),
    }
}

fn format_inspect_variables(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<Value, DriverError> {
    let families = &program.variable_families;
    let overrides = &program.variable_overrides;

    if let Some(target_name) = name {
        // Detail mode: look for a specific variable family
        let target = families.iter().find(|f| f.target == target_name);
        if let Some(family) = target {
            let mut fields = vec![
                ("name", Value::String(family.target.clone())),
                (
                    "notation",
                    Value::String(render_variable_math_notation(family)),
                ),
                (
                    "set",
                    Value::Array(render_variable_domains(family, &program.set_registry)),
                ),
                (
                    "domain",
                    render_variable_value_domain(family, overrides.get(target_name)),
                ),
            ];

            // Look for overrides (overrides is a BTreeMap<String, VariableDeclOverrides>)
            if let Some(override_def) = overrides.get(target_name) {
                if let Some(kind) = &override_def.kind {
                    fields.push(("override_kind", Value::String(format!("{:?}", kind))));
                }
                if let Some(lower) = &override_def.lower {
                    fields.push(("override_lower", Value::String(format!("{:?}", lower))));
                }
                if let Some(upper) = &override_def.upper {
                    fields.push(("override_upper", Value::String(format!("{:?}", upper))));
                }
            }
            Ok(render_named_card("variable", fields))
        } else {
            let available: Vec<_> = families.iter().map(|f| f.target.as_str()).collect();
            Err(DriverError::InspectLookup {
                message: format!(
                    "variable '{}' not found. Available variables: {}",
                    target_name,
                    available.join(", ")
                ),
            })
        }
    } else {
        let items = families
            .iter()
            .map(|family| {
                render_named_card(
                    "variable",
                    vec![
                        ("name", Value::String(family.target.clone())),
                        (
                            "notation",
                            Value::String(render_variable_math_notation(family)),
                        ),
                        (
                            "set",
                            Value::Array(render_variable_domains(family, &program.set_registry)),
                        ),
                        (
                            "domain",
                            render_variable_value_domain(family, overrides.get(&family.target)),
                        ),
                    ],
                )
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "kind": "variables",
            "items": items,
        }))
    }
}

fn render_variable_math_notation(family: &arco_kdl::semantic::FamilySignature) -> String {
    if family.indices.is_empty() {
        return family.target.clone();
    }

    format!("{}[{}]", family.target, family.indices.join(", "))
}

fn render_variable_domains(
    family: &arco_kdl::semantic::FamilySignature,
    set_registry: &std::collections::BTreeMap<String, arco_kdl::semantic::ResolvedSet>,
) -> Vec<Value> {
    family
        .indices
        .iter()
        .map(|index| {
            let set_name = family
                .index_domains
                .get(index)
                .cloned()
                .unwrap_or_else(|| index.clone());
            let cardinality = set_registry
                .get(&set_name)
                .map_or(0, |resolved_set| resolved_set.values.len());

            json!({
                "index": index,
                "name": set_name,
                "cardinality": cardinality,
            })
        })
        .collect::<Vec<_>>()
}

fn render_variable_value_domain(
    family: &arco_kdl::semantic::FamilySignature,
    overrides: Option<&arco_kdl::semantic::VariableDeclOverrides>,
) -> Value {
    let (mut kind, mut lower, mut upper) = match family.target.as_str() {
        "build" => (
            Value::String("continuous".to_string()),
            json!(0.0),
            Value::String("max_build[a]".to_string()),
        ),
        "unserved_energy" | "charge" | "discharge" | "generation" => (
            Value::String("continuous".to_string()),
            json!(0.0),
            Value::Null,
        ),
        "dispatch" => (
            Value::String("continuous".to_string()),
            Value::String("asset-dependent".to_string()),
            Value::Null,
        ),
        "commit" | "start" | "shutdown" => {
            (Value::String("binary".to_string()), json!(0.0), json!(1.0))
        }
        _ => (
            Value::String("continuous".to_string()),
            Value::String("-inf".to_string()),
            Value::Null,
        ),
    };

    if let Some(override_def) = overrides {
        if let Some(kind_override) = &override_def.kind {
            let label = match kind_override {
                arco_kdl::source::VariableKindDecl::Continuous => "continuous",
                arco_kdl::source::VariableKindDecl::Integer => "integer",
                arco_kdl::source::VariableKindDecl::Binary => "binary",
            };
            kind = Value::String(label.to_string());
        }

        if let Some(lower_override) = &override_def.lower {
            lower = render_bound_expr(lower_override);
        }

        if let Some(upper_override) = &override_def.upper {
            upper = render_bound_expr(upper_override);
        }
    }

    json!({
        "kind": kind,
        "lower": lower,
        "upper": upper,
    })
}

fn render_bound_expr(bound: &arco_kdl::source::BoundExpr) -> Value {
    match bound {
        arco_kdl::source::BoundExpr::Literal(arco_kdl::source::LiteralValue::Integer(value)) => {
            serde_json::Number::from_i128(*value)
                .map_or_else(|| Value::String(value.to_string()), Value::Number)
        }
        arco_kdl::source::BoundExpr::Literal(arco_kdl::source::LiteralValue::Decimal(text)) => {
            match text.parse::<f64>() {
                Ok(parsed) => serde_json::Number::from_f64(parsed)
                    .map_or_else(|| Value::String(text.clone()), Value::Number),
                Err(_) => Value::String(text.clone()),
            }
        }
        arco_kdl::source::BoundExpr::Literal(other) => Value::String(format!("{other:?}")),
        arco_kdl::source::BoundExpr::Formula(expr) => Value::String(format!("{expr:?}")),
    }
}

fn format_inspect_parameters(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<Value, DriverError> {
    let catalog = build_parameter_catalog(program)?;
    let catalog_object = catalog
        .as_object()
        .ok_or_else(|| DriverError::InspectLookup {
            message: "parameter catalog is not an object".to_string(),
        })?;

    if let Some(target_name) = name {
        if let Some(parameter) = catalog_object.get(target_name) {
            Ok(render_named_card(
                "parameter",
                vec![
                    ("name", Value::String(target_name.to_string())),
                    (
                        "type",
                        parameter
                            .get("kind")
                            .cloned()
                            .unwrap_or_else(|| Value::String("inferred".to_string())),
                    ),
                    (
                        "set",
                        parameter
                            .get("set")
                            .cloned()
                            .unwrap_or_else(|| Value::Array(Vec::new())),
                    ),
                ],
            ))
        } else {
            let mut available = catalog_object.keys().cloned().collect::<Vec<_>>();
            available.sort();
            Err(DriverError::InspectLookup {
                message: format!(
                    "parameter '{}' not found. Available parameters: {}",
                    target_name,
                    available.join(", ")
                ),
            })
        }
    } else {
        let mut parameter_names = catalog_object.keys().cloned().collect::<Vec<_>>();
        parameter_names.sort();
        let items = parameter_names
            .into_iter()
            .filter_map(|param_name| {
                catalog_object.get(&param_name).map(|parameter| {
                    render_named_card(
                        "parameter",
                        vec![
                            ("name", Value::String(param_name)),
                            (
                                "type",
                                parameter
                                    .get("kind")
                                    .cloned()
                                    .unwrap_or_else(|| Value::String("inferred".to_string())),
                            ),
                            (
                                "set",
                                parameter
                                    .get("set")
                                    .cloned()
                                    .unwrap_or_else(|| Value::Array(Vec::new())),
                            ),
                        ],
                    )
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "kind": "parameters",
            "items": items,
        }))
    }
}

fn build_parameter_catalog(
    program: &arco_kdl::semantic::SemanticProgram,
) -> Result<Value, DriverError> {
    let sets_array = payload_as_array(format_inspect_sets(program, None)?);
    let mut variables = payload_as_array(format_inspect_variables(program, None)?);
    let sets = compose_set_catalog_and_ref_variables(&mut variables, &sets_array);
    let parameter_targets = collect_parameter_targets(program);
    Ok(compose_parameter_catalog(
        program,
        &parameter_targets,
        &sets,
    ))
}

fn format_inspect_expressions(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<Value, DriverError> {
    let expressions = &program.active_expressions;

    if let Some(target_name) = name {
        // Detail mode: look for a specific expression
        let target = expressions.iter().find(|e| e.name == target_name);
        if let Some(expr) = target {
            Ok(render_named_card(
                "expression",
                vec![
                    ("name", Value::String(target_name.to_string())),
                    ("formula", Value::String(expr.formula_text.clone())),
                ],
            ))
        } else {
            let available: Vec<_> = expressions.iter().map(|e| e.name.as_str()).collect();
            Err(DriverError::InspectLookup {
                message: format!(
                    "expression '{}' not found. Available expressions: {}",
                    target_name,
                    available.join(", ")
                ),
            })
        }
    } else {
        let items = expressions
            .iter()
            .map(|expr| {
                render_named_card(
                    "expression",
                    vec![
                        ("name", Value::String(expr.name.clone())),
                        ("formula", Value::String(expr.formula_text.clone())),
                    ],
                )
            })
            .collect::<Vec<_>>();
        Ok(json!({
            "kind": "expressions",
            "items": items,
        }))
    }
}

fn format_inspect_objective_with_params(
    program: &arco_kdl::semantic::SemanticProgram,
    parameter_targets: &std::collections::BTreeSet<String>,
) -> Value {
    let objective = &program.active_objective;
    let variable_targets = collect_variable_targets(program);
    let mut fields = vec![
        ("name", Value::String(objective.name.clone())),
        ("sense", Value::String(objective.sense.clone())),
    ];

    match &objective.expression {
        arco_kdl::algebra::Expr::Reduction(reduction) => {
            let aggregation = match reduction.op {
                arco_kdl::algebra::ReductionOp::Sum => "sum",
            };
            fields.push(("aggregation", Value::String(aggregation.to_string())));
            fields.push(("scope", Value::Array(render_reduction_scope(reduction))));
            if !reduction.filters.is_empty() {
                fields.push((
                    "conditions",
                    Value::Array(
                        reduction
                            .filters
                            .iter()
                            .map(|filter| Value::String(filter.to_string()))
                            .collect::<Vec<_>>(),
                    ),
                ));
            }
            fields.push((
                "terms",
                Value::Array(
                    expr_additive_terms(&reduction.body)
                        .into_iter()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ));
        }
        expression => {
            fields.push(("aggregation", Value::String("scalar".to_string())));
            fields.push((
                "terms",
                Value::Array(
                    expr_additive_terms(expression)
                        .into_iter()
                        .map(Value::String)
                        .collect::<Vec<_>>(),
                ),
            ));
        }
    }

    fields.push((
        "variable_refs",
        Value::Array(extract_expr_variable_refs(
            &objective.expression,
            &variable_targets,
        )),
    ));
    fields.push((
        "parameter_refs",
        Value::Array(extract_expr_parameter_refs(
            &objective.expression,
            parameter_targets,
        )),
    ));

    render_named_card("objective", fields)
}

fn compose_parameter_catalog(
    program: &arco_kdl::semantic::SemanticProgram,
    parameter_targets: &std::collections::BTreeSet<String>,
    sets: &Value,
) -> Value {
    let mut catalog = Map::new();

    let symbol_to_set = sets
        .as_object()
        .map(|set_catalog| {
            set_catalog
                .iter()
                .filter_map(|(set_name, set_data)| {
                    set_data
                        .get("symbol")
                        .and_then(Value::as_str)
                        .map(|symbol| (symbol.to_string(), set_name.clone()))
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let make_parameter_entry = |kind: &str, name: &str| {
        let mut entry = Map::new();
        entry.insert("kind".to_string(), Value::String(kind.to_string()));
        let set_refs = collect_parameter_set_refs(program, name, &symbol_to_set)
            .into_iter()
            .map(|set_name| json!({"$ref": format!("#/sets/{set_name}")}))
            .collect::<Vec<_>>();
        if !set_refs.is_empty() {
            entry.insert("set".to_string(), Value::Array(set_refs));
        }
        Value::Object(entry)
    };

    for name in &program.parameters.series {
        catalog.insert(name.clone(), make_parameter_entry("series", name));
    }
    for name in &program.parameters.indexed {
        catalog.insert(name.clone(), make_parameter_entry("indexed", name));
    }
    for name in &program.parameters.asset {
        catalog.insert(name.clone(), make_parameter_entry("asset", name));
    }

    for name in parameter_targets {
        catalog
            .entry(name.clone())
            .or_insert_with(|| make_parameter_entry("inferred", name));
    }

    Value::Object(catalog)
}

fn collect_parameter_set_refs(
    program: &arco_kdl::semantic::SemanticProgram,
    parameter_name: &str,
    symbol_to_set: &std::collections::BTreeMap<String, String>,
) -> std::collections::BTreeSet<String> {
    let mut refs = std::collections::BTreeSet::new();

    for constraint in &program.active_constraints {
        match &constraint.expression {
            arco_kdl::algebra::ConstraintBody::Comparison { left, right, .. } => {
                collect_parameter_sets_from_expr(left, parameter_name, symbol_to_set, &mut refs);
                collect_parameter_sets_from_expr(right, parameter_name, symbol_to_set, &mut refs);
            }
            arco_kdl::algebra::ConstraintBody::Range {
                lower,
                middle,
                upper,
                ..
            } => {
                collect_parameter_sets_from_expr(lower, parameter_name, symbol_to_set, &mut refs);
                collect_parameter_sets_from_expr(middle, parameter_name, symbol_to_set, &mut refs);
                collect_parameter_sets_from_expr(upper, parameter_name, symbol_to_set, &mut refs);
            }
        }
        if let Some(condition) = &constraint.generation_filter {
            collect_parameter_sets_from_expr(condition, parameter_name, symbol_to_set, &mut refs);
        }
    }

    collect_parameter_sets_from_expr(
        &program.active_objective.expression,
        parameter_name,
        symbol_to_set,
        &mut refs,
    );
    for report in &program.active_reports {
        collect_parameter_sets_from_expr(&report.formula, parameter_name, symbol_to_set, &mut refs);
    }
    for expression in &program.active_expressions {
        collect_parameter_sets_from_expr(
            &expression.formula,
            parameter_name,
            symbol_to_set,
            &mut refs,
        );
    }

    refs
}

fn collect_parameter_sets_from_expr(
    expr: &arco_kdl::algebra::Expr,
    parameter_name: &str,
    symbol_to_set: &std::collections::BTreeMap<String, String>,
    refs: &mut std::collections::BTreeSet<String>,
) {
    match expr {
        arco_kdl::algebra::Expr::Indexed { target, indices } => {
            if target == parameter_name {
                for index in indices {
                    if let arco_kdl::algebra::Expr::Identifier(symbol) = index {
                        if let Some(set_name) = symbol_to_set.get(symbol) {
                            refs.insert(set_name.clone());
                        }
                    }
                }
            }
            for index in indices {
                collect_parameter_sets_from_expr(index, parameter_name, symbol_to_set, refs);
            }
        }
        arco_kdl::algebra::Expr::Unary { expr, .. } => {
            collect_parameter_sets_from_expr(expr, parameter_name, symbol_to_set, refs);
        }
        arco_kdl::algebra::Expr::Binary { left, right, .. }
        | arco_kdl::algebra::Expr::Comparison { left, right, .. } => {
            collect_parameter_sets_from_expr(left, parameter_name, symbol_to_set, refs);
            collect_parameter_sets_from_expr(right, parameter_name, symbol_to_set, refs);
        }
        arco_kdl::algebra::Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_parameter_sets_from_expr(arg, parameter_name, symbol_to_set, refs);
            }
        }
        arco_kdl::algebra::Expr::Reduction(reduction) => {
            collect_parameter_sets_from_expr(&reduction.body, parameter_name, symbol_to_set, refs);
            for filter in &reduction.filters {
                collect_parameter_sets_from_expr(filter, parameter_name, symbol_to_set, refs);
            }
        }
        arco_kdl::algebra::Expr::Number(_)
        | arco_kdl::algebra::Expr::String(_)
        | arco_kdl::algebra::Expr::Boolean(_)
        | arco_kdl::algebra::Expr::Identifier(_) => {}
    }
}

fn collect_parameter_targets(
    program: &arco_kdl::semantic::SemanticProgram,
) -> std::collections::BTreeSet<String> {
    let variable_targets = collect_variable_targets(program);
    let set_targets = program
        .set_registry
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    let mut targets = std::collections::BTreeSet::new();

    for constraint in &program.active_constraints {
        match &constraint.expression {
            arco_kdl::algebra::ConstraintBody::Comparison { left, right, .. } => {
                collect_expr_indexed_targets(left, &mut targets);
                collect_expr_indexed_targets(right, &mut targets);
            }
            arco_kdl::algebra::ConstraintBody::Range {
                lower,
                middle,
                upper,
                ..
            } => {
                collect_expr_indexed_targets(lower, &mut targets);
                collect_expr_indexed_targets(middle, &mut targets);
                collect_expr_indexed_targets(upper, &mut targets);
            }
        }
        if let Some(condition) = &constraint.generation_filter {
            collect_expr_indexed_targets(condition, &mut targets);
        }
    }

    collect_expr_indexed_targets(&program.active_objective.expression, &mut targets);
    for report in &program.active_reports {
        collect_expr_indexed_targets(&report.formula, &mut targets);
    }
    for expression in &program.active_expressions {
        collect_expr_indexed_targets(&expression.formula, &mut targets);
    }

    targets
        .into_iter()
        .filter(|target| !variable_targets.contains(target) && !set_targets.contains(target))
        .collect::<std::collections::BTreeSet<_>>()
}

fn collect_variable_targets(
    program: &arco_kdl::semantic::SemanticProgram,
) -> std::collections::BTreeSet<String> {
    program
        .variable_families
        .iter()
        .map(|family| family.target.clone())
        .collect::<std::collections::BTreeSet<_>>()
}

fn extract_constraint_parameter_refs(
    constraint: &arco_kdl::semantic::ResolvedConstraint,
    parameter_targets: &std::collections::BTreeSet<String>,
) -> Vec<Value> {
    let targets = collect_constraint_indexed_targets(constraint);
    extract_indexed_refs(&targets, parameter_targets, "#/parameters")
}

fn extract_expr_parameter_refs(
    expr: &arco_kdl::algebra::Expr,
    parameter_targets: &std::collections::BTreeSet<String>,
) -> Vec<Value> {
    let mut targets = std::collections::BTreeSet::new();
    collect_expr_indexed_targets(expr, &mut targets);
    extract_indexed_refs(&targets, parameter_targets, "#/parameters")
}

fn extract_constraint_variable_refs(
    constraint: &arco_kdl::semantic::ResolvedConstraint,
    variable_targets: &std::collections::BTreeSet<String>,
) -> Vec<Value> {
    let targets = collect_constraint_indexed_targets(constraint);
    extract_indexed_refs(&targets, variable_targets, "#/variables")
}

fn extract_expr_variable_refs(
    expr: &arco_kdl::algebra::Expr,
    variable_targets: &std::collections::BTreeSet<String>,
) -> Vec<Value> {
    let mut targets = std::collections::BTreeSet::new();
    collect_expr_indexed_targets(expr, &mut targets);
    extract_indexed_refs(&targets, variable_targets, "#/variables")
}

fn collect_constraint_indexed_targets(
    constraint: &arco_kdl::semantic::ResolvedConstraint,
) -> std::collections::BTreeSet<String> {
    let mut targets = std::collections::BTreeSet::new();
    match &constraint.expression {
        arco_kdl::algebra::ConstraintBody::Comparison { left, right, .. } => {
            collect_expr_indexed_targets(left, &mut targets);
            collect_expr_indexed_targets(right, &mut targets);
        }
        arco_kdl::algebra::ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            collect_expr_indexed_targets(lower, &mut targets);
            collect_expr_indexed_targets(middle, &mut targets);
            collect_expr_indexed_targets(upper, &mut targets);
        }
    }

    if let Some(condition) = &constraint.generation_filter {
        collect_expr_indexed_targets(condition, &mut targets);
    }
    targets
}

fn extract_indexed_refs(
    indexed_targets: &std::collections::BTreeSet<String>,
    allowed_targets: &std::collections::BTreeSet<String>,
    ref_prefix: &str,
) -> Vec<Value> {
    indexed_targets
        .iter()
        .filter(|target| allowed_targets.contains(*target))
        .map(|target| json!({"$ref": format!("{ref_prefix}/{target}")}))
        .collect::<Vec<_>>()
}

fn collect_expr_indexed_targets(
    expr: &arco_kdl::algebra::Expr,
    out: &mut std::collections::BTreeSet<String>,
) {
    match expr {
        arco_kdl::algebra::Expr::Indexed { target, indices } => {
            out.insert(target.clone());
            for index in indices {
                collect_expr_indexed_targets(index, out);
            }
        }
        arco_kdl::algebra::Expr::Unary { expr, .. } => collect_expr_indexed_targets(expr, out),
        arco_kdl::algebra::Expr::Binary { left, right, .. }
        | arco_kdl::algebra::Expr::Comparison { left, right, .. } => {
            collect_expr_indexed_targets(left, out);
            collect_expr_indexed_targets(right, out);
        }
        arco_kdl::algebra::Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_expr_indexed_targets(arg, out);
            }
        }
        arco_kdl::algebra::Expr::Reduction(reduction) => {
            collect_expr_indexed_targets(&reduction.body, out);
            for filter in &reduction.filters {
                collect_expr_indexed_targets(filter, out);
            }
        }
        arco_kdl::algebra::Expr::Number(_)
        | arco_kdl::algebra::Expr::String(_)
        | arco_kdl::algebra::Expr::Boolean(_)
        | arco_kdl::algebra::Expr::Identifier(_) => {}
    }
}

fn render_reduction_scope(reduction: &arco_kdl::algebra::ReductionExpr) -> Vec<Value> {
    reduction
        .bindings
        .iter()
        .map(|binding| {
            let symbol = match &binding.pattern {
                arco_kdl::algebra::BindingPattern::Name(name) => name.clone(),
                arco_kdl::algebra::BindingPattern::Tuple(parts) => format!("({})", parts.join(",")),
            };
            json!({
                "symbol": symbol,
                "$ref": format!("#/sets/{}", binding.domain),
            })
        })
        .collect::<Vec<_>>()
}

fn format_inspect_reports(
    program: &arco_kdl::semantic::SemanticProgram,
    name: Option<&str>,
) -> Result<Value, DriverError> {
    let reports = &program.active_reports;

    if let Some(target_name) = name {
        // Detail mode: look for a specific report
        let target = reports.iter().find(|r| r.name == target_name);
        if let Some(report) = target {
            Ok(render_named_card(
                "report",
                vec![
                    ("name", Value::String(target_name.to_string())),
                    ("formula", Value::String(report.formula_text.clone())),
                ],
            ))
        } else {
            let available: Vec<_> = reports.iter().map(|r| r.name.as_str()).collect();
            Err(DriverError::InspectLookup {
                message: format!(
                    "report '{}' not found. Available reports: {}",
                    target_name,
                    available.join(", ")
                ),
            })
        }
    } else {
        let items = reports
            .iter()
            .map(|report| {
                render_named_card(
                    "report",
                    vec![
                        ("name", Value::String(report.name.clone())),
                        ("formula", Value::String(report.formula_text.clone())),
                    ],
                )
            })
            .collect::<Vec<_>>();
        Ok(json!({
            "kind": "reports",
            "items": items,
        }))
    }
}

fn format_inspect_chronology(program: &arco_kdl::semantic::SemanticProgram) -> Value {
    let chronology = &program.chronology;
    let mut fields = Vec::new();
    if let Some(initial_boundary) = &chronology.initial_boundary {
        fields.push(("initial_boundary", Value::String(initial_boundary.clone())));
    }
    if let Some(terminal_boundary) = &chronology.terminal_boundary {
        fields.push((
            "terminal_boundary",
            Value::String(terminal_boundary.clone()),
        ));
    }
    if let Some(initial_commitment_boundary) = &chronology.initial_commitment_boundary {
        fields.push((
            "initial_commitment_boundary",
            Value::String(initial_commitment_boundary.clone()),
        ));
    }

    render_named_card("chronology", fields)
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

#[cfg(test)]
mod tests {
    use super::{
        collect_constraint_indexed_targets, expr_additive_terms, extract_expr_variable_refs,
        extract_indexed_refs, format_validation_summary, render_variable_domains,
        render_variable_math_notation, render_variable_value_domain,
    };
    use arco_kdl::semantic::{FamilySignature, ResolvedSet};
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::path::Path;

    #[test]
    fn format_validation_summary_includes_model_counts() {
        let summary = format_validation_summary(Path::new("example.kdl"), "ScenarioA", 4, 9, 7);

        assert_eq!(summary["kind"], json!("summary"));
        assert_eq!(summary["entrypoint"], json!("example.kdl"));
        assert_eq!(summary["scenario"], json!("ScenarioA"));
        assert_eq!(summary["counts"]["sets"], json!(4));
        assert_eq!(summary["counts"]["variables"], json!(9));
        assert_eq!(summary["counts"]["constraints"], json!(7));
        assert!(summary["counts"].get("assets").is_none());
    }

    #[test]
    fn render_variable_domains_are_structured() {
        let mut family = FamilySignature::new("dispatch", ["g", "n", "t"]);
        family.index_domains = BTreeMap::from([
            ("g".to_string(), "generators".to_string()),
            ("n".to_string(), "nodes".to_string()),
        ]);

        let set_registry = BTreeMap::from([
            (
                "generators".to_string(),
                ResolvedSet {
                    values: vec!["G1".to_string(), "G2".to_string()],
                },
            ),
            (
                "nodes".to_string(),
                ResolvedSet {
                    values: vec!["N1".to_string()],
                },
            ),
        ]);

        let domains = render_variable_domains(&family, &set_registry);

        assert_eq!(
            domains,
            vec![
                json!({"index": "g", "name": "generators", "cardinality": 2}),
                json!({"index": "n", "name": "nodes", "cardinality": 1}),
                json!({"index": "t", "name": "t", "cardinality": 0}),
            ]
        );
    }

    #[test]
    fn render_variable_math_notation_uses_index_domains() {
        let mut family = FamilySignature::new("dispatch", ["g", "n", "t"]);
        family.index_domains = BTreeMap::from([
            ("g".to_string(), "generators".to_string()),
            ("n".to_string(), "nodes".to_string()),
            ("t".to_string(), "time".to_string()),
        ]);

        let pretty = render_variable_math_notation(&family);

        assert_eq!(pretty, "dispatch[g, n, t]");
    }

    #[test]
    fn render_variable_value_domain_shows_default_bounds() {
        let family = FamilySignature::new("new_capacity", ["g", "n"]);

        let value_domain = render_variable_value_domain(&family, None);

        assert_eq!(value_domain["kind"], json!("continuous"));
        assert_eq!(value_domain["lower"], json!("-inf"));
        assert_eq!(value_domain["upper"], json!(null));
    }

    #[test]
    fn expr_additive_terms_splits_ast_sum_terms() {
        let expression =
            arco_kdl::algebra::parse_value_formula("a + b - c").expect("parse expression");

        let terms = expr_additive_terms(&expression);

        assert_eq!(terms, vec!["a", "b", "-(c)"]);
    }

    #[test]
    fn expr_additive_terms_distributes_parenthesized_multiplication() {
        let expression =
            arco_kdl::algebra::parse_value_formula("(a + b + c) * x[i]").expect("parse expression");

        let terms = expr_additive_terms(&expression);

        assert_eq!(terms, vec!["a * x[i]", "b * x[i]", "c * x[i]"]);
    }

    #[test]
    fn extract_expr_variable_refs_filters_to_variable_targets() {
        let expression = arco_kdl::algebra::parse_value_formula("dispatch[g,n,t] + MWLoad[n]")
            .expect("parse expression");
        let variable_targets = std::collections::BTreeSet::from(["dispatch".to_string()]);

        let refs = extract_expr_variable_refs(&expression, &variable_targets);

        assert_eq!(refs, vec![json!({"$ref": "#/variables/dispatch"})]);
    }

    #[test]
    fn collect_constraint_indexed_targets_collects_both_sides_and_condition() {
        let parsed = arco_kdl::algebra::parse_constraint_formula("dispatch[g,n,t] <= MWLoad[n]")
            .expect("parse constraint");
        let filter =
            arco_kdl::algebra::parse_value_formula("pair_exists[g,n]").expect("parse filter");
        let constraint = arco_kdl::semantic::ResolvedConstraint {
            name: "c".to_string(),
            source_kind: "model".to_string(),
            source_name: "m".to_string(),
            expression_text: "dispatch[g,n,t] <= MWLoad[n]".to_string(),
            expression: parsed,
            generation_bindings: Vec::new(),
            generation_filter_text: Some("pair_exists[g,n]".to_string()),
            generation_filter: Some(filter),
        };

        let targets = collect_constraint_indexed_targets(&constraint);

        assert!(targets.contains("dispatch"));
        assert!(targets.contains("MWLoad"));
        assert!(targets.contains("pair_exists"));
    }

    #[test]
    fn extract_indexed_refs_filters_and_formats_refs() {
        let indexed =
            std::collections::BTreeSet::from(["dispatch".to_string(), "MWLoad".to_string()]);
        let allowed = std::collections::BTreeSet::from(["dispatch".to_string()]);

        let refs = extract_indexed_refs(&indexed, &allowed, "#/variables");

        assert_eq!(refs, vec![json!({"$ref": "#/variables/dispatch"})]);
    }
}
