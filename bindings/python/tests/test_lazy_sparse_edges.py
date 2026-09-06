from __future__ import annotations

import gc
import math

import numpy as np
import pytest

import arco


def _coefficient_signature(snapshot: object) -> list[tuple[int, int, str]]:
    coefficients = snapshot.coefficients
    assert coefficients is not None
    return sorted(
        (
            int(coefficient.constraint_id),
            int(coefficient.variable_id),
            coefficient.value.hex(),
        )
        for coefficient in coefficients
    )


def _sparse_variables() -> tuple[arco.Model, arco.IndexSet, object, object, object]:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=range(3))
    active = [True, False, True]
    left = model.add_variables(
        axes=(axis,), bounds=arco.NonNegativeFloat, active=active, name="left"
    )
    middle = model.add_variables(
        axes=(axis,), bounds=arco.NonNegativeFloat, active=active, name="middle"
    )
    right = model.add_variables(
        axes=(axis,), bounds=arco.NonNegativeFloat, active=active, name="right"
    )
    return model, axis, left, middle, right


def test_sparse_diff_maps_multiple_outer_and_middle_axis_rows() -> None:
    model = arco.Model()
    outer = arco.IndexSet(name="outer", members=["o0", "o1"])
    middle = arco.IndexSet(name="middle", members=[0, 1, 2, 3])
    inner = arco.IndexSet(name="inner", members=["i0", "i1", "i2"])
    active = np.zeros((2, 4, 3), dtype=bool)
    active[0, 0, 0] = True
    active[0, 1, 0] = True
    active[0, 2, 1] = True
    active[0, 3, 2] = True
    active[1, 0, 1] = True
    active[1, 1, 1] = True
    active[1, 2, 2] = True
    active[1, 3, 0] = True

    values = model.add_variables(
        axes=(outer, middle, inner),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="values",
    )
    comparison = np.diff(values, axis=middle) >= 0.0
    model.add_constraints(comparison)

    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.metadata.coefficients == 12
    assert [
        (variable_id, value)
        for _, variable_id, value in _coefficient_signature(snapshot)
    ] == [
        (0, "-0x1.0000000000000p+0"),
        (1, "0x1.0000000000000p+0"),
        (1, "-0x1.0000000000000p+0"),
        (2, "0x1.0000000000000p+0"),
        (2, "-0x1.0000000000000p+0"),
        (3, "0x1.0000000000000p+0"),
        (4, "-0x1.0000000000000p+0"),
        (5, "0x1.0000000000000p+0"),
        (5, "-0x1.0000000000000p+0"),
        (6, "0x1.0000000000000p+0"),
        (7, "0x1.0000000000000p+0"),
        (6, "-0x1.0000000000000p+0"),
    ]


def test_sparse_roll_accepts_isize_minimum_shift() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=range(5))
    values = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False, True],
        name="values",
    )

    rolled = np.roll(values, int(np.iinfo(np.int64).min), axis=axis)
    model.add_constraints(rolled >= 0.0)

    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (1, 2, "0x1.0000000000000p+0"),
        (2, 0, "0x1.0000000000000p+0"),
        (4, 1, "0x1.0000000000000p+0"),
    ]


def test_deep_scale_add_chain_preserves_sparse_comparison_rows() -> None:
    model, _, values, _, _ = _sparse_variables()
    expression = values
    for _ in range(40):
        expression = expression * 1.0 + values * 0.0

    comparison = expression >= values
    assert len(comparison) == 2
    assert [value.hex() for value in comparison.rhs] == [
        "-0x0.0p+0",
        "-0x0.0p+0",
    ]
    model.add_constraints(comparison)

    assert model.num_constraints == 2
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.metadata.coefficients == 0


def test_sparse_comparison_survives_source_lifetime() -> None:
    model, _, left, _, right = _sparse_variables()
    comparison = left * 2.0 >= right * 3.0
    del left, right
    gc.collect()

    model.add_constraints(comparison)
    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (0, 0, "0x1.0000000000000p+1"),
        (0, 4, "-0x1.8000000000000p+1"),
        (1, 1, "0x1.0000000000000p+1"),
        (1, 5, "-0x1.8000000000000p+1"),
    ]


def test_sparse_comparison_can_be_inserted_repeatedly() -> None:
    model, _, left, _, right = _sparse_variables()
    comparison = left * 2.0 >= right * 3.0

    model.add_constraints(comparison)
    model.add_constraints(comparison)

    assert model.num_constraints == 4
    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (0, 0, "0x1.0000000000000p+1"),
        (0, 4, "-0x1.8000000000000p+1"),
        (1, 1, "0x1.0000000000000p+1"),
        (1, 5, "-0x1.8000000000000p+1"),
        (2, 0, "0x1.0000000000000p+1"),
        (2, 4, "-0x1.8000000000000p+1"),
        (3, 1, "0x1.0000000000000p+1"),
        (3, 5, "-0x1.8000000000000p+1"),
    ]


def test_sparse_arithmetic_comparison_honors_active_mask() -> None:
    model, _, left, _, right = _sparse_variables()
    comparison = left * 2.0 >= right * 3.0

    inserted = model.add_constraints(comparison, active=[True, False, False])

    assert len(inserted) == 1
    assert model.num_constraints == 1
    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (0, 0, "0x1.0000000000000p+1"),
        (0, 4, "-0x1.8000000000000p+1"),
    ]


def test_sparse_arithmetic_invalid_active_mask_does_not_mutate_model() -> None:
    model, _, left, _, right = _sparse_variables()
    comparison = left * 2.0 >= right * 3.0

    with pytest.raises(ValueError, match="broadcast"):
        model.add_constraints(comparison, active=[True, False])

    assert model.num_constraints == 0


def test_zero_rows_are_dropped_before_infinite_scale() -> None:
    model, _, left, middle, right = _sparse_variables()
    empty_then_infinite = (left * 0.0 + middle * 0.0) * float("inf")
    comparison = empty_then_infinite >= right

    assert [value.hex() for value in comparison.rhs] == [
        "-0x0.0p+0",
        "-0x0.0p+0",
    ]
    model.add_constraints(comparison)

    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (0, 4, "-0x1.0000000000000p+0"),
        (1, 5, "-0x1.0000000000000p+0"),
    ]


def test_zero_then_infinite_scale_retains_nan_rows_and_errors() -> None:
    model, _, left, _, right = _sparse_variables()
    direct_infinite = (left * 0.0) * float("inf")
    comparison = direct_infinite >= right

    assert all(math.isnan(value) for value in comparison.rhs)
    with pytest.raises(arco.ConstraintInvalidBoundsError):
        model.add_constraints(comparison)
    assert model.num_constraints == 0


def test_underflow_keeps_operation_order() -> None:
    def coefficients(build: object) -> list[tuple[int, str]]:
        model, _, values, _, _ = _sparse_variables()
        expression = build(values)
        model.add_constraints(expression >= 0.0)
        snapshot = model.inspect(include_coeffs=True)
        return sorted(
            (int(coefficient.variable_id), coefficient.value.hex())
            for coefficient in snapshot.coefficients or []
        )

    tiny = 1.0e-200
    assert coefficients(lambda values: (values * tiny + values * tiny) * tiny) == []
    assert coefficients(lambda values: values * tiny + (values * tiny) * tiny) == [
        (0, tiny.hex()),
        (1, tiny.hex()),
    ]


def test_signed_zero_addition_is_dropped_before_infinite_scale() -> None:
    model, _, left, middle, right = _sparse_variables()
    signed_zero_then_infinite = (left * -0.0 + middle * -0.0) * float("-inf")
    comparison = signed_zero_then_infinite >= right

    model.add_constraints(comparison)
    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (0, 4, "-0x1.0000000000000p+0"),
        (1, 5, "-0x1.0000000000000p+0"),
    ]


def test_lazy_labeled_parameter_fallback_preserves_coefficients() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=["a", "b"])
    hour = arco.IndexSet(name="hour", members=[0, 1, 2])
    values = model.add_variables(
        axes=(technology, hour),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, False]]),
        name="values",
    )
    weights = arco.param(
        np.array([[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]]),
        axes=(technology, hour),
        name="weights",
    )

    weighted = (values * 2.0) * weights
    model.add_constraints(weighted >= 0.0)

    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (0, 0, "0x1.0000000000000p+2"),
        (2, 1, "0x1.0000000000000p+3"),
        (4, 2, "0x1.8000000000000p+3"),
    ]


def test_mixed_sparse_comparison_keeps_union_rows() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=["a", "b"])
    hour = arco.IndexSet(name="hour", members=[0, 1, 2])
    values = model.add_variables(
        axes=(technology, hour),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, False]]),
        name="values",
    )
    weights = arco.param(
        np.array([[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]]),
        axes=(technology, hour),
        name="weights",
    )

    lazy_left = values * 2.0
    eager_right = values * weights
    comparison = lazy_left >= eager_right

    assert len(comparison) == 3
    model.add_constraints(comparison)
    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (1, 1, "-0x1.0000000000000p+1"),
        (2, 2, "-0x1.0000000000000p+2"),
    ]


def test_reverse_mixed_sparse_comparison_keeps_union_rows() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=["a", "b"])
    hour = arco.IndexSet(name="hour", members=[0, 1, 2])
    values = model.add_variables(
        axes=(technology, hour),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, False]]),
        name="values",
    )
    weights = arco.param(
        np.array([[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]]),
        axes=(technology, hour),
        name="weights",
    )

    eager_left = values * weights
    lazy_right = values * 2.0
    comparison = eager_left >= lazy_right

    assert len(comparison) == 3
    model.add_constraints(comparison)
    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (1, 1, "0x1.0000000000000p+1"),
        (2, 2, "0x1.0000000000000p+2"),
    ]


def test_rolled_sparse_comparison_keeps_labeled_weight_union_rows() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=["a", "b"])
    hour = arco.IndexSet(name="hour", members=[0, 1, 2])
    values = model.add_variables(
        axes=(technology, hour),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, False]]),
        name="values",
    )
    weights = arco.param(
        np.array([[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]]),
        axes=(technology, hour),
        name="weights",
    )

    rolled = np.roll(values, -1, axis=hour)
    comparison = rolled >= values * weights

    assert len(comparison) == 5
    model.add_constraints(comparison)
    snapshot = model.inspect(include_coeffs=True)
    assert _coefficient_signature(snapshot) == [
        (0, 0, "-0x1.0000000000000p+1"),
        (1, 1, "0x1.0000000000000p+0"),
        (2, 0, "0x1.0000000000000p+0"),
        (2, 1, "-0x1.0000000000000p+2"),
        (3, 2, "0x1.0000000000000p+0"),
        (4, 2, "-0x1.8000000000000p+2"),
    ]
