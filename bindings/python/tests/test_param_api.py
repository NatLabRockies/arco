from __future__ import annotations

import numpy as np
import pytest

import arco


def test_param_validates_shape_and_exposes_metadata() -> None:
    i = arco.IndexSet(name="i", members=["a", "b"])
    t = arco.IndexSet(name="t", members=[2020, 2025, 2030])

    values = np.arange(6).reshape(2, 3)
    p = arco.param(
        values,
        axes=(
            i,
            t,
        ),
        name="weight",
    )

    assert p.axes == (i, t)
    assert p.shape == (2, 3)
    assert p.name == "weight"
    np.testing.assert_array_equal(p.values, values)
    assert p[0].name == "weight"
    assert p.sum(over=i).name == "weight"

    with pytest.raises(arco.ArrayDimensionError):
        arco.param(values, axes=(i,))

    with pytest.raises(arco.ArrayShapeMismatchError):
        arco.param(
            values,
            axes=(
                t,
                i,
            ),
        )


def test_param_requires_keyword_axes() -> None:
    i = arco.IndexSet(name="i", members=["a", "b"])

    with pytest.raises(TypeError):
        arco.param(np.array([1.0, 2.0]), i)


def test_add_variables_requires_keyword_axes() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=["a", "b"])

    with pytest.raises(TypeError):
        model.add_variables(i, bounds=arco.NonNegativeFloat)


def test_param_rejects_duplicate_axes_without_alias() -> None:
    i = arco.IndexSet(name="i", members=["a", "b"])

    with pytest.raises(arco.ArrayDimensionError):
        arco.param(np.arange(4).reshape(2, 2), axes=(i, i))


def test_param_rejects_duplicate_axis_name_even_if_size_differs() -> None:
    h = arco.IndexSet(name="h", members=[0, 1, 2])
    h_ramp = h[:-1]

    with pytest.raises(arco.ArrayDimensionError):
        arco.param(np.arange(6).reshape(3, 2), axes=(h, h_ramp))


def test_add_variables_rejects_duplicate_axes_through_shared_shape_contract() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=["a", "b"])

    with pytest.raises(arco.ArrayDimensionError, match="duplicate axis"):
        model.add_variables(axes=(i, i), bounds=arco.NonNegativeFloat)


def test_param_arithmetic_aligns_by_axis_identity() -> None:
    i = arco.IndexSet(name="i", members=["a", "b"])
    h = arco.IndexSet(name="h", members=[0, 1, 2])
    t = arco.IndexSet(name="t", members=[2020, 2025])

    hourly = arco.param(np.arange(6).reshape(2, 3), axes=(i, h))
    yearly = arco.param(np.array([[1.0, 2.0], [3.0, 4.0]]), axes=(i, t))
    product = hourly * yearly

    assert product.axes == (i, h, t)
    assert product.shape == (2, 3, 2)
    np.testing.assert_array_equal(
        product.values,
        np.array(
            [
                [[0.0, 0.0], [1.0, 2.0], [2.0, 4.0]],
                [[9.0, 12.0], [12.0, 16.0], [15.0, 20.0]],
            ]
        ),
    )


def test_param_mask_broadcasts_into_sparse_variable_creation() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=["a", "b"])
    r = arco.IndexSet(name="r", members=["x", "y"])
    h = arco.IndexSet(name="h", members=[0, 1, 2])

    active = arco.param(np.array([[True, False], [False, True]]), axes=(i, r))
    _ = model.add_variables(
        axes=(
            i,
            r,
            h,
        ),
        bounds=arco.NonNegativeFloat,
        active=active,
    )

    assert model.num_variables == 6


def test_param_array_array_protocol_accepts_dtype() -> None:
    i = arco.IndexSet(name="i", members=["a", "b"])
    p = arco.param(np.array([1.0, 2.0]), axes=(i,))

    arr = np.asarray(p, dtype=np.float32)

    assert arr.dtype == np.float32
    np.testing.assert_allclose(arr, np.array([1.0, 2.0], dtype=np.float32))


def test_param_array_numpy_sum_rejects_unsupported_kwargs() -> None:
    i = arco.IndexSet(name="i", members=["a", "b"])
    p = arco.param(np.array([1.0, 2.0]), axes=(i,))
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.ArrayTypeError) as exc:
        np.sum(p, axis=i, where=np.array([True, False]))

    assert exc.value.code == codes["ARRAY_TYPE"]
    assert arco.error_code(exc.value) == codes["ARRAY_TYPE"]
    assert "unsupported keyword 'where'" in str(exc.value)


def test_numpy_sum_and_diff_accept_named_axes() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=["a", "b"])
    h = arco.IndexSet(name="h", members=[0, 1, 2])
    t = arco.IndexSet(name="t", members=[2020, 2025])

    gen = model.add_variables(
        axes=(
            i,
            h,
            t,
        ),
        bounds=arco.NonNegativeFloat,
        name="GEN",
    )

    summed = np.sum(gen, axis=i)
    assert summed.shape == (3, 2)
    assert tuple(axis.name for axis in summed.index_sets) == ("h", "t")

    delta = np.diff(gen, axis=h)
    assert delta.shape == (2, 2, 2)
    assert delta.index_sets[1].members == [1, 2]


def test_named_axis_reduction_rejects_duplicate_axes_through_shared_contract() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=["a", "b"])
    h = arco.IndexSet(name="h", members=[0, 1])
    gen = model.add_variables(
        axes=(i, h),
        bounds=arco.NonNegativeFloat,
        name="GEN",
    )

    with pytest.raises(arco.ArrayDimensionError, match="duplicate axis"):
        np.sum(gen, axis=(i, i))


def test_solve_result_exposes_public_api_value_accessor() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.Bounds(lower=1.0, upper=10.0), name="x")
    model.minimize(x)

    result = model.solve(log_to_console=False)

    assert result.is_optimal()
    assert round(result.value(x), 6) == 1.0


def test_index_set_alias_keeps_members_and_changes_name() -> None:
    r = arco.IndexSet(name="r", members=["north", "south"])

    r_from = r.alias("from")

    assert r_from.name == "from"
    assert r_from.size == r.size
    assert r_from.members == r.members


def test_index_set_slice_preserves_axis_name_and_subset_members() -> None:
    h = arco.IndexSet(name="h", members=[0, 1, 2, 3])

    ramp = h[:-1]
    tail = h[1:]

    assert ramp.name == "h"
    assert ramp.members == [0, 1, 2]
    assert tail.name == "h"
    assert tail.members == [1, 2, 3]
