from __future__ import annotations

import gc
import math

import numpy as np
import pytest

import arco


def _axes() -> tuple[arco.IndexSet, arco.IndexSet, arco.IndexSet]:
    return (
        arco.IndexSet(name="outer", members=range(2)),
        arco.IndexSet(name="middle", members=range(3)),
        arco.IndexSet(name="inner", members=range(2)),
    )


def _add_sparse_variables(
    model: arco.Model,
) -> tuple[tuple[arco.IndexSet, ...], object]:
    axes = _axes()
    active = np.zeros((2, 3, 2), dtype=bool)
    active[0, 0, 0] = True
    active[1, 2, 1] = True
    values = model.add_variables(
        axes=axes,
        bounds=arco.NonNegativeFloat,
        active=active,
        name="values",
    )
    return axes, values


def _objective_signature(model: arco.Model) -> list[tuple[int, str]]:
    objective = model.inspect(include_coeffs=True).objective
    assert objective is not None
    return [
        (int(variable_id), coefficient.hex())
        for variable_id, coefficient in objective.terms
    ]


@pytest.mark.parametrize("axis_index", [0, 1, 2])
def test_single_axis_reduction_keeps_logical_output_groups(axis_index: int) -> None:
    model = arco.Model()
    axes, values = _add_sparse_variables(model)

    reduced = values @ axes[axis_index]
    expected_shape = tuple(
        size for index, size in enumerate((2, 3, 2)) if index != axis_index
    )

    assert reduced.shape == expected_shape
    estimate = reduced.memory_estimate()
    assert estimate["dense_slots"] == len(reduced)
    assert estimate["active_slots"] == len(reduced)

    comparison = reduced == reduced  # noqa: PLR0124 - exercise zero-row retention.
    assert len(comparison) == len(reduced)
    inserted = model.add_constraints(comparison)
    assert len(inserted) == len(reduced)
    assert model.num_constraints == len(reduced)


@pytest.mark.parametrize(
    ("axis_index", "expected_rows"),
    [(0, 6), (1, 4)],
)
def test_reduction_broadcast_to_sparse_rhs_keeps_zero_output_groups(
    axis_index: int, expected_rows: int
) -> None:
    model = arco.Model()
    axes, values = _add_sparse_variables(model)
    rhs = model.add_variables(
        axes=(axes[2],),
        bounds=arco.NonNegativeFloat,
        active=np.array([True, False]),
        name="rhs",
    )

    reduced = values @ axes[axis_index]
    comparison = reduced >= rhs

    assert comparison.shape == reduced.shape
    assert len(comparison) == expected_rows
    inserted = model.add_constraints(comparison)
    assert len(inserted) == expected_rows
    assert model.num_constraints == expected_rows


def test_at_and_sum_over_single_axis_preserve_variable_order_and_unit_bits() -> None:
    def objective_signature(reduction: str) -> list[tuple[int, str]]:
        model = arco.Model()
        axes, values = _add_sparse_variables(model)
        reduced = (
            values @ axes[1] if reduction == "matmul" else values.sum(over=axes[1])
        )
        model.minimize(reduced.sum())
        return _objective_signature(model)

    expected = [(0, (1.0).hex()), (1, (1.0).hex())]
    assert objective_signature("matmul") == expected
    assert objective_signature("sum") == expected


def test_multi_axis_and_scalar_reductions_match_sum_fallback() -> None:
    def signature(
        reduction: str, axes_to_reduce: tuple[int, ...]
    ) -> list[tuple[int, str]]:
        model = arco.Model()
        axes, values = _add_sparse_variables(model)
        selected = tuple(axes[index] for index in axes_to_reduce)
        reduced = (
            values @ selected if reduction == "matmul" else values.sum(over=selected)
        )
        model.minimize(reduced if not hasattr(reduced, "sum") else reduced.sum())
        return _objective_signature(model)

    expected = [(0, (1.0).hex()), (1, (1.0).hex())]
    assert signature("matmul", (0, 2)) == expected
    assert signature("sum", (0, 2)) == expected
    assert signature("matmul", (0, 1, 2)) == expected
    assert signature("sum", (0, 1, 2)) == expected


def test_relabelled_reduction_survives_source_gc_and_repeated_insertion() -> None:
    model = arco.Model()
    axes, values = _add_sparse_variables(model)
    expression = values * 1.0
    relabelled = expression.relabel_axis(axes[1], axes[1].alias("middle_alias"))
    del values, expression
    gc.collect()

    reduced = relabelled @ relabelled.index_sets[1]
    assert reduced.shape == (2, 2)
    estimate = reduced.memory_estimate()
    assert estimate["dense_slots"] == len(reduced)
    assert estimate["active_slots"] == len(reduced)
    assert len(model.add_constraints(reduced >= 0.0)) == len(reduced)
    assert len(model.add_constraints(reduced <= 1.0)) == len(reduced)
    assert model.num_constraints == 2 * len(reduced)


def test_mixed_finite_and_signed_zero_scaling_keeps_exact_coefficients() -> None:
    model = arco.Model()
    axes, values = _add_sparse_variables(model)
    reduced = ((values * 2.0) + (values * -0.0)) @ axes[1]

    assert reduced.shape == (2, 2)
    estimate = reduced.memory_estimate()
    assert estimate["dense_slots"] == len(reduced)
    assert estimate["active_slots"] == len(reduced)
    model.minimize(reduced.sum())
    assert _objective_signature(model) == [(0, (2.0).hex()), (1, (2.0).hex())]


def test_zero_and_underflow_scaling_keep_logical_output_groups() -> None:
    for factor in (-0.0, 1.0e-200 * 1.0e-200):
        model = arco.Model()
        axes, values = _add_sparse_variables(model)
        reduced = (values * factor) @ axes[1]
        assert reduced.shape == (2, 2)
        estimate = reduced.memory_estimate()
        assert estimate["dense_slots"] == len(reduced)
        assert estimate["active_slots"] == len(reduced)
        model.minimize(reduced.sum())
        assert _objective_signature(model) == []


def test_nonfinite_scaling_keeps_reduction_shape_and_errors_at_model_boundary() -> None:
    model = arco.Model()
    axes, values = _add_sparse_variables(model)
    reduced = (values * math.inf) @ axes[1]

    assert reduced.shape == (2, 2)
    estimate = reduced.memory_estimate()
    assert estimate["dense_slots"] == len(reduced)
    assert estimate["active_slots"] == len(reduced)
    with pytest.raises(arco.ExprCoefficientError, match="finite"):
        model.minimize(reduced.sum())
