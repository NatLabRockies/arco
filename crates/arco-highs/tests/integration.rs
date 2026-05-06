#![allow(clippy::float_cmp)]

use arco_highs::Solver;
use arco_targets::{
    AlgebraicProblem, ConstraintSense, LinearConstraint, LinearObjective, LinearTerm,
    ObjectiveSense, VariableInstance, VariableKind,
};

fn variable(name: &str, lower: f64, upper: Option<f64>, kind: VariableKind) -> VariableInstance {
    VariableInstance {
        name: name.to_string(),
        family: name.to_string(),
        lower,
        upper,
        kind,
    }
}

fn term(variable_name: &str, coefficient: f64) -> LinearTerm {
    LinearTerm {
        variable_name: variable_name.to_string(),
        coefficient,
    }
}

fn objective(sense: ObjectiveSense, terms: Vec<LinearTerm>) -> LinearObjective {
    LinearObjective {
        name: "objective".to_string(),
        sense,
        constant: 0.0,
        terms,
    }
}

fn constraint(
    name: &str,
    sense: ConstraintSense,
    rhs: f64,
    terms: Vec<LinearTerm>,
) -> LinearConstraint {
    LinearConstraint {
        name: name.to_string(),
        sense,
        rhs,
        terms,
    }
}

fn simple_problem() -> AlgebraicProblem {
    AlgebraicProblem {
        variable_instances: vec![
            variable("x", 0.0, Some(10.0), VariableKind::Continuous),
            variable("y", 0.0, Some(10.0), VariableKind::Continuous),
        ],
        constraints: vec![constraint(
            "limit",
            ConstraintSense::LessEqual,
            5.0,
            vec![term("x", 1.0), term("y", 1.0)],
        )],
        objective: objective(
            ObjectiveSense::Minimize,
            vec![term("x", 1.0), term("y", 1.0)],
        ),
        reports: Vec::new(),
    }
}

#[test]
fn simple_lp_solves() {
    let problem = AlgebraicProblem {
        variable_instances: vec![
            variable("x", 0.0, None, VariableKind::Continuous),
            variable("y", 0.0, None, VariableKind::Continuous),
        ],
        constraints: vec![constraint(
            "demand",
            ConstraintSense::GreaterEqual,
            5.0,
            vec![term("x", 1.0), term("y", 1.0)],
        )],
        objective: objective(
            ObjectiveSense::Minimize,
            vec![term("x", 2.0), term("y", 3.0)],
        ),
        reports: Vec::new(),
    };

    let mut solver = Solver::new(problem).expect("solver initializes");
    let solution = solver.solve().expect("solve succeeds");

    assert!((solution.objective_value() - 10.0).abs() < 1e-6);
}

#[test]
fn integer_variable_solution_respects_integrality() {
    let problem = AlgebraicProblem {
        variable_instances: vec![variable("x", 0.0, Some(10.0), VariableKind::Integer)],
        constraints: vec![constraint(
            "cap",
            ConstraintSense::LessEqual,
            1.5,
            vec![term("x", 1.0)],
        )],
        objective: objective(ObjectiveSense::Maximize, vec![term("x", 1.0)]),
        reports: Vec::new(),
    };

    let mut solver = Solver::new(problem).expect("solver initializes");
    let solution = solver.solve().expect("solve succeeds");

    assert_eq!(solution.get_primal(0), Some(1.0));
    assert!((solution.objective_value() - 1.0).abs() < 1e-6);
}

#[test]
fn dual_values_exposed() {
    let problem = simple_problem();
    let num_variables = problem.variable_instances.len();
    let num_constraints = problem.constraints.len();

    let mut solver = Solver::new(problem).expect("solver initializes");
    let solution = solver.solve().expect("solve succeeds");

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

#[test]
fn solution_metadata_accessors() {
    let start_time = std::time::Instant::now();
    let mut solver = Solver::new(simple_problem()).expect("solver initializes");
    let solution = solver.solve().expect("solve succeeds");
    let elapsed = start_time.elapsed();

    let solve_time = solution.solve_time_seconds();
    assert!(solve_time > 0.0);
    assert!(solve_time <= elapsed.as_secs_f64() + 0.1);
    assert_eq!(
        solution.total_iterations(),
        solution.simplex_iterations() + solution.barrier_iterations()
    );
    assert!(solution.total_iterations() <= 10000);
    assert_eq!(solution.primal_feasibility_tolerance(), 1e-6);
    assert_eq!(solution.dual_feasibility_tolerance(), 1e-6);
    assert!(solution.mip_gap() >= 0.0);
}

#[test]
fn solution_status_methods() {
    let mut solver = Solver::new(simple_problem()).expect("solver initializes");
    let solution = solver.solve().expect("solve succeeds");

    assert!(solution.is_optimal());
    assert!(solution.is_feasible());
    assert!(!solution.is_infeasible());
    assert!(!solution.is_unbounded());
    assert_eq!(solution.status_string(), "optimal");
}

#[test]
fn infeasible_and_unbounded_problems_fail() {
    let infeasible = AlgebraicProblem {
        variable_instances: vec![variable("x", 0.0, Some(10.0), VariableKind::Continuous)],
        constraints: vec![
            constraint(
                "lower",
                ConstraintSense::GreaterEqual,
                10.0,
                vec![term("x", 1.0)],
            ),
            constraint(
                "upper",
                ConstraintSense::LessEqual,
                5.0,
                vec![term("x", 1.0)],
            ),
        ],
        objective: objective(ObjectiveSense::Minimize, vec![term("x", 1.0)]),
        reports: Vec::new(),
    };
    assert!(Solver::new(infeasible).unwrap().solve().is_err());

    let unbounded = AlgebraicProblem {
        variable_instances: vec![variable("y", 0.0, None, VariableKind::Continuous)],
        constraints: Vec::new(),
        objective: objective(ObjectiveSense::Maximize, vec![term("y", 1.0)]),
        reports: Vec::new(),
    };
    assert!(Solver::new(unbounded).unwrap().solve().is_err());
}
