from __future__ import annotations

import gc

import numpy as np

import arco


def _sparse_variables() -> tuple[
    arco.Model, tuple[arco.IndexSet, arco.IndexSet], object
]:
    model = arco.Model()
    region = arco.IndexSet("region", size=2)
    time = arco.IndexSet("time", size=3)
    active = np.array([[True, False, False], [False, True, False]])
    values = model.add_variables(
        axes=(region, time),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="flow",
    )
    return model, (region, time), values


def test_sparse_variable_reduction_defers_without_dropping_zero_rows() -> None:
    model, (region, _time), values = _sparse_variables()

    reduced = values.sum(over=region)

    assert reduced.shape == (3,)
    estimate = reduced.memory_estimate()
    assert estimate["storage"] == "deferred_variable_reduction"
    assert estimate["dense_slots"] == 3
    assert estimate["active_slots"] == 3
    assert estimate["linear_terms"] == 2
    comparison = reduced >= 0.0
    model.add_constraints(comparison)
    assert model.num_constraints == 3


def test_sparse_variable_reduction_preserves_terms_and_relabeling() -> None:
    model, (region, time), values = _sparse_variables()
    reduced = values @ region
    renamed = reduced.relabel_axis(time, time.alias("renamed_time"))

    del values
    gc.collect()
    model.minimize(renamed.sum())

    assert model.inspect(include_coeffs=True).objective.terms == [(0, 1.0), (1, 1.0)]
    assert renamed.index_sets[0].name == "renamed_time"


def test_sparse_variable_reduction_broadcasts_against_sparse_rhs() -> None:
    model, (region, time), values = _sparse_variables()
    other_region = arco.IndexSet("other_region", size=2)
    rhs = model.add_variables(
        axes=(other_region, time),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, False], [False, False, True]]),
        name="rhs",
    )

    comparison = values.sum(over=region) >= rhs
    model.add_constraints(comparison)

    assert model.num_constraints == 6


def test_sparse_variable_reduction_keeps_multiaxis_eager_and_single_axis_deferred() -> (
    None
):
    _model, (region, time), values = _sparse_variables()

    all_axes = values.sum(over=(region, time))
    single_axis = values.sum(over=time)

    assert all_axes.__class__.__name__ == "Expr"
    assert single_axis.memory_estimate()["storage"] == "deferred_variable_reduction"
