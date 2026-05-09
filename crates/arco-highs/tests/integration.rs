#![allow(clippy::float_cmp)]

use arco_highs::solve_model_view;
use arco_model::{Bounds, Constraint, Model, Objective, Sense, Variable};
use arco_solver::SolverConfig;

fn add_variable(
    model: &mut Model,
    lower: f64,
    upper: f64,
    is_integer: bool,
) -> arco_model::VariableId {
    let variable = if is_integer {
        Variable::integer(Bounds::new(lower, upper))
    } else {
        Variable::continuous(Bounds::new(lower, upper))
    };
    model.add_variable(variable).expect("variable")
}

#[test]
fn simple_lp_solves() {
    let mut model = Model::new();
    let x = add_variable(&mut model, 0.0, f64::INFINITY, false);
    let y = add_variable(&mut model, 0.0, f64::INFINITY, false);
    let demand = model
        .add_constraint(Constraint {
            bounds: Bounds::new(5.0, f64::INFINITY),
        })
        .expect("constraint");
    model.set_coefficient(x, demand, 1.0).expect("x coeff");
    model.set_coefficient(y, demand, 1.0).expect("y coeff");
    model
        .set_objective(Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(x, 2.0), (y, 3.0)],
        })
        .expect("objective");

    let solution = solve_model_view(&model, &SolverConfig::new()).expect("solve succeeds");

    assert!((solution.objective_value - 10.0).abs() < 1e-6);
}

#[test]
fn integer_variable_solution_respects_integrality() {
    let mut model = Model::new();
    let x = add_variable(&mut model, 0.0, 10.0, true);
    let cap = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 1.5),
        })
        .expect("constraint");
    model.set_coefficient(x, cap, 1.0).expect("coeff");
    model
        .set_objective(Objective {
            sense: Some(Sense::Maximize),
            terms: vec![(x, 1.0)],
        })
        .expect("objective");

    let solution = solve_model_view(&model, &SolverConfig::new()).expect("solve succeeds");

    assert_eq!(solution.primal_values.first().copied(), Some(1.0));
    assert!((solution.objective_value - 1.0).abs() < 1e-6);
}

#[test]
fn dual_values_exposed() {
    let mut model = Model::new();
    let x = add_variable(&mut model, 0.0, 10.0, false);
    let y = add_variable(&mut model, 0.0, 10.0, false);
    let limit = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 5.0),
        })
        .expect("constraint");
    model.set_coefficient(x, limit, 1.0).expect("x coeff");
    model.set_coefficient(y, limit, 1.0).expect("y coeff");
    model
        .set_objective(Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(x, 1.0), (y, 1.0)],
        })
        .expect("objective");

    let solution = solve_model_view(&model, &SolverConfig::new()).expect("solve succeeds");

    assert_eq!(solution.variable_duals.len(), 2);
    assert_eq!(solution.constraint_duals.len(), 1);
    assert!(
        solution
            .variable_duals
            .iter()
            .all(|value| value.is_finite())
    );
    assert!(
        solution
            .constraint_duals
            .iter()
            .all(|value| value.is_finite())
    );
}

#[test]
fn infeasible_and_unbounded_problems_fail() {
    let mut infeasible = Model::new();
    let x = add_variable(&mut infeasible, 0.0, 10.0, false);
    let lower = infeasible
        .add_constraint(Constraint {
            bounds: Bounds::new(10.0, f64::INFINITY),
        })
        .expect("lower");
    let upper = infeasible
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 5.0),
        })
        .expect("upper");
    infeasible
        .set_coefficient(x, lower, 1.0)
        .expect("lower coeff");
    infeasible
        .set_coefficient(x, upper, 1.0)
        .expect("upper coeff");
    infeasible
        .set_objective(Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(x, 1.0)],
        })
        .expect("objective");
    assert!(solve_model_view(&infeasible, &SolverConfig::new()).is_err());

    let mut unbounded = Model::new();
    let y = add_variable(&mut unbounded, 0.0, f64::INFINITY, false);
    unbounded
        .set_objective(Objective {
            sense: Some(Sense::Maximize),
            terms: vec![(y, 1.0)],
        })
        .expect("objective");
    assert!(solve_model_view(&unbounded, &SolverConfig::new()).is_err());
}
