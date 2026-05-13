use arco_solver::{SolverDiagnostic, SolverError, SolverModelStats};

#[test]
fn solver_specific_display_uses_message_without_extra_wrapper() {
    let error = SolverError::SolverSpecific("clean message".to_string());

    assert_eq!(error.to_string(), "clean message");
}

#[test]
fn model_size_limit_diagnostic_display_includes_actual_size_and_actions() {
    let error = SolverError::Diagnostic(SolverDiagnostic::ModelSizeLimit {
        solver: "Example Solver".to_string(),
        operation: "optimize".to_string(),
        return_code: 120,
        limit: 5000,
        model: SolverModelStats {
            variables: 2300,
            constraints: 2800,
            coefficients: 12_345,
        },
    });
    let message = error.to_string();

    assert!(message.contains("Example Solver cannot solve this model"));
    assert!(message.contains("rows: 2800"));
    assert!(message.contains("columns: 2300"));
    assert!(message.contains("nonzeros: 12345"));
    assert!(message.contains("rows + columns: 5100"));
    assert!(message.contains("limit: 5000"));
    assert!(message.contains("arco solver set highs"));
    assert!(message.contains("Details: solver=Example Solver, operation=optimize, rc=120"));
}
