use arco_cli::execution::{
    AdapterSolveOutput, ExecutionError, OptimizationAdapter, RustArcoAdapter, ScalarArtifactValue,
    SolveStatus, VariableArtifactValue, execute_problem,
};
use arco_kdl::lowering::lower_program;
use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_file;
use std::path::PathBuf;

#[test]
fn executes_price_taker_battery_fixture_through_rust_arco_adapter()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("price-taker-battery")
        .join("input.kdl");

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;
    let execution_result = execute_problem(&lowered_problem, &RustArcoAdapter::new())?;

    assert_eq!(execution_result.backend, "arco-rust-highs");
    assert_eq!(execution_result.status, SolveStatus::Optimal);
    assert_eq!(execution_result.objective.dsl_name, "ArbitrageProfit");
    assert_eq!(execution_result.objective_sense, "maximize");
    assert!(execution_result.objective.value.is_finite());
    assert!(execution_result.objective.value > 0.0);
    assert_eq!(execution_result.reports.len(), 1);
    assert_eq!(execution_result.reports[0].dsl_name, "ArbitrageRevenue");
    assert!(execution_result.reports[0].value > 0.0);

    let variable_families: Vec<&str> = execution_result
        .variables
        .iter()
        .map(|v| v.dsl_name.as_str())
        .collect();
    assert_eq!(
        variable_families,
        &["charge[a,t]", "discharge[a,t]", "soc[a,t]"]
    );

    Ok(())
}

#[test]
fn executes_simple_electricity_market_storage_fixture_with_expected_dispatch()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("simple-electricity-market-storage")
        .join("input.kdl");

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;
    let execution_result = execute_problem(&lowered_problem, &RustArcoAdapter::new())?;

    assert_eq!(execution_result.backend, "arco-rust-highs");
    assert_eq!(execution_result.status, SolveStatus::Optimal);
    assert_eq!(execution_result.objective.dsl_name, "TotalSystemCost");
    assert_eq!(execution_result.objective_sense, "minimize");
    assert!((execution_result.objective.value - 6_046_000.0).abs() < 1e-6);

    let dispatch = execution_result
        .variables
        .iter()
        .find(|variable| variable.dsl_name == "dispatch[a,t]")
        .ok_or("missing dispatch family")?;
    let soc = execution_result
        .variables
        .iter()
        .find(|variable| variable.dsl_name == "soc[a,t]")
        .ok_or("missing soc family")?;

    let dispatch_points = variable_points(dispatch);
    let soc_points = variable_points(soc);
    let pypsa_dispatch = vec![
        ("dispatch[Coal,1]".to_string(), 35_000.0),
        ("dispatch[Coal,2]".to_string(), 35_000.0),
        ("dispatch[Coal,3]".to_string(), 35_000.0),
        ("dispatch[Coal,4]".to_string(), 35_000.0),
        ("dispatch[Gas,1]".to_string(), 6_900.0),
        ("dispatch[Gas,2]".to_string(), 7_200.0),
        ("dispatch[Gas,3]".to_string(), 8_000.0),
        ("dispatch[Gas,4]".to_string(), 8_000.0),
        ("dispatch[Oil,1]".to_string(), 0.0),
        ("dispatch[Oil,2]".to_string(), 0.0),
        ("dispatch[Oil,3]".to_string(), 0.0),
        ("dispatch[Oil,4]".to_string(), 500.0),
        ("dispatch[PumpedHydro,1]".to_string(), -800.0),
        ("dispatch[PumpedHydro,2]".to_string(), -1_000.0),
        ("dispatch[PumpedHydro,3]".to_string(), 800.0),
        ("dispatch[PumpedHydro,4]".to_string(), 1_000.0),
        ("dispatch[Wind,1]".to_string(), 900.0),
        ("dispatch[Wind,2]".to_string(), 1_800.0),
        ("dispatch[Wind,3]".to_string(), 1_200.0),
        ("dispatch[Wind,4]".to_string(), 1_500.0),
    ];
    let equivalent_dispatch = vec![
        ("dispatch[Coal,1]".to_string(), 35_000.0),
        ("dispatch[Coal,2]".to_string(), 35_000.0),
        ("dispatch[Coal,3]".to_string(), 35_000.0),
        ("dispatch[Coal,4]".to_string(), 35_000.0),
        ("dispatch[Gas,1]".to_string(), 7_100.0),
        ("dispatch[Gas,2]".to_string(), 7_000.0),
        ("dispatch[Gas,3]".to_string(), 8_000.0),
        ("dispatch[Gas,4]".to_string(), 8_000.0),
        ("dispatch[Oil,1]".to_string(), 0.0),
        ("dispatch[Oil,2]".to_string(), 0.0),
        ("dispatch[Oil,3]".to_string(), 0.0),
        ("dispatch[Oil,4]".to_string(), 500.0),
        ("dispatch[PumpedHydro,1]".to_string(), -1_000.0),
        ("dispatch[PumpedHydro,2]".to_string(), -800.0),
        ("dispatch[PumpedHydro,3]".to_string(), 800.0),
        ("dispatch[PumpedHydro,4]".to_string(), 1_000.0),
        ("dispatch[Wind,1]".to_string(), 900.0),
        ("dispatch[Wind,2]".to_string(), 1_800.0),
        ("dispatch[Wind,3]".to_string(), 1_200.0),
        ("dispatch[Wind,4]".to_string(), 1_500.0),
    ];
    let pypsa_soc = vec![
        ("soc[PumpedHydro,1]".to_string(), 800.0),
        ("soc[PumpedHydro,2]".to_string(), 1_800.0),
        ("soc[PumpedHydro,3]".to_string(), 1_000.0),
        ("soc[PumpedHydro,4]".to_string(), 0.0),
    ];
    let equivalent_soc = vec![
        ("soc[PumpedHydro,1]".to_string(), 1_000.0),
        ("soc[PumpedHydro,2]".to_string(), 1_800.0),
        ("soc[PumpedHydro,3]".to_string(), 1_000.0),
        ("soc[PumpedHydro,4]".to_string(), 0.0),
    ];

    assert!(dispatch_points == pypsa_dispatch || dispatch_points == equivalent_dispatch);
    assert!(soc_points == pypsa_soc || soc_points == equivalent_soc);

    Ok(())
}

#[test]
fn execution_rejects_missing_report_mapping() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("pcm")
        .join("input.kdl");

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;
    let adapter = MissingReportAdapter;

    let error = execute_problem(&lowered_problem, &adapter).expect_err("execution should fail");
    assert!(matches!(error, ExecutionError::MissingReportValue { .. }));

    Ok(())
}

struct MissingReportAdapter;

impl OptimizationAdapter for MissingReportAdapter {
    fn backend_name(&self) -> &'static str {
        "missing-report"
    }

    fn solve(
        &self,
        problem: &arco_kdl::lowering::LoweredProblem,
        _include_variable_values: bool,
    ) -> Result<AdapterSolveOutput, ExecutionError> {
        Ok(AdapterSolveOutput {
            status: SolveStatus::Optimal,
            objective_value: ScalarArtifactValue {
                lowered_name: problem.objective.name.clone(),
                value: 0.0,
            },
            report_values: Vec::new(),
            variable_values: problem
                .variables
                .iter()
                .map(|variable| VariableArtifactValue {
                    lowered_name: variable.family.clone(),
                    representative_value: 0.0,
                    values: Vec::new(),
                })
                .collect(),
        })
    }
}

#[test]
fn executes_sdom_fixture_through_full_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/e2e/sdom/input.kdl");

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let lowered = lower_program(&semantic, &parsed.program, &path)?;
    let result = execute_problem(&lowered, &RustArcoAdapter::new())?;

    assert_eq!(result.status, SolveStatus::Optimal);
    assert_eq!(result.objective.dsl_name, "SystemCost");
    assert_eq!(result.objective_sense, "minimize");
    assert!(result.objective.value.is_finite());
    assert!(result.objective.value > 0.0);

    let families: Vec<&str> = result
        .variables
        .iter()
        .map(|v| v.dsl_name.as_str())
        .collect();
    assert!(
        families.contains(&"pv_fraction[a]"),
        "missing pv_fraction: {families:?}"
    );
    assert!(
        families.contains(&"bal_capacity[a]"),
        "missing bal_capacity: {families:?}"
    );
    assert!(
        families.contains(&"energy_cap[a]"),
        "missing energy_cap: {families:?}"
    );
    assert!(
        families.contains(&"charge[a,t]"),
        "missing charge: {families:?}"
    );
    assert!(
        families.contains(&"discharge[a,t]"),
        "missing discharge: {families:?}"
    );
    assert!(families.contains(&"soc[a,t]"), "missing soc: {families:?}");
    assert!(
        families.contains(&"charge_indicator[a,t]"),
        "missing charge_indicator: {families:?}"
    );

    assert_eq!(result.reports.len(), 5, "expected 5 report components");

    Ok(())
}

#[test]
fn sdom_and_sdom_canonical_produce_matching_objectives() -> Result<(), Box<dyn std::error::Error>> {
    let sdom_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/e2e/sdom/input.kdl");
    let canonical_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/e2e/sdom-canonical/input.kdl");

    let sdom_parsed = parse_program_file(&sdom_path)?;
    let sdom_semantic = validate_program(&sdom_parsed.program, &sdom_path)?;
    let sdom_lowered = lower_program(&sdom_semantic, &sdom_parsed.program, &sdom_path)?;
    let sdom_result = execute_problem(&sdom_lowered, &RustArcoAdapter::new())?;

    let canon_parsed = parse_program_file(&canonical_path)?;
    let canon_semantic = validate_program(&canon_parsed.program, &canonical_path)?;
    let canon_lowered = lower_program(&canon_semantic, &canon_parsed.program, &canonical_path)?;
    let canon_result = execute_problem(&canon_lowered, &RustArcoAdapter::new())?;

    assert_eq!(sdom_result.status, SolveStatus::Optimal);
    assert_eq!(canon_result.status, SolveStatus::Optimal);

    let relative_error = (sdom_result.objective.value - canon_result.objective.value).abs()
        / sdom_result.objective.value.abs();
    assert!(
        relative_error < 1e-6,
        "objectives should match: sdom={}, canonical={}, relative_error={}",
        sdom_result.objective.value,
        canon_result.objective.value,
        relative_error
    );

    Ok(())
}

fn variable_points(variable: &arco_cli::execution::MappedVariableResult) -> Vec<(String, f64)> {
    variable
        .values
        .iter()
        .map(|value| (value.lowered_name.clone(), value.value))
        .collect()
}
