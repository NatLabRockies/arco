use super::support::{active_continuous_variable, bounded_constraint};
use super::*;

fn column_entry_count(model: &Model) -> usize {
    model.columns().map(|(_, column)| column.len()).sum()
}

fn assert_count_matches_columns(model: &Model) {
    assert_eq!(model.num_coefficients(), column_entry_count(model));
}

#[test]
fn coefficient_count_matches_columns_through_mixed_mutations() {
    let mut model = Model::new();
    assert_count_matches_columns(&model);
    let first = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    assert_count_matches_columns(&model);
    let second = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    model.deactivate_variable(second).unwrap();
    assert_count_matches_columns(&model);
    let first_constraint = model.add_constraint(bounded_constraint(0.0, 10.0)).unwrap();
    assert_count_matches_columns(&model);
    assert!(
        model
            .set_coefficient(first, first_constraint, f64::NAN)
            .is_err()
    );
    assert_count_matches_columns(&model);
    model.set_coefficient(first, first_constraint, 1.0).unwrap();
    assert_count_matches_columns(&model);
    model.set_coefficient(first, first_constraint, 2.0).unwrap();
    assert_count_matches_columns(&model);
    model
        .set_coefficient(second, first_constraint, 0.0)
        .unwrap();
    assert_count_matches_columns(&model);

    model
        .add_constraints_compact(
            &[(first.inner(), 3.0), (first.inner(), 0.0)],
            &[Bounds::new(0.0, 10.0)],
        )
        .unwrap();
    assert_count_matches_columns(&model);

    model
        .add_constraints_batch_streaming(
            1,
            vec![(vec![(first, 4.0), (first, 0.0)], Bounds::new(0.0, 10.0))],
        )
        .unwrap();
    assert_count_matches_columns(&model);

    let normalized = Expr::term(first, 3.0)
        .add(&Expr::term(first, -3.0))
        .add(&Expr::term(second, 4.0));
    model
        .add_expr_constraint(normalized, Bounds::new(0.0, 10.0))
        .unwrap();
    assert_count_matches_columns(&model);

    assert_eq!(model.num_coefficients(), 7);
    assert_eq!(model.clone().num_coefficients(), 7);
    let mut columns = model.columns();
    assert_eq!(columns.next().unwrap().1.len(), 5);
    assert_eq!(columns.next().unwrap().1.len(), 2);
}

#[test]
fn coefficient_count_tracks_partial_compact_and_batch_failures() {
    let mut compact_model = Model::new();
    compact_model
        .add_variables_uniform(active_continuous_variable(0.0, 1.0), 1)
        .unwrap();
    let result = compact_model.add_constraints_compact_indexed(
        &[(0, 1.0), (0, f64::INFINITY)],
        &[0],
        &[Bounds::new(0.0, 1.0)],
    );
    assert!(matches!(result, Err(ModelError::InvalidCoefficient { .. })));
    assert_eq!(compact_model.num_coefficients(), 1);
    assert_count_matches_columns(&compact_model);

    let mut batch_model = Model::new();
    let first = batch_model
        .add_variable(active_continuous_variable(0.0, 1.0))
        .unwrap();
    let second = batch_model
        .add_variable(active_continuous_variable(0.0, 1.0))
        .unwrap();
    let invalid = VariableId::new(99);
    let result = batch_model.add_constraints_batch_streaming(
        2,
        vec![
            (vec![(first, 1.0)], Bounds::new(0.0, 1.0)),
            (vec![(second, 2.0), (invalid, 3.0)], Bounds::new(0.0, 1.0)),
        ],
    );
    assert_eq!(result, Err(ModelError::InvalidVariableId(invalid)));
    assert_eq!(batch_model.num_coefficients(), 2);
    assert_count_matches_columns(&batch_model);
}

#[test]
fn coefficient_count_preserves_csc_duplicates_and_explicit_zero() {
    let model = Model::from_csc(
        CscInput {
            num_constraints: 2,
            num_variables: 1,
            col_ptrs: &[0, 3],
            row_indices: &[0, 0, 1],
            values: &[1.0, 0.0, -2.0],
            var_lower: &[0.0],
            var_upper: &[1.0],
            con_lower: &[0.0, 0.0],
            con_upper: &[1.0, 1.0],
            is_integer: &[false],
        },
        SimplifyLevel::None,
    )
    .unwrap();

    assert_eq!(model.num_coefficients(), 3);
    assert_count_matches_columns(&model);
    assert_eq!(model.columns().next().unwrap().1[1].1, 0.0);
}
