from __future__ import annotations

import math

import numpy as np
import pytest

import arco


def _coefficient_rows(snapshot: object) -> list[tuple[int, int, float]]:
    coefficients = snapshot.coefficients
    assert coefficients is not None
    return [(item.constraint_id, item.variable_id, item.value) for item in coefficients]


def test_weighted_sparse_sum_preserves_target_order_and_duplicate_broadcast_terms() -> (
    None
):
    model = arco.Model()
    extra = arco.IndexSet(name="extra", members=[0, 1])
    group = arco.IndexSet(name="group", members=[0, 1])
    variable_axis = arco.IndexSet(name="variable", members=[0, 1])
    variables = model.add_variables(
        axes=(group, variable_axis),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, True], [False, False]]),
        name="variables",
    )
    weights = arco.param(
        np.array([2.0, 3.0]),
        axes=(extra,),
    )

    weighted = weights * variables
    assert weighted.memory_estimate()["storage"] == "sparse_weighted"
    objective = weighted.sum()
    model.minimize(objective)

    snapshot = model.inspect()
    assert snapshot.objective is not None
    assert snapshot.objective.terms == [
        (0, 5.0),
        (1, 5.0),
    ]


def test_weighted_sparse_axis_reduction_preserves_labels_and_coefficients() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=["a", "b"])
    hour = arco.IndexSet(name="hour", members=[0, 1, 2])
    variables = model.add_variables(
        axes=(technology, hour),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, False]]),
        name="variables",
    )
    weights = arco.param(np.array([2.0, 3.0, 4.0]), axes=(hour,))

    weighted = weights * variables
    reduced = weighted.sum(over=hour)
    assert reduced.shape == (2,)
    assert [axis.name for axis in reduced.index_sets] == ["technology"]

    model.add_constraints(reduced >= 0.0)
    assert _coefficient_rows(model.inspect(include_coeffs=True)) == [
        (0, 0, 2.0),
        (0, 1, 4.0),
        (1, 2, 3.0),
    ]


def test_weighted_sparse_zero_weight_is_omitted_before_reduction() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=[0, 1, 2])
    variables = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=np.array([True, True, False]),
        name="variables",
    )
    weighted = arco.param(np.array([0.0, 2.0, -3.0]), axes=(axis,)) * variables

    assert weighted.memory_estimate()["active_slots"] == 1
    model.minimize(weighted.sum())
    assert model.inspect().objective.terms == [(1, 2.0)]


def test_weighted_sparse_nonfinite_weight_keeps_native_error_boundary() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=[0, 1])
    variables = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=np.ones(2, dtype=bool),
        name="variables",
    )
    weighted = arco.param(np.array([math.inf, 2.0]), axes=(axis,)) * variables

    with pytest.raises(arco.ExprCoefficientError, match="finite"):
        model.minimize(weighted.sum())
