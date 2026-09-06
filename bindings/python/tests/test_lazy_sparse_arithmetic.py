from __future__ import annotations

import numpy as np

import arco


def test_sparse_arithmetic_keeps_rows_lazy_until_constraint_insertion() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=range(2))
    hour = arco.IndexSet(name="hour", members=range(3))
    active = np.array([[True, False, True], [False, True, False]])
    soc = model.add_variables(
        axes=(technology, hour),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="soc",
    )
    generation = model.add_variables(
        axes=(technology, hour),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="generation",
    )

    rolled = np.roll(soc, -1, axis=hour)
    rhs = soc + 2.0 * soc - generation

    assert rolled.memory_estimate()["storage"] == "sparse_lazy"
    assert rhs.memory_estimate()["storage"] == "sparse_lazy"

    comparison = rolled == rhs
    model.add_constraints(comparison, name="soc_balance")

    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert model.num_constraints == 5
    assert snapshot.metadata.coefficients == 9
    assert sorted(
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ) == [
        (0, 0, -3.0),
        (0, 3, 1.0),
        (1, 1, 1.0),
        (2, 0, 1.0),
        (2, 1, -3.0),
        (2, 4, 1.0),
        (3, 2, 1.0),
        (4, 2, -3.0),
        (4, 5, 1.0),
    ]


def test_sparse_arithmetic_depth_fallback_preserves_sparse_rows() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=range(4))
    variables = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=np.array([True, False, True, False]),
        name="variables",
    )

    result = variables
    for _ in range(40):
        result = result * 1.0

    assert result.memory_estimate()["storage"] == "sparse"
    assert result.memory_estimate()["active_slots"] == 2
    comparison = result == variables
    assert len(comparison) == 2
    assert comparison.rhs == [0.0, 0.0]
    model.add_constraints(comparison)
    assert model.num_constraints == 2


def test_sparse_roll_normalizes_minimum_isize_shift() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=range(4))
    variables = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=np.array([True, False, True, False]),
        name="variables",
    )
    minimum_shift = -(1 << 63)

    shifted = np.roll(variables, minimum_shift, axis=axis)
    equivalent = np.roll(variables, minimum_shift % 4, axis=axis)
    comparison = shifted == equivalent

    assert shifted.memory_estimate()["storage"] == "sparse_lazy"
    assert len(comparison) == 2
    assert comparison.rhs == [0.0, 0.0]
    model.add_constraints(comparison)
    assert model.num_constraints == 2
