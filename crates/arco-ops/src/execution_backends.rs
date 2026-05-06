#[cfg(feature = "xpress")]
use crate::execution::XpressArcoAdapter;
use crate::execution::{
    AdapterSolveOutput, ExecutionError, OptimizationAdapter, RustArcoAdapter, ScalarArtifactValue,
    ScipArcoAdapter, SolveStatus, VariableArtifactValue, VariableInstanceArtifactValue,
    evaluate_linear_report, extract_dual_report_values, lookup_primal_value, map_solver_status,
};
use crate::highs::Solver as HighsSolver;
use crate::kdl::compile::CompiledProblem;
use crate::scip;
#[cfg(feature = "xpress")]
use crate::xpress::Solver as XpressSolver;
use std::time::Instant;
use tracing::info;

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
        info!("building solver index maps");
        let build_started = Instant::now();
        let variable_indices = problem
            .algebra
            .variable_instances
            .iter()
            .enumerate()
            .map(|(index, instance)| (instance.name.clone(), index))
            .collect();
        let constraint_indices = problem
            .algebra
            .constraints
            .iter()
            .enumerate()
            .map(|(index, constraint)| (constraint.name.clone(), index))
            .collect();
        info!(
            "solver index map build completed in {:.2} ms",
            build_started.elapsed().as_secs_f64() * 1000.0,
        );
        info!("initializing solver backend instance");
        let mut solver = HighsSolver::new(problem.algebra.clone()).map_err(|source| {
            ExecutionError::SolverInitialization {
                backend: backend.clone(),
                source,
            }
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

impl OptimizationAdapter for ScipArcoAdapter {
    fn backend_name(&self) -> &'static str {
        scip::BACKEND_NAME
    }

    fn solve(
        &self,
        problem: &CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = scip::BACKEND_NAME.to_string();
        let options = scip::ExternalProcessOptions {
            executable: self.executable.clone(),
            arguments: self.arguments.clone(),
            environment: self.environment.clone(),
        };
        let variable_families = problem
            .variables
            .iter()
            .map(|variable| variable.family.clone())
            .collect::<Vec<_>>();
        let scip_problem = scip::ScipProblem {
            algebra: &problem.algebra,
            variable_families: &variable_families,
        };
        let solution = scip::solve_problem_with_options(
            scip_problem,
            include_variable_values,
            self.log_to_console,
            &options,
        )
        .map_err(|source| match source {
            scip::Error::Io { source } => ExecutionError::ExternalSolverIo {
                backend: backend.clone(),
                source,
            },
            scip::Error::Process { message } => ExecutionError::ExternalSolverProcess {
                backend: backend.clone(),
                message,
            },
            scip::Error::Parse { message } => ExecutionError::ExternalSolverParse {
                backend: backend.clone(),
                message,
            },
            scip::Error::NoFeasibleSolution { status } => ExecutionError::NoFeasibleSolution {
                backend: backend.clone(),
                status,
            },
        })?;

        Ok(AdapterSolveOutput {
            status: match solution.status {
                scip::SolveStatus::Optimal => SolveStatus::Optimal,
                scip::SolveStatus::Infeasible => SolveStatus::Infeasible,
                scip::SolveStatus::Failed => SolveStatus::Failed,
            },
            objective_value: ScalarArtifactValue {
                compiled_name: problem.objective.name.clone(),
                value: solution.objective_value,
            },
            report_values: solution
                .report_values
                .into_iter()
                .map(|report| ScalarArtifactValue {
                    compiled_name: report.compiled_name,
                    value: report.value,
                })
                .collect(),
            variable_values: solution
                .variable_values
                .into_iter()
                .map(|variable| VariableArtifactValue {
                    compiled_name: variable.compiled_name,
                    representative_value: variable.representative_value,
                    values: variable
                        .values
                        .into_iter()
                        .map(|value| VariableInstanceArtifactValue {
                            compiled_name: value.compiled_name,
                            value: value.value,
                        })
                        .collect(),
                })
                .collect(),
            dual_report_values: Vec::new(),
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
