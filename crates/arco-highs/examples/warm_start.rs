#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_expr::VariableId;
use arco_highs::Solver;

fn build_simple_model() -> Model {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable {
            bounds: Bounds::new(0.0, 10.0),
            is_integer: false,
            is_active: true,
        })
        .unwrap();

    let y = model
        .add_variable(Variable {
            bounds: Bounds::new(0.0, 10.0),
            is_integer: false,
            is_active: true,
        })
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(0.0, 5.0),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();
    model.set_coefficient(y, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0), (y, 1.0)],
    };
    model.set_objective(objective).unwrap();

    model
}

/// Example: Using warm start (primal start) for faster solves
fn main() {
    println!("Running warm start example...\n");

    let model = build_simple_model();
    let mut solver = Solver::new(model).unwrap();

    // Set warm start hints
    let hints = vec![(VariableId::new(0), 2.0), (VariableId::new(1), 1.0)];
    solver.set_primal_start(&hints).unwrap();
    println!("Set primal start hints: x0=2.0, x1=1.0");

    // Solve with warm start
    let solution = solver.solve().unwrap();
    let obj_value = solution.objective_value();

    println!("Solution with warm start:");
    println!("  Objective value: {}", obj_value);
    println!("\n✅ Warm start example passed!");
}
