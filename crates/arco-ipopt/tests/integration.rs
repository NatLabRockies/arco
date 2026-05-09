use arco_ipopt::{IpoptModelViewBackend, Solver, SolverError, solve_model_view};
use arco_model::types::Bounds;
use arco_model::{Model, Variable};
use arco_solver::{ModelViewBackend, SolverConfig};

fn continuous_model() -> Model {
    let mut model = Model::new();
    model
        .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
        .unwrap();
    model
}

fn solver_new_error(model: &Model) -> SolverError {
    match Solver::new(model) {
        Ok(_) => panic!("expected IPOPT solver construction to fail"),
        Err(error) => error,
    }
}

#[test]
fn backend_identifies_ipopt_family() {
    assert_eq!(IpoptModelViewBackend.family(), "ipopt");
}

#[test]
fn constructor_rejects_empty_model() {
    let model = Model::new();
    let error = solver_new_error(&model);

    assert!(matches!(error, SolverError::EmptyModel));
}

#[test]
fn constructor_rejects_integer_variables() {
    let mut model = Model::new();
    model
        .add_variable(Variable::integer(Bounds::new(0.0, 10.0)))
        .unwrap();

    let error = solver_new_error(&model);

    assert!(matches!(error, SolverError::SolverSpecific(message) if message.contains("integer")));
}

#[test]
fn constructor_reports_unavailable_adapter_for_supported_model_shape() {
    let model = continuous_model();
    let error = solver_new_error(&model);

    assert!(
        matches!(error, SolverError::SolverNotAvailable(message) if message.contains("not implemented"))
    );
}

#[test]
fn model_view_solve_reports_unavailable_adapter() {
    let model = continuous_model();
    let config = SolverConfig::new();
    let error = solve_model_view(&model, &config).unwrap_err();

    assert!(
        matches!(error, SolverError::SolverNotAvailable(message) if message.contains("not implemented"))
    );
}
