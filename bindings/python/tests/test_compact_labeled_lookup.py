from __future__ import annotations

import numpy as np

import arco


def _objective_terms(model: arco.Model) -> list[tuple[int, float]]:
    snapshot = model.inspect()
    assert snapshot.objective is not None
    return snapshot.objective.terms


def test_weighted_lookup_keeps_edge_active_rows_and_skips_holes() -> None:
    model = arco.Model()
    axis = arco.IndexSet(name="axis", members=range(6))
    values = model.add_variables(
        axes=(axis,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, False, False, False, True],
    )
    weights = arco.param(np.array([2.0, 3.0, 4.0, 5.0, 6.0, 7.0]), axes=(axis,))

    model.minimize((weights * values).sum())

    assert _objective_terms(model) == [(0, 2.0), (1, 7.0)]


def test_weighted_lookup_follows_reordered_broadcast_axes() -> None:
    model = arco.Model()
    item = arco.IndexSet(name="item", members=["a", "b"])
    hour = arco.IndexSet(name="hour", members=[0, 1, 2])
    values = model.add_variables(
        axes=(item, hour),
        bounds=arco.NonNegativeFloat,
        active=np.array([[True, False, True], [False, True, False]]),
    )
    weights = arco.param(
        np.array([[1.0, 2.0], [3.0, 5.0], [4.0, 6.0]]),
        axes=(hour, item),
    )

    weighted = weights * values
    assert weighted.shape == (3, 2)
    model.minimize(weighted.sum())

    assert _objective_terms(model) == [(0, 1.0), (1, 4.0), (2, 5.0)]
