from __future__ import annotations

import math

import numpy as np
import pytest

import arco


def test_three_term_axis_reduction_reuses_source_and_preserves_csc_columns() -> None:
    model = arco.Model()
    outer = arco.IndexSet(name="outer", members=range(2))
    inner = arco.IndexSet(name="inner", members=range(2))
    reduced_axis = arco.IndexSet(name="reduced", members=range(3))
    source = model.add_variables(
        axes=(outer, inner, reduced_axis),
        bounds=arco.NonNegativeFloat,
        name="source",
    )
    reference = model.add_variables(
        axes=(outer, inner),
        bounds=arco.NonNegativeFloat,
        name="reference",
    )

    reduced = source.sum(over=reduced_axis)
    assert reduced.shape == (2, 2)
    assert tuple(axis.name for axis in reduced.index_sets) == ("outer", "inner")
    comparison = reduced >= reference + 2.5

    model.add_constraints(comparison)
    model.add_constraints(comparison)

    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert model.num_constraints == 8
    assert [
        (constraint.bounds.lower, constraint.bounds.upper)
        for constraint in snapshot.constraints
    ] == [(2.5, math.inf)] * 8

    csc = model.export_csc()
    expected_col_ptrs = [0]
    expected_rows: list[int] = []
    expected_values: list[float] = []
    for variable_id in range(16):
        if variable_id < 12:
            row = variable_id // 3
            coefficient = 1.0
        else:
            row = variable_id - 12
            coefficient = -1.0
        expected_rows.extend((row, row + 4))
        expected_values.extend((coefficient, coefficient))
        expected_col_ptrs.append(len(expected_rows))

    assert csc == {
        "col_ptrs": expected_col_ptrs,
        "row_indices": expected_rows,
        "values": expected_values,
        "shape": (8, 16),
    }


def test_nonfinite_owned_axis_expression_still_fails_validation() -> None:
    model = arco.Model()
    outer = arco.IndexSet(name="outer", members=range(2))
    inner = arco.IndexSet(name="inner", members=range(2))
    reduced_axis = arco.IndexSet(name="reduced", members=range(3))
    source = model.add_variables(
        axes=(outer, inner, reduced_axis),
        bounds=arco.NonNegativeFloat,
    )
    reference = model.add_variables(
        axes=(outer, inner),
        bounds=arco.NonNegativeFloat,
    )

    reduced = source.sum(over=reduced_axis)
    with pytest.raises(arco.ConstraintInvalidBoundsError):
        model.add_constraints(reduced * np.nan >= reference)


def test_full_param_comparison_can_be_reused_with_an_active_mask() -> None:
    model = arco.Model()
    index = arco.IndexSet(name="index", members=range(3))
    variables = model.add_variables(
        axes=(index,), bounds=arco.NonNegativeFloat, name="variables"
    )
    rhs = arco.param(np.array([2.0, 3.0, 4.0]), axes=(index,))
    comparison = variables + 1.0 == rhs

    model.add_constraints(comparison)
    model.add_constraints(comparison, active=np.array([True, False, True]))

    assert model.num_constraints == 5
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert [constraint.nnz for constraint in snapshot.constraints] == [1, 1, 1, 1, 1]
    assert [
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ] == [(0, 0, 1.0), (3, 0, 1.0), (1, 1, 1.0), (2, 2, 1.0), (4, 2, 1.0)]
    assert [
        (constraint.bounds.lower, constraint.bounds.upper)
        for constraint in snapshot.constraints
    ] == [(1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (1.0, 1.0), (3.0, 3.0)]


def test_full_param_comparison_rejects_invalid_mask_before_model_mutation() -> None:
    model = arco.Model()
    index = arco.IndexSet(name="index", members=range(3))
    variables = model.add_variables(axes=(index,), bounds=arco.NonNegativeFloat)
    comparison = variables + 1.0 == arco.param(np.array([2.0, 3.0, 4.0]), axes=(index,))

    with pytest.raises(ValueError, match="broadcast"):
        model.add_constraints(comparison, active=np.array([True, False]))

    assert model.num_constraints == 0
