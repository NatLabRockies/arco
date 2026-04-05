#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_highs::Solver;

/// Example: maximize integer x subject to x <= 1.5, x integer
fn main() {
    println!("Running integer variable example...\n");

    let mut model = Model::new();

    let x = model
        .add_variable(Variable {
            bounds: Bounds::new(0.0, 10.0),
            is_integer: true,
            is_active: true,
        })
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 1.5),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    println!("Model built:");
    println!("  Variable: x (integer, 0 <= x <= 10)");
    println!("  Constraint: x <= 1.5");
    println!("  Objective: maximize x\n");

    let mut solver = Solver::new(model).expect("Failed to create solver");
    let solution = solver.solve().expect("Failed to solve");

    let x_value = solution
        .get_primal(x.inner() as usize)
        .expect("missing primal value");

    println!("Solution found:");
    println!("  x = {} (should be 1.0 for integer)", x_value);
    println!("  Objective = {}\n", solution.objective_value());

    assert!(
        (x_value - 1.0).abs() < 1e-6,
        "Expected integer x = 1.0, got {}",
        x_value
    );

    println!("✅ Integer variable example passed!");
}
