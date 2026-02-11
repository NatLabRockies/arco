use arco_highs::{HighsModel, HighsStatus, ObjectiveSense};

#[test]
fn test_minimize_simple() {
    // Initialize tracing for diagnostics
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    // Create a simple minimization problem: minimize x
    // subject to: x >= 1
    let mut model = HighsModel::new();

    // Add a variable x with bounds [1, infinity)
    let x = model.add_col(1.0, f64::INFINITY, 1.0);

    // Objective: minimize x (coefficient is 1.0)
    // Already set in add_col

    // Set objective sense to minimize
    model.set_objective_sense(ObjectiveSense::Minimize);

    assert_eq!(model.columns(), 1);
    model
        .set_primal_start(vec![2.0])
        .expect("failed to set primal start");

    // Solve
    let status = model.solve();

    // Verify
    assert_eq!(status, HighsStatus::Optimal);
    assert_eq!(model.columns(), 0);

    let obj_value = model.objective_value().expect("missing objective value");
    let snapshot = model.solution_snapshot().expect("missing solution");
    let x_value = snapshot.col_values()[x];

    // The optimal value should be approximately 1.0
    assert!(
        (obj_value - 1.0).abs() < 1e-6,
        "Expected objective value ~1.0, got {}",
        obj_value
    );
    assert!(
        (x_value - 1.0).abs() < 1e-6,
        "Expected x ~1.0, got {}",
        x_value
    );
}

#[test]
fn test_integer_variable_is_enforced() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let mut model = HighsModel::new();
    let x = model.add_integer_col(0.0, 10.0, 1.0);
    model
        .add_row(f64::NEG_INFINITY, 1.5, &[x], &[1.0])
        .expect("failed to add row");
    model.set_objective_sense(ObjectiveSense::Maximize);

    let status = model.solve();
    assert_eq!(status, HighsStatus::Optimal);

    let snapshot = model.solution_snapshot().expect("missing solution");
    let x_value = snapshot.col_values()[x];
    assert!(
        (x_value - 1.0).abs() < 1e-6,
        "Expected integer x = 1.0, got {}",
        x_value
    );
}

#[test]
fn test_primal_start_length_mismatch() {
    let mut model = HighsModel::new();
    model.add_col(0.0, 1.0, 1.0);

    assert!(model.set_primal_start(vec![0.0, 1.0]).is_err());
}
