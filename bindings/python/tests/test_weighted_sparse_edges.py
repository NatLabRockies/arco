from __future__ import annotations

import math

import numpy as np
import pytest

import arco


def _objective_signature(snapshot: object) -> list[tuple[int, str]]:
    objective = snapshot.objective
    assert objective is not None
    return [
        (int(variable_id), coefficient.hex())
        for variable_id, coefficient in objective.terms
    ]


def _coefficient_signature(snapshot: object) -> list[tuple[int, int, str]]:
    coefficients = snapshot.coefficients
    assert coefficients is not None
    return [
        (
            int(coefficient.constraint_id),
            int(coefficient.variable_id),
            coefficient.value.hex(),
        )
        for coefficient in coefficients
    ]


def test_lower_rank_weights_expand_sparse_variable_axes() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=["a", "b"])
    region = arco.IndexSet(name="region", members=["north", "south", "west"])
    hour = arco.IndexSet(name="hour", members=[0, 1])
    values = model.add_variables(
        axes=(technology, region, hour),
        bounds=arco.NonNegativeFloat,
        active=np.array(
            [
                [[True, False], [False, True], [True, False]],
                [[False, True], [True, False], [True, True]],
            ]
        ),
        name="values",
    )
    technology_weights = arco.param(np.array([10.0, 20.0]), axes=(technology,))

    weighted = values * technology_weights

    assert weighted.shape == (2, 3, 2)
    model.minimize(weighted.sum())
    assert _objective_signature(model.inspect(include_coeffs=True)) == [
        (0, (10.0).hex()),
        (1, (10.0).hex()),
        (2, (10.0).hex()),
        (3, (20.0).hex()),
        (4, (20.0).hex()),
        (5, (20.0).hex()),
        (6, (20.0).hex()),
    ]


def test_duplicate_broadcast_variables_sum_in_target_flat_order() -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b", "c"])
    region = arco.IndexSet(name="region", members=["north", "south"])
    values = model.add_variables(
        axes=(item,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True],
        name="values",
    )
    weights = arco.param(
        np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
        axes=(region, item),
    )

    reduced = (values * weights).sum(over=region)

    assert reduced.shape == (3,)
    assert tuple(axis.name for axis in reduced.index_sets) == ("item",)
    model.minimize(reduced)
    assert _objective_signature(model.inspect(include_coeffs=True)) == [
        (0, (5.0).hex()),
        (1, (9.0).hex()),
    ]


@pytest.mark.parametrize("reduction", ["no_axis", "all_axes", "partial_axis"])
def test_weighted_sparse_sum_supports_no_all_and_partial_axes(reduction: str) -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b"])
    region = arco.IndexSet(name="region", members=["north", "south", "west"])
    values = model.add_variables(
        axes=(item, region),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, True]]),
        name="values",
    )
    weights = arco.param(
        np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
        axes=(item, region),
    )
    weighted = values * weights
    if reduction == "no_axis":
        reduced = weighted.sum()
    elif reduction == "all_axes":
        reduced = weighted.sum(over=(item, region))
    else:
        reduced = weighted.sum(over=item)

    model.minimize(reduced)
    assert (getattr(reduced, "shape", None) is None) is (reduction != "partial_axis")
    if reduction == "partial_axis":
        assert reduced.shape == (3,)
        assert tuple(axis.name for axis in reduced.index_sets) == ("region",)
    assert _objective_signature(model.inspect(include_coeffs=True)) == [
        (0, (1.0).hex()),
        (1, (3.0).hex()),
        (2, (5.0).hex()),
        (3, (6.0).hex()),
    ]


@pytest.mark.parametrize("weight", [0.0, -0.0])
def test_zero_weights_drop_terms_without_sign_artifacts(weight: float) -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=[0, 1])
    values = model.add_variables(
        axes=(axis,), bounds=arco.NonNegativeFloat, active=[True, True]
    )
    weights = arco.param(np.array([weight, weight]), axes=(axis,))

    model.minimize((values * weights).sum())

    assert _objective_signature(model.inspect(include_coeffs=True)) == []


@pytest.mark.parametrize("weight", [math.nan, math.inf, -math.inf])
def test_nonfinite_weights_are_rejected_at_model_boundary(weight: float) -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=[0])
    values = model.add_variables(
        axes=(axis,), bounds=arco.NonNegativeFloat, active=[True]
    )
    weights = arco.param(np.array([weight]), axes=(axis,))

    with pytest.raises(arco.ExprCoefficientError, match="finite"):
        model.minimize((values * weights).sum())


def test_subnormal_weights_preserve_coefficient_bits_and_underflow_to_zero() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=[0, 1, 2, 3])
    values = model.add_variables(
        axes=(axis,), bounds=arco.NonNegativeFloat, active=[True, True, True, True]
    )
    weights = arco.param(np.array([5e-324, -5e-324, 1e-320, -1e-320]), axes=(axis,))
    model.minimize((values * weights).sum())

    assert _objective_signature(model.inspect(include_coeffs=True)) == [
        (0, "0x0.0000000000001p-1022"),
        (1, "-0x0.0000000000001p-1022"),
        (2, "0x0.00000000007e8p-1022"),
        (3, "-0x0.00000000007e8p-1022"),
    ]

    underflow_model = arco.Model()
    underflow_values = underflow_model.add_variables(
        axes=(axis,), bounds=arco.NonNegativeFloat, active=[True, True, True, True]
    )
    underflow_model.minimize(((underflow_values * 1.0e-200) * 1.0e-200).sum())
    assert _objective_signature(underflow_model.inspect(include_coeffs=True)) == []


def test_sequential_labeled_weights_preserve_float_operation_order() -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b"])
    region = arco.IndexSet(name="region", members=[0, 1, 2])
    values = model.add_variables(
        axes=(item, region),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, True]]),
    )
    item_weights = arco.param(np.array([0.1, 0.3]), axes=(item,))
    region_weights = arco.param(np.array([0.2, 0.7, 0.11]), axes=(region,))

    model.minimize((values * item_weights * region_weights).sum())

    assert _objective_signature(model.inspect(include_coeffs=True)) == [
        (0, "0x1.47ae147ae147cp-6"),
        (1, "0x1.6872b020c49bbp-7"),
        (2, "0x1.ae147ae147ae1p-3"),
        (3, "0x1.0e5604189374cp-5"),
    ]


def test_weighted_sparse_comparison_applies_active_mask_after_broadcast() -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b"])
    region = arco.IndexSet(name="region", members=[0, 1, 2])
    values = model.add_variables(
        axes=(item, region),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, True]]),
    )
    region_weights = arco.param(np.array([2.0, 3.0, 4.0]), axes=(region,))
    comparison = values * region_weights >= 0.0

    inserted = model.add_constraints(
        comparison,
        active=np.array([[True, False], [False, True], [False, False]]),
    )

    assert len(inserted) == 2
    assert _coefficient_signature(model.inspect(include_coeffs=True)) == [
        (0, 0, (2.0).hex()),
        (1, 2, (3.0).hex()),
    ]


def test_relabelled_weighted_sparse_sum_and_compare_preserve_coefficients() -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b"])
    region = arco.IndexSet(name="region", members=[0, 1, 2])
    relabelled_region = region.alias("relabelled_region")
    values = model.add_variables(
        axes=(item, region),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, True]]),
    )
    region_weights = arco.param(np.array([2.0, 3.0, 4.0]), axes=(region,))
    weighted = (values * region_weights).relabel_axis(region, relabelled_region)

    reduced = weighted.sum(over=relabelled_region)
    comparison = weighted >= 0.0
    model.minimize(reduced)
    assert len(comparison) == 6
    model.add_constraints(comparison)
    snapshot = model.inspect(include_coeffs=True)

    assert _objective_signature(snapshot) == [
        (0, (2.0).hex()),
        (1, (4.0).hex()),
        (2, (3.0).hex()),
        (3, (4.0).hex()),
    ]
    assert _coefficient_signature(snapshot) == [
        (0, 0, (2.0).hex()),
        (4, 1, (4.0).hex()),
        (3, 2, (3.0).hex()),
        (5, 3, (4.0).hex()),
    ]


@pytest.mark.parametrize("reverse", [False, True])
def test_weighted_sparse_comparison_with_variable_preserves_direction(
    reverse: bool,
) -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b"])
    region = arco.IndexSet(name="region", members=[0, 1, 2])
    values = model.add_variables(
        axes=(item, region),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, True]]),
    )
    weights = arco.param(
        np.array([[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]]),
        axes=(item, region),
    )
    weighted = values * weights
    comparison = values >= weighted if reverse else weighted >= values

    assert len(comparison) == 4
    model.add_constraints(comparison)
    sign = -1.0 if reverse else 1.0
    assert _coefficient_signature(model.inspect(include_coeffs=True)) == [
        (0, 0, (sign * 1.0).hex()),
        (1, 1, (sign * 3.0).hex()),
        (2, 2, (sign * 5.0).hex()),
        (3, 3, (sign * 6.0).hex()),
    ]


@pytest.mark.parametrize("reverse", [False, True])
def test_weighted_sparse_comparison_with_lazy_expression_preserves_union_rows(
    reverse: bool,
) -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b"])
    region = arco.IndexSet(name="region", members=[0, 1, 2])
    values = model.add_variables(
        axes=(item, region),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, True]]),
    )
    weights = arco.param(
        np.array([[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]]),
        axes=(item, region),
    )
    weighted = values * weights
    lazy = values * 2.0
    comparison = lazy >= weighted if reverse else weighted >= lazy

    assert len(comparison) == 4
    model.add_constraints(comparison)
    sign = -1.0 if reverse else 1.0
    assert _coefficient_signature(model.inspect(include_coeffs=True)) == [
        (1, 1, (sign * 2.0).hex()),
        (2, 2, (sign * 4.0).hex()),
        (3, 3, (sign * 5.0).hex()),
    ]


@pytest.mark.parametrize("reverse", [False, True])
def test_weighted_sparse_comparison_with_eager_sparse_preserves_union_rows(
    reverse: bool,
) -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=range(5))
    left = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False, False],
    )
    right = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[False, True, False, True, True],
    )
    weights = arco.param(np.array([2.0, 3.0, 4.0, 5.0, 6.0]), axes=(axis,))
    weighted = left * weights
    eager = right
    for _ in range(40):
        eager = eager * 1.0 + right * 0.0

    assert eager.memory_estimate()["storage"] == "sparse"
    comparison = eager >= weighted if reverse else weighted >= eager

    assert len(comparison) == 5
    model.add_constraints(comparison)
    sign = -1.0 if reverse else 1.0
    assert _coefficient_signature(model.inspect(include_coeffs=True)) == [
        (0, 0, (sign * 2.0).hex()),
        (2, 1, (sign * 4.0).hex()),
        (1, 2, (-sign * 1.0).hex()),
        (3, 3, (-sign * 1.0).hex()),
        (4, 4, (-sign * 1.0).hex()),
    ]
