use super::support::{active_continuous_variable, bounded_constraint};
use super::*;
use crate::slack::SlackBound;

#[test]
fn test_variable_name_lifecycle() {
    let mut model = Model::new();
    let var = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    assert!(model.get_variable_name(var).is_none());
    model.set_variable_name(var, "x".to_string()).unwrap();
    assert_eq!(model.get_variable_name(var), Some("x"));
}

#[test]
fn test_variable_metadata() {
    let mut model = Model::new();
    let var = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    let meta = serde_json::json!({"unit": "MW", "description": "power"});
    model.set_variable_metadata(var, meta.clone()).unwrap();
    assert_eq!(model.get_variable_metadata(var), Some(&meta));
}

#[test]
fn test_constraint_name_lifecycle() {
    let mut model = Model::new();
    let con = model
        .add_constraint(bounded_constraint(0.0, 100.0))
        .unwrap();
    assert!(model.get_constraint_name(con).is_none());
    model
        .set_constraint_name(con, "capacity".to_string())
        .unwrap();
    assert_eq!(model.get_constraint_name(con), Some("capacity"));
}

#[test]
fn test_name_lookup_helpers() {
    let mut model = Model::new();
    let var = model
        .add_variable(active_continuous_variable(0.0, 1.0))
        .unwrap();
    let con = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();
    model.set_variable_name(var, "x".to_string()).unwrap();
    model.set_constraint_name(con, "limit".to_string()).unwrap();

    assert_eq!(model.get_variable_by_name("x"), Some(var));
    assert_eq!(model.get_constraint_by_name("limit"), Some(con));
    assert!(model.get_variable_by_name("missing").is_none());
    assert!(model.get_constraint_by_name("missing").is_none());
}

#[test]
fn test_constraint_metadata() {
    let mut model = Model::new();
    let con = model
        .add_constraint(bounded_constraint(0.0, 100.0))
        .unwrap();
    let meta = serde_json::json!({"type": "capacity", "region": "north"});
    model.set_constraint_metadata(con, meta.clone()).unwrap();
    assert_eq!(model.get_constraint_metadata(con), Some(&meta));
}

#[test]
fn inspect_includes_coefficients_and_slacks() {
    let mut model = Model::new();
    let x = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    model
        .set_variable_name(x, "x".to_string())
        .expect("name set");
    let var_meta = serde_json::json!({"unit": "MW"});
    model
        .set_variable_metadata(x, var_meta.clone())
        .expect("meta set");

    let c = model.add_constraint(bounded_constraint(5.0, 5.0)).unwrap();
    model
        .set_constraint_name(c, "balance".to_string())
        .expect("name set");
    let con_meta = serde_json::json!({"type": "balance"});
    model
        .set_constraint_metadata(c, con_meta.clone())
        .expect("meta set");

    model.set_coefficient(x, c, 1.5).unwrap();
    model.minimize(Expr::term(x, 1.0)).expect("objective set");
    model
        .add_slack(c, SlackBound::Lower, 1e4, Some("balance_slack".into()))
        .expect("slack added");

    let snapshot = model.inspect(InspectOptions {
        include_coefficients: true,
        include_slacks: true,
        variable_filter: None,
        constraint_filter: None,
    });

    assert_eq!(snapshot.variables.len(), 2);
    let var_view = snapshot
        .variables
        .iter()
        .find(|v| v.name.as_deref() == Some("x"))
        .expect("x variable missing");
    assert_eq!(var_view.metadata, Some(var_meta));

    assert_eq!(snapshot.constraints.len(), 1);
    let con_view = &snapshot.constraints[0];
    assert_eq!(con_view.name.as_deref(), Some("balance"));
    assert_eq!(con_view.nnz, 2);
    assert_eq!(con_view.metadata, Some(con_meta));

    let coeffs = snapshot
        .coefficients
        .as_ref()
        .expect("coefficients missing");
    assert_eq!(coeffs.len(), 2);
    assert!(
        coeffs
            .iter()
            .any(|coeff| coeff.variable_id == x && coeff.constraint_id == c && coeff.value == 1.5)
    );

    let slacks = snapshot.slacks.as_ref().expect("slacks missing");
    assert_eq!(slacks.len(), 1);
    let slack_view = &slacks[0];
    assert_eq!(slack_view.constraint_id, c);
    assert_eq!(slack_view.bound, SlackBound::Lower);
    assert_eq!(slack_view.penalty, 1e4);
    assert_eq!(slack_view.name.as_deref(), Some("balance_slack"));
    assert!(slack_view.variable_ids.lower.is_some());
    assert!(slack_view.variable_ids.upper.is_none());

    let objective = snapshot.objective.as_ref().expect("objective missing");
    assert_eq!(objective.sense, Some(Sense::Minimize));
    assert_eq!(objective.terms.len(), 2);
    let mut objective_coeffs = objective
        .terms
        .iter()
        .map(|(_, coeff)| *coeff)
        .collect::<Vec<_>>();
    objective_coeffs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(objective_coeffs, vec![1.0, 1e4]);
}

#[test]
fn inspect_respects_filters() {
    let mut model = Model::new();
    let x = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    let y = model
        .add_variable(active_continuous_variable(0.0, 5.0))
        .unwrap();

    let c1 = model.add_constraint(bounded_constraint(0.0, 10.0)).unwrap();
    let c2 = model.add_constraint(bounded_constraint(0.0, 10.0)).unwrap();

    model.set_coefficient(x, c1, 1.0).unwrap();
    model.set_coefficient(y, c2, 2.0).unwrap();
    model.minimize(Expr::term(x, 1.0)).expect("objective set");
    model
        .add_slack(c2, SlackBound::Upper, 5.0, None)
        .expect("slack added");

    let snapshot = model.inspect(InspectOptions {
        include_coefficients: true,
        include_slacks: true,
        variable_filter: Some(vec![x]),
        constraint_filter: Some(vec![c1]),
    });

    assert_eq!(snapshot.variables.len(), 1);
    assert_eq!(snapshot.variables[0].id, x);
    assert_eq!(snapshot.constraints.len(), 1);
    assert_eq!(snapshot.constraints[0].id, c1);

    let coeffs = snapshot.coefficients.as_ref().expect("coeffs missing");
    assert_eq!(coeffs.len(), 1);
    assert_eq!(coeffs[0].variable_id, x);
    assert_eq!(coeffs[0].constraint_id, c1);

    assert!(snapshot.slacks.as_ref().is_none_or(|s| s.is_empty()));
}
