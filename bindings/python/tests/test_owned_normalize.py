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
