use arco_highs::HighsModelViewBackend;
use arco_ops::execution::{
    AdapterSolveOutput, ExecutionError, OptimizationAdapter, ScalarArtifactValue, SolveStatus,
    VariableArtifactValue, VariableInstanceArtifactValue, adapter_output_from_model_view_solution,
    build_model,
};
use arco_ops::portable_problem_from_algebraic;
use arco_ops::solve::{
    ModelViewBackend, ResolvedSelection, SolverConfig, SolverProfile, SolverTransport,
};
use arco_solver::{ModelViewBackendRegistry, SolverRegistry};
use std::collections::BTreeMap;

pub fn register_builtin_solver_families(registry: &mut SolverRegistry) {
    arco_scip::register_solver_family(registry);
}

pub fn register_builtin_model_view_backends(registry: &mut ModelViewBackendRegistry<'_>) {
    let highs = Box::leak(Box::new(HighsModelViewBackend));
    registry.register(highs);
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
struct ScipExternalAdapter {
    log_to_console: bool,
    executable: Option<String>,
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
}

impl ScipExternalAdapter {
    fn with_external_process_profile(
        log_to_console: bool,
        executable: Option<String>,
        arguments: Vec<String>,
        environment: BTreeMap<String, String>,
    ) -> Self {
        Self {
            log_to_console,
            executable,
            arguments,
            environment,
        }
    }
}

impl OptimizationAdapter for ScipExternalAdapter {
    fn backend_name(&self) -> &'static str {
        arco_scip::BACKEND_NAME
    }

    fn solve(
        &self,
        problem: &arco_ops::compile::compile::CompiledProblem,
        include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        let backend = self.backend_name().to_string();
        let options = arco_scip::ExternalProcessOptions {
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
            arco_scip::Error::Io { source } => ExecutionError::ExternalSolverIo {
                backend: backend.clone(),
                source,
            },
            arco_scip::Error::Process { message } => ExecutionError::ExternalSolverProcess {
                backend: backend.clone(),
                message,
            },
            arco_scip::Error::Parse { message } => ExecutionError::ExternalSolverParse {
                backend: backend.clone(),
                message,
            },
            arco_scip::Error::NoFeasibleSolution { status } => ExecutionError::NoFeasibleSolution {
                backend: backend.clone(),
                status,
            },
        })?;

        Ok(AdapterSolveOutput {
            status: match solution.status {
                arco_scip::SolveStatus::Optimal => SolveStatus::Optimal,
                arco_scip::SolveStatus::Infeasible => SolveStatus::Infeasible,
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

pub fn adapter_for_selection(
    selection: &ResolvedSelection,
    log_to_console: bool,
    profile: Option<&SolverProfile>,
) -> Result<Box<dyn OptimizationAdapter>, String> {
    match selection.transport {
        SolverTransport::Embedded => match selection.family.as_str() {
            "highs" => Ok(Box::new(HighsArcoAdapter::with_console_log(log_to_console))),
            family => Err(format!(
                "embedded solver family '{family}' is not available"
            )),
        },
        SolverTransport::ExternalProcess => match selection.family.as_str() {
            "scip" => Ok(Box::new(
                ScipExternalAdapter::with_external_process_profile(
                    log_to_console,
                    profile.and_then(|value| value.executable.clone()),
                    profile.map_or_else(Vec::new, |value| value.arguments.clone()),
                    profile.map_or_else(Default::default, |value| value.environment.clone()),
                ),
            )),
            family => Err(format!(
                "external-process solver family '{family}' is not available"
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::adapter_for_selection;
    use arco_ops::solve::{ResolvedSelection, SolverConfig, SolverProfile, SolverTransport};
    use std::collections::BTreeMap;

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
    fn external_scip_selection_returns_scip_adapter() {
        let selection = ResolvedSelection {
            token: "scip".to_string(),
            family: "scip".to_string(),
            profile: Some("scip-local".to_string()),
            transport: SolverTransport::ExternalProcess,
        };
        let profile = SolverProfile {
            name: "scip-local".to_string(),
            family: "scip".to_string(),
            transport: SolverTransport::ExternalProcess,
            executable: Some("/usr/bin/scip".to_string()),
            arguments: vec!["-q".to_string()],
            environment: BTreeMap::from([("SCIP_SETTINGS".to_string(), "fast".to_string())]),
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
