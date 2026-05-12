from __future__ import annotations

import numpy as np
import pytest

import arco


def _objective_terms(expr: object) -> list[tuple[int, float]]:
    model = arco.Model()
    i = arco.IndexSet("i", members=["a", "b"])
    r = arco.IndexSet("r", members=["north", "south"])
    t = arco.IndexSet("t", members=[2020, 2025])
    inv = model.add_variables(i, r, t, bounds=arco.NonNegativeFloat, name="INV")
    pvf = arco.param(np.array([0.95, 0.90]), t)
    cost_inv = arco.param(np.array([10.0, 20.0]), i)

    target = expr(inv, pvf, cost_inv, i, r, t)
    model.minimize(target)
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.objective is not None
    return snapshot.objective.terms


def test_total_reduction_equivalent_forms_match_coefficients() -> None:
    canonical = _objective_terms(
        lambda inv, pvf, cost_inv, _i, _r, _t: (pvf * cost_inv * inv).sum()
    )

    assert canonical == _objective_terms(
        lambda inv, pvf, cost_inv, _i, _r, _t: sum(pvf * cost_inv * inv)
    )
    assert canonical == _objective_terms(
        lambda inv, pvf, cost_inv, _i, _r, _t: np.sum(pvf * cost_inv * inv)
    )
    assert canonical == _objective_terms(
        lambda inv, pvf, cost_inv, i, r, t: np.sum(
            pvf * cost_inv * inv,
            axis=(i, r, t),
        )
    )
    assert canonical == _objective_terms(
        lambda inv, pvf, cost_inv, i, r, t: (pvf * cost_inv * inv) @ (i, r, t)
    )
    assert canonical == _objective_terms(
        lambda inv, pvf, cost_inv, i, r, t: (((pvf * cost_inv * inv) @ i) @ r) @ t
    )


def test_numpy_time_axis_equivalent_forms_match_shape() -> None:
    model = arco.Model()
    i = arco.IndexSet("i", members=["a", "b"])
    h = arco.IndexSet("h", members=[0, 1, 2])
    t = arco.IndexSet("t", members=[2020, 2025])
    gen = model.add_variables(i, h, t, bounds=arco.NonNegativeFloat, name="GEN")

    ramp_a = np.diff(gen, axis=h)
    ramp_b = gen.diff(over=h)
    ramp_c = gen[:, 1:, :] - gen[:, :-1, :]

    assert ramp_a.shape == (2, 2, 2)
    assert ramp_b.shape == ramp_a.shape
    assert ramp_c.shape == ramp_a.shape
    assert tuple(axis.members for axis in ramp_a.index_sets) == tuple(
        axis.members for axis in ramp_b.index_sets
    )

    rolled_a = np.roll(gen, -1, axis=h)
    rolled_b = gen.roll(shift=-1, over=h)
    rolled_c = np.concatenate((gen[:, 1:, :], gen[:, :1, :]), axis=h)

    assert rolled_a.shape == gen.shape
    assert rolled_b.shape == gen.shape
    assert rolled_c.shape == gen.shape
    assert tuple(axis.name for axis in rolled_a.index_sets) == tuple(
        axis.name for axis in rolled_c.index_sets
    )


def test_alias_axes_keep_directed_pair_dimensions_distinct() -> None:
    model = arco.Model()
    r = arco.IndexSet("r", members=["north", "south"])
    r_from = r.alias("from")
    r_to = r.alias("to")
    h = arco.IndexSet("h", members=[0, 1])
    t = arco.IndexSet("t", members=[2020])

    flow = model.add_variables(r_from, r_to, h, t, bounds=arco.NonNegativeFloat)
    imports = flow @ r_from
    exports = flow @ r_to

    assert imports.shape == (2, 2, 1)
    assert exports.shape == (2, 2, 1)
    assert tuple(axis.name for axis in imports.index_sets) == ("to", "h", "t")
    assert tuple(axis.name for axis in exports.index_sets) == ("from", "h", "t")


def _constraint_signature(
    builder: object,
) -> tuple[list[tuple[int, float, float]], list[tuple[int, int, float]]]:
    model = arco.Model()
    i = arco.IndexSet("i", members=["a", "b"])
    r = arco.IndexSet("r", members=["north", "south"])
    h = arco.IndexSet("h", members=[0, 1, 2])
    t = arco.IndexSet("t", members=[2020, 2025])

    cap = model.add_variables(i, r, t, bounds=arco.NonNegativeFloat, name="CAP")
    gen = model.add_variables(i, r, h, t, bounds=arco.NonNegativeFloat, name="GEN")
    cf = arco.param(np.arange(12, dtype=float).reshape(2, 2, 3) / 10.0, i, r, h)

    model.add_constraints(builder(gen, cap, cf), name="eq_cap_limit")
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    bounds = [
        (constraint.id, constraint.bounds.lower, constraint.bounds.upper)
        for constraint in snapshot.constraints
    ]
    coeffs = [
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ]
    return bounds, coeffs


def test_axis_alignment_equivalent_forms_match_coefficients() -> None:
    canonical = _constraint_signature(lambda gen, cap, cf: gen <= cf * cap)

    assert canonical == _constraint_signature(
        lambda gen, cap, _cf: (
            gen
            <= np.einsum(
                "irh,irt->irht",
                np.arange(12, dtype=float).reshape(2, 2, 3) / 10.0,
                cap,
            )
        )
    )


def test_labeled_bounds_broadcast_over_missing_axes() -> None:
    model = arco.Model()
    r = arco.IndexSet("r", members=["north", "south"])
    r_from = r.alias("from")
    r_to = r.alias("to")
    h = arco.IndexSet("h", members=[0, 1])
    t = arco.IndexSet("t", members=[2020, 2025])

    route_active = arco.param(np.array([[False, True], [True, False]]), r_from, r_to)
    transcap = arco.param(np.array([[0.0, 12.0], [8.0, 0.0]]), r_from, r_to)

    _ = model.add_variables(
        r_from,
        r_to,
        h,
        t,
        bounds=arco.Bounds(0, transcap),
        active=route_active,
        name="FLOW",
    )

    snapshot = model.inspect()
    uppers = [variable.bounds.upper for variable in snapshot.variables]
    assert model.num_variables == 8
    assert uppers == [12.0, 12.0, 12.0, 12.0, 8.0, 8.0, 8.0, 8.0]


def test_einsum_accepts_arco_array_operand() -> None:
    canonical = _objective_terms(
        lambda inv, pvf, cost_inv, _i, _r, _t: (pvf * cost_inv * inv).sum()
    )

    assert canonical == _objective_terms(
        lambda inv, _pvf, _cost_inv, _i, _r, _t: np.einsum(
            "t,i,irt->",
            np.array([0.95, 0.90]),
            np.array([10.0, 20.0]),
            inv,
        )
    )


def test_einsum_rejects_output_labels_not_in_inputs() -> None:
    model = arco.Model()
    i = arco.IndexSet("i", members=["a", "b"])
    x = model.add_variables(i, bounds=arco.NonNegativeFloat)

    with pytest.raises(arco.ArrayDimensionError):
        np.einsum("i->ij", x)


def test_array_array_ops_align_by_axis_labels_not_position() -> None:
    model = arco.Model()
    i = arco.IndexSet("i", members=["a", "b"])
    t = arco.IndexSet("t", members=[2020, 2025])
    b = model.add_variables(t, i, bounds=arco.NonNegativeFloat, name="B")
    weight = arco.param(np.array([[1.0, 2.0], [10.0, 20.0]]), i, t)

    model.minimize((weight * b).sum())
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.objective is not None

    coeffs = [float(term[1]) for term in snapshot.objective.terms]
    assert coeffs == pytest.approx([1.0, 10.0, 2.0, 20.0])
