use arco_highs::{HighsModelViewBackend, solve_model_view};
use arco_solver::{LpAlgorithm, SolverConfig, SolverError, check_small_lp, small_lp_model};

#[test]
fn solves_with_supported_lp_algorithms() {
    let backend = HighsModelViewBackend;
    for algorithm in [
        LpAlgorithm::Automatic,
        LpAlgorithm::PrimalSimplex,
        LpAlgorithm::DualSimplex,
        LpAlgorithm::Barrier,
        LpAlgorithm::BarrierWithCrossover,
        LpAlgorithm::PrimalDualFirstOrder,
    ] {
        let config = SolverConfig::new()
            .with_log_to_console(false)
            .with_lp_algorithm(algorithm);
        let report = check_small_lp(&backend, &config)
            .unwrap_or_else(|error| panic!("HiGHS should solve with {algorithm:?}: {error}"));
        assert!((report.objective_value - 2.0).abs() < 1e-9);
    }
}

#[test]
fn rejects_unsupported_concurrent_algorithm() {
    let model = small_lp_model();
    let config = SolverConfig::new().with_lp_algorithm(LpAlgorithm::Concurrent);

    let error = solve_model_view(&model, &config)
        .expect_err("HiGHS should reject unsupported concurrent selection");
    assert!(matches!(
        error,
        SolverError::InvalidSettings(message)
            if message.contains("concurrent") && message.contains("HiGHS backend")
    ));
}
