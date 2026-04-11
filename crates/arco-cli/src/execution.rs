use arco_core::{
    Bounds, Constraint, Model, ModelError as ArcoModelError, Objective, PrettyPrintOptions, Sense,
    SolverError as ArcoSolverError, SolverStatus as ArcoSolverStatus, Variable,
};
use arco_highs::Solver as HighsSolver;
use arco_kdl::compile::{
    CompiledProblem, ConstraintSense, LinearReport, LinearTerm, ObjectiveSense, VariableKind,
};
#[cfg(feature = "xpress")]
use arco_xpress::Solver as XpressSolver;
use std::collections::BTreeMap;
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
    pub reports: Vec<MappedScalarResult>,
    pub variables: Vec<MappedVariableResult>,
    pub dual_reports: Vec<DualReportResult>,
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

    let reports = problem
        .reports
        .iter()
        .map(|report| {
            let value = report_values.get(&report.name).copied().ok_or_else(|| {
                ExecutionError::MissingReportValue {
                    backend: backend.to_string(),
                    compiled_name: report.name.clone(),
                }
            })?;
            Ok(MappedScalarResult {
                dsl_name: report.name.clone(),
                compiled_name: report.name.clone(),
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

fn build_model(problem: &CompiledProblem, backend: &str) -> Result<BuiltModel, ExecutionError> {
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
