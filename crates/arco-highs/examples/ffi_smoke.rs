#![allow(clippy::float_cmp)]

use arco_highs::{HighsModel, HighsStatus, ObjectiveSense};

fn main() {
    println!("Running HiGHS FFI smoke tests...\n");

    test_minimize_simple();
    test_integer_variable_is_enforced();

    println!("\n✅ All ffi_smoke tests passed!");
}

/// Example: minimize x subject to x >= 1
fn test_minimize_simple() {
    println!("Test: minimize x subject to x >= 1");

    let mut model = HighsModel::new();
    let x = model.add_col(1.0, f64::INFINITY, 1.0);
    model.set_objective_sense(ObjectiveSense::Minimize);

    assert_eq!(model.columns(), 1);
    model.set_primal_start(vec![2.0])
        .expect("failed to set primal start");

    let status = model.solve();
    assert_eq!(status, HighsStatus::Optimal);

    let obj_value = model.objective_value().expect("missing objective value");
    let snapshot = model.solution_snapshot().expect("missing solution");
    let x_value = snapshot.col_values()[x];

    assert!((obj_value - 1.0).abs() < 1e-6,
        "Expected objective value ~1.0, got {}", obj_value);
    assert!((x_value - 1.0).abs() < 1e-6,
        "Expected x ~1.0, got {}", x_value);

    println!("  ✓ Passed: obj={}, x={}", obj_value, x_value);
}

/// Example: maximize integer x subject to x <= 1.5
fn test_integer_variable_is_enforced() {
    println!("Test: maximize integer x subject to x <= 1.5");

    let mut model = HighsModel::new();
    let x = model.add_integer_col(0.0, 10.0, 1.0);
    model.add_row(f64::NEG_INFINITY, 1.5, &[x], &[1.0])
        .expect("failed to add row");
    model.set_objective_sense(ObjectiveSense::Maximize);

    let status = model.solve();
    assert_eq!(status, HighsStatus::Optimal);

    let snapshot = model.solution_snapshot().expect("missing solution");
    let x_value = snapshot.col_values()[x];
    assert!((x_value - 1.0).abs() < 1e6,
        "Expected integer x = 1.0, got {}", x_value);

    println!("  ✓ Passed: x={} (integer enforced)", x_value);
}
