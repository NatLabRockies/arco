#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
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

/// Example: Inspecting solution metadata and dual values
fn main() {
    println!("Running solution inspection example...\n");

    let model = build_simple_model();
    let num_variables = model.num_variables();
    let num_constraints = model.num_constraints();

    let mut solver = Solver::new(model).unwrap();
    let solution = solver.solve().unwrap();

    println!("Solution found! Inspecting results...\n");

    // Dual values
    let var_duals = solution.variable_duals();
    let constr_duals = solution.constraint_duals();
    println!("Dual values:");
    println!("  Variable duals: {} entries", var_duals.len());
    println!("  Constraint duals: {} entries", constr_duals.len());
    assert_eq!(var_duals.len(), num_variables);
    assert_eq!(constr_duals.len(), num_constraints);

    // Timing
    let solve_time = solution.solve_time_seconds();
    println!("\nTiming:");
    println!("  Solve time: {:.6} seconds", solve_time);

    // Iterations
    let simplex_iters = solution.simplex_iterations();
    let barrier_iters = solution.barrier_iterations();
    let total_iters = solution.total_iterations();
    println!("\nIterations:");
    println!("  Simplex: {}", simplex_iters);
    println!("  Barrier: {}", barrier_iters);
    println!("  Total: {}", total_iters);

    // Tolerances
    let primal_tol = solution.primal_feasibility_tolerance();
    let dual_tol = solution.dual_feasibility_tolerance();
    println!("\nTolerances:");
    println!("  Primal: {}", primal_tol);
    println!("  Dual: {}", dual_tol);

    // Status
    println!("\nStatus:");
    println!("  Is optimal: {}", solution.is_optimal());
    println!("  Is feasible: {}", solution.is_feasible());
    println!("  Status string: {}", solution.status_string());

    println!("\n✅ Solution inspection example passed!");
}
