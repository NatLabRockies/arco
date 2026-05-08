use arco_highs::HighsModelViewBackend;
use arco_ops::execution::{OptimizationAdapter, RustArcoAdapter, ScipArcoAdapter};
use arco_ops::solve::{ResolvedSelection, SolverProfile, SolverTransport};
use arco_solver::{ModelViewBackendRegistry, SolverRegistry};

pub fn register_builtin_solver_families(registry: &mut SolverRegistry) {
    arco_scip::register_solver_family(registry);
}

pub fn register_builtin_model_view_backends(registry: &mut ModelViewBackendRegistry<'_>) {
    let highs = Box::leak(Box::new(HighsModelViewBackend));
    registry.register(highs);
}

pub fn adapter_for_selection(
    selection: &ResolvedSelection,
    log_to_console: bool,
    profile: Option<&SolverProfile>,
) -> Result<Box<dyn OptimizationAdapter>, String> {
    match selection.transport {
        SolverTransport::Embedded => match selection.family.as_str() {
            "highs" => Ok(Box::new(RustArcoAdapter::with_console_log(log_to_console))),
            family => Err(format!(
                "embedded solver family '{family}' is not available"
            )),
        },
        SolverTransport::ExternalProcess => match selection.family.as_str() {
            "scip" => Ok(Box::new(ScipArcoAdapter::with_external_process_profile(
                log_to_console,
                profile.and_then(|value| value.executable.clone()),
                profile.map_or_else(Vec::new, |value| value.arguments.clone()),
                profile.map_or_else(Default::default, |value| value.environment.clone()),
            ))),
            family => Err(format!(
                "external-process solver family '{family}' is not available"
            )),
        },
    }
}
