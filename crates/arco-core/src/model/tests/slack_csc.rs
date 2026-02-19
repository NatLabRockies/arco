use super::support::{active_continuous_variable, bounded_constraint};
use super::*;
use crate::slack::SlackBound;
use crate::types::SimplifyLevel;

#[test]
fn test_add_slack_upper_adds_variable_and_objective() {
    let mut model = Model::new();
    let var_id = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    let constraint_id = model.add_constraint(bounded_constraint(0.0, 5.0)).unwrap();
    model
        .set_constraint_name(constraint_id, "limit".to_string())
        .unwrap();
    model.minimize(Expr::term(var_id, 1.0)).unwrap();

    let slack = model
        .add_slack(constraint_id, SlackBound::Upper, 10.0, None)
        .unwrap();
    assert!(slack.var_ids.lower.is_none());
    let slack_var = slack.var_ids.upper.expect("upper slack missing");

    let slack_variable = model.get_variable(slack_var).unwrap();
    assert_eq!(slack_variable.bounds.lower, 0.0);
    assert_eq!(slack_variable.bounds.upper, f64::INFINITY);
    assert!(!slack_variable.is_integer);

    let slack_column = model.get_column(slack_var).expect("slack column missing");
    assert_eq!(slack_column, &vec![(constraint_id, -1.0)]);

    let slack_coeff = model
        .objective()
        .terms
        .iter()
        .find(|(id, _)| *id == slack_var)
        .map(|(_, coeff)| *coeff)
        .expect("slack objective term missing");
    assert_eq!(slack_coeff, 10.0);

    assert_eq!(
        model.get_variable_name(slack_var),
        Some("limit:slack_upper")
    );
}

#[test]
fn test_add_slack_both_sets_names_and_coefficients() {
    let mut model = Model::new();
    let var_id = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    let constraint_id = model.add_constraint(bounded_constraint(2.0, 4.0)).unwrap();
    model.minimize(Expr::term(var_id, 1.0)).unwrap();

    let slack = model
        .add_slack(
            constraint_id,
            SlackBound::Both,
            7.0,
            Some("balance".to_string()),
        )
        .unwrap();
    let lower_var = slack.var_ids.lower.expect("lower slack missing");
    let upper_var = slack.var_ids.upper.expect("upper slack missing");

    assert_eq!(
        model.get_variable_name(lower_var),
        Some("balance:slack_lower")
    );
    assert_eq!(
        model.get_variable_name(upper_var),
        Some("balance:slack_upper")
    );
    assert_eq!(
        model.get_column(lower_var).expect("lower column missing"),
        &vec![(constraint_id, 1.0)]
    );
    assert_eq!(
        model.get_column(upper_var).expect("upper column missing"),
        &vec![(constraint_id, -1.0)]
    );

    let mut slack_terms = model
        .objective()
        .terms
        .iter()
        .filter(|(id, _)| *id == lower_var || *id == upper_var)
        .map(|(_, coeff)| *coeff)
        .collect::<Vec<_>>();
    slack_terms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(slack_terms, vec![7.0, 7.0]);
}

#[test]
fn test_add_slack_penalty_flips_on_maximize() {
    let mut model = Model::new();
    let var_id = model
        .add_variable(active_continuous_variable(0.0, 1.0))
        .unwrap();
    let constraint_id = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();
    model.maximize(Expr::term(var_id, 1.0)).unwrap();

    let slack = model
        .add_slack(constraint_id, SlackBound::Lower, 3.0, None)
        .unwrap();
    let slack_var = slack.var_ids.lower.expect("lower slack missing");

    let slack_coeff = model
        .objective()
        .terms
        .iter()
        .find(|(id, _)| *id == slack_var)
        .map(|(_, coeff)| *coeff)
        .expect("slack objective term missing");
    assert_eq!(slack_coeff, -3.0);
}

#[test]
fn test_add_slack_requires_objective() {
    let mut model = Model::new();
    let constraint_id = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();

    let result = model.add_slack(constraint_id, SlackBound::Upper, 1.0, None);
    assert_eq!(result, Err(ModelError::NoObjective));
}

#[test]
fn test_make_elastic_respects_optional_bounds() {
    let mut model = Model::new();
    let var_id = model
        .add_variable(active_continuous_variable(0.0, 1.0))
        .unwrap();
    let constraint_id = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();
    model.minimize(Expr::term(var_id, 1.0)).unwrap();

    let elastic = model
        .make_elastic(constraint_id, Some(4.0), None, Some("cap".to_string()))
        .unwrap();
    assert!(elastic.lower.is_none());
    let upper = elastic.upper.expect("upper slack missing");
    let upper_var = upper.var_ids.upper.expect("upper var missing");
    assert_eq!(model.get_variable_name(upper_var), Some("cap:slack_upper"));
}

#[test]
fn test_from_csc_builds_model() {
    let model = Model::from_csc(
        CscInput {
            num_constraints: 1,
            num_variables: 2,
            col_ptrs: &[0, 1, 2],
            row_indices: &[0, 0],
            values: &[1.0_f32, 2.0_f32],
            var_lower: &[0.0_f32, 0.0_f32],
            var_upper: &[10.0_f32, 5.0_f32],
            con_lower: &[5.0_f32],
            con_upper: &[5.0_f32],
            is_integer: &[false, true],
        },
        SimplifyLevel::None,
    )
    .expect("model should build");

    assert_eq!(model.num_variables(), 2);
    assert_eq!(model.num_constraints(), 1);
    assert_eq!(
        model.get_column(VariableId::new(0)).unwrap(),
        &vec![(ConstraintId::new(0), 1.0)]
    );
    assert_eq!(
        model.get_column(VariableId::new(1)).unwrap(),
        &vec![(ConstraintId::new(0), 2.0)]
    );
    let var = model.get_variable(VariableId::new(1)).unwrap();
    assert!(var.is_integer);
}

#[test]
fn test_from_csc_rejects_bad_col_ptrs() {
    let err = Model::from_csc(
        CscInput {
            num_constraints: 1,
            num_variables: 1,
            col_ptrs: &[0],
            row_indices: &[],
            values: &[],
            var_lower: &[0.0_f32],
            var_upper: &[1.0_f32],
            con_lower: &[0.0_f32],
            con_upper: &[1.0_f32],
            is_integer: &[false],
        },
        SimplifyLevel::None,
    )
    .expect_err("expected invalid CSC error");
    assert!(matches!(err, ModelError::InvalidCscData { .. }));
}

#[test]
fn test_from_csc_only_stores_non_empty_columns() {
    let model = Model::from_csc(
        CscInput {
            num_constraints: 2,
            num_variables: 3,
            col_ptrs: &[0, 0, 2, 2],
            row_indices: &[0, 1],
            values: &[1.0_f32, 2.0_f32],
            var_lower: &[0.0_f32, 0.0_f32, 0.0_f32],
            var_upper: &[1.0_f32, 1.0_f32, 1.0_f32],
            con_lower: &[0.0_f32, 0.0_f32],
            con_upper: &[1.0_f32, 1.0_f32],
            is_integer: &[false, false, false],
        },
        SimplifyLevel::None,
    )
    .expect("model should build");

    assert_eq!(model.columns.len(), 1);
    assert!(
        model
            .get_column(VariableId::new(0))
            .expect("missing column")
            .is_empty()
    );
    assert_eq!(
        model
            .get_column(VariableId::new(1))
            .expect("missing column"),
        &vec![(ConstraintId::new(0), 1.0), (ConstraintId::new(1), 2.0)]
    );
    assert!(
        model
            .get_column(VariableId::new(2))
            .expect("missing column")
            .is_empty()
    );
}

#[test]
fn test_from_csc_rejects_non_finite_values() {
    let err = Model::from_csc(
        CscInput {
            num_constraints: 1,
            num_variables: 1,
            col_ptrs: &[0, 1],
            row_indices: &[0],
            values: &[f32::INFINITY],
            var_lower: &[0.0_f32],
            var_upper: &[1.0_f32],
            con_lower: &[0.0_f32],
            con_upper: &[1.0_f32],
            is_integer: &[false],
        },
        SimplifyLevel::None,
    )
    .expect_err("expected invalid CSC error");
    assert!(matches!(err, ModelError::InvalidCscData { .. }));
}
