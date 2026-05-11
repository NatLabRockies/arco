from __future__ import annotations

import numpy as np
import pytest

import arco


def test_param_validates_shape_and_exposes_metadata() -> None:
    i = arco.IndexSet("i", members=["a", "b"])
    t = arco.IndexSet("t", members=[2020, 2025, 2030])

    values = np.arange(6).reshape(2, 3)
    p = arco.param(values, i, t)

    assert p.axes == (i, t)
    assert p.shape == (2, 3)
    np.testing.assert_array_equal(p.values, values)

    with pytest.raises(arco.ArrayDimensionError):
        arco.param(values, i)

    with pytest.raises(arco.ArrayShapeMismatchError):
        arco.param(values, t, i)


def test_index_set_alias_keeps_members_and_changes_name() -> None:
    r = arco.IndexSet("r", members=["north", "south"])

    r_from = r.alias("from")

    assert r_from.name == "from"
    assert r_from.size == r.size
    assert r_from.members == r.members
