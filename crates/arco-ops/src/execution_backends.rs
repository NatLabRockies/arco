#[cfg(feature = "compile")]
use crate::compile::compile::CompiledProblem;
#[cfg(all(feature = "compile", feature = "xpress"))]
use crate::execution::XpressArcoAdapter;
#[cfg(feature = "compile")]
use crate::execution::{
    AdapterSolveOutput, ExecutionError, OptimizationAdapter, RustArcoAdapter, ScalarArtifactValue,
    VariableArtifactValue, VariableInstanceArtifactValue, build_model, evaluate_linear_report,
    extract_dual_report_values, lookup_primal_value, map_solver_status,
};
#[cfg(all(
    feature = "compile",
    any(feature = "scip-bundled", feature = "scip-from-source")
))]
use crate::execution::{ScipArcoAdapter, SolveStatus};
#[cfg(all(
    feature = "compile",
    any(feature = "scip-bundled", feature = "scip-from-source")
))]
use crate::{ops_problem_from_algebraic, portable_problem_from_ops};
use arco_highs::{HighsModelViewBackend, highs_version};
#[cfg(any(feature = "scip-bundled", feature = "scip-from-source"))]
use arco_scip as scip;
use arco_solver::{
    ModelViewBackendRegistry, ModelViewSolveResult, SolverConfig, SolverError, SolverRegistry,
};
#[cfg(feature = "compile")]
use arco_solver::{ResolvedSelection, SolverProfile, SolverTransport};
#[cfg(feature = "xpress")]
use arco_xpress::{XpressModelViewBackend, xpress_runtime_available};
#[cfg(feature = "compile")]
use std::time::Instant;
#[cfg(feature = "compile")]
use tracing::info;

pub(crate) fn solver_registry_with_builtin_families() -> SolverRegistry {
    #[cfg(any(feature = "scip-bundled", feature = "scip-from-source"))]
    {
        let mut registry = SolverRegistry::with_builtin_families();
        scip::register_solver_family(&mut registry);
        registry
    }
    #[cfg(not(any(feature = "scip-bundled", feature = "scip-from-source")))]
    {
        SolverRegistry::with_builtin_families()
    }
}

pub(crate) fn solve_model_view_with_builtin_backend(
    family: &str,
    model: &dyn arco_model::ModelView,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let family = normalize_model_view_backend_family(family);
    if family == "ipopt" {
        return Err(SolverError::SolverNotAvailable(
            "IPOPT model-view backend is not implemented yet; use a supported backend such as 'highs'"
                .to_string(),
        ));
    }
    #[cfg(not(any(feature = "scip-bundled", feature = "scip-from-source")))]
    if family == "scip" {
        return Err(SolverError::SolverNotAvailable(
            "SCIP model-view backend is not enabled; rebuild with --features scip-bundled or scip-from-source".to_string(),
        ));
    }
    #[cfg(not(feature = "xpress"))]
    if family == "xpress" {
        return Err(SolverError::SolverNotAvailable(
            "Xpress model-view backend is not enabled; rebuild with --features xpress".to_string(),
        ));
    }

    let highs = HighsModelViewBackend;
    let mut registry = ModelViewBackendRegistry::new();
    registry.register(&highs);
    #[cfg(any(feature = "scip-bundled", feature = "scip-from-source"))]
    let scip = scip::ScipModelViewBackend;
    #[cfg(any(feature = "scip-bundled", feature = "scip-from-source"))]
    registry.register(&scip);
    #[cfg(feature = "xpress")]
    let xpress = XpressModelViewBackend;
    #[cfg(feature = "xpress")]
    registry.register(&xpress);
    registry.solve(family, model, config)
}

pub(crate) fn builtin_solver_version(family: &str) -> Option<String> {
    match normalize_model_view_backend_family(family) {
        "highs" => highs_version(),
        #[cfg(feature = "xpress")]
        "xpress" => Some(if xpress_runtime_available() {
            "available".to_string()
        } else {
            "not-found".to_string()
        }),
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

#[cfg(feature = "compile")]
pub(crate) fn adapter_for_selection(
    selection: &ResolvedSelection,
    log_to_console: bool,
    _profile: Option<&SolverProfile>,
) -> Result<Box<dyn OptimizationAdapter>, String> {
    match selection.transport {
        SolverTransport::Embedded => match selection.family.as_str() {
            "highs" => Ok(Box::new(RustArcoAdapter::with_console_log(log_to_console))),
            #[cfg(feature = "xpress")]
            "xpress" => Ok(Box::new(XpressArcoAdapter::with_console_log(
                log_to_console,
            ))),
            #[cfg(not(feature = "xpress"))]
            "xpress" => Err(
                "embedded solver family 'xpress' is not available (rebuild with --features xpress)"
                    .to_string(),
            ),
            #[cfg(any(feature = "scip-bundled", feature = "scip-from-source"))]
            "scip" => Ok(Box::new(ScipArcoAdapter::with_native_profile(
                log_to_console,
                _profile.map_or_else(SolverConfig::default, |value| value.options.clone()),
            ))),
            #[cfg(not(any(feature = "scip-bundled", feature = "scip-from-source")))]
            "scip" => Err(
                "embedded solver family 'scip' is not available (rebuild with --features scip-bundled or scip-from-source)"
                    .to_string(),
            ),
            #[cfg(feature = "ipopt")]
            "ipopt" => Ok(Box::new(
                crate::execution::IpoptArcoAdapter::with_console_log(log_to_console),
            )),
            #[cfg(not(feature = "ipopt"))]
            "ipopt" => Err(
                "embedded solver family 'ipopt' is not available (rebuild with --features ipopt)"
                    .to_string(),
            ),
            family => Err(format!(
                "embedded solver family '{family}' is not available"
            )),
        },
        SolverTransport::ExternalProcess => {
            let family = selection.family.as_str();
            Err(format!(
                "external-process solver family '{family}' is not available"
            ))
        }
    }
}

#[cfg(feature = "compile")]
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
        if !problem.algebra.linearized {
            return Err(ExecutionError::UnsupportedNonlinearBackend { backend });
        }
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

#[cfg(all(
    feature = "compile",
    any(feature = "scip-bundled", feature = "scip-from-source")
))]
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
        let options = scip::NativeSolveOptions {
            time_limit: self.solver_config.time_limit,
            mip_gap: self.solver_config.mip_gap,
            presolve: self.solver_config.presolve,
            threads: self.solver_config.threads,
            tolerance: self.solver_config.tolerance,
            verbosity: self.solver_config.verbosity,
            lp_algorithm: self.solver_config.lp_algorithm,
        };
        let variable_families = problem
            .variables
            .iter()
            .map(|variable| variable.family.clone())
            .collect::<Vec<_>>();
        let ops_problem = ops_problem_from_algebraic(&problem.algebra);
        let portable = portable_problem_from_ops(&ops_problem);
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
            scip::Error::Build { message } | scip::Error::Process { message } => {
                ExecutionError::ExternalSolverProcess {
                    backend: backend.clone(),
                    message,
                }
            }
            scip::Error::NoFeasibleSolution { status } => ExecutionError::NoFeasibleSolution {
                backend: backend.clone(),
                status,
            },
        })?;

        Ok(AdapterSolveOutput {
            status: match solution.status {
                scip::SolveStatus::Optimal => SolveStatus::Optimal,
                scip::SolveStatus::Infeasible => SolveStatus::Infeasible,
                scip::SolveStatus::TimeLimit => SolveStatus::TimeLimit,
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

#[cfg(all(feature = "compile", feature = "xpress"))]
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

#[cfg(all(feature = "compile", feature = "xpress"))]
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
        let config = SolverConfig::default().with_log_to_console(self.log_to_console);
        let solution = solve_model_view_with_builtin_backend("xpress", &built.model, &config)
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

    #[cfg(any(feature = "scip-bundled", feature = "scip-from-source"))]
    #[test]
    fn builtin_model_view_backend_reports_scip_empty_model() {
        let model = Model::new();
        let error = solve_model_view_with_builtin_backend("scip", &model, &SolverConfig::new())
            .expect_err("empty model should fail before SCIP solve");

        assert!(matches!(error, SolverError::EmptyModel));
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
