use arco_model::{Bounds, Constraint, Model, ModelView, Objective, Sense, Variable};
use arco_solver::{
    LpAlgorithm, SolverConfig, SolverError, check_small_lp, check_small_milp, small_lp_model,
    small_milp_model,
};
use arco_xpress::{PreparedXpressModel, Solver, XpressModelViewBackend, detect_xpress_dir};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

static XPRESS_TEST_SESSION: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_xpress_tests() -> MutexGuard<'static, ()> {
    XPRESS_TEST_SESSION
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("Xpress integration test lock is not poisoned")
}

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

fn build_infeasible_model() -> Model {
    let mut model = Model::new();
    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 0.0)))
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
            terms: vec![(x, 1.0)],
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
    let _test_guard = lock_xpress_tests();

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
    let _test_guard = lock_xpress_tests();

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
fn model_view_solves_with_selected_lp_algorithms() {
    let Some(_xpress_dir) = local_xpress_dir() else {
        return;
    };
    let _test_guard = lock_xpress_tests();

    let backend = XpressModelViewBackend;
    for algorithm in [
        LpAlgorithm::PrimalSimplex,
        LpAlgorithm::DualSimplex,
        LpAlgorithm::Barrier,
    ] {
        let config = SolverConfig::new()
            .with_log_to_console(false)
            .with_lp_algorithm(algorithm);
        match check_small_lp(&backend, &config) {
            Ok(report) => assert_close(report.objective_value, 2.0),
            Err(SolverError::SolverSpecific(message))
                if message.contains("Xpress license initialization failed") =>
            {
                return;
            }
            Err(error) => panic!("xpress {algorithm:?} LP solve succeeds: {error:?}"),
        }
    }
}

#[test]
fn solver_wrapper_smoke_solves_with_local_xpress_install() {
    let Some(_xpress_dir) = local_xpress_dir() else {
        return;
    };
    let _test_guard = lock_xpress_tests();

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

#[test]
#[ignore = "requires local Xpress runtime and license"]
fn prepared_xpress_solves_after_source_models_are_dropped() {
    let xpress_dir = local_xpress_dir().expect("set XPRESSDIR to run owned Xpress regression");
    assert!(xpress_dir.exists(), "Xpress directory must exist");
    let _test_guard = lock_xpress_tests();
    let config = SolverConfig::new()
        .with_log_to_console(false)
        .with_threads(1);

    let lp_model = small_lp_model();
    let lp_fingerprint = lp_model.fingerprint();
    let lp_prepared =
        PreparedXpressModel::prepare(&lp_model, &config).expect("prepare canonical Xpress LP");
    assert_eq!(lp_prepared.fingerprint(), lp_fingerprint);
    drop(lp_model);
    let lp_result = lp_prepared
        .solve_model_view()
        .expect("solve prepared Xpress LP");
    assert_eq!(lp_result.fingerprint, lp_fingerprint);
    assert_close(lp_result.objective_value, 2.0);
    assert_eq!(lp_result.primal_values, [1.0]);

    let mip_model = small_milp_model();
    let mip_fingerprint = mip_model.fingerprint();
    let mip_prepared =
        PreparedXpressModel::prepare(&mip_model, &config).expect("prepare canonical Xpress MILP");
    assert_eq!(mip_prepared.fingerprint(), mip_fingerprint);
    drop(mip_model);
    let mip_solution = mip_prepared.solve().expect("solve prepared Xpress MILP");
    assert!(mip_solution.is_feasible());
    assert_close(mip_solution.objective_value(), 1.0);
    assert_eq!(mip_solution.primal_values(), &[1.0]);
}

#[test]
#[ignore = "requires local Xpress runtime and license"]
fn prepared_xpress_rejects_overlap_and_allows_retry_after_drop() {
    let xpress_dir = local_xpress_dir().expect("set XPRESSDIR to run owned Xpress regression");
    assert!(xpress_dir.exists(), "Xpress directory must exist");
    let _test_guard = lock_xpress_tests();
    let config = SolverConfig::new()
        .with_log_to_console(false)
        .with_threads(1);
    let first_model = small_lp_model();
    let first =
        PreparedXpressModel::prepare(&first_model, &config).expect("prepare first Xpress problem");
    drop(first_model);

    let second_model = small_lp_model();
    let error = match PreparedXpressModel::prepare(&second_model, &config) {
        Err(error) => error,
        Ok(_) => panic!("overlapping prepared Xpress problems must be rejected"),
    };
    assert!(
        matches!(error, SolverError::SolverSpecific(ref message) if message.contains("session is busy")),
        "unexpected overlap error: {error:?}"
    );

    drop(first);
    let retry = PreparedXpressModel::prepare(&second_model, &config)
        .expect("prepare after the first problem is dropped");
    drop(second_model);
    retry
        .solve()
        .expect("solve retried prepared Xpress problem");
}

#[test]
#[ignore = "requires local Xpress runtime and license"]
fn prepared_xpress_releases_native_state_after_infeasible_solve() {
    let xpress_dir = local_xpress_dir().expect("set XPRESSDIR to run owned Xpress regression");
    assert!(xpress_dir.exists(), "Xpress directory must exist");
    let _test_guard = lock_xpress_tests();
    let config = SolverConfig::new()
        .with_log_to_console(false)
        .with_threads(1);

    let infeasible_model = build_infeasible_model();
    let infeasible = PreparedXpressModel::prepare(&infeasible_model, &config)
        .expect("prepare infeasible Xpress problem");
    drop(infeasible_model);
    let error = match infeasible.solve() {
        Err(error) => error,
        Ok(_) => panic!("infeasible Xpress problem must fail to solve"),
    };
    assert!(matches!(error, SolverError::SolveFailure { .. }));

    let retry_model = small_lp_model();
    let retry = PreparedXpressModel::prepare(&retry_model, &config)
        .expect("prepare after infeasible Xpress solve");
    drop(retry_model);
    retry.solve().expect("solve after infeasible Xpress solve");
}
