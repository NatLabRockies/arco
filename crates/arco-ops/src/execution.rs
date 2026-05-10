use crate::compile::compile::CompiledProblem;
use crate::compile::compile::{
    ConstraintSense, LinearReport, LinearTerm, TargetObjectiveSense as ObjectiveSense, VariableKind,
};
use crate::solve::{
    ModelViewSolveResult, SolverError as ArcoSolverError, SolverStatus as ArcoSolverStatus,
};
use arco_model::{
    Bounds, Constraint, Model, ModelError as ArcoModelError, Objective, PrettyPrintOptions, Sense,
    Variable,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct DualReportResult {
    pub constraint_family: String,
    pub values: Vec<DualReportValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DualReportValue {
    pub instance_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdapterSolveOutput {
    pub status: SolveStatus,
    pub objective_value: ScalarArtifactValue,
    pub report_values: Vec<ScalarArtifactValue>,
    pub variable_values: Vec<VariableArtifactValue>,
    pub dual_report_values: Vec<DualReportResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    Optimal,
    Infeasible,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarArtifactValue {
    pub compiled_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableArtifactValue {
    pub compiled_name: String,
    pub representative_value: f64,
    pub values: Vec<VariableInstanceArtifactValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableInstanceArtifactValue {
    pub compiled_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub backend: &'static str,
    pub status: SolveStatus,
    pub objective_sense: ObjectiveSense,
    pub objective: MappedScalarResult,
    pub reports: Vec<ReportResult>,
    pub variables: Vec<MappedVariableResult>,
    pub dual_reports: Vec<DualReportResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportResult {
    pub name: String,
    pub index: Vec<String>,
    pub values: Vec<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedScalarResult {
    pub dsl_name: String,
    pub compiled_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedVariableResult {
    pub dsl_name: String,
    pub compiled_name: String,
    pub representative_value: f64,
    pub values: Vec<MappedVariableValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedVariableValue {
    pub dsl_name: String,
    pub compiled_name: String,
    pub value: f64,
}

pub trait OptimizationAdapter {
    fn backend_name(&self) -> &'static str;

    fn solve(
        &self,
        problem: &CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError>;
}

#[derive(Debug, Default)]
pub struct MockArcoAdapter;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct RustArcoAdapter {
    pub(crate) log_to_console: bool,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ScipArcoAdapter {
    pub(crate) log_to_console: bool,
    pub(crate) executable: Option<String>,
    pub(crate) arguments: Vec<String>,
    pub(crate) environment: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("adapter backend `{backend}` failed to add variable `{compiled_name}`: {source}")]
    AddVariable {
        backend: String,
        compiled_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to name variable `{compiled_name}`: {source}")]
    NameVariable {
        backend: String,
        compiled_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to add constraint `{compiled_name}`: {source}")]
    AddConstraint {
        backend: String,
        compiled_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to name constraint `{compiled_name}`: {source}")]
    NameConstraint {
        backend: String,
        compiled_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` references unknown compiled variable `{compiled_name}`")]
    UnknownCompiledVariable {
        backend: String,
        compiled_name: String,
    },
    #[error(
        "adapter backend `{backend}` failed to set coefficient for `{constraint_name}`: {source}"
    )]
    SetCoefficient {
        backend: String,
        constraint_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to set objective `{compiled_name}`: {source}")]
    SetObjective {
        backend: String,
        compiled_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to name objective `{compiled_name}`: {source}")]
    NameObjective {
        backend: String,
        compiled_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to initialize solver: {source}")]
    SolverInitialization {
        backend: String,
        #[source]
        source: ArcoSolverError,
    },
    #[error("adapter backend `{backend}` failed to solve model: {source}")]
    Solve {
        backend: String,
        #[source]
        source: ArcoSolverError,
    },
    #[error(
        "adapter backend `{backend}` produced no usable primal solution because status was `{status}`"
    )]
    NoFeasibleSolution { backend: String, status: String },
    #[error("adapter backend `{backend}` did not return objective `{compiled_name}`")]
    MissingObjectiveValue {
        backend: String,
        compiled_name: String,
    },
    #[error("adapter backend `{backend}` did not return report `{compiled_name}`")]
    MissingReportValue {
        backend: String,
        compiled_name: String,
    },
    #[error("adapter backend `{backend}` did not return variable `{compiled_name}`")]
    MissingVariableValue {
        backend: String,
        compiled_name: String,
    },
    #[error("adapter backend `{backend}` failed during external solver I/O: {source}")]
    ExternalSolverIo {
        backend: String,
        #[source]
        source: std::io::Error,
    },
    #[error("adapter backend `{backend}` external solver invocation failed: {message}")]
    ExternalSolverProcess { backend: String, message: String },
    #[error("adapter backend `{backend}` produced an invalid solution file: {message}")]
    ExternalSolverParse { backend: String, message: String },
    #[error("adapter backend `{backend}` only supports linearized models")]
    UnsupportedNonlinearBackend { backend: String },
    #[error("adapter backend `{backend}` failed to evaluate nonlinear expression: {message}")]
    NonlinearEvaluation { backend: String, message: String },
    #[error("{message}")]
    BackendNotAvailable { message: String },
}

impl MockArcoAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl RustArcoAdapter {
    pub fn new() -> Self {
        Self {
            log_to_console: false,
        }
    }

    pub fn with_console_log(log_to_console: bool) -> Self {
        Self { log_to_console }
    }
}

pub fn builtin_adapter_for_selection(
    selection: &crate::solve::ResolvedSelection,
    log_to_console: bool,
    profile: Option<&crate::solve::SolverProfile>,
) -> Result<Box<dyn OptimizationAdapter>, String> {
    crate::execution_backends::adapter_for_selection(selection, log_to_console, profile)
}

impl ScipArcoAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_console_log(log_to_console: bool) -> Self {
        Self {
            log_to_console,
            ..Self::default()
        }
    }

    pub fn with_external_process_profile(
        log_to_console: bool,
        executable: Option<String>,
        arguments: Vec<String>,
        environment: std::collections::BTreeMap<String, String>,
    ) -> Self {
        Self {
            log_to_console,
            executable,
            arguments,
            environment,
        }
    }
}

impl OptimizationAdapter for MockArcoAdapter {
    fn backend_name(&self) -> &'static str {
        "mock-arco"
    }

    fn solve(
        &self,
        problem: &CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        Ok(AdapterSolveOutput {
            status: SolveStatus::Optimal,
            objective_value: ScalarArtifactValue {
                compiled_name: problem.objective.name.clone(),
                value: 0.0,
            },
            report_values: problem
                .reports
                .iter()
                .map(|report| ScalarArtifactValue {
                    compiled_name: report.name.clone(),
                    value: 0.0,
                })
                .collect(),
            variable_values: problem
                .variables
                .iter()
                .map(|variable| VariableArtifactValue {
                    compiled_name: variable.family.clone(),
                    representative_value: 0.0,
                    values: if include_variable_values {
                        problem
                            .algebra
                            .variable_instances
                            .iter()
                            .filter(|instance| instance.family == variable.family)
                            .map(|instance| VariableInstanceArtifactValue {
                                compiled_name: instance.name.clone(),
                                value: 0.0,
                            })
                            .collect()
                    } else {
                        Vec::new()
                    },
                })
                .collect(),
            dual_report_values: problem
                .dual_reports
                .iter()
                .map(|dr| DualReportResult {
                    constraint_family: dr.constraint_name.clone(),
                    values: Vec::new(),
                })
                .collect(),
        })
    }
}

pub fn execute_problem(
    problem: &CompiledProblem,
    adapter: &dyn OptimizationAdapter,
) -> Result<ExecutionResult, ExecutionError> {
    execute_problem_with_options(problem, adapter, true)
}

pub fn execute_problem_with_options(
    problem: &CompiledProblem,
    adapter: &dyn OptimizationAdapter,
    include_variable_values: bool,
) -> Result<ExecutionResult, ExecutionError> {
    let solve_output = adapter.solve(problem, include_variable_values)?;
    let backend = adapter.backend_name();

    let objective = if solve_output.objective_value.compiled_name == problem.objective.name {
        MappedScalarResult {
            dsl_name: problem.objective.name.clone(),
            compiled_name: solve_output.objective_value.compiled_name.clone(),
            value: solve_output.objective_value.value,
        }
    } else {
        return Err(ExecutionError::MissingObjectiveValue {
            backend: backend.to_string(),
            compiled_name: problem.objective.name.clone(),
        });
    };

    let report_values = solve_output
        .report_values
        .iter()
        .map(|report| (report.compiled_name.clone(), report.value))
        .collect::<BTreeMap<_, _>>();
    let variable_values = solve_output
        .variable_values
        .iter()
        .map(|variable| (variable.compiled_name.clone(), variable))
        .collect::<BTreeMap<_, _>>();

    // Build unified reports: expression reports (scalar) and variable reports
    let mut reports = Vec::new();

    for report in &problem.reports {
        let value = report_values.get(&report.name).copied().ok_or_else(|| {
            ExecutionError::MissingReportValue {
                backend: backend.to_string(),
                compiled_name: report.name.clone(),
            }
        })?;
        let mut record = BTreeMap::new();
        record.insert("value".to_string(), serde_json::Value::from(value));
        reports.push(ReportResult {
            name: report.name.clone(),
            index: Vec::new(),
            values: vec![record],
        });
    }

    for vr in &problem.variable_reports {
        if let Some(family) = variable_values.get(&vr.compiled_family) {
            let values = family
                .values
                .iter()
                .filter_map(|v| {
                    let (raw, typed) = extract_index_parts(&vr.control_name, &v.compiled_name);
                    if let Some(ref filter) = vr.filter {
                        let bindings: BTreeMap<&str, &str> = vr
                            .indices
                            .iter()
                            .zip(raw.iter())
                            .map(|(k, v)| (k.as_str(), v.as_str()))
                            .collect();
                        let filter_result = try_eval_filter(filter, &bindings);
                        match filter_result {
                            Some(false) => return None,
                            None => {
                                tracing::warn!(
                                    "Unsupported filter expression for {}: {}. Skipping row.",
                                    vr.control_name,
                                    filter
                                );
                                return None;
                            }
                            Some(true) => {} // Continue processing
                        }
                    }
                    let mut record = BTreeMap::new();
                    for (idx_name, idx_val) in vr.indices.iter().zip(typed) {
                        record.insert(idx_name.clone(), idx_val);
                    }
                    record.insert("value".to_string(), serde_json::Value::from(v.value));
                    Some(record)
                })
                .collect();
            reports.push(ReportResult {
                name: vr.control_name.clone(),
                index: vr.indices.clone(),
                values,
            });
        }
    }

    let variables = problem
        .variables
        .iter()
        .map(|variable| {
            let solved_variable =
                variable_values
                    .get(&variable.family)
                    .copied()
                    .ok_or_else(|| ExecutionError::MissingVariableValue {
                        backend: backend.to_string(),
                        compiled_name: variable.family.clone(),
                    })?;
            Ok(MappedVariableResult {
                dsl_name: variable.family.clone(),
                compiled_name: variable.family.clone(),
                representative_value: solved_variable.representative_value,
                values: solved_variable
                    .values
                    .iter()
                    .map(|value| MappedVariableValue {
                        dsl_name: value.compiled_name.clone(),
                        compiled_name: value.compiled_name.clone(),
                        value: value.value,
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ExecutionResult {
        backend,
        status: solve_output.status,
        objective_sense: problem.objective.sense,
        objective,
        reports,
        variables,
        dual_reports: solve_output.dual_report_values,
    })
}

pub fn render_problem_model(problem: &CompiledProblem) -> Result<String, ExecutionError> {
    let built = build_model(problem, "arco-rust-highs")?;
    Ok(built.model.format_ascii(PrettyPrintOptions::full()))
}

pub struct BuiltModel {
    pub model: Model,
    pub variable_indices: BTreeMap<String, usize>,
    pub constraint_indices: BTreeMap<String, usize>,
}

pub fn adapter_output_from_model_view_solution(
    problem: &CompiledProblem,
    include_variable_values: bool,
    backend: &str,
    variable_indices: BTreeMap<String, usize>,
    constraint_indices: BTreeMap<String, usize>,
    solution: ModelViewSolveResult,
) -> Result<AdapterSolveOutput, ExecutionError> {
    if !solution.status.is_feasible() {
        return Err(ExecutionError::NoFeasibleSolution {
            backend: backend.to_string(),
            status: solution.status.to_string(),
        });
    }

    let objective_value = problem.algebra.objective.constant + solution.objective_value;
    let report_values = problem
        .algebra
        .reports
        .iter()
        .map(|report| {
            Ok(ScalarArtifactValue {
                compiled_name: report.name.clone(),
                value: evaluate_linear_report(
                    backend,
                    report,
                    &variable_indices,
                    &solution.primal_values,
                )?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let variable_values = problem
        .variables
        .iter()
        .map(|variable| {
            let representative_value = problem
                .algebra
                .variable_instances
                .iter()
                .find(|instance| instance.family == variable.family)
                .map(|instance| {
                    lookup_primal_value(
                        backend,
                        &instance.name,
                        &variable_indices,
                        &solution.primal_values,
                    )
                })
                .transpose()?
                .unwrap_or(0.0);
            let values = if include_variable_values {
                problem
                    .algebra
                    .variable_instances
                    .iter()
                    .filter(|instance| instance.family == variable.family)
                    .map(|instance| {
                        Ok(VariableInstanceArtifactValue {
                            compiled_name: instance.name.clone(),
                            value: lookup_primal_value(
                                backend,
                                &instance.name,
                                &variable_indices,
                                &solution.primal_values,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            };

            Ok(VariableArtifactValue {
                compiled_name: variable.family.clone(),
                representative_value,
                values,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let dual_report_values =
        extract_dual_report_values(problem, &constraint_indices, &solution.constraint_duals);

    Ok(AdapterSolveOutput {
        status: map_solver_status(solution.status),
        objective_value: ScalarArtifactValue {
            compiled_name: problem.objective.name.clone(),
            value: objective_value,
        },
        report_values,
        variable_values,
        dual_report_values,
    })
}

pub fn build_model(problem: &CompiledProblem, backend: &str) -> Result<BuiltModel, ExecutionError> {
    let mut model = Model::with_capacities(
        problem.algebra.variable_instances.len(),
        problem.algebra.constraints.len(),
    );
    let mut variable_ids = BTreeMap::new();
    let mut constraint_ids = BTreeMap::new();

    for variable in &problem.algebra.variable_instances {
        let upper = variable.upper.unwrap_or(f64::INFINITY);
        let variable_def = match variable.kind {
            VariableKind::Continuous => Variable::continuous(Bounds::new(variable.lower, upper)),
            VariableKind::Integer => Variable::integer(Bounds::new(variable.lower, upper)),
            VariableKind::Binary => Variable::binary(),
        };
        let variable_id =
            model
                .add_variable(variable_def)
                .map_err(|source| ExecutionError::AddVariable {
                    backend: backend.to_string(),
                    compiled_name: variable.name.clone(),
                    source,
                })?;
        model
            .set_variable_name(variable_id, variable.name.clone())
            .map_err(|source| ExecutionError::NameVariable {
                backend: backend.to_string(),
                compiled_name: variable.name.clone(),
                source,
            })?;
        variable_ids.insert(variable.name.clone(), variable_id);
    }

    for constraint in &problem.algebra.constraints {
        let constraint_id = model
            .add_constraint(Constraint {
                bounds: to_bounds(constraint.sense, constraint.rhs),
            })
            .map_err(|source| ExecutionError::AddConstraint {
                backend: backend.to_string(),
                compiled_name: constraint.name.clone(),
                source,
            })?;
        model
            .set_constraint_name(constraint_id, constraint.name.clone())
            .map_err(|source| ExecutionError::NameConstraint {
                backend: backend.to_string(),
                compiled_name: constraint.name.clone(),
                source,
            })?;
        constraint_ids.insert(constraint.name.clone(), constraint_id.inner() as usize);

        for term in &constraint.terms {
            let variable_id = variable_ids
                .get(&term.variable_name)
                .copied()
                .ok_or_else(|| ExecutionError::UnknownCompiledVariable {
                    backend: backend.to_string(),
                    compiled_name: term.variable_name.clone(),
                })?;
            model
                .set_coefficient(variable_id, constraint_id, term.coefficient)
                .map_err(|source| ExecutionError::SetCoefficient {
                    backend: backend.to_string(),
                    constraint_name: constraint.name.clone(),
                    source,
                })?;
        }
    }

    let objective_terms = problem
        .algebra
        .objective
        .terms
        .iter()
        .map(|term| {
            let variable_id = variable_ids
                .get(&term.variable_name)
                .copied()
                .ok_or_else(|| ExecutionError::UnknownCompiledVariable {
                    backend: backend.to_string(),
                    compiled_name: term.variable_name.clone(),
                })?;
            Ok((variable_id, term.coefficient))
        })
        .collect::<Result<Vec<_>, _>>()?;

    model
        .set_objective(Objective {
            sense: Some(to_objective_sense(problem.algebra.objective.sense)),
            terms: objective_terms,
        })
        .map_err(|source| ExecutionError::SetObjective {
            backend: backend.to_string(),
            compiled_name: problem.algebra.objective.name.clone(),
            source,
        })?;
    model
        .set_objective_name(Some(problem.algebra.objective.name.clone()))
        .map_err(|source| ExecutionError::NameObjective {
            backend: backend.to_string(),
            compiled_name: problem.algebra.objective.name.clone(),
            source,
        })?;

    let variable_indices = variable_ids
        .iter()
        .map(|(name, id)| (name.clone(), id.inner() as usize))
        .collect();

    Ok(BuiltModel {
        model,
        variable_indices,
        constraint_indices: constraint_ids,
    })
}

pub(crate) fn evaluate_linear_report(
    backend: &str,
    report: &LinearReport,
    variable_indices: &BTreeMap<String, usize>,
    primal_values: &[f64],
) -> Result<f64, ExecutionError> {
    let terms_value =
        evaluate_linear_terms(backend, &report.terms, variable_indices, primal_values)?;
    Ok(report.constant + terms_value)
}

fn evaluate_linear_terms(
    backend: &str,
    terms: &[LinearTerm],
    variable_indices: &BTreeMap<String, usize>,
    primal_values: &[f64],
) -> Result<f64, ExecutionError> {
    terms.iter().try_fold(0.0, |accumulator, term| {
        let value = lookup_primal_value(
            backend,
            &term.variable_name,
            variable_indices,
            primal_values,
        )?;
        Ok(accumulator + (term.coefficient * value))
    })
}

pub(crate) fn lookup_primal_value(
    backend: &str,
    compiled_name: &str,
    variable_indices: &BTreeMap<String, usize>,
    primal_values: &[f64],
) -> Result<f64, ExecutionError> {
    let index = variable_indices
        .get(compiled_name)
        .copied()
        .ok_or_else(|| ExecutionError::UnknownCompiledVariable {
            backend: backend.to_string(),
            compiled_name: compiled_name.to_string(),
        })?;
    primal_values
        .get(index)
        .copied()
        .ok_or_else(|| ExecutionError::UnknownCompiledVariable {
            backend: backend.to_string(),
            compiled_name: compiled_name.to_string(),
        })
}

pub(crate) fn extract_dual_report_values(
    problem: &CompiledProblem,
    constraint_indices: &BTreeMap<String, usize>,
    constraint_duals: &[f64],
) -> Vec<DualReportResult> {
    problem
        .dual_reports
        .iter()
        .map(|dual_report| {
            let family = &dual_report.constraint_name;
            let prefix = format!("{family}[");
            let mut upper = prefix.clone();
            // Increment last char to form exclusive upper bound for range query.
            // '[' + 1 == '\\' in ASCII, so range "name[".."name\\" captures all "name[...]" keys.
            if let Some(last) = upper.pop() {
                upper.push((last as u8 + 1) as char);
            }
            let values = constraint_indices
                .range(prefix..upper)
                .map(|(name, &index)| {
                    let dual = constraint_duals.get(index).copied().unwrap_or(0.0);
                    DualReportValue {
                        instance_name: name.clone(),
                        value: dual,
                    }
                })
                .collect();
            DualReportResult {
                constraint_family: family.clone(),
                values,
            }
        })
        .collect()
}

fn to_bounds(sense: ConstraintSense, rhs: f64) -> Bounds {
    match sense {
        ConstraintSense::GreaterEqual => Bounds::new(rhs, f64::INFINITY),
        ConstraintSense::LessEqual => Bounds::new(f64::NEG_INFINITY, rhs),
        ConstraintSense::Equal => Bounds::new(rhs, rhs),
    }
}

fn to_objective_sense(sense: ObjectiveSense) -> Sense {
    match sense {
        ObjectiveSense::Minimize => Sense::Minimize,
        ObjectiveSense::Maximize => Sense::Maximize,
    }
}

pub(crate) fn map_solver_status(status: ArcoSolverStatus) -> SolveStatus {
    match status {
        ArcoSolverStatus::Optimal => SolveStatus::Optimal,
        ArcoSolverStatus::Infeasible => SolveStatus::Infeasible,
        ArcoSolverStatus::Unbounded
        | ArcoSolverStatus::TimeLimit
        | ArcoSolverStatus::IterationLimit
        | ArcoSolverStatus::Unknown => SolveStatus::Failed,
    }
}

/// Split a variable instance name into raw string parts and typed JSON values.
/// E.g. `("pc", "pc[1,Li-Ion]")` → `(["1", "Li-Ion"], [Number(1), String("Li-Ion")])`.
fn extract_index_parts(
    family_name: &str,
    instance_name: &str,
) -> (Vec<String>, Vec<serde_json::Value>) {
    let inner = instance_name
        .strip_prefix(family_name)
        .and_then(|s| s.strip_prefix('['))
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or("");
    if inner.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let parts: Vec<&str> = inner.split(',').collect();
    let strings = parts.iter().map(|s| (*s).to_string()).collect();
    let typed = parts
        .iter()
        .map(|part| {
            if let Ok(n) = part.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else if let Some(n) = part
                .parse::<f64>()
                .ok()
                .and_then(serde_json::Number::from_f64)
            {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::String((*part).to_string())
            }
        })
        .collect();
    (strings, typed)
}

/// Evaluate a simple filter expression against index bindings.
/// Only supports comparison expressions with identifier/string/number operands.
/// Returns `None` for unsupported expression types to allow caller to decide handling.
fn try_eval_filter(
    expr: &crate::kdl::algebra::Expr,
    bindings: &BTreeMap<&str, &str>,
) -> Option<bool> {
    use crate::kdl::algebra::{ComparisonOp, Expr};
    match expr {
        Expr::Comparison { op, left, right } => {
            let lhs = match left.as_ref() {
                Expr::Identifier(name) => bindings.get(name.as_str()).copied(),
                Expr::String(s) => Some(s.as_str()),
                Expr::Number(n) => Some(n.as_str()),
                _ => return None,
            };
            let rhs = match right.as_ref() {
                Expr::Identifier(name) => bindings.get(name.as_str()).copied(),
                Expr::String(s) => Some(s.as_str()),
                Expr::Number(n) => Some(n.as_str()),
                _ => return None,
            };
            match (lhs, rhs) {
                (Some(l), Some(r)) => Some(match op {
                    ComparisonOp::Equal | ComparisonOp::DoubleEqual => l == r,
                    ComparisonOp::NotEqual => l != r,
                    ComparisonOp::Less => l < r,
                    ComparisonOp::LessEqual => l <= r,
                    ComparisonOp::Greater => l > r,
                    ComparisonOp::GreaterEqual => l >= r,
                }),
                _ => None,
            }
        }
        // Unsupported filter expressions return None instead of silently failing closed
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::compile::compile::{
        AlgebraicProblem, LinearObjective, TargetObjectiveSense as ObjectiveSense,
        VariableInstance, VariableKind,
    };
    use crate::compile::compile::{CompiledObjective, CompiledProblem, CompiledVariable};
    use crate::execution::{MockArcoAdapter, execute_problem_with_options};

    #[test]
    #[allow(clippy::float_cmp)]
    fn compact_execution_can_skip_detailed_variable_values() {
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
                linearized: true,
                nonlinear: None,
            },
        };

        let execution = execute_problem_with_options(&problem, &MockArcoAdapter::new(), false)
            .expect("compact execution should succeed");

        assert_eq!(execution.status, crate::execution::SolveStatus::Optimal);
        assert_eq!(execution.objective.dsl_name, "obj");
        assert_eq!(execution.objective.value, 0.0);
        assert_eq!(execution.objective_sense, ObjectiveSense::Maximize);
        assert!(execution.reports.is_empty());

        assert_eq!(execution.variables.len(), 1);
        assert_eq!(execution.variables[0].dsl_name, "x[a,t]");
        assert_eq!(execution.variables[0].representative_value, 0.0);
        assert!(execution.variables[0].values.is_empty());
    }
}

// =============================================================================
// IPOPT integration (gated by `feature = "ipopt"`).
// =============================================================================

#[cfg(feature = "ipopt")]
use crate::compile::compile::NonlinearExpr;
#[cfg(feature = "ipopt")]
use arco_ipopt::Solver as IpoptSolver;
#[cfg(feature = "ipopt")]
use ipopt::{BasicProblem, ConstrainedProblem, Index, Ipopt, Number, SolveStatus as IpoptStatus};
#[cfg(feature = "ipopt")]
use std::cell::RefCell;
#[cfg(feature = "ipopt")]
use std::time::Instant;
#[cfg(feature = "ipopt")]
use tracing::info;

#[cfg(feature = "ipopt")]
#[derive(Debug, Default)]
pub struct IpoptArcoAdapter {
    log_to_console: bool,
}

#[cfg(feature = "ipopt")]
impl IpoptArcoAdapter {
    pub fn new() -> Self {
        Self {
            log_to_console: false,
        }
    }

    pub fn with_console_log(log_to_console: bool) -> Self {
        Self { log_to_console }
    }
}

#[cfg(feature = "ipopt")]
impl OptimizationAdapter for IpoptArcoAdapter {
    fn backend_name(&self) -> &'static str {
        "arco-rust-ipopt"
    }

    fn solve(
        &self,
        problem: &CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();

        if !problem.algebra.linearized {
            return solve_with_nonlinear_ipopt(
                problem,
                include_variable_values,
                &backend,
                self.log_to_console,
            );
        }

        info!("solving with {}", backend);
        info!("translating lowered algebra into solver model");
        let build_started = Instant::now();
        let BuiltModel {
            model,
            variable_indices,
            constraint_indices,
        } = build_model(problem, &backend)?;
        info!(
            "solver model translation completed in {:.2} ms",
            build_started.elapsed().as_secs_f64() * 1000.0
        );
        info!("initializing solver backend instance");
        let mut solver =
            IpoptSolver::new(&model).map_err(|source| ExecutionError::SolverInitialization {
                backend: backend.clone(),
                source,
            })?;
        solver.set_log_to_console(self.log_to_console);

        info!("starting solver backend run: {}", backend);
        let solver_started = Instant::now();
        let solution = solver.solve().map_err(|source| ExecutionError::Solve {
            backend: backend.clone(),
            source,
        })?;
        info!(
            "solver backend run completed in {:.2} ms: {}",
            solver_started.elapsed().as_secs_f64() * 1000.0,
            backend
        );
        info!("solve status: {:?}", solution.core_status());
        if !solution.is_feasible() {
            return Err(ExecutionError::NoFeasibleSolution {
                backend,
                status: format!("{:?}", solution.core_status()),
            });
        }

        let objective_value = problem.algebra.objective.constant + solution.objective_value();
        let report_values = problem
            .algebra
            .reports
            .iter()
            .map(|report| {
                Ok(ScalarArtifactValue {
                    compiled_name: report.name.clone(),
                    value: evaluate_linear_report(
                        &backend,
                        report,
                        &variable_indices,
                        solution.primal_values(),
                    )?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let variable_values = problem
            .variables
            .iter()
            .map(|variable| {
                let representative_value = problem
                    .algebra
                    .variable_instances
                    .iter()
                    .find(|instance| instance.family == variable.family)
                    .map(|instance| {
                        lookup_primal_value(
                            &backend,
                            &instance.name,
                            &variable_indices,
                            solution.primal_values(),
                        )
                    })
                    .transpose()?
                    .unwrap_or(0.0);
                let values = if include_variable_values {
                    problem
                        .algebra
                        .variable_instances
                        .iter()
                        .filter(|instance| instance.family == variable.family)
                        .map(|instance| {
                            Ok(VariableInstanceArtifactValue {
                                compiled_name: instance.name.clone(),
                                value: lookup_primal_value(
                                    &backend,
                                    &instance.name,
                                    &variable_indices,
                                    solution.primal_values(),
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    Vec::new()
                };

                Ok(VariableArtifactValue {
                    compiled_name: variable.family.clone(),
                    representative_value,
                    values,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let dual_report_values =
            extract_dual_report_values(problem, &constraint_indices, solution.constraint_duals());

        Ok(AdapterSolveOutput {
            status: map_solver_status(solution.core_status()),
            objective_value: ScalarArtifactValue {
                compiled_name: problem.objective.name.clone(),
                value: objective_value,
            },
            report_values,
            variable_values,
            dual_report_values,
        })
    }
}
#[cfg(feature = "ipopt")]
const IPOPT_INF: f64 = 1e19;

#[cfg(feature = "ipopt")]
fn clamp_bound(value: f64) -> f64 {
    value.clamp(-IPOPT_INF, IPOPT_INF)
}

#[cfg(feature = "ipopt")]
fn default_primal_value(lower: f64, upper: f64) -> f64 {
    if lower.is_finite() && upper.is_finite() {
        if (upper - lower).abs() <= f64::EPSILON {
            return lower;
        }

        return 0.5 * (lower + upper);
    }

    if lower <= 0.0 && 0.0 <= upper {
        0.0
    } else if lower > 0.0 && lower.is_finite() {
        lower + 1.0
    } else if upper < 0.0 && upper.is_finite() {
        upper - 1.0
    } else {
        0.0
    }
}

#[cfg(feature = "ipopt")]
fn eval_nonlinear_expr(
    expr: &NonlinearExpr,
    values: &[f64],
    var_positions: &BTreeMap<String, usize>,
) -> Result<f64, String> {
    match expr {
        NonlinearExpr::Constant(value) => Ok(*value),
        NonlinearExpr::Variable(name) => {
            let Some(position) = var_positions.get(name) else {
                return Err(format!("unknown variable `{name}`"));
            };
            values
                .get(*position)
                .copied()
                .ok_or_else(|| format!("variable index out of bounds for `{name}`"))
        }
        NonlinearExpr::Unary { op, expr } => {
            let value = eval_nonlinear_expr(expr, values, var_positions)?;
            match op {
                arco_kdl::algebra::UnaryOp::Negate => Ok(-value),
            }
        }
        NonlinearExpr::Binary { op, left, right } => {
            let left = eval_nonlinear_expr(left, values, var_positions)?;
            let right = eval_nonlinear_expr(right, values, var_positions)?;
            match op {
                arco_kdl::algebra::BinaryOp::Add => Ok(left + right),
                arco_kdl::algebra::BinaryOp::Subtract => Ok(left - right),
                arco_kdl::algebra::BinaryOp::Multiply => Ok(left * right),
                arco_kdl::algebra::BinaryOp::Divide => Ok(left / right),
            }
        }
        NonlinearExpr::FunctionCall { name, args } => {
            let evaluated = args
                .iter()
                .map(|arg| eval_nonlinear_expr(arg, values, var_positions))
                .collect::<Result<Vec<_>, _>>()?;
            match (name.as_str(), evaluated.len()) {
                ("sqrt", 1) => Ok(evaluated[0].sqrt()),
                ("abs", 1) => Ok(evaluated[0].abs()),
                ("exp", 1) => Ok(evaluated[0].exp()),
                ("ln", 1) => Ok(evaluated[0].ln()),
                ("sin", 1) => Ok(evaluated[0].sin()),
                ("cos", 1) => Ok(evaluated[0].cos()),
                ("atan", 1) => Ok(evaluated[0].atan()),
                ("pow", 2) => Ok(evaluated[0].powf(evaluated[1])),
                _ => Err(format!(
                    "unsupported function `{}` with {} argument(s)",
                    name,
                    evaluated.len()
                )),
            }
        }
    }
}

#[cfg(feature = "ipopt")]
#[derive(Clone, Copy)]
enum TapeOp {
    Const(f64),
    Var(u32),
    Negate(u32),
    Add(u32, u32),
    Sub(u32, u32),
    Mul(u32, u32),
    Div(u32, u32),
    Sqrt(u32),
    Abs(u32),
    Exp(u32),
    Ln(u32),
    Sin(u32),
    Cos(u32),
    Atan(u32),
    Pow(u32, u32),
}

#[cfg(feature = "ipopt")]
struct Tape {
    ops: Vec<TapeOp>,
    /// local index -> global variable index in the IPOPT `x` vector.
    local_to_global: Vec<usize>,
    /// True if the tape contains any operation that produces a non-zero second
    /// derivative (multiplication of two non-constant operands, division by a
    /// non-constant denominator, transcendentals, etc.). Linear tapes can skip
    /// the Hessian pass entirely.
    is_nonlinear: bool,
}

#[cfg(feature = "ipopt")]
fn compile_tape(
    expr: &NonlinearExpr,
    var_positions: &BTreeMap<String, usize>,
) -> Result<Tape, String> {
    let mut ops: Vec<TapeOp> = Vec::new();
    let mut local_for_global: BTreeMap<usize, u32> = BTreeMap::new();
    let mut local_to_global: Vec<usize> = Vec::new();
    fn visit(
        expr: &NonlinearExpr,
        ops: &mut Vec<TapeOp>,
        local_for_global: &mut BTreeMap<usize, u32>,
        local_to_global: &mut Vec<usize>,
        var_positions: &BTreeMap<String, usize>,
    ) -> Result<u32, String> {
        let op = match expr {
            NonlinearExpr::Constant(value) => TapeOp::Const(*value),
            NonlinearExpr::Variable(name) => {
                let global = *var_positions
                    .get(name)
                    .ok_or_else(|| format!("unknown variable `{name}`"))?;
                let local = if let Some(&l) = local_for_global.get(&global) {
                    l
                } else {
                    let l = local_to_global.len() as u32;
                    local_for_global.insert(global, l);
                    local_to_global.push(global);
                    l
                };
                TapeOp::Var(local)
            }
            NonlinearExpr::Unary { op, expr } => {
                let a = visit(expr, ops, local_for_global, local_to_global, var_positions)?;
                match op {
                    arco_kdl::algebra::UnaryOp::Negate => TapeOp::Negate(a),
                }
            }
            NonlinearExpr::Binary { op, left, right } => {
                let a = visit(left, ops, local_for_global, local_to_global, var_positions)?;
                let b = visit(right, ops, local_for_global, local_to_global, var_positions)?;
                match op {
                    arco_kdl::algebra::BinaryOp::Add => TapeOp::Add(a, b),
                    arco_kdl::algebra::BinaryOp::Subtract => TapeOp::Sub(a, b),
                    arco_kdl::algebra::BinaryOp::Multiply => TapeOp::Mul(a, b),
                    arco_kdl::algebra::BinaryOp::Divide => TapeOp::Div(a, b),
                }
            }
            NonlinearExpr::FunctionCall { name, args } => {
                let arg_indices = args
                    .iter()
                    .map(|arg| visit(arg, ops, local_for_global, local_to_global, var_positions))
                    .collect::<Result<Vec<_>, _>>()?;
                match (name.as_str(), arg_indices.len()) {
                    ("sqrt", 1) => TapeOp::Sqrt(arg_indices[0]),
                    ("abs", 1) => TapeOp::Abs(arg_indices[0]),
                    ("exp", 1) => TapeOp::Exp(arg_indices[0]),
                    ("ln", 1) => TapeOp::Ln(arg_indices[0]),
                    ("sin", 1) => TapeOp::Sin(arg_indices[0]),
                    ("cos", 1) => TapeOp::Cos(arg_indices[0]),
                    ("atan", 1) => TapeOp::Atan(arg_indices[0]),
                    ("pow", 2) => TapeOp::Pow(arg_indices[0], arg_indices[1]),
                    _ => {
                        return Err(format!(
                            "unsupported function `{}` with {} argument(s)",
                            name,
                            arg_indices.len()
                        ));
                    }
                }
            }
        };
        ops.push(op);
        Ok((ops.len() - 1) as u32)
    }

    visit(
        expr,
        &mut ops,
        &mut local_for_global,
        &mut local_to_global,
        var_positions,
    )?;
    let is_nonlinear = ops.iter().any(|op| match op {
        TapeOp::Const(_)
        | TapeOp::Var(_)
        | TapeOp::Negate(_)
        | TapeOp::Add(_, _)
        | TapeOp::Sub(_, _)
        | TapeOp::Abs(_) => false,
        TapeOp::Mul(a, b) | TapeOp::Pow(a, b) => {
            !matches!(ops[*a as usize], TapeOp::Const(_))
                && !matches!(ops[*b as usize], TapeOp::Const(_))
        }
        TapeOp::Div(_, b) => !matches!(ops[*b as usize], TapeOp::Const(_)),
        TapeOp::Sqrt(_)
        | TapeOp::Exp(_)
        | TapeOp::Ln(_)
        | TapeOp::Sin(_)
        | TapeOp::Cos(_)
        | TapeOp::Atan(_) => true,
    });
    Ok(Tape {
        ops,
        local_to_global,
        is_nonlinear,
    })
}

/// Reverse-mode AD scratch buffers reused across IPOPT callbacks.
#[cfg(feature = "ipopt")]
#[derive(Default)]
struct TapeScratch {
    values: Vec<f64>,
    adj: Vec<f64>,
    var_grad: Vec<f64>,
    /// Forward tangent buffer for second-order AD (one column at a time).
    v_dot: Vec<f64>,
    /// Adjoint tangent buffer for second-order AD.
    adj_dot: Vec<f64>,
    /// Per-local-variable Hessian column accumulator for second-order AD.
    var_grad_dot: Vec<f64>,
    /// Dense local Hessian (row-major, `local_n * local_n`) for one tape.
    hess_local: Vec<f64>,
}

#[cfg(feature = "ipopt")]
fn tape_eval(tape: &Tape, x: &[f64], values: &mut Vec<f64>) -> f64 {
    values.clear();
    values.reserve(tape.ops.len());
    for op in &tape.ops {
        let v = match *op {
            TapeOp::Const(v) => v,
            TapeOp::Var(local) => x[tape.local_to_global[local as usize]],
            TapeOp::Negate(a) => -values[a as usize],
            TapeOp::Add(a, b) => values[a as usize] + values[b as usize],
            TapeOp::Sub(a, b) => values[a as usize] - values[b as usize],
            TapeOp::Mul(a, b) => values[a as usize] * values[b as usize],
            TapeOp::Div(a, b) => values[a as usize] / values[b as usize],
            TapeOp::Sqrt(a) => values[a as usize].sqrt(),
            TapeOp::Abs(a) => values[a as usize].abs(),
            TapeOp::Exp(a) => values[a as usize].exp(),
            TapeOp::Ln(a) => values[a as usize].ln(),
            TapeOp::Sin(a) => values[a as usize].sin(),
            TapeOp::Cos(a) => values[a as usize].cos(),
            TapeOp::Atan(a) => values[a as usize].atan(),
            TapeOp::Pow(a, b) => values[a as usize].powf(values[b as usize]),
        };
        values.push(v);
    }
    *values.last().expect("tape must produce a value")
}

/// Reverse pass: assumes `values` was populated by `tape_eval` for the same `x`.
/// Writes per-local-variable gradient into `var_grad` (zeroed and resized).
#[cfg(feature = "ipopt")]
fn tape_grad(tape: &Tape, values: &[f64], adj: &mut Vec<f64>, var_grad: &mut Vec<f64>) {
    adj.clear();
    adj.resize(tape.ops.len(), 0.0);
    if let Some(last) = adj.last_mut() {
        *last = 1.0;
    }
    var_grad.clear();
    var_grad.resize(tape.local_to_global.len(), 0.0);

    for i in (0..tape.ops.len()).rev() {
        let a_self = adj[i];
        if a_self == 0.0 {
            continue;
        }
        match tape.ops[i] {
            TapeOp::Const(_) => {}
            TapeOp::Var(local) => var_grad[local as usize] += a_self,
            TapeOp::Negate(a) => adj[a as usize] -= a_self,
            TapeOp::Add(a, b) => {
                adj[a as usize] += a_self;
                adj[b as usize] += a_self;
            }
            TapeOp::Sub(a, b) => {
                adj[a as usize] += a_self;
                adj[b as usize] -= a_self;
            }
            TapeOp::Mul(a, b) => {
                let av = values[a as usize];
                let bv = values[b as usize];
                adj[a as usize] += bv * a_self;
                adj[b as usize] += av * a_self;
            }
            TapeOp::Div(a, b) => {
                let av = values[a as usize];
                let bv = values[b as usize];
                adj[a as usize] += a_self / bv;
                adj[b as usize] -= av / (bv * bv) * a_self;
            }
            TapeOp::Sqrt(a) => {
                let out = values[i];
                adj[a as usize] += 0.5 / out * a_self;
            }
            TapeOp::Abs(a) => {
                let av = values[a as usize];
                let s = if av > 0.0 {
                    1.0
                } else if av < 0.0 {
                    -1.0
                } else {
                    0.0
                };
                adj[a as usize] += s * a_self;
            }
            TapeOp::Exp(a) => adj[a as usize] += values[i] * a_self,
            TapeOp::Ln(a) => adj[a as usize] += a_self / values[a as usize],
            TapeOp::Sin(a) => adj[a as usize] += values[a as usize].cos() * a_self,
            TapeOp::Cos(a) => adj[a as usize] -= values[a as usize].sin() * a_self,
            TapeOp::Atan(a) => {
                let av = values[a as usize];
                adj[a as usize] += a_self / (1.0 + av * av);
            }
            TapeOp::Pow(a, b) => {
                let av = values[a as usize];
                let out = values[i];
                if av != 0.0 {
                    adj[a as usize] += values[b as usize] * out / av * a_self;
                }
                if av > 0.0 {
                    adj[b as usize] += out * av.ln() * a_self;
                }
            }
        }
    }
}

/// Dense local Hessian for one tape via second-order reverse-mode AD
/// (one forward tangent + reverse tangent pass per local variable).
///
/// `values` and `adj` must have been populated by `tape_eval`/`tape_grad` for the same `x`.
/// Output `hess_local` is row-major, length `local_n * local_n`, symmetric.
#[cfg(feature = "ipopt")]
fn tape_hessian(
    tape: &Tape,
    values: &[f64],
    adj: &[f64],
    v_dot: &mut Vec<f64>,
    adj_dot: &mut Vec<f64>,
    var_grad_dot: &mut Vec<f64>,
    hess_local: &mut Vec<f64>,
) {
    let local_n = tape.local_to_global.len();
    let n_ops = tape.ops.len();
    hess_local.clear();
    hess_local.resize(local_n * local_n, 0.0);

    for k in 0..local_n {
        // Forward tangent pass (direction = e_k).
        v_dot.clear();
        v_dot.resize(n_ops, 0.0);
        for i in 0..n_ops {
            let dot = match tape.ops[i] {
                TapeOp::Const(_) => 0.0,
                TapeOp::Var(local) => {
                    if local as usize == k {
                        1.0
                    } else {
                        0.0
                    }
                }
                TapeOp::Negate(a) => -v_dot[a as usize],
                TapeOp::Add(a, b) => v_dot[a as usize] + v_dot[b as usize],
                TapeOp::Sub(a, b) => v_dot[a as usize] - v_dot[b as usize],
                TapeOp::Mul(a, b) => {
                    values[a as usize] * v_dot[b as usize] + values[b as usize] * v_dot[a as usize]
                }
                TapeOp::Div(a, b) => {
                    let bv = values[b as usize];
                    v_dot[a as usize] / bv - values[a as usize] * v_dot[b as usize] / (bv * bv)
                }
                TapeOp::Sqrt(a) => {
                    let out = values[i];
                    if out == 0.0 {
                        0.0
                    } else {
                        0.5 / out * v_dot[a as usize]
                    }
                }
                TapeOp::Abs(a) => {
                    let av = values[a as usize];
                    let s = if av > 0.0 {
                        1.0
                    } else if av < 0.0 {
                        -1.0
                    } else {
                        0.0
                    };
                    s * v_dot[a as usize]
                }
                TapeOp::Exp(a) => values[i] * v_dot[a as usize],
                TapeOp::Ln(a) => v_dot[a as usize] / values[a as usize],
                TapeOp::Sin(a) => values[a as usize].cos() * v_dot[a as usize],
                TapeOp::Cos(a) => -values[a as usize].sin() * v_dot[a as usize],
                TapeOp::Atan(a) => {
                    let av = values[a as usize];
                    v_dot[a as usize] / (1.0 + av * av)
                }
                TapeOp::Pow(a, b) => {
                    let av = values[a as usize];
                    let out = values[i];
                    let mut t = 0.0;
                    if av != 0.0 {
                        t += values[b as usize] * out / av * v_dot[a as usize];
                    }
                    if av > 0.0 {
                        t += out * av.ln() * v_dot[b as usize];
                    }
                    t
                }
            };
            v_dot[i] = dot;
        }

        // Reverse tangent pass: differentiate the gradient pass w.r.t. the
        // perturbation in direction e_k. Boundary condition: adj[last] = 1
        // is constant in `x`, so adj_dot[last] = 0.
        adj_dot.clear();
        adj_dot.resize(n_ops, 0.0);
        var_grad_dot.clear();
        var_grad_dot.resize(local_n, 0.0);

        for i in (0..n_ops).rev() {
            let a_self = adj[i];
            let ad_self = adj_dot[i];
            match tape.ops[i] {
                TapeOp::Const(_) => {}
                TapeOp::Var(local) => var_grad_dot[local as usize] += ad_self,
                TapeOp::Negate(a) => adj_dot[a as usize] -= ad_self,
                TapeOp::Add(a, b) => {
                    adj_dot[a as usize] += ad_self;
                    adj_dot[b as usize] += ad_self;
                }
                TapeOp::Sub(a, b) => {
                    adj_dot[a as usize] += ad_self;
                    adj_dot[b as usize] -= ad_self;
                }
                TapeOp::Mul(a, b) => {
                    let av = values[a as usize];
                    let bv = values[b as usize];
                    adj_dot[a as usize] += bv * ad_self + v_dot[b as usize] * a_self;
                    adj_dot[b as usize] += av * ad_self + v_dot[a as usize] * a_self;
                }
                TapeOp::Div(a, b) => {
                    let av = values[a as usize];
                    let bv = values[b as usize];
                    let bv2 = bv * bv;
                    adj_dot[a as usize] += ad_self / bv - v_dot[b as usize] / bv2 * a_self;
                    adj_dot[b as usize] += -av / bv2 * ad_self - v_dot[a as usize] / bv2 * a_self
                        + 2.0 * av * v_dot[b as usize] / (bv2 * bv) * a_self;
                }
                TapeOp::Sqrt(a) => {
                    let av = values[a as usize];
                    let out = values[i];
                    if out != 0.0 {
                        let coef = 0.5 / out;
                        let coef_dd = if av > 0.0 { -0.25 / (av * out) } else { 0.0 };
                        adj_dot[a as usize] +=
                            coef * ad_self + coef_dd * v_dot[a as usize] * a_self;
                    }
                }
                TapeOp::Abs(a) => {
                    let av = values[a as usize];
                    let s = if av > 0.0 {
                        1.0
                    } else if av < 0.0 {
                        -1.0
                    } else {
                        0.0
                    };
                    adj_dot[a as usize] += s * ad_self;
                }
                TapeOp::Exp(a) => {
                    let out = values[i];
                    adj_dot[a as usize] += out * ad_self + out * v_dot[a as usize] * a_self;
                }
                TapeOp::Ln(a) => {
                    let av = values[a as usize];
                    adj_dot[a as usize] += ad_self / av - v_dot[a as usize] / (av * av) * a_self;
                }
                TapeOp::Sin(a) => {
                    let av = values[a as usize];
                    adj_dot[a as usize] +=
                        av.cos() * ad_self - av.sin() * v_dot[a as usize] * a_self;
                }
                TapeOp::Cos(a) => {
                    let av = values[a as usize];
                    adj_dot[a as usize] +=
                        -av.sin() * ad_self - av.cos() * v_dot[a as usize] * a_self;
                }
                TapeOp::Atan(a) => {
                    let av = values[a as usize];
                    let denom = 1.0 + av * av;
                    adj_dot[a as usize] +=
                        ad_self / denom - 2.0 * av * v_dot[a as usize] / (denom * denom) * a_self;
                }
                TapeOp::Pow(a, b) => {
                    let av = values[a as usize];
                    let bv = values[b as usize];
                    let out = values[i];
                    if av != 0.0 {
                        let coef_a = bv * out / av;
                        let f_aa = bv * (bv - 1.0) * out / (av * av);
                        adj_dot[a as usize] += coef_a * ad_self + f_aa * v_dot[a as usize] * a_self;
                        if av > 0.0 {
                            let f_ab = (out / av) * (1.0 + bv * av.ln());
                            adj_dot[a as usize] += f_ab * v_dot[b as usize] * a_self;
                            adj_dot[b as usize] += f_ab * v_dot[a as usize] * a_self;
                        }
                    }
                    if av > 0.0 {
                        let lna = av.ln();
                        let coef_b = out * lna;
                        let f_bb = out * lna * lna;
                        adj_dot[b as usize] += coef_b * ad_self + f_bb * v_dot[b as usize] * a_self;
                    }
                }
            }
        }

        // Column k of the local Hessian.
        for j in 0..local_n {
            hess_local[j * local_n + k] = var_grad_dot[j];
        }
    }
}

#[cfg(feature = "ipopt")]
struct NonlinearIpoptProblem {
    x_lower: Vec<f64>,
    x_upper: Vec<f64>,
    x_init: Vec<f64>,
    g_lower: Vec<f64>,
    g_upper: Vec<f64>,
    objective_tape: Tape,
    objective_sign: f64,
    constraint_tapes: Vec<Tape>,
    jac_rows: Vec<Index>,
    jac_cols: Vec<Index>,
    /// For each constraint, the position in `jac_rows`/`jac_cols`/`vals`
    /// corresponding to each local variable in `constraint_tapes[i].local_to_global`.
    jac_value_positions: Vec<Vec<usize>>,
    hess_rows: Vec<Index>,
    hess_cols: Vec<Index>,
    /// `(local_j, local_k, value_position)` triples for the objective tape.
    /// Empty if the objective is linear.
    obj_hess_pairs: Vec<(u32, u32, u32)>,
    /// Same per constraint tape; empty entry means the constraint is linear.
    constraint_hess_pairs: Vec<Vec<(u32, u32, u32)>>,
    scratch: RefCell<TapeScratch>,
}

#[cfg(feature = "ipopt")]
impl BasicProblem for NonlinearIpoptProblem {
    fn num_variables(&self) -> usize {
        self.x_lower.len()
    }

    fn bounds(&self, x_l: &mut [Number], x_u: &mut [Number]) -> bool {
        x_l.copy_from_slice(&self.x_lower);
        x_u.copy_from_slice(&self.x_upper);
        true
    }

    fn initial_point(&self, x: &mut [Number]) -> bool {
        x.copy_from_slice(&self.x_init);
        true
    }

    fn objective(&self, x: &[Number], obj: &mut Number) -> bool {
        let mut scratch = self.scratch.borrow_mut();
        let value = tape_eval(&self.objective_tape, x, &mut scratch.values);
        *obj = self.objective_sign * value;
        true
    }

    fn objective_grad(&self, x: &[Number], grad_f: &mut [Number]) -> bool {
        let mut scratch = self.scratch.borrow_mut();
        let TapeScratch {
            values,
            adj,
            var_grad,
            ..
        } = &mut *scratch;
        tape_eval(&self.objective_tape, x, values);
        tape_grad(&self.objective_tape, values, adj, var_grad);
        grad_f.fill(0.0);
        for (local, &global) in self.objective_tape.local_to_global.iter().enumerate() {
            grad_f[global] = self.objective_sign * var_grad[local];
        }
        true
    }
}

#[cfg(feature = "ipopt")]
impl ConstrainedProblem for NonlinearIpoptProblem {
    fn num_constraints(&self) -> usize {
        self.constraint_tapes.len()
    }

    fn num_constraint_jacobian_non_zeros(&self) -> usize {
        self.jac_rows.len()
    }

    fn constraint(&self, x: &[Number], g: &mut [Number]) -> bool {
        let mut scratch = self.scratch.borrow_mut();
        for (i, tape) in self.constraint_tapes.iter().enumerate() {
            g[i] = tape_eval(tape, x, &mut scratch.values);
        }
        true
    }

    fn constraint_bounds(&self, g_l: &mut [Number], g_u: &mut [Number]) -> bool {
        g_l.copy_from_slice(&self.g_lower);
        g_u.copy_from_slice(&self.g_upper);
        true
    }

    fn constraint_jacobian_indices(&self, rows: &mut [Index], cols: &mut [Index]) -> bool {
        rows.copy_from_slice(&self.jac_rows);
        cols.copy_from_slice(&self.jac_cols);
        true
    }

    fn constraint_jacobian_values(&self, x: &[Number], vals: &mut [Number]) -> bool {
        let mut scratch = self.scratch.borrow_mut();
        let TapeScratch {
            values,
            adj,
            var_grad,
            ..
        } = &mut *scratch;
        for (i, tape) in self.constraint_tapes.iter().enumerate() {
            tape_eval(tape, x, values);
            tape_grad(tape, values, adj, var_grad);
            let positions = &self.jac_value_positions[i];
            for (local, &pos) in positions.iter().enumerate() {
                vals[pos] = var_grad[local];
            }
        }
        true
    }

    fn num_hessian_non_zeros(&self) -> usize {
        self.hess_rows.len()
    }

    fn hessian_indices(&self, rows: &mut [Index], cols: &mut [Index]) -> bool {
        rows.copy_from_slice(&self.hess_rows);
        cols.copy_from_slice(&self.hess_cols);
        true
    }

    fn hessian_values(
        &self,
        x: &[Number],
        obj_factor: Number,
        lambda: &[Number],
        vals: &mut [Number],
    ) -> bool {
        vals.fill(0.0);
        let mut scratch = self.scratch.borrow_mut();
        let TapeScratch {
            values,
            adj,
            var_grad,
            v_dot,
            adj_dot,
            var_grad_dot,
            hess_local,
        } = &mut *scratch;

        // Objective contribution.
        if !self.obj_hess_pairs.is_empty() {
            let weight = obj_factor * self.objective_sign;
            tape_eval(&self.objective_tape, x, values);
            tape_grad(&self.objective_tape, values, adj, var_grad);
            tape_hessian(
                &self.objective_tape,
                values,
                adj,
                v_dot,
                adj_dot,
                var_grad_dot,
                hess_local,
            );
            let local_n = self.objective_tape.local_to_global.len();
            for &(lj, lk, pos) in &self.obj_hess_pairs {
                vals[pos as usize] += weight * hess_local[lj as usize * local_n + lk as usize];
            }
        }

        // Constraint contributions.
        for (i, tape) in self.constraint_tapes.iter().enumerate() {
            let pairs = &self.constraint_hess_pairs[i];
            if pairs.is_empty() {
                continue;
            }
            let weight = lambda[i];
            if weight == 0.0 {
                continue;
            }
            tape_eval(tape, x, values);
            tape_grad(tape, values, adj, var_grad);
            tape_hessian(tape, values, adj, v_dot, adj_dot, var_grad_dot, hess_local);
            let local_n = tape.local_to_global.len();
            for &(lj, lk, pos) in pairs {
                vals[pos as usize] += weight * hess_local[lj as usize * local_n + lk as usize];
            }
        }

        true
    }
}

#[cfg(feature = "ipopt")]
#[allow(clippy::too_many_arguments)]
fn solve_with_nonlinear_ipopt(
    problem: &CompiledProblem,
    include_variable_values: bool,
    backend: &str,
    log_to_console: bool,
) -> Result<AdapterSolveOutput, ExecutionError> {
    let Some(nonlinear) = &problem.algebra.nonlinear else {
        return Err(ExecutionError::UnsupportedNonlinearBackend {
            backend: backend.to_string(),
        });
    };

    let mut variable_positions = BTreeMap::new();
    let mut x_lower = Vec::with_capacity(problem.algebra.variable_instances.len());
    let mut x_upper = Vec::with_capacity(problem.algebra.variable_instances.len());
    let mut x_init = Vec::with_capacity(problem.algebra.variable_instances.len());
    for (index, variable) in problem.algebra.variable_instances.iter().enumerate() {
        variable_positions.insert(variable.name.clone(), index);
        let upper = variable.upper.unwrap_or(f64::INFINITY);
        x_lower.push(clamp_bound(variable.lower));
        x_upper.push(clamp_bound(upper));
        x_init.push(default_primal_value(variable.lower, upper));
    }

    let mut g_lower = Vec::with_capacity(nonlinear.constraints.len());
    let mut g_upper = Vec::with_capacity(nonlinear.constraints.len());
    for row in &nonlinear.constraints {
        match row.sense {
            ConstraintSense::GreaterEqual => {
                g_lower.push(clamp_bound(row.rhs));
                g_upper.push(clamp_bound(f64::INFINITY));
            }
            ConstraintSense::LessEqual => {
                g_lower.push(clamp_bound(f64::NEG_INFINITY));
                g_upper.push(clamp_bound(row.rhs));
            }
            ConstraintSense::Equal => {
                let bound = clamp_bound(row.rhs);
                g_lower.push(bound);
                g_upper.push(bound);
            }
        }
    }

    let mut jac_rows: Vec<Index> = Vec::new();
    let mut jac_cols: Vec<Index> = Vec::new();
    let mut jac_value_positions: Vec<Vec<usize>> = Vec::with_capacity(nonlinear.constraints.len());
    let mut constraint_tapes: Vec<Tape> = Vec::with_capacity(nonlinear.constraints.len());
    for (row_index, row) in nonlinear.constraints.iter().enumerate() {
        let tape = compile_tape(&row.expression, &variable_positions).map_err(|message| {
            ExecutionError::NonlinearEvaluation {
                backend: backend.to_string(),
                message,
            }
        })?;
        let mut positions = Vec::with_capacity(tape.local_to_global.len());
        for &col_index in &tape.local_to_global {
            let value_position = jac_rows.len();
            jac_rows.push(row_index as Index);
            jac_cols.push(col_index as Index);
            positions.push(value_position);
        }
        jac_value_positions.push(positions);
        constraint_tapes.push(tape);
    }

    let objective_tape = compile_tape(&nonlinear.objective.expression, &variable_positions)
        .map_err(|message| ExecutionError::NonlinearEvaluation {
            backend: backend.to_string(),
            message,
        })?;

    let objective_sign = match nonlinear.objective.sense {
        arco_kdl::ObjectiveSense::Minimize => 1.0,
        arco_kdl::ObjectiveSense::Maximize => -1.0,
    };

    // Build Hessian sparsity (lower-triangular in global coordinates).
    // Each tape with `is_nonlinear == true` contributes upper-triangular pairs
    // of its local variables; duplicate (row, col) entries are summed by IPOPT.
    let mut hess_rows: Vec<Index> = Vec::new();
    let mut hess_cols: Vec<Index> = Vec::new();
    let mut obj_hess_pairs: Vec<(u32, u32, u32)> = Vec::new();
    if objective_tape.is_nonlinear {
        let local_n = objective_tape.local_to_global.len();
        for lj in 0..local_n {
            for lk in lj..local_n {
                let g_j = objective_tape.local_to_global[lj];
                let g_k = objective_tape.local_to_global[lk];
                let (row, col) = if g_j >= g_k { (g_j, g_k) } else { (g_k, g_j) };
                let pos = hess_rows.len() as u32;
                hess_rows.push(row as Index);
                hess_cols.push(col as Index);
                obj_hess_pairs.push((lj as u32, lk as u32, pos));
            }
        }
    }
    let mut constraint_hess_pairs: Vec<Vec<(u32, u32, u32)>> =
        Vec::with_capacity(constraint_tapes.len());
    for tape in &constraint_tapes {
        let mut pairs: Vec<(u32, u32, u32)> = Vec::new();
        if tape.is_nonlinear {
            let local_n = tape.local_to_global.len();
            for lj in 0..local_n {
                for lk in lj..local_n {
                    let g_j = tape.local_to_global[lj];
                    let g_k = tape.local_to_global[lk];
                    let (row, col) = if g_j >= g_k { (g_j, g_k) } else { (g_k, g_j) };
                    let pos = hess_rows.len() as u32;
                    hess_rows.push(row as Index);
                    hess_cols.push(col as Index);
                    pairs.push((lj as u32, lk as u32, pos));
                }
            }
        }
        constraint_hess_pairs.push(pairs);
    }

    let g_lower_diag = g_lower.clone();
    let g_upper_diag = g_upper.clone();

    let nlp_problem = NonlinearIpoptProblem {
        x_lower,
        x_upper,
        x_init,
        g_lower,
        g_upper,
        objective_tape,
        objective_sign,
        constraint_tapes,
        jac_rows,
        jac_cols,
        jac_value_positions,
        hess_rows,
        hess_cols,
        obj_hess_pairs,
        constraint_hess_pairs,
        scratch: RefCell::new(TapeScratch::default()),
    };

    let mut ipopt =
        Ipopt::new(nlp_problem).map_err(|source| ExecutionError::SolverInitialization {
            backend: backend.to_string(),
            source: ArcoSolverError::SolverSpecific(format!(
                "Failed to create IPOPT NLP: {source:?}"
            )),
        })?;

    ipopt.set_option("mu_strategy", "adaptive");
    ipopt.set_option("max_iter", 300);
    ipopt.set_option("acceptable_tol", 1e-4);
    ipopt.set_option("acceptable_iter", 8);
    ipopt.set_option("print_level", if log_to_console { 5 } else { 0 });

    let result = ipopt.solve();
    let status = result.status;
    if !ipopt_has_solution(status) {
        if log_to_console {
            log_top_nonlinear_violations(
                nonlinear,
                result.solver_data.solution.primal_variables,
                &variable_positions,
                &g_lower_diag,
                &g_upper_diag,
            );
        }
        return Err(ExecutionError::NoFeasibleSolution {
            backend: backend.to_string(),
            status: format!("{status:?}"),
        });
    }

    let primal_values = &result.solver_data.solution.primal_variables;
    let objective_value = objective_sign * result.objective_value;

    let report_values = nonlinear
        .reports
        .iter()
        .map(|report| {
            let value = eval_nonlinear_expr(&report.expression, primal_values, &variable_positions)
                .map_err(|message| ExecutionError::NonlinearEvaluation {
                    backend: backend.to_string(),
                    message,
                })?;
            Ok(ScalarArtifactValue {
                compiled_name: report.name.clone(),
                value,
            })
        })
        .collect::<Result<Vec<_>, ExecutionError>>()?;

    let variable_values = problem
        .variables
        .iter()
        .map(|variable| {
            let representative_value = problem
                .algebra
                .variable_instances
                .iter()
                .find(|instance| instance.family == variable.family)
                .map(|instance| {
                    lookup_primal_value(backend, &instance.name, &variable_positions, primal_values)
                })
                .transpose()?
                .unwrap_or(0.0);
            let values = if include_variable_values {
                problem
                    .algebra
                    .variable_instances
                    .iter()
                    .filter(|instance| instance.family == variable.family)
                    .map(|instance| {
                        Ok(VariableInstanceArtifactValue {
                            compiled_name: instance.name.clone(),
                            value: lookup_primal_value(
                                backend,
                                &instance.name,
                                &variable_positions,
                                primal_values,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            };

            Ok(VariableArtifactValue {
                compiled_name: variable.family.clone(),
                representative_value,
                values,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let constraint_indices = nonlinear
        .constraints
        .iter()
        .enumerate()
        .map(|(index, row)| (row.name.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let dual_report_values = extract_dual_report_values(
        problem,
        &constraint_indices,
        result.solver_data.solution.constraint_multipliers,
    );

    Ok(AdapterSolveOutput {
        status: map_ipopt_status(status),
        objective_value: ScalarArtifactValue {
            compiled_name: problem.objective.name.clone(),
            value: objective_value,
        },
        report_values,
        variable_values,
        dual_report_values,
    })
}

#[cfg(feature = "ipopt")]
fn ipopt_has_solution(status: IpoptStatus) -> bool {
    matches!(
        status,
        IpoptStatus::SolveSucceeded
            | IpoptStatus::SolvedToAcceptableLevel
            | IpoptStatus::MaximumIterationsExceeded
            | IpoptStatus::MaximumCpuTimeExceeded
            | IpoptStatus::FeasiblePointFound
    )
}

#[cfg(feature = "ipopt")]
fn constraint_violation(value: f64, lower: f64, upper: f64) -> f64 {
    if value < lower {
        lower - value
    } else if value > upper {
        value - upper
    } else {
        0.0
    }
}

#[cfg(feature = "ipopt")]
fn log_top_nonlinear_violations(
    nonlinear: &crate::compile::compile::NonlinearProblem,
    primal_values: &[f64],
    variable_positions: &BTreeMap<String, usize>,
    g_lower: &[f64],
    g_upper: &[f64],
) {
    if primal_values.is_empty() {
        eprintln!(
            "nonlinear IPOPT diagnostics: no primal values available for infeasibility analysis"
        );
        return;
    }

    let mut violations: Vec<(f64, String, f64, f64, f64)> = Vec::new();
    for (idx, constraint) in nonlinear.constraints.iter().enumerate() {
        let Ok(value) =
            eval_nonlinear_expr(&constraint.expression, primal_values, variable_positions)
        else {
            continue;
        };
        let lower = g_lower.get(idx).copied().unwrap_or(f64::NEG_INFINITY);
        let upper = g_upper.get(idx).copied().unwrap_or(f64::INFINITY);
        let violation = constraint_violation(value, lower, upper);
        if violation > 1e-8 {
            violations.push((violation, constraint.name.clone(), value, lower, upper));
        }
    }

    if violations.is_empty() {
        eprintln!(
            "nonlinear IPOPT diagnostics: no violated constraints identified at returned point"
        );
        return;
    }

    violations.sort_by(|a, b| b.0.total_cmp(&a.0));
    eprintln!(
        "nonlinear IPOPT diagnostics: top {} violated constraints:",
        violations.len().min(12)
    );
    for (violation, name, value, lower, upper) in violations.into_iter().take(12) {
        eprintln!(
            "  {name}: violation={violation:.6e}, value={value:.6e}, bounds=[{lower:.6e}, {upper:.6e}]"
        );
    }
}

#[cfg(feature = "ipopt")]
fn map_ipopt_status(status: IpoptStatus) -> SolveStatus {
    match status {
        IpoptStatus::SolveSucceeded | IpoptStatus::SolvedToAcceptableLevel => SolveStatus::Optimal,
        IpoptStatus::InfeasibleProblemDetected | IpoptStatus::RestorationFailed => {
            SolveStatus::Infeasible
        }
        _ => SolveStatus::Failed,
    }
}
