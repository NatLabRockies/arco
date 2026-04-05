#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_highs::Solver;

/// Example: minimize 2x + 3y subject to x + y >= 5, x,y >= 0
fn main() {
    println!("Running simple LP example...\n");

    // Build model
    let mut model = Model::new();

    // Add variables: x and y, both continuous, non-negative
    let x = model
        .add_variable(Variable {
            bounds: Bounds::new(0.0, f64::INFINITY),
            is_integer: false,
            is_active: true,
        })
        .unwrap();

    let y = model
        .add_variable(Variable {
            bounds: Bounds::new(0.0, f64::INFINITY),
            is_integer: false,
            is_active: true,
        })
        .unwrap();

    // Add constraint: x + y >= 5
    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(5.0, f64::INFINITY),
        })
        .unwrap();

    // Set coefficients: x and y both have coefficient 1 in the constraint
    model.set_coefficient(x, constraint, 1.0).unwrap();
    model.set_coefficient(y, constraint, 1.0).unwrap();

    // Set objective: minimize 2x + 3y
    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 2.0), (y, 3.0)],
    };
    model.set_objective(objective).unwrap();

    println!("Model built:");
    println!("  Variables: x, y (continuous, non-negative)");
    println!("  Constraint: x + y >= 5");
    println!("  Objective: minimize 2x + 3y\n");

    // Create solver and solve
    let mut solver = Solver::new(model).expect("Failed to create solver");
    let solution = solver.solve().expect("Failed to solve");

    // Expected optimal solution: x = 5, y = 0, objective = 10.
    let obj_value = solution.objective_value();
    println!("Solution found:");
    println!("  Objective value: {}", obj_value);
    println!("  Expected: 10.0\n");

    assert!(
        (obj_value - 10.0).abs() < 1e-6,
        "Expected objective value 10.0, got {}",
        obj_value
    );

    println!("✅ Simple LP example passed!");
}
