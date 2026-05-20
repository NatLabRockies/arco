use arco_model::{Bounds, Constraint, Model, Objective, Sense, Variable};
use arco_solver::{SolverConfig, SolverError, check_small_lp, check_small_milp};
use arco_xpress::{Solver, XpressModelViewBackend, detect_xpress_dir};
use std::path::PathBuf;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {actual} to be within tolerance of {expected}"
    );
}

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
fn model_view_shared_small_lp_conformance_with_local_xpress_install() {
    let Some(_xpress_dir) = local_xpress_dir() else {
        return;
    };

    let backend = XpressModelViewBackend;
    let report = match check_small_lp(&backend, &SolverConfig::new().with_log_to_console(false)) {
        Ok(report) => report,
        Err(SolverError::SolverSpecific(message))
            if message.contains("Xpress license initialization failed") =>
        {
            return;
        }
        Err(error) => panic!("xpress shared small-LP conformance succeeds: {error:?}"),
    };

    assert_eq!(report.family, "xpress");
    assert_close(report.objective_value, 2.0);
    assert_eq!(report.variables, 1);
    assert_eq!(report.constraints, 1);
    assert_eq!(report.coefficients, 1);
}

#[test]
fn model_view_shared_small_milp_conformance_with_local_xpress_install() {
    let Some(_xpress_dir) = local_xpress_dir() else {
        return;
    };

    let backend = XpressModelViewBackend;
    let report = match check_small_milp(&backend, &SolverConfig::new().with_log_to_console(false)) {
        Ok(report) => report,
        Err(SolverError::SolverSpecific(message))
            if message.contains("Xpress license initialization failed") =>
        {
            return;
        }
        Err(error) => panic!("xpress shared small-MILP conformance succeeds: {error:?}"),
    };

    assert_eq!(report.family, "xpress");
    assert_close(report.objective_value, 1.0);
    assert_eq!(report.variables, 1);
    assert_eq!(report.constraints, 1);
    assert_eq!(report.coefficients, 1);
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
    assert_close(solution.objective_value(), 2.0);
    assert_eq!(solution.primal_values(), &[1.0]);
}
