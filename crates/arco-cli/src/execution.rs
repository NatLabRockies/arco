use arco_core::{
    Bounds, Constraint, Model, ModelError as ArcoModelError, Objective, PrettyPrintOptions, Sense,
    SolverError as ArcoSolverError, SolverStatus as ArcoSolverStatus, Variable,
};
use arco_highs::Solver as HighsSolver;
use arco_kdl::lowering::{
    ConstraintSense, LinearReport, LinearTerm, LoweredProblem, ObjectiveSense, VariableKind,
};
#[cfg(feature = "xpress")]
use arco_xpress::Solver as XpressSolver;
use std::collections::BTreeMap;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Debug, Clone, PartialEq)]
pub struct AdapterSolveOutput {
    pub status: SolveStatus,
    pub objective_value: ScalarArtifactValue,
    pub report_values: Vec<ScalarArtifactValue>,
    pub variable_values: Vec<VariableArtifactValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    Optimal,
    Infeasible,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarArtifactValue {
    pub lowered_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableArtifactValue {
    pub lowered_name: String,
    pub representative_value: f64,
    pub values: Vec<VariableInstanceArtifactValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableInstanceArtifactValue {
    pub lowered_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub backend: &'static str,
    pub status: SolveStatus,
    pub objective_sense: String,
    pub objective: MappedScalarResult,
    pub reports: Vec<MappedScalarResult>,
    pub variables: Vec<MappedVariableResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedScalarResult {
    pub dsl_name: String,
    pub lowered_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedVariableResult {
    pub dsl_name: String,
    pub lowered_name: String,
    pub representative_value: f64,
    pub values: Vec<MappedVariableValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedVariableValue {
    pub dsl_name: String,
    pub lowered_name: String,
    pub value: f64,
}

pub trait OptimizationAdapter {
    fn backend_name(&self) -> &'static str;

    fn solve(
        &self,
        problem: &LoweredProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError>;
}

#[derive(Debug, Default)]
pub struct MockArcoAdapter;

#[derive(Debug, Default)]
pub struct RustArcoAdapter {
    log_to_console: bool,
}

#[cfg(feature = "xpress")]
#[derive(Debug, Default)]
pub struct XpressArcoAdapter {
    log_to_console: bool,
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("adapter backend `{backend}` failed to add variable `{lowered_name}`: {source}")]
    AddVariable {
        backend: String,
        lowered_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to name variable `{lowered_name}`: {source}")]
    NameVariable {
        backend: String,
        lowered_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to add constraint `{lowered_name}`: {source}")]
    AddConstraint {
        backend: String,
        lowered_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to name constraint `{lowered_name}`: {source}")]
    NameConstraint {
        backend: String,
        lowered_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` references unknown lowered variable `{lowered_name}`")]
    UnknownLoweredVariable {
        backend: String,
        lowered_name: String,
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
    #[error("adapter backend `{backend}` failed to set objective `{lowered_name}`: {source}")]
    SetObjective {
        backend: String,
        lowered_name: String,
        #[source]
        source: ArcoModelError,
    },
    #[error("adapter backend `{backend}` failed to name objective `{lowered_name}`: {source}")]
    NameObjective {
        backend: String,
        lowered_name: String,
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
    #[error("adapter backend `{backend}` did not return objective `{lowered_name}`")]
    MissingObjectiveValue {
        backend: String,
        lowered_name: String,
    },
    #[error("adapter backend `{backend}` did not return report `{lowered_name}`")]
    MissingReportValue {
        backend: String,
        lowered_name: String,
    },
    #[error("adapter backend `{backend}` did not return variable `{lowered_name}`")]
    MissingVariableValue {
        backend: String,
        lowered_name: String,
    },
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
        problem: &LoweredProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        Ok(AdapterSolveOutput {
            status: SolveStatus::Optimal,
            objective_value: ScalarArtifactValue {
                lowered_name: problem.objective.name.clone(),
                value: 0.0,
            },
            report_values: problem
                .reports
                .iter()
                .map(|report| ScalarArtifactValue {
                    lowered_name: report.name.clone(),
                    value: 0.0,
                })
                .collect(),
            variable_values: problem
                .variables
                .iter()
                .map(|variable| VariableArtifactValue {
                    lowered_name: variable.family.clone(),
                    representative_value: 0.0,
                    values: if include_variable_values {
                        problem
                            .algebra
                            .variable_instances
                            .iter()
                            .filter(|instance| instance.family == variable.family)
                            .map(|instance| VariableInstanceArtifactValue {
                                lowered_name: instance.name.clone(),
                                value: 0.0,
                            })
                            .collect()
                    } else {
                        Vec::new()
                    },
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
        problem: &LoweredProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();
        info!("solving with {}", backend);
        debug!("building solver model from lowered algebra");
        let build_started = Instant::now();
        let BuiltModel {
            model,
            variable_indices,
        } = build_model(problem, &backend)?;
        debug!(
            "solver model built in {:.2} ms ({} variable instances, {} constraints)",
            build_started.elapsed().as_secs_f64() * 1000.0,
            problem.algebra.variable_instances.len(),
            problem.algebra.constraints.len()
        );
        let mut solver =
            HighsSolver::new(model).map_err(|source| ExecutionError::SolverInitialization {
                backend: backend.clone(),
                source,
            })?;
        solver.set_log_to_console(self.log_to_console);

        let solution = solver.solve().map_err(|source| ExecutionError::Solve {
            backend: backend.clone(),
            source,
        })?;
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
                    lowered_name: report.name.clone(),
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
                                lowered_name: instance.name.clone(),
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
                    lowered_name: variable.family.clone(),
                    representative_value,
                    values,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AdapterSolveOutput {
            status: map_solver_status(solution.core_status()),
            objective_value: ScalarArtifactValue {
                lowered_name: problem.objective.name.clone(),
                value: objective_value,
            },
            report_values,
            variable_values,
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
        problem: &LoweredProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();
        info!("solving with {}", backend);
        let BuiltModel {
            model,
            variable_indices,
        } = build_model(problem, &backend)?;
        let mut solver =
            XpressSolver::new(model).map_err(|source| ExecutionError::SolverInitialization {
                backend: backend.clone(),
                source,
            })?;
        solver.set_log_to_console(self.log_to_console);

        let solution = solver.solve().map_err(|source| ExecutionError::Solve {
            backend: backend.clone(),
            source,
        })?;
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
                    lowered_name: report.name.clone(),
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
                                lowered_name: instance.name.clone(),
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
                    lowered_name: variable.family.clone(),
                    representative_value,
                    values,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AdapterSolveOutput {
            status: map_solver_status(solution.core_status()),
            objective_value: ScalarArtifactValue {
                lowered_name: problem.objective.name.clone(),
                value: objective_value,
            },
            report_values,
            variable_values,
        })
    }
}

pub fn execute_problem(
    problem: &LoweredProblem,
    adapter: &dyn OptimizationAdapter,
) -> Result<ExecutionResult, ExecutionError> {
    execute_problem_with_options(problem, adapter, true)
}

pub fn execute_problem_with_options(
    problem: &LoweredProblem,
    adapter: &dyn OptimizationAdapter,
    include_variable_values: bool,
) -> Result<ExecutionResult, ExecutionError> {
    let execution_started = Instant::now();
    debug!(
        "starting backend execution pipeline (include_variable_values={})",
        include_variable_values
    );
    let solve_output = adapter.solve(problem, include_variable_values)?;
    debug!(
        "backend execution pipeline returned in {:.2} ms",
        execution_started.elapsed().as_secs_f64() * 1000.0
    );
    let backend = adapter.backend_name();

    let objective = if solve_output.objective_value.lowered_name == problem.objective.name {
        MappedScalarResult {
            dsl_name: problem.objective.name.clone(),
            lowered_name: solve_output.objective_value.lowered_name.clone(),
            value: solve_output.objective_value.value,
        }
    } else {
        return Err(ExecutionError::MissingObjectiveValue {
            backend: backend.to_string(),
            lowered_name: problem.objective.name.clone(),
        });
    };

    let report_values = solve_output
        .report_values
        .iter()
        .map(|report| (report.lowered_name.clone(), report.value))
        .collect::<BTreeMap<_, _>>();
    let variable_values = solve_output
        .variable_values
        .iter()
        .map(|variable| (variable.lowered_name.clone(), variable))
        .collect::<BTreeMap<_, _>>();

    let reports = problem
        .reports
        .iter()
        .map(|report| {
            let value = report_values.get(&report.name).copied().ok_or_else(|| {
                ExecutionError::MissingReportValue {
                    backend: backend.to_string(),
                    lowered_name: report.name.clone(),
                }
            })?;
            Ok(MappedScalarResult {
                dsl_name: report.name.clone(),
                lowered_name: report.name.clone(),
                value,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

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
                        lowered_name: variable.family.clone(),
                    })?;
            Ok(MappedVariableResult {
                dsl_name: variable.family.clone(),
                lowered_name: variable.family.clone(),
                representative_value: solved_variable.representative_value,
                values: solved_variable
                    .values
                    .iter()
                    .map(|value| MappedVariableValue {
                        dsl_name: value.lowered_name.clone(),
                        lowered_name: value.lowered_name.clone(),
                        value: value.value,
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ExecutionResult {
        backend,
        status: solve_output.status,
        objective_sense: problem.objective.sense.clone(),
        objective,
        reports,
        variables,
    })
}

pub fn render_problem_model(problem: &LoweredProblem) -> Result<String, ExecutionError> {
    let built = build_model(problem, "arco-rust-highs")?;
    Ok(built.model.format_ascii(PrettyPrintOptions::full()))
}

struct BuiltModel {
    model: Model,
    variable_indices: BTreeMap<String, usize>,
}

fn build_model(problem: &LoweredProblem, backend: &str) -> Result<BuiltModel, ExecutionError> {
    let total_variables = problem.algebra.variable_instances.len();
    let total_constraints = problem.algebra.constraints.len();
    let variable_progress_step = 50_000usize;
    let constraint_progress_step = 10_000usize;
    let build_started = Instant::now();
    let debug_progress_enabled = tracing::enabled!(tracing::Level::DEBUG);
    if debug_progress_enabled {
        debug!(
            "translating lowered algebra into solver model ({} variable instances, {} constraints)",
            total_variables, total_constraints
        );
    }

    let mut model = Model::with_capacities(total_variables, total_constraints);
    let mut variable_indices = BTreeMap::new();
    let mut variable_ids = BTreeMap::new();

    let mut next_variable_progress = variable_progress_step;
    for (i, variable) in problem.algebra.variable_instances.iter().enumerate() {
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
                    lowered_name: variable.name.clone(),
                    source,
                })?;
        model
            .set_variable_name(variable_id, variable.name.clone())
            .map_err(|source| ExecutionError::NameVariable {
                backend: backend.to_string(),
                lowered_name: variable.name.clone(),
                source,
            })?;
        variable_indices.insert(variable.name.clone(), variable_id.inner() as usize);
        variable_ids.insert(variable.name.clone(), variable_id);

        let processed = i + 1;
        if debug_progress_enabled
            && (processed >= next_variable_progress || processed == total_variables)
        {
            debug!("model translation progress: variables {processed}/{total_variables}");
            next_variable_progress += variable_progress_step;
        }
    }

    if debug_progress_enabled {
        debug!("starting constraint coefficient population");
    }
    let mut next_constraint_progress = constraint_progress_step;
    for (i, constraint) in problem.algebra.constraints.iter().enumerate() {
        let constraint_id = model
            .add_constraint(Constraint {
                bounds: to_bounds(constraint.sense, constraint.rhs),
            })
            .map_err(|source| ExecutionError::AddConstraint {
                backend: backend.to_string(),
                lowered_name: constraint.name.clone(),
                source,
            })?;
        model
            .set_constraint_name(constraint_id, constraint.name.clone())
            .map_err(|source| ExecutionError::NameConstraint {
                backend: backend.to_string(),
                lowered_name: constraint.name.clone(),
                source,
            })?;

        for term in &constraint.terms {
            let variable_id = variable_ids
                .get(&term.variable_name)
                .copied()
                .ok_or_else(|| ExecutionError::UnknownLoweredVariable {
                    backend: backend.to_string(),
                    lowered_name: term.variable_name.clone(),
                })?;
            model
                .set_coefficient(variable_id, constraint_id, term.coefficient)
                .map_err(|source| ExecutionError::SetCoefficient {
                    backend: backend.to_string(),
                    constraint_name: constraint.name.clone(),
                    source,
                })?;
        }

        let processed = i + 1;
        if debug_progress_enabled
            && (processed >= next_constraint_progress || processed == total_constraints)
        {
            debug!("model translation progress: constraints {processed}/{total_constraints}");
            next_constraint_progress += constraint_progress_step;
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
                .ok_or_else(|| ExecutionError::UnknownLoweredVariable {
                    backend: backend.to_string(),
                    lowered_name: term.variable_name.clone(),
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
            lowered_name: problem.algebra.objective.name.clone(),
            source,
        })?;
    model
        .set_objective_name(Some(problem.algebra.objective.name.clone()))
        .map_err(|source| ExecutionError::NameObjective {
            backend: backend.to_string(),
            lowered_name: problem.algebra.objective.name.clone(),
            source,
        })?;
    if debug_progress_enabled {
        debug!(
            "model translation completed in {:.2} ms",
            build_started.elapsed().as_secs_f64() * 1000.0
        );
    }

    Ok(BuiltModel {
        model,
        variable_indices,
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
    lowered_name: &str,
    variable_indices: &BTreeMap<String, usize>,
    primal_values: &[f64],
) -> Result<f64, ExecutionError> {
    let index = variable_indices.get(lowered_name).copied().ok_or_else(|| {
        ExecutionError::UnknownLoweredVariable {
            backend: backend.to_string(),
            lowered_name: lowered_name.to_string(),
        }
    })?;
    primal_values
        .get(index)
        .copied()
        .ok_or_else(|| ExecutionError::UnknownLoweredVariable {
            backend: backend.to_string(),
            lowered_name: lowered_name.to_string(),
        })
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

#[cfg(test)]
mod tests {
    use crate::execution::{MockArcoAdapter, execute_problem_with_options};
    use arco_kdl::lowering::{
        AlgebraicProblem, LinearObjective, LoweredObjective, LoweredProblem, LoweredVariable,
        ObjectiveSense, VariableInstance, VariableKind,
    };

    #[test]
    #[allow(clippy::float_cmp)]
    fn compact_execution_can_skip_detailed_variable_values() {
        let problem = LoweredProblem {
            parameters: Vec::new(),
            variables: vec![LoweredVariable {
                family: "x[a,t]".to_string(),
            }],
            constraints: Vec::new(),
            objective: LoweredObjective {
                name: "obj".to_string(),
                sense: "maximize".to_string(),
                expression: "0".to_string(),
            },
            reports: Vec::new(),
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

        let execution = execute_problem_with_options(&problem, &MockArcoAdapter::new(), false)
            .expect("compact execution should succeed");

        assert_eq!(execution.status, crate::execution::SolveStatus::Optimal);
        assert_eq!(execution.objective.dsl_name, "obj");
        assert_eq!(execution.objective.value, 0.0);
        assert_eq!(execution.objective_sense, "maximize");
        assert!(execution.reports.is_empty());

        assert_eq!(execution.variables.len(), 1);
        assert_eq!(execution.variables[0].dsl_name, "x[a,t]");
        assert_eq!(execution.variables[0].representative_value, 0.0);
        assert!(execution.variables[0].values.is_empty());
    }
}
