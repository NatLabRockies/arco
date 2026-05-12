use arco_model::{Bounds, Constraint, Model, Objective, Sense, Variable};
use arco_solver::{SolverConfig, SolverError};
use arco_xpress::{Solver, detect_xpress_dir, solve_model_view};
use std::path::PathBuf;

fn build_simple_model() -> Model {
    let mut model = Model::new();
    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .expect("variable");
    let demand = model
        .add_constraint(Constraint {
            bounds: Bounds::new(1.0, f64::INFINITY),
        })
        .expect("constraint");
    model.set_coefficient(x, demand, 1.0).expect("coefficient");
    model
        .set_objective(Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(x, 2.0)],
        })
        .expect("objective");
    model
}

fn local_xpress_dir() -> Option<PathBuf> {
    let mounted = PathBuf::from("/Volumes/FICO Xpress Installer/FICO Xpress/xpressmp");
    if mounted.exists() {
        return Some(mounted);
    }
    detect_xpress_dir()
}

#[test]
fn model_view_smoke_solves_with_local_xpress_install() {
    let Some(_xpress_dir) = local_xpress_dir() else {
        return;
    };

    let model = build_simple_model();
    let result = match solve_model_view(&model, &SolverConfig::new().with_log_to_console(false)) {
        Ok(result) => result,
        Err(SolverError::SolverSpecific(message))
            if message.contains("Xpress license initialization failed") =>
        {
            return;
        }
        Err(error) => panic!("xpress solve succeeds: {error:?}"),
    };

    assert!(result.status.is_feasible());
    assert_eq!(result.objective_value, 2.0);
    assert_eq!(result.primal_values, vec![1.0]);
}

#[test]
fn solver_wrapper_smoke_solves_with_local_xpress_install() {
    let Some(_xpress_dir) = local_xpress_dir() else {
        return;
    };

    let model = build_simple_model();
    let mut solver = Solver::new(&model).expect("solver wrapper");
    solver.set_log_to_console(false);
    let solution = match solver.solve() {
        Ok(solution) => solution,
        Err(SolverError::SolverSpecific(message))
            if message.contains("Xpress license initialization failed") =>
        {
            return;
        }
        Err(error) => panic!("xpress wrapper solve succeeds: {error:?}"),
    };

    assert!(solution.is_feasible());
    assert_eq!(solution.objective_value(), 2.0);
    assert_eq!(solution.primal_values(), &[1.0]);
}
