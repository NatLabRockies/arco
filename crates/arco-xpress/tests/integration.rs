#![allow(clippy::float_cmp)]

use arco_model::types::Bounds;
use arco_model::{Constraint, Model, Objective, Sense, Variable, VariableId};
use arco_xpress::Solver;

/// Helper to build a simple model used by multiple tests:
/// minimize x + y, subject to x + y <= 5, x in [0,10], y in [0,10].
fn build_simple_model() -> Model {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .expect("add variable x");

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .expect("add variable y");

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(0.0, 5.0),
        })
        .expect("add constraint");

    model.set_coefficient(x, constraint, 1.0).expect("coeff x");
    model.set_coefficient(y, constraint, 1.0).expect("coeff y");

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0), (y, 1.0)],
    };
    model.set_objective(objective).expect("set objective");

    model
}

/// Test: minimize 2x + 3y subject to x + y >= 5, x,y >= 0
/// Expected optimal: x = 5, y = 0, objective = 10.0
#[test]
fn test_simple_lp() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .expect("add variable x");

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .expect("add variable y");

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(5.0, f64::INFINITY),
        })
        .expect("add constraint");

    model.set_coefficient(x, constraint, 1.0).expect("coeff x");
    model.set_coefficient(y, constraint, 1.0).expect("coeff y");

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 2.0), (y, 3.0)],
    };
    model.set_objective(objective).expect("set objective");

    let mut solver = Solver::new(model).expect("failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("failed to solve");

    assert!(
        (solution.objective_value() - 10.0).abs() < 1e-4,
        "Expected objective value 10.0, got {}",
        solution.objective_value()
    );
}

/// Test: maximize x subject to x <= 10
/// Expected optimal: x = 10, objective = 10.0
#[test]
fn test_maximize_lp() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .expect("add variable x");

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 10.0),
        })
        .expect("add constraint");

    model.set_coefficient(x, constraint, 1.0).expect("coeff x");

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).expect("set objective");

    let mut solver = Solver::new(model).expect("failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("failed to solve");

    assert!(
        (solution.objective_value() - 10.0).abs() < 1e-4,
        "Expected objective value 10.0, got {}",
        solution.objective_value()
    );
}

/// Test: maximize integer x subject to x <= 1.5, x integer, x in [0,10]
/// Expected optimal: x = 1.0, objective = 1.0
#[test]
fn test_integer_variable() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::integer(Bounds::new(0.0, 10.0)))
        .expect("add integer variable x");

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 1.5),
        })
        .expect("add constraint");

    model.set_coefficient(x, constraint, 1.0).expect("coeff x");

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).expect("set objective");

    let mut solver = Solver::new(model).expect("failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("failed to solve");

    let x_value = solution
        .get_primal(x.inner() as usize)
        .expect("missing primal value for x");

    assert!(
        (x_value - 1.0).abs() < 1e-4,
        "Expected integer x = 1.0, got {}",
        x_value
    );
    assert!(
        (solution.objective_value() - 1.0).abs() < 1e-4,
        "Expected objective value 1.0, got {}",
        solution.objective_value()
    );
}

/// Test: dual values have correct lengths and are finite
/// Model: minimize x + y, subject to x + y <= 5, x in [0,10], y in [0,10]
/// (2 variables, 1 constraint)
#[test]
fn test_dual_values() {
    let model = build_simple_model();
    let num_variables = model.num_variables();
    let num_constraints = model.num_constraints();

    let mut solver = Solver::new(model).expect("failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("failed to solve");

    assert_eq!(
        solution.variable_duals().len(),
        num_variables,
        "Variable duals length should match number of variables"
    );
    assert_eq!(
        solution.constraint_duals().len(),
        num_constraints,
        "Constraint duals length should match number of constraints"
    );
    assert!(
        solution
            .variable_duals()
            .iter()
            .all(|value| value.is_finite()),
        "All variable duals should be finite"
    );
    assert!(
        solution
            .constraint_duals()
            .iter()
            .all(|value| value.is_finite()),
        "All constraint duals should be finite"
    );
}

/// Test: infeasible model returns error
/// x in [0,10] with constraint x >= 20 is infeasible
#[test]
fn test_infeasible() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .expect("add variable x");

    // x >= 20 but x has upper bound 10, so infeasible
    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(20.0, f64::INFINITY),
        })
        .expect("add constraint");

    model.set_coefficient(x, constraint, 1.0).expect("coeff x");

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).expect("set objective");

    let mut solver = Solver::new(model).expect("failed to create solver");
    solver.set_log_to_console(false);
    let result = solver.solve();

    assert!(result.is_err(), "Infeasible problem should fail to solve");
}

/// Test: primal start (warm-start hints) storage, validation, clear, then solve with hints
#[test]
fn test_primal_start() {
    let model = build_simple_model();

    let mut solver = Solver::new(model).expect("failed to create solver");
    solver.set_log_to_console(false);

    // Storage: set valid hints and verify they are stored
    let hints = vec![(VariableId::new(0), 2.0), (VariableId::new(1), 1.0)];
    assert!(
        solver.set_primal_start(&hints).is_ok(),
        "Setting valid primal start should succeed"
    );
    assert_eq!(
        solver.get_primal_start(),
        Some(hints.as_slice()),
        "Stored hints should match what was set"
    );

    // Validation: reject invalid variable IDs
    let bad_hints = vec![(VariableId::new(9999), 0.5)];
    assert!(
        solver.set_primal_start(&bad_hints).is_err(),
        "Setting primal start with invalid variable ID should fail"
    );

    // Clear: primal start should be gone after clearing
    solver
        .set_primal_start(&hints)
        .expect("re-setting valid hints should succeed");
    solver.clear_primal_start();
    assert!(
        solver.get_primal_start().is_none(),
        "Primal start should be None after clearing"
    );

    // Solve with hints: solver should still produce a valid solution
    solver
        .set_primal_start(&hints)
        .expect("setting hints before solve should succeed");
    let solution = solver
        .solve()
        .expect("solve with primal start should succeed");
    assert!(
        (solution.objective_value() - 0.0).abs() < 1e-4,
        "Expected objective value 0.0, got {}",
        solution.objective_value()
    );
}

/// Test: solution metadata (solve_time_seconds >= 0)
#[test]
fn test_solution_metadata() {
    let model = build_simple_model();

    let mut solver = Solver::new(model).expect("failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("failed to solve");

    assert!(
        solution.solve_time_seconds() >= 0.0,
        "Solve time should be non-negative, got {}",
        solution.solve_time_seconds()
    );
    assert!(solution.is_optimal(), "Solution should be optimal");
    assert!(solution.is_feasible(), "Solution should be feasible");
    assert!(
        !solution.is_infeasible(),
        "Solution should not be infeasible"
    );
    assert!(!solution.is_unbounded(), "Solution should not be unbounded");
}
