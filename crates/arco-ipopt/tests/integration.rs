#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_expr::VariableId;
use arco_ipopt::Solver;

/// Test: minimize 2x + 3y subject to x + y >= 5, x,y >= 0
#[test]
fn test_simple_lp() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .unwrap();

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(5.0, f64::INFINITY),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();
    model.set_coefficient(y, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 2.0), (y, 3.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).expect("Failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("Failed to solve");

    assert!(
        (solution.objective_value() - 10.0).abs() < 1e-4,
        "Expected objective value 10.0, got {}",
        solution.objective_value()
    );
}

/// Test: maximize x subject to x <= 10
#[test]
fn test_maximize_lp() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 10.0),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).expect("Failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("Failed to solve");

    assert!(
        (solution.objective_value() - 10.0).abs() < 1e-4,
        "Expected objective value 10.0, got {}",
        solution.objective_value()
    );
}

/// Test: dual values have correct lengths and are finite
#[test]
fn test_dual_values() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
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

    let num_variables = model.num_variables();
    let num_constraints = model.num_constraints();

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);
    let solution = solver.solve().unwrap();

    assert_eq!(solution.variable_duals().len(), num_variables);
    assert_eq!(solution.constraint_duals().len(), num_constraints);
    assert!(solution
        .variable_duals()
        .iter()
        .all(|value| value.is_finite()));
    assert!(solution
        .constraint_duals()
        .iter()
        .all(|value| value.is_finite()));
}

/// Test: model with integer variable is rejected
#[test]
fn test_integer_rejection() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::integer(Bounds::new(0.0, 10.0)))
        .unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);
    let result = solver.solve();
    assert!(result.is_err(), "Should reject integer variables");
}

/// Test: infeasible model returns SolveFailure
#[test]
fn test_infeasible() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    // x >= 20 AND x <= 10 (infeasible)
    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(20.0, f64::INFINITY),
        })
        .unwrap();
    model.set_coefficient(x, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);
    let result = solver.solve();
    assert!(result.is_err(), "Infeasible problem should fail to solve");
}

/// Test: primal start (warm-start hints) storage, validation, clear, solve
#[test]
fn test_primal_start() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
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

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);

    // Storage
    let hints = vec![(VariableId::new(0), 2.0), (VariableId::new(1), 1.0)];
    assert!(solver.set_primal_start(&hints).is_ok());
    assert_eq!(solver.get_primal_start(), Some(hints.as_slice()));

    // Validation
    let bad_hints = vec![(VariableId::new(9999), 0.5)];
    assert!(solver.set_primal_start(&bad_hints).is_err());

    // Clear
    solver.set_primal_start(&hints).unwrap();
    solver.clear_primal_start();
    assert!(solver.get_primal_start().is_none());

    // Solve with hints
    solver.set_primal_start(&hints).unwrap();
    let solution = solver.solve().unwrap();
    assert!(
        (solution.objective_value() - 0.0).abs() < 1e-4,
        "Expected objective value 0.0, got {}",
        solution.objective_value()
    );
}

/// Test: solution status methods
#[test]
fn test_solution_status_methods() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
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
    solver.set_log_to_console(false);
    let solution = solver.solve().unwrap();

    assert!(solution.is_optimal(), "Solution should be optimal");
    assert!(solution.is_feasible(), "Solution should be feasible");
    assert!(
        !solution.is_infeasible(),
        "Solution should not be infeasible"
    );
    assert!(!solution.is_unbounded(), "Solution should not be unbounded");

    let status_str = solution.status_string();
    assert!(
        status_str == "optimal" || status_str == "acceptable",
        "Status string should be 'optimal' or 'acceptable', got '{}'",
        status_str
    );
}

/// Test: solution metadata (solve_time > 0)
#[test]
fn test_solution_metadata() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
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
    solver.set_log_to_console(false);
    let solution = solver.solve().unwrap();

    assert!(
        solution.solve_time_seconds() > 0.0,
        "Solve time should be positive"
    );
}
