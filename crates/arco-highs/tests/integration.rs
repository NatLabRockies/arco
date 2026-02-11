#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_expr::VariableId;
use arco_highs::Solver;

/// Test: minimize 2x + 3y subject to x + y >= 5, x,y >= 0
#[test]
fn test_simple_lp() {
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

    // Create solver and solve
    let mut solver = Solver::new(model).expect("Failed to create solver");
    let solution = solver.solve().expect("Failed to solve");

    // Expected optimal solution: x = 5, y = 0, objective = 10.

    assert!(
        (solution.objective_value() - 10.0).abs() < 1e-6,
        "Expected objective value 10.0, got {}",
        solution.objective_value()
    );
}

/// Test: maximize integer x subject to x <= 1.5, x integer
#[test]
fn test_integer_variable_solution() {
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

    let mut solver = Solver::new(model).expect("Failed to create solver");
    let solution = solver.solve().expect("Failed to solve");

    let x_value = solution
        .get_primal(x.inner() as usize)
        .expect("missing primal value");
    assert!(
        (x_value - 1.0).abs() < 1e-6,
        "Expected integer x = 1.0, got {}",
        x_value
    );
    assert!(
        (solution.objective_value() - 1.0).abs() < 1e-6,
        "Expected integer objective value 1.0, got {}",
        solution.objective_value()
    );
}

/// Helper to build a simple model for testing warm-start
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

#[test]
fn test_primal_start_storage() {
    let model = build_simple_model();

    let mut solver = Solver::new(model).unwrap();
    let hints = vec![(VariableId::new(0), 0.5), (VariableId::new(1), 1.0)];
    assert!(solver.set_primal_start(&hints).is_ok());
    assert_eq!(solver.get_primal_start(), Some(hints.as_slice()));
}

#[test]
fn test_primal_start_validation() {
    let model = build_simple_model();

    let mut solver = Solver::new(model).unwrap();
    let invalid_hints = vec![(VariableId::new(9999), 0.5)]; // Non-existent variable
    assert!(solver.set_primal_start(&invalid_hints).is_err());
}

#[test]
fn test_primal_start_clear() {
    let model = build_simple_model();

    let mut solver = Solver::new(model).unwrap();
    let hints = vec![(VariableId::new(0), 0.5)];
    solver.set_primal_start(&hints).unwrap();
    assert!(solver.get_primal_start().is_some());
    solver.clear_primal_start();
    assert!(solver.get_primal_start().is_none());
}

#[test]
fn test_primal_start_solve() {
    let model = build_simple_model();

    let mut solver = Solver::new(model).unwrap();
    let hints = vec![(VariableId::new(0), 2.0), (VariableId::new(1), 1.0)];
    solver.set_primal_start(&hints).unwrap();

    let solution = solver.solve().unwrap();
    assert!(
        (solution.objective_value() - 0.0).abs() < 1e-6,
        "Expected objective value 0.0, got {}",
        solution.objective_value()
    );
}

#[test]
fn test_dual_values_exposed() {
    let model = build_simple_model();
    let num_variables = model.num_variables();
    let num_constraints = model.num_constraints();

    let mut solver = Solver::new(model).unwrap();
    let solution = solver.solve().unwrap();

    assert_eq!(solution.variable_duals().len(), num_variables);
    assert_eq!(solution.constraint_duals().len(), num_constraints);
    assert!(
        solution
            .variable_duals()
            .iter()
            .all(|value| value.is_finite())
    );
    assert!(
        solution
            .constraint_duals()
            .iter()
            .all(|value| value.is_finite())
    );
}

/// Test solution metadata accessors
#[test]
fn test_solution_metadata_accessors() {
    let model = build_simple_model();

    let start_time = std::time::Instant::now();
    let mut solver = Solver::new(model).unwrap();
    let solution = solver.solve().unwrap();
    let elapsed = start_time.elapsed();

    // Test timing accessor
    let solve_time = solution.solve_time_seconds();
    assert!(solve_time > 0.0, "Solve time should be positive");
    assert!(
        solve_time <= elapsed.as_secs_f64() + 0.1,
        "Solve time should be reasonable"
    );

    // Test iteration accessors
    let simplex_iters = solution.simplex_iterations();
    let barrier_iters = solution.barrier_iterations();
    let total_iters = solution.total_iterations();

    assert_eq!(
        total_iters,
        simplex_iters + barrier_iters,
        "Total iterations should equal sum of simplex and barrier iterations"
    );
    // For small trivial problems, HiGHS might solve without iterations
    // This is expected behavior - solution is still valid
    assert!(
        total_iters <= 10000,
        "Total iterations should be reasonable"
    );

    // Test tolerance accessors
    let primal_tol = solution.primal_feasibility_tolerance();
    let dual_tol = solution.dual_feasibility_tolerance();

    assert!(primal_tol.is_finite(), "Primal tolerance should be finite");
    assert!(dual_tol.is_finite(), "Dual tolerance should be finite");

    // Should return default tolerance values
    assert_eq!(primal_tol, 1e-6, "Primal tolerance should be default");
    assert_eq!(dual_tol, 1e-6, "Dual tolerance should be default");

    // Test MIP gap (should be 0.0 or infinity for LP problems)
    let mip_gap = solution.mip_gap();
    // MIP gap might be infinity for pure LP problems (no integer variables)
    assert!(mip_gap >= 0.0, "MIP gap should be non-negative");
}

/// Test solution status methods
#[test]
fn test_solution_status_methods() {
    let mut model = Model::new();

    // Add variable and constraint for feasible problem
    let x = model
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

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).unwrap();
    let solution = solver.solve().unwrap();

    // Test status methods
    assert!(solution.is_optimal(), "Solution should be optimal");
    assert!(solution.is_feasible(), "Solution should be feasible");
    assert!(
        !solution.is_infeasible(),
        "Solution should not be infeasible"
    );
    assert!(!solution.is_unbounded(), "Solution should not be unbounded");

    // Test status string
    let status_str = solution.status_string();
    assert_eq!(status_str, "optimal", "Status string should be 'optimal'");
}

/// Test solution accessor edge cases
#[test]
fn test_solution_accessor_edge_cases() {
    let mut model = Model::new();

    // Add variable
    let x = model
        .add_variable(Variable {
            bounds: Bounds::new(0.0, 10.0),
            is_integer: false,
            is_active: true,
        })
        .unwrap();

    // Create infeasible constraint: x >= 10 AND x <= 5
    let constraint1 = model
        .add_constraint(Constraint {
            bounds: Bounds::new(10.0, f64::INFINITY),
        })
        .unwrap();
    let constraint2 = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 5.0),
        })
        .unwrap();

    model.set_coefficient(x, constraint1, 1.0).unwrap();
    model.set_coefficient(x, constraint2, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).unwrap();
    let solution = solver.solve();

    // For infeasible problems, solver should return error
    assert!(solution.is_err(), "Infeasible problem should fail to solve");

    // Test status methods on successful solution for edge cases
    // Build a feasible unbounded problem
    let mut unbounded_model = Model::new();
    let y = unbounded_model
        .add_variable(Variable {
            bounds: Bounds::new(0.0, f64::INFINITY),
            is_integer: false,
            is_active: true,
        })
        .unwrap();

    let unbounded_objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(y, 1.0)],
    };
    unbounded_model.set_objective(unbounded_objective).unwrap();

    let mut unbounded_solver = Solver::new(unbounded_model).unwrap();
    let unbounded_result = unbounded_solver.solve();

    assert!(
        unbounded_result.is_err(),
        "Unbounded problem should fail to solve"
    );
}
