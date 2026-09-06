from __future__ import annotations

import numpy as np

import arco


def _coefficient_signature(snapshot: object) -> list[tuple[int, int, str]]:
    coefficients = snapshot.coefficients
    assert coefficients is not None
    return sorted(
        [
            (int(item.constraint_id), int(item.variable_id), item.value.hex())
            for item in coefficients
        ]
    )


def _eager_sparse(values: object) -> object:
    """Cross the bounded sparse-node depth fallback into eager sparse storage."""
    eager = values
    for _ in range(40):
        eager = eager * 1.0 + values * 0.0
    return eager


def _weighted_route_fixture() -> tuple[arco.Model, object, object, object]:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", size=6)
    left = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False, False, False],
        name="left",
    )
    right = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[False, True, False, True, False, False],
        name="right",
    )
    weights = arco.param(np.array([2.0, 3.0, 4.0, 5.0, 6.0, 7.0]), axes=(axis,))
    return model, left, right, right * weights


def test_eager_left_weighted_right_retains_dense_zero_rows() -> None:
    model, left, _right, weighted = _weighted_route_fixture()
    eager = _eager_sparse(left)

    comparison = eager >= weighted
    assert len(comparison) == 6
    model.add_constraints(comparison)

    assert model.num_constraints == 6
    assert _coefficient_signature(model.inspect(include_coeffs=True)) == [
        (0, 0, "0x1.0000000000000p+0"),
        (1, 2, "-0x1.8000000000000p+1"),
        (2, 1, "0x1.0000000000000p+0"),
        (3, 3, "-0x1.4000000000000p+2"),
    ]


def test_weighted_left_eager_right_filters_missing_zero_rows() -> None:
    model, left, _right, weighted = _weighted_route_fixture()
    eager = _eager_sparse(left)

    comparison = weighted >= eager
    assert len(comparison) == 4
    model.add_constraints(comparison)

    assert model.num_constraints == 4
    assert _coefficient_signature(model.inspect(include_coeffs=True)) == [
        (0, 0, "-0x1.0000000000000p+0"),
        (1, 2, "0x1.8000000000000p+1"),
        (2, 1, "-0x1.0000000000000p+0"),
        (3, 3, "0x1.4000000000000p+2"),
    ]


def test_sparse_variable_left_eager_rhs_filters_missing_zero_rows() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", size=6)
    left = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False, False, False],
        name="left",
    )
    rhs_source = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, False, True, False, False],
        name="rhs",
    )
    eager = _eager_sparse(rhs_source)

    comparison = left >= eager
    assert len(comparison) == 3
    model.add_constraints(comparison)

    assert model.num_constraints == 3
    assert _coefficient_signature(model.inspect(include_coeffs=True)) == [
        (0, 0, "0x1.0000000000000p+0"),
        (0, 2, "-0x1.0000000000000p+0"),
        (1, 1, "0x1.0000000000000p+0"),
        (2, 3, "-0x1.0000000000000p+0"),
    ]
