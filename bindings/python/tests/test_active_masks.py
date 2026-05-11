from __future__ import annotations

import numpy as np

import arco


def test_add_variables_active_mask_controls_activation() -> None:
    model = arco.Model()
    i = arco.IndexSet("i", members=[0, 1, 2])

    _ = model.add_variables(i, bounds=arco.NonNegativeFloat, active=[True, False, True])

    assert model.num_variables == 2
    snapshot = model.inspect()
    statuses = [v.is_active for v in snapshot.variables]
    assert statuses == [True, True]


def test_add_constraints_active_mask_skips_inactive_rows() -> None:
    model = arco.Model()
    i = arco.IndexSet("i", members=[0, 1, 2])
    x = model.add_variables(i, bounds=arco.NonNegativeFloat)

    _ = model.add_constraints(
        x,
        sense="ge",
        rhs=0.0,
        active=[True, False, True],
    )

    assert model.num_constraints == 2


def test_add_variables_active_mask_broadcasts_with_numpy_rules() -> None:
    model = arco.Model()
    i = arco.IndexSet("i", members=[0, 1])
    r = arco.IndexSet("r", members=[0, 1])
    h = arco.IndexSet("h", members=[0, 1])

    mask = np.array([[True, False], [False, True]], dtype=bool)
    _ = model.add_variables(i, r, h, bounds=arco.NonNegativeFloat, active=mask)

    snapshot = model.inspect()
    assert model.num_variables == 4
    assert len(snapshot.variables) == 4
    assert all(variable.is_active for variable in snapshot.variables)
