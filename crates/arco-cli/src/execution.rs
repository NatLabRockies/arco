use arco_core::{
    Bounds, Constraint, Model, ModelError as ArcoModelError, Objective, PrettyPrintOptions, Sense,
    SolverError as ArcoSolverError, SolverStatus as ArcoSolverStatus, Variable,
};
use arco_highs::Solver as HighsSolver;
#[cfg(feature = "ipopt")]
use arco_ipopt::Solver as IpoptSolver;
use arco_kdl::compile::{
    CompiledProblem, ConstraintSense, LinearReport, LinearTerm, ObjectiveSense, VariableKind,
};
#[cfg(feature = "ipopt")]
use arco_kdl::compile::{NonlinearConstraint, NonlinearExpr};
#[cfg(feature = "xpress")]
use arco_xpress::Solver as XpressSolver;
#[cfg(feature = "ipopt")]
use ipopt::{BasicProblem, ConstrainedProblem, Index, Ipopt, Number, SolveStatus as IpoptStatus};
use std::collections::BTreeMap;
#[cfg(feature = "ipopt")]
use std::collections::BTreeSet;
use std::time::Instant;
use thiserror::Error;
use tracing::info;

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

#[derive(Debug, Default)]
pub struct RustArcoAdapter {
    log_to_console: bool,
}

#[cfg(feature = "ipopt")]
#[derive(Debug, Default)]
pub struct IpoptArcoAdapter {
    log_to_console: bool,
}

#[cfg(feature = "xpress")]
#[derive(Debug, Default)]
pub struct XpressArcoAdapter {
    log_to_console: bool,
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
    #[error("adapter backend `{backend}` only supports linearized models")]
    UnsupportedNonlinearBackend { backend: String },
    #[error("adapter backend `{backend}` failed to evaluate nonlinear expression: {message}")]
    NonlinearEvaluation { backend: String, message: String },
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

impl OptimizationAdapter for RustArcoAdapter {
    fn backend_name(&self) -> &'static str {
        "arco-rust-highs"
    }

    fn solve(
        &self,
        problem: &CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();
        ensure_linearized_problem(problem, &backend)?;

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
            build_started.elapsed().as_secs_f64() * 1000.0,
        );
        info!("initializing solver backend instance");
        let mut solver =
            HighsSolver::new(model).map_err(|source| ExecutionError::SolverInitialization {
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
        info!("solve status: {}", solution.status_string());
        if !solution.is_feasible() {
            return Err(ExecutionError::NoFeasibleSolution {
                backend,
                status: solution.status_string().to_string(),
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
            IpoptSolver::new(model).map_err(|source| ExecutionError::SolverInitialization {
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

#[cfg(feature = "xpress")]
impl XpressArcoAdapter {
    pub fn new() -> Self {
        Self {
            log_to_console: false,
        }
    }

    pub fn with_console_log(log_to_console: bool) -> Self {
        Self { log_to_console }
    }
}

#[cfg(feature = "xpress")]
impl OptimizationAdapter for XpressArcoAdapter {
    fn backend_name(&self) -> &'static str {
        "arco-rust-xpress"
    }

    fn solve(
        &self,
        problem: &CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();
        ensure_linearized_problem(problem, &backend)?;

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
            XpressSolver::new(model).map_err(|source| ExecutionError::SolverInitialization {
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
        let family_key = format!("{}[{}]", vr.control_name, vr.indices.join(","));
        if let Some(family) = variable_values.get(&family_key) {
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

struct BuiltModel {
    model: Model,
    variable_indices: BTreeMap<String, usize>,
    constraint_indices: BTreeMap<String, usize>,
}

fn ensure_linearized_problem(
    problem: &CompiledProblem,
    backend: &str,
) -> Result<(), ExecutionError> {
    if problem.algebra.linearized {
        return Ok(());
    }

    Err(ExecutionError::UnsupportedNonlinearBackend {
        backend: backend.to_string(),
    })
}

fn build_model(problem: &CompiledProblem, backend: &str) -> Result<BuiltModel, ExecutionError> {
    ensure_linearized_problem(problem, backend)?;
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

fn evaluate_linear_report(
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

fn lookup_primal_value(
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
fn collect_nonlinear_variables(expr: &NonlinearExpr, output: &mut BTreeSet<String>) {
    match expr {
        NonlinearExpr::Constant(_) => {}
        NonlinearExpr::Variable(name) => {
            output.insert(name.clone());
        }
        NonlinearExpr::Unary { expr, .. } => collect_nonlinear_variables(expr, output),
        NonlinearExpr::Binary { left, right, .. } => {
            collect_nonlinear_variables(left, output);
            collect_nonlinear_variables(right, output);
        }
        NonlinearExpr::FunctionCall { args, .. } => {
            for arg in args {
                collect_nonlinear_variables(arg, output);
            }
        }
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
#[derive(Debug, Clone)]
struct AutoDiffValue {
    value: f64,
    grad: BTreeMap<usize, f64>,
}

#[cfg(feature = "ipopt")]
fn add_scaled_grad(target: &mut BTreeMap<usize, f64>, source: &BTreeMap<usize, f64>, scale: f64) {
    for (idx, coeff) in source {
        let entry = target.entry(*idx).or_insert(0.0);
        *entry += coeff * scale;
    }
    target.retain(|_, coeff| coeff.abs() >= 1e-14);
}

#[cfg(feature = "ipopt")]
fn eval_nonlinear_expr_autodiff(
    expr: &NonlinearExpr,
    values: &[f64],
    var_positions: &BTreeMap<String, usize>,
) -> Result<AutoDiffValue, String> {
    match expr {
        NonlinearExpr::Constant(value) => Ok(AutoDiffValue {
            value: *value,
            grad: BTreeMap::new(),
        }),
        NonlinearExpr::Variable(name) => {
            let Some(position) = var_positions.get(name) else {
                return Err(format!("unknown variable `{name}`"));
            };
            let value = values
                .get(*position)
                .copied()
                .ok_or_else(|| format!("variable index out of bounds for `{name}`"))?;
            let mut grad = BTreeMap::new();
            grad.insert(*position, 1.0);
            Ok(AutoDiffValue { value, grad })
        }
        NonlinearExpr::Unary { op, expr } => {
            let inner = eval_nonlinear_expr_autodiff(expr, values, var_positions)?;
            match op {
                arco_kdl::algebra::UnaryOp::Negate => Ok(AutoDiffValue {
                    value: -inner.value,
                    grad: inner
                        .grad
                        .into_iter()
                        .map(|(idx, val)| (idx, -val))
                        .collect(),
                }),
            }
        }
        NonlinearExpr::Binary { op, left, right } => {
            let left = eval_nonlinear_expr_autodiff(left, values, var_positions)?;
            let right = eval_nonlinear_expr_autodiff(right, values, var_positions)?;
            match op {
                arco_kdl::algebra::BinaryOp::Add => {
                    let mut grad = left.grad;
                    add_scaled_grad(&mut grad, &right.grad, 1.0);
                    Ok(AutoDiffValue {
                        value: left.value + right.value,
                        grad,
                    })
                }
                arco_kdl::algebra::BinaryOp::Subtract => {
                    let mut grad = left.grad;
                    add_scaled_grad(&mut grad, &right.grad, -1.0);
                    Ok(AutoDiffValue {
                        value: left.value - right.value,
                        grad,
                    })
                }
                arco_kdl::algebra::BinaryOp::Multiply => {
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &left.grad, right.value);
                    add_scaled_grad(&mut grad, &right.grad, left.value);
                    Ok(AutoDiffValue {
                        value: left.value * right.value,
                        grad,
                    })
                }
                arco_kdl::algebra::BinaryOp::Divide => {
                    let denom = right.value;
                    let denom_sq = denom * denom;
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &left.grad, 1.0 / denom);
                    add_scaled_grad(&mut grad, &right.grad, -left.value / denom_sq);
                    Ok(AutoDiffValue {
                        value: left.value / denom,
                        grad,
                    })
                }
            }
        }
        NonlinearExpr::FunctionCall { name, args } => {
            let evaluated = args
                .iter()
                .map(|arg| eval_nonlinear_expr_autodiff(arg, values, var_positions))
                .collect::<Result<Vec<_>, _>>()?;
            match (name.as_str(), evaluated.len()) {
                ("sqrt", 1) => {
                    let base = &evaluated[0];
                    let out = base.value.sqrt();
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &base.grad, 0.5 / out);
                    Ok(AutoDiffValue { value: out, grad })
                }
                ("abs", 1) => {
                    let base = &evaluated[0];
                    let factor = if base.value > 0.0 {
                        1.0
                    } else if base.value < 0.0 {
                        -1.0
                    } else {
                        0.0
                    };
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &base.grad, factor);
                    Ok(AutoDiffValue {
                        value: base.value.abs(),
                        grad,
                    })
                }
                ("exp", 1) => {
                    let base = &evaluated[0];
                    let out = base.value.exp();
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &base.grad, out);
                    Ok(AutoDiffValue { value: out, grad })
                }
                ("ln", 1) => {
                    let base = &evaluated[0];
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &base.grad, 1.0 / base.value);
                    Ok(AutoDiffValue {
                        value: base.value.ln(),
                        grad,
                    })
                }
                ("sin", 1) => {
                    let base = &evaluated[0];
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &base.grad, base.value.cos());
                    Ok(AutoDiffValue {
                        value: base.value.sin(),
                        grad,
                    })
                }
                ("cos", 1) => {
                    let base = &evaluated[0];
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &base.grad, -base.value.sin());
                    Ok(AutoDiffValue {
                        value: base.value.cos(),
                        grad,
                    })
                }
                ("atan", 1) => {
                    let base = &evaluated[0];
                    let mut grad = BTreeMap::new();
                    add_scaled_grad(&mut grad, &base.grad, 1.0 / (1.0 + base.value * base.value));
                    Ok(AutoDiffValue {
                        value: base.value.atan(),
                        grad,
                    })
                }
                ("pow", 2) => {
                    let base = &evaluated[0];
                    let exponent = &evaluated[1];
                    let out = base.value.powf(exponent.value);

                    if !exponent.grad.is_empty() && base.value <= 0.0 {
                        return Err(
                            "pow(base, exp) with variable exponent requires base > 0".to_string()
                        );
                    }

                    let mut grad = BTreeMap::new();
                    if base.value != 0.0 {
                        add_scaled_grad(&mut grad, &base.grad, exponent.value * out / base.value);
                    }
                    if !exponent.grad.is_empty() {
                        add_scaled_grad(&mut grad, &exponent.grad, out * base.value.ln());
                    }

                    Ok(AutoDiffValue { value: out, grad })
                }
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
struct NonlinearIpoptProblem {
    x_lower: Vec<f64>,
    x_upper: Vec<f64>,
    x_init: Vec<f64>,
    g_lower: Vec<f64>,
    g_upper: Vec<f64>,
    objective_expr: NonlinearExpr,
    objective_sign: f64,
    constraints: Vec<NonlinearConstraint>,
    jac_rows: Vec<Index>,
    jac_cols: Vec<Index>,
    jac_positions_by_row: Vec<BTreeMap<usize, usize>>,
    var_positions: BTreeMap<String, usize>,
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
        match eval_nonlinear_expr(&self.objective_expr, x, &self.var_positions) {
            Ok(value) => {
                *obj = self.objective_sign * value;
                true
            }
            Err(_) => false,
        }
    }

    fn objective_grad(&self, x: &[Number], grad_f: &mut [Number]) -> bool {
        let objective =
            match eval_nonlinear_expr_autodiff(&self.objective_expr, x, &self.var_positions) {
                Ok(value) => value,
                Err(_) => return false,
            };

        grad_f.fill(0.0);
        for (idx, deriv) in objective.grad {
            grad_f[idx] = self.objective_sign * deriv;
        }

        true
    }
}

#[cfg(feature = "ipopt")]
impl ConstrainedProblem for NonlinearIpoptProblem {
    fn num_constraints(&self) -> usize {
        self.constraints.len()
    }

    fn num_constraint_jacobian_non_zeros(&self) -> usize {
        self.jac_rows.len()
    }

    fn constraint(&self, x: &[Number], g: &mut [Number]) -> bool {
        for (row_index, row) in self.constraints.iter().enumerate() {
            let Ok(value) = eval_nonlinear_expr(&row.expression, x, &self.var_positions) else {
                return false;
            };
            g[row_index] = value;
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
        vals.fill(0.0);
        for (row_index, row) in self.constraints.iter().enumerate() {
            let evaluated =
                match eval_nonlinear_expr_autodiff(&row.expression, x, &self.var_positions) {
                    Ok(value) => value,
                    Err(_) => return false,
                };

            for (col_idx, deriv) in evaluated.grad {
                if let Some(position) = self.jac_positions_by_row[row_index].get(&col_idx) {
                    vals[*position] = deriv;
                }
            }
        }

        true
    }

    fn num_hessian_non_zeros(&self) -> usize {
        0
    }

    fn hessian_indices(&self, _rows: &mut [Index], _cols: &mut [Index]) -> bool {
        true
    }

    fn hessian_values(
        &self,
        _x: &[Number],
        _obj_factor: Number,
        _lambda: &[Number],
        _vals: &mut [Number],
    ) -> bool {
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

    let mut jac_rows = Vec::new();
    let mut jac_cols = Vec::new();
    let mut jac_positions_by_row =
        vec![BTreeMap::<usize, usize>::new(); nonlinear.constraints.len()];
    for (row_index, row) in nonlinear.constraints.iter().enumerate() {
        let mut vars = BTreeSet::new();
        collect_nonlinear_variables(&row.expression, &mut vars);
        for name in vars {
            let Some(&col_index) = variable_positions.get(&name) else {
                return Err(ExecutionError::UnknownCompiledVariable {
                    backend: backend.to_string(),
                    compiled_name: name,
                });
            };
            let value_position = jac_rows.len();
            jac_rows.push(row_index as Index);
            jac_cols.push(col_index as Index);
            jac_positions_by_row[row_index].insert(col_index, value_position);
        }
    }

    let objective_sign = match nonlinear.objective.sense {
        ObjectiveSense::Minimize => 1.0,
        ObjectiveSense::Maximize => -1.0,
    };

    let g_lower_diag = g_lower.clone();
    let g_upper_diag = g_upper.clone();

    let nlp_problem = NonlinearIpoptProblem {
        x_lower,
        x_upper,
        x_init,
        g_lower,
        g_upper,
        objective_expr: nonlinear.objective.expression.clone(),
        objective_sign,
        constraints: nonlinear.constraints.clone(),
        jac_rows,
        jac_cols,
        jac_positions_by_row,
        var_positions: variable_positions.clone(),
    };

    let mut ipopt =
        Ipopt::new(nlp_problem).map_err(|source| ExecutionError::SolverInitialization {
            backend: backend.to_string(),
            source: ArcoSolverError::SolverSpecific(format!(
                "Failed to create IPOPT NLP: {source:?}"
            )),
        })?;

    ipopt.set_option("hessian_approximation", "limited-memory");
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
                &result.solver_data.solution.primal_variables,
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
        &result.solver_data.solution.constraint_multipliers,
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
    nonlinear: &arco_kdl::compile::NonlinearProblem,
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

fn extract_dual_report_values(
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

fn map_solver_status(status: ArcoSolverStatus) -> SolveStatus {
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
    expr: &arco_kdl::algebra::Expr,
    bindings: &BTreeMap<&str, &str>,
) -> Option<bool> {
    use arco_kdl::algebra::{ComparisonOp, Expr};
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
    use crate::execution::{MockArcoAdapter, execute_problem_with_options};
    use arco_kdl::compile::{
        AlgebraicProblem, CompiledObjective, CompiledProblem, CompiledVariable, LinearObjective,
        ObjectiveSense, VariableInstance, VariableKind,
    };

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
                linearized: true,
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
