use arco_highs::HighsModelViewBackend;
use arco_ops::execution::{
    AdapterSolveOutput, ExecutionError, OptimizationAdapter, ScalarArtifactValue, SolveStatus,
    VariableArtifactValue, VariableInstanceArtifactValue, adapter_output_from_model_view_solution,
    build_model,
};
use arco_ops::solve::{
    ModelViewBackend, ResolvedSelection, SolverConfig, SolverProfile, SolverTransport,
};
use arco_ops::{ops_problem_from_algebraic, portable_problem_from_ops};
use arco_solver::{ModelViewBackendRegistry, SolverRegistry};

pub fn register_builtin_solver_families(registry: &mut SolverRegistry) {
    arco_scip::register_solver_family(registry);
}

pub fn register_builtin_model_view_backends(registry: &mut ModelViewBackendRegistry<'_>) {
    let highs = Box::leak(Box::new(HighsModelViewBackend));
    let scip = Box::leak(Box::new(arco_scip::ScipModelViewBackend));
    registry.register(highs);
    registry.register(scip);
}

#[derive(Debug, Default)]
struct HighsArcoAdapter {
    log_to_console: bool,
}

impl HighsArcoAdapter {
    fn with_console_log(log_to_console: bool) -> Self {
        Self { log_to_console }
    }
}

impl OptimizationAdapter for HighsArcoAdapter {
    fn backend_name(&self) -> &'static str {
        "arco-rust-highs"
    }

    fn solve(
        &self,
        problem: &arco_ops::compile::compile::CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();
        let built = build_model(problem, &backend)?;
        let config = SolverConfig::default().with_log_to_console(self.log_to_console);
        let highs = HighsModelViewBackend;
        let solution = highs
            .solve_model_view(&built.model, &config)
            .map_err(|source| ExecutionError::Solve {
                backend: backend.clone(),
                source,
            })?;

        adapter_output_from_model_view_solution(
            problem,
            include_variable_values,
            &backend,
            built.variable_indices,
            built.constraint_indices,
            solution,
        )
    }
}

#[derive(Debug, Default)]
struct ScipArcoAdapter {
    log_to_console: bool,
    solver_config: SolverConfig,
}

impl ScipArcoAdapter {
    fn with_native_profile(log_to_console: bool, solver_config: SolverConfig) -> Self {
        Self {
            log_to_console,
            solver_config,
        }
    }
}

impl OptimizationAdapter for ScipArcoAdapter {
    fn backend_name(&self) -> &'static str {
        arco_scip::BACKEND_NAME
    }

    fn solve(
        &self,
        problem: &arco_ops::compile::compile::CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();
        let options = arco_scip::NativeSolveOptions {
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
        let scip_problem = arco_scip::ScipProblem {
            portable: &portable,
            variable_families: &variable_families,
        };
        let solution = arco_scip::solve_problem_with_options(
            scip_problem,
            include_variable_values,
            self.log_to_console,
            &options,
        )
        .map_err(|source| match source {
            arco_scip::Error::Build { message } | arco_scip::Error::Process { message } => {
                ExecutionError::ExternalSolverProcess {
                    backend: backend.clone(),
                    message,
                }
            }
            arco_scip::Error::NoFeasibleSolution { status } => ExecutionError::NoFeasibleSolution {
                backend: backend.clone(),
                status,
            },
        })?;

        Ok(AdapterSolveOutput {
            status: match solution.status {
                arco_scip::SolveStatus::Optimal => SolveStatus::Optimal,
                arco_scip::SolveStatus::Infeasible => SolveStatus::Infeasible,
                arco_scip::SolveStatus::TimeLimit => SolveStatus::TimeLimit,
                arco_scip::SolveStatus::Failed => SolveStatus::Failed,
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

pub(crate) fn adapter_for_selection(
    selection: &ResolvedSelection,
    log_to_console: bool,
    profile: Option<&SolverProfile>,
) -> Result<Box<dyn OptimizationAdapter>, String> {
    match selection.transport {
        SolverTransport::Embedded => match selection.family.as_str() {
            "highs" => Ok(Box::new(HighsArcoAdapter::with_console_log(log_to_console))),
            "scip" => Ok(Box::new(ScipArcoAdapter::with_native_profile(
                log_to_console,
                profile.map_or_else(SolverConfig::default, |value| value.options.clone()),
            ))),
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

#[cfg(test)]
mod tests {
    use super::adapter_for_selection;
    use arco_ops::solve::{ResolvedSelection, SolverConfig, SolverProfile, SolverTransport};

    #[test]
    fn embedded_highs_selection_returns_highs_adapter() {
        let selection = ResolvedSelection {
            token: "highs".to_string(),
            family: "highs".to_string(),
            profile: None,
            transport: SolverTransport::Embedded,
        };

        let adapter = adapter_for_selection(&selection, false, None)
            .unwrap_or_else(|err| panic!("unexpected selection error: {err}"));

        assert_eq!(adapter.backend_name(), "arco-rust-highs");
    }

    #[test]
    fn embedded_scip_selection_returns_scip_adapter() {
        let selection = ResolvedSelection {
            token: "scip".to_string(),
            family: "scip".to_string(),
            profile: Some("scip-local".to_string()),
            transport: SolverTransport::Embedded,
        };
        let profile = SolverProfile {
            name: "scip-local".to_string(),
            family: "scip".to_string(),
            transport: SolverTransport::Embedded,
            executable: None,
            arguments: Vec::new(),
            environment: Default::default(),
            options: SolverConfig::default(),
        };

        let adapter = adapter_for_selection(&selection, true, Some(&profile))
            .unwrap_or_else(|err| panic!("unexpected selection error: {err}"));

        assert_eq!(adapter.backend_name(), arco_scip::BACKEND_NAME);
    }

    #[test]
    fn unsupported_family_returns_error() {
        let selection = ResolvedSelection {
            token: "xpress".to_string(),
            family: "xpress".to_string(),
            profile: None,
            transport: SolverTransport::Embedded,
        };

        let error = match adapter_for_selection(&selection, false, None) {
            Ok(_) => panic!("unsupported family must return error"),
            Err(error) => error,
        };

        assert!(error.contains("embedded solver family 'xpress' is not available"));
    }
}
