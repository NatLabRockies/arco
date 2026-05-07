use crate::compile::compile::CompiledProblem;
#[cfg(feature = "xpress")]
use crate::execution::XpressArcoAdapter;
use crate::execution::{
    AdapterSolveOutput, ExecutionError, OptimizationAdapter, RustArcoAdapter, ScalarArtifactValue,
    ScipArcoAdapter, SolveStatus, VariableArtifactValue, VariableInstanceArtifactValue,
    build_model, evaluate_linear_report, extract_dual_report_values, lookup_primal_value,
    map_solver_status,
};
use crate::portable_problem_from_algebraic;
#[cfg(feature = "xpress")]
use crate::xpress::Solver as XpressSolver;
use arco_highs::{HighsModelViewBackend, highs_version};
use arco_scip as scip;
use arco_solver::{
    ModelViewBackendRegistry, ModelViewSolveResult, SolverConfig, SolverError, SolverRegistry,
};
use std::time::Instant;
use tracing::info;

pub(crate) fn solver_registry_with_builtin_families() -> SolverRegistry {
    let mut registry = SolverRegistry::with_builtin_families();
    scip::register_solver_family(&mut registry);
    registry
}

pub(crate) fn solve_model_view_with_builtin_backend(
    family: &str,
    model: &dyn arco_model::ModelView,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    match normalize_model_view_backend_family(family) {
        "highs" => {
            let highs = HighsModelViewBackend;
            let mut registry = ModelViewBackendRegistry::new();
            registry.register(&highs);
            registry.solve("highs", model, config)
        }
        "ipopt" => Err(SolverError::SolverNotAvailable(
            "IPOPT model-view backend is not implemented yet; use a supported backend such as 'highs'"
                .to_string(),
        )),
        "xpress" => Err(SolverError::SolverNotAvailable(
            "Xpress model-view backend is not implemented yet; use a supported backend such as 'highs'"
                .to_string(),
        )),
        "scip" => Err(SolverError::SolverNotAvailable(
            "SCIP is available as an external-process adapter, not as a builtin model-view backend"
                .to_string(),
        )),
        other => Err(SolverError::SolverNotAvailable(format!(
            "no builtin model-view backend registered for '{other}'"
        ))),
    }
}

pub(crate) fn builtin_solver_version(family: &str) -> Option<String> {
    match normalize_model_view_backend_family(family) {
        "highs" => highs_version(),
        _ => None,
    }
}

fn normalize_model_view_backend_family(family: &str) -> &str {
    match family {
        "arco-rust-highs" => "highs",
        "arco-rust-xpress" => "xpress",
        "arco-rust-scip" => "scip",
        other => other,
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
        info!("building primitive model view");
        let build_started = Instant::now();
        let built = build_model(problem, &backend)?;
        let variable_indices = built.variable_indices;
        let constraint_indices = built.constraint_indices;
        info!(
            "primitive model build completed in {:.2} ms",
            build_started.elapsed().as_secs_f64() * 1000.0,
        );

        info!("starting solver backend run: {}", backend);
        let solver_started = Instant::now();
        let config = SolverConfig::default().with_log_to_console(self.log_to_console);
        let solution = solve_model_view_with_builtin_backend("highs", &built.model, &config)
            .map_err(|source| ExecutionError::Solve {
                backend: backend.clone(),
                source,
            })?;
        info!(
            "solver backend run completed in {:.2} ms: {}",
            solver_started.elapsed().as_secs_f64() * 1000.0,
            backend
        );
        info!("solve status: {}", solution.status);
        if !solution.status.is_feasible() {
            return Err(ExecutionError::NoFeasibleSolution {
                backend,
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
                        &backend,
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
                            &backend,
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
                                    &backend,
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
        let portable = portable_problem_from_algebraic(&problem.algebra);
        let scip_problem = scip::ScipProblem {
            portable: &portable,
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
        let built = build_model(problem, &backend)?;
        let variable_indices = built.variable_indices;
        let constraint_indices = built.constraint_indices;
        info!(
            "solver model translation completed in {:.2} ms",
            build_started.elapsed().as_secs_f64() * 1000.0
        );

        info!("starting solver backend run: {}", backend);
        let solver_started = Instant::now();
        info!("initializing solver backend instance");
        let mut solver = XpressSolver::new(&built.model).map_err(|source| {
            ExecutionError::SolverInitialization {
                backend: backend.clone(),
                source,
            }
        })?;
        solver.set_log_to_console(self.log_to_console);

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

#[cfg(test)]
mod tests {
    use super::{normalize_model_view_backend_family, solve_model_view_with_builtin_backend};
    use arco_model::Model;
    use arco_solver::{SolverConfig, SolverError};

    #[test]
    fn normalize_model_view_backend_family_maps_known_aliases() {
        assert_eq!(
            normalize_model_view_backend_family("arco-rust-highs"),
            "highs"
        );
        assert_eq!(
            normalize_model_view_backend_family("arco-rust-xpress"),
            "xpress"
        );
        assert_eq!(
            normalize_model_view_backend_family("arco-rust-scip"),
            "scip"
        );
    }

    #[test]
    fn builtin_model_view_backend_reports_ipopt_not_implemented() {
        let model = Model::new();
        let error = solve_model_view_with_builtin_backend("ipopt", &model, &SolverConfig::new())
            .expect_err("ipopt backend should report unavailable status");

        assert!(matches!(error, SolverError::SolverNotAvailable(_)));
        assert!(error.to_string().contains("IPOPT model-view backend"));
    }

    #[test]
    fn builtin_model_view_backend_reports_scip_external_only() {
        let model = Model::new();
        let error = solve_model_view_with_builtin_backend("scip", &model, &SolverConfig::new())
            .expect_err("scip backend should report external-process only path");

        assert!(matches!(error, SolverError::SolverNotAvailable(_)));
        assert!(error.to_string().contains("external-process adapter"));
    }

    #[test]
    fn builtin_model_view_backend_reports_unknown_family() {
        let model = Model::new();
        let error = solve_model_view_with_builtin_backend("unknown", &model, &SolverConfig::new())
            .expect_err("unknown backend should report unavailable status");

        assert!(matches!(error, SolverError::SolverNotAvailable(_)));
        assert!(error.to_string().contains("unknown"));
    }
}
