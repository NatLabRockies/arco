//! XPRESS Solver Integration Examples
//!
//! This example demonstrates various use cases of the XPRESS solver:
//! - Simple LP minimization
//! - Maximization problems
//! - Integer variable support (MIP solving)
//! - Dual value extraction
//! - Infeasible problem handling
//! - Warm-start hints (primal start)
//! - Solution metadata and status checking

#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_expr::VariableId;
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

fn main() {
    println!("Running XPRESS solver integration examples...\n");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Simple LP
    print!("Test 1: Simple LP minimization... ");
    match test_simple_lp() {
        Ok(_) => { println!("✓ PASSED"); passed += 1; }
        Err(e) => { println!("✗ FAILED: {}", e); failed += 1; }
    }

    // Test 2: Maximize LP
    print!("Test 2: Maximization problem... ");
    match test_maximize_lp() {
        Ok(_) => { println!("✓ PASSED"); passed += 1; }
        Err(e) => { println!("✗ FAILED: {}", e); failed += 1; }
    }

    // Test 3: Integer variable
    print!("Test 3: Integer variable (MIP) support... ");
    match test_integer_variable() {
        Ok(_) => { println!("✓ PASSED"); passed += 1; }
        Err(e) => { println!("✗ FAILED: {}", e); failed += 1; }
    }

    // Test 4: Dual values
    print!("Test 4: Dual value extraction... ");
    match test_dual_values() {
        Ok(_) => { println!("✓ PASSED"); passed += 1; }
        Err(e) => { println!("✗ FAILED: {}", e); failed += 1; }
    }

    // Test 5: Infeasible
    print!("Test 5: Infeasible problem handling... ");
    match test_infeasible() {
        Ok(_) => { println!("✓ PASSED"); passed += 1; }
        Err(e) => { println!("✗ FAILED: {}", e); failed += 1; }
    }

    // Test 6: Primal start
    print!("Test 6: Warm-start hints (primal start)... ");
    match test_primal_start() {
        Ok(_) => { println!("✓ PASSED"); passed += 1; }
        Err(e) => { println!("✗ FAILED: {}", e); failed += 1; }
    }

    // Test 7: Solution metadata
    print!("Test 7: Solution metadata and status... ");
    match test_solution_metadata() {
        Ok(_) => { println!("✓ PASSED"); passed += 1; }
        Err(e) => { println!("✗ FAILED: {}", e); failed += 1; }
    }

    println!("\n========================================");
    println!("Results: {} passed, {} failed", passed, failed);
    println!("========================================");

    if failed > 0 {
        std::process::exit(1);
    }
}

/// Test: minimize 2x + 3y subject to x + y >= 5, x,y >= 0
/// Expected optimal: x = 5, y = 0, objective = 10.0
fn test_simple_lp() -> Result<(), String> {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .map_err(|e| format!("Failed to add variable x: {}", e))?;

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .map_err(|e| format!("Failed to add variable y: {}", e))?;

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(5.0, f64::INFINITY),
        })
        .map_err(|e| format!("Failed to add constraint: {}", e))?;

    model
        .set_coefficient(x, constraint, 1.0)
        .map_err(|e| format!("Failed to set coefficient: {}", e))?;
    model
        .set_coefficient(y, constraint, 1.0)
        .map_err(|e| format!("Failed to set coefficient: {}", e))?;

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 2.0), (y, 3.0)],
    };
    model
        .set_objective(objective)
        .map_err(|e| format!("Failed to set objective: {}", e))?;

    let mut solver =
        Solver::new(model).map_err(|e| format!("Failed to create solver: {}", e))?;
    solver.set_log_to_console(false);
    let solution = solver.solve().map_err(|e| format!("Failed to solve: {}", e))?;

    let expected = 10.0;
    let actual = solution.objective_value();
    if (actual - expected).abs() >= 1e-4 {
        return Err(format!(
            "Expected objective value {}, got {}",
            expected, actual
        ));
    }

    Ok(())
}

/// Test: maximize x subject to x <= 10
/// Expected optimal: x = 10, objective = 10.0
fn test_maximize_lp() -> Result<(), String> {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .map_err(|e| format!("Failed to add variable: {}", e))?;

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 10.0),
        })
        .map_err(|e| format!("Failed to add constraint: {}", e))?;

    model
        .set_coefficient(x, constraint, 1.0)
        .map_err(|e| format!("Failed to set coefficient: {}", e))?;

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model
        .set_objective(objective)
        .map_err(|e| format!("Failed to set objective: {}", e))?;

    let mut solver =
        Solver::new(model).map_err(|e| format!("Failed to create solver: {}", e))?;
    solver.set_log_to_console(false);
    let solution = solver.solve().map_err(|e| format!("Failed to solve: {}", e))?;

    let expected = 10.0;
    let actual = solution.objective_value();
    if (actual - expected).abs() >= 1e-4 {
        return Err(format!(
            "Expected objective value {}, got {}",
            expected, actual
        ));
    }

    Ok(())
}

/// Test: maximize integer x subject to x <= 1.5, x integer, x in [0,10]
/// Expected optimal: x = 1.0, objective = 1.0
fn test_integer_variable() -> Result<(), String> {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::integer(Bounds::new(0.0, 10.0)))
        .map_err(|e| format!("Failed to add integer variable: {}", e))?;

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 1.5),
        })
        .map_err(|e| format!("Failed to add constraint: {}", e))?;

    model
        .set_coefficient(x, constraint, 1.0)
        .map_err(|e| format!("Failed to set coefficient: {}", e))?;

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model
        .set_objective(objective)
        .map_err(|e| format!("Failed to set objective: {}", e))?;

    let mut solver =
        Solver::new(model).map_err(|e| format!("Failed to create solver: {}", e))?;
    solver.set_log_to_console(false);
    let solution = solver.solve().map_err(|e| format!("Failed to solve: {}", e))?;

    let x_value = solution
        .get_primal(x.inner() as usize)
        .ok_or("Missing primal value for x")?;

    if (x_value - 1.0).abs() >= 1e-4 {
        return Err(format!("Expected integer x = 1.0, got {}", x_value));
    }

    let expected_obj = 1.0;
    let actual_obj = solution.objective_value();
    if (actual_obj - expected_obj).abs() >= 1e-4 {
        return Err(format!(
            "Expected objective value {}, got {}",
            expected_obj, actual_obj
        ));
    }

    Ok(())
}

/// Test: dual values have correct lengths and are finite
/// Model: minimize x + y, subject to x + y <= 5, x in [0,10], y in [0,10]
/// (2 variables, 1 constraint)
fn test_dual_values() -> Result<(), String> {
    let model = build_simple_model();
    let num_variables = model.num_variables();
    let num_constraints = model.num_constraints();

    let mut solver =
        Solver::new(model).map_err(|e| format!("Failed to create solver: {}", e))?;
    solver.set_log_to_console(false);
    let solution = solver.solve().map_err(|e| format!("Failed to solve: {}", e))?;

    if solution.variable_duals().len() != num_variables {
        return Err(format!(
            "Variable duals length mismatch: expected {}, got {}",
            num_variables,
            solution.variable_duals().len()
        ));
    }
    if solution.constraint_duals().len() != num_constraints {
        return Err(format!(
            "Constraint duals length mismatch: expected {}, got {}",
            num_constraints,
            solution.constraint_duals().len()
        ));
    }
    if !solution.variable_duals().iter().all(|value| value.is_finite()) {
        return Err("Not all variable duals are finite".to_string());
    }
    if !solution.constraint_duals().iter().all(|value| value.is_finite()) {
        return Err("Not all constraint duals are finite".to_string());
    }

    Ok(())
}

/// Test: infeasible model returns error
/// x in [0,10] with constraint x >= 20 is infeasible
fn test_infeasible() -> Result<(), String> {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .map_err(|e| format!("Failed to add variable: {}", e))?;

    // x >= 20 but x has upper bound 10, so infeasible
    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(20.0, f64::INFINITY),
        })
        .map_err(|e| format!("Failed to add constraint: {}", e))?;

    model
        .set_coefficient(x, constraint, 1.0)
        .map_err(|e| format!("Failed to set coefficient: {}", e))?;

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model
        .set_objective(objective)
        .map_err(|e| format!("Failed to set objective: {}", e))?;

    let mut solver =
        Solver::new(model).map_err(|e| format!("Failed to create solver: {}", e))?;
    solver.set_log_to_console(false);
    let result = solver.solve();

    if result.is_ok() {
        return Err("Infeasible problem should fail to solve".to_string());
    }

    Ok(())
}

/// Test: primal start (warm-start hints) storage, validation, clear, then solve with hints
fn test_primal_start() -> Result<(), String> {
    let model = build_simple_model();

    let mut solver =
        Solver::new(model).map_err(|e| format!("Failed to create solver: {}", e))?;
    solver.set_log_to_console(false);

    // Storage: set valid hints and verify they are stored
    let hints = vec![(VariableId::new(0), 2.0), (VariableId::new(1), 1.0)];
    solver
        .set_primal_start(&hints)
        .map_err(|e| format!("Setting valid primal start should succeed: {}", e))?;
    if solver.get_primal_start() != Some(hints.as_slice()) {
        return Err("Stored hints don't match what was set".to_string());
    }

    // Validation: reject invalid variable IDs
    let bad_hints = vec![(VariableId::new(9999), 0.5)];
    if solver.set_primal_start(&bad_hints).is_ok() {
        return Err("Setting primal start with invalid variable ID should fail".to_string());
    }

    // Clear: primal start should be gone after clearing
    solver
        .set_primal_start(&hints)
        .map_err(|e| format!("Re-setting valid hints should succeed: {}", e))?;
    solver.clear_primal_start();
    if solver.get_primal_start().is_some() {
        return Err("Primal start should be None after clearing".to_string());
    }

    // Solve with hints: solver should still produce a valid solution
    solver
        .set_primal_start(&hints)
        .map_err(|e| format!("Setting hints before solve should succeed: {}", e))?;
    let solution = solver
        .solve()
        .map_err(|e| format!("Solve with primal start should succeed: {}", e))?;

    let expected = 0.0;
    let actual = solution.objective_value();
    if (actual - expected).abs() >= 1e-4 {
        return Err(format!("Expected objective value {}, got {}", expected, actual));
    }

    Ok(())
}

/// Test: solution metadata (solve_time_seconds >= 0)
fn test_solution_metadata() -> Result<(), String> {
    let model = build_simple_model();

    let mut solver =
        Solver::new(model).map_err(|e| format!("Failed to create solver: {}", e))?;
    solver.set_log_to_console(false);
    let solution = solver.solve().map_err(|e| format!("Failed to solve: {}", e))?;

    if solution.solve_time_seconds() < 0.0 {
        return Err(format!(
            "Solve time should be non-negative, got {}",
            solution.solve_time_seconds()
        ));
    }
    if !solution.is_optimal() {
        return Err("Solution should be optimal".to_string());
    }
    if !solution.is_feasible() {
        return Err("Solution should be feasible".to_string());
    }
    if solution.is_infeasible() {
        return Err("Solution should not be infeasible".to_string());
    }
    if solution.is_unbounded() {
        return Err("Solution should not be unbounded".to_string());
    }

    Ok(())
}
