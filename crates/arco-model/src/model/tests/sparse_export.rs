use super::support::{active_continuous_variable, bounded_constraint};
use super::*;

fn build_interleaved_sparse_model() -> Model {
    let mut model = Model::new();

    let v0 = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    let _v1 = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    let v2 = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();
    let v3 = model
        .add_variable(active_continuous_variable(0.0, 10.0))
        .unwrap();

    let c0 = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();
    let c1 = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();
    let _c2 = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();
    let c3 = model.add_constraint(bounded_constraint(0.0, 1.0)).unwrap();

    model.set_coefficient(v0, c1, 10.0).unwrap();
    model.set_coefficient(v0, c3, 11.0).unwrap();
    model.set_coefficient(v2, c0, 20.0).unwrap();
    model.set_coefficient(v2, c3, 21.0).unwrap();
    model.set_coefficient(v3, c0, 30.0).unwrap();

    model
}

#[test]
fn sparse_export_csc_has_expected_shape_and_arrays() {
    let model = build_interleaved_sparse_model();
    let csc = model.export_csc();

    assert_eq!(csc.shape, (4, 4));
    assert_eq!(csc.col_ptrs, vec![0, 2, 2, 4, 5]);
    assert_eq!(csc.row_indices, vec![1, 3, 0, 3, 0]);
    assert_eq!(csc.values, vec![10.0, 11.0, 20.0, 21.0, 30.0]);
    assert_eq!(*csc.col_ptrs.last().unwrap(), csc.values.len());
}

#[test]
fn sparse_export_crs_groups_values_by_row_and_keeps_monotonic_ptrs() {
    let model = build_interleaved_sparse_model();
    let crs = model.export_crs();

    assert_eq!(crs.shape, (4, 4));
    assert_eq!(crs.row_ptrs, vec![0, 2, 3, 3, 5]);
    assert!(crs.row_ptrs.windows(2).all(|w| w[0] <= w[1]));
    assert_eq!(crs.col_indices, vec![2, 3, 0, 0, 2]);
    assert_eq!(crs.values, vec![20.0, 30.0, 10.0, 11.0, 21.0]);
    assert_eq!(*crs.row_ptrs.last().unwrap(), crs.values.len());
}

#[test]
fn sparse_export_coo_emits_triplets_in_column_scan_order() {
    let model = build_interleaved_sparse_model();
    let coo = model.export_coo();

    assert_eq!(coo.shape, (4, 4));
    assert_eq!(coo.rows, vec![1, 3, 0, 3, 0]);
    assert_eq!(coo.cols, vec![0, 0, 2, 2, 3]);
    assert_eq!(coo.values, vec![10.0, 11.0, 20.0, 21.0, 30.0]);
}

#[test]
fn sparse_export_empty_model_returns_empty_payloads() {
    let model = Model::new();

    let csc = model.export_csc();
    assert_eq!(csc.shape, (0, 0));
    assert_eq!(csc.col_ptrs, vec![0]);
    assert!(csc.row_indices.is_empty());
    assert!(csc.values.is_empty());

    let crs = model.export_crs();
    assert_eq!(crs.shape, (0, 0));
    assert_eq!(crs.row_ptrs, vec![0]);
    assert!(crs.col_indices.is_empty());
    assert!(crs.values.is_empty());

    let coo = model.export_coo();
    assert_eq!(coo.shape, (0, 0));
    assert!(coo.rows.is_empty());
    assert!(coo.cols.is_empty());
    assert!(coo.values.is_empty());
}
