from __future__ import annotations

import numpy as np
import pytest

import arco


class LabeledMask:
    def __init__(self, values: object, axes: tuple[object, ...]) -> None:
        self.values = values
        self.axes = axes


def test_add_variables_active_mask_controls_activation() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0, 1, 2])

    _ = model.add_variables(
        axes=(i,), bounds=arco.NonNegativeFloat, active=[True, False, True]
    )

    assert model.num_variables == 2
    snapshot = model.inspect()
    statuses = [v.is_active for v in snapshot.variables]
    assert statuses == [True, True]


def test_add_constraints_active_mask_skips_inactive_rows() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0, 1, 2])
    x = model.add_variables(axes=(i,), bounds=arco.NonNegativeFloat)

    _ = model.add_constraints(
        x,
        sense="ge",
        rhs=0.0,
        active=[True, False, True],
    )

    assert model.num_constraints == 2


def test_sparse_comparison_applies_active_mask_before_insertion() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(8))
    left = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False, True, False, True, False],
        name="left",
    )
    right = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, True, False, False, True, True, False, False],
        name="right",
    )

    comparison = (left * 2.0) >= (right * 3.0)
    model.add_constraints(
        comparison, active=[True, False, False, False, False, False, False, False]
    )

    assert model.num_constraints == 1
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert [
        (coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ] == [(0, 2.0), (4, -3.0)]


def test_sparse_comparison_can_be_reused_after_masked_insertion() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(4))
    left = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False],
    )
    right = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, True, False, False],
    )

    comparison = (left * 2.0) >= (right * 3.0)
    model.add_constraints(comparison, active=[True, False, False, False])
    model.add_constraints(comparison, active=[True, False, False, False])

    assert model.num_constraints == 2
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert [
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ] == [
        (0, 0, 2.0),
        (1, 0, 2.0),
        (0, 2, -3.0),
        (1, 2, -3.0),
    ]


def test_sparse_comparison_accessors_preserve_filtered_rows() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(4))
    left = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False],
    )
    right = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, True, False, False],
    )

    comparison = (left * 2.0) >= (right * 3.0)

    assert len(comparison) == 3
    assert comparison.rhs == [0.0, 0.0, 0.0]
    with pytest.raises(arco.ArrayTypeError, match="has not been added"):
        _ = comparison[0]

    inserted = model.add_constraints(comparison, name="limit")
    assert len(inserted) == 3
    assert inserted.rhs == [0.0, 0.0, 0.0]
    assert [int(inserted[index]) for index in range(len(inserted))] == [0, 1, 2]
    assert [inserted[index].bounds.lower for index in range(len(inserted))] == [
        0.0,
        0.0,
        0.0,
    ]


def test_sparse_comparison_temporary_can_be_deleted_after_insertion() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(4))
    left = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False],
    )
    right = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, True, False, False],
    )

    model.add_constraints((left * 2.0) >= (right * 3.0))
    model.add_constraints((left * 2.0) >= (right * 3.0))

    assert model.num_constraints == 6


def test_sparse_comparison_invalid_mask_does_not_mutate_model() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(4))
    left = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False],
    )
    right = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, True, False, False],
    )

    with pytest.raises(ValueError, match="broadcast"):
        model.add_constraints((left * 2.0) >= (right * 3.0), active=[True, False])

    assert model.num_constraints == 0


def test_sparse_comparison_cancellation_retains_zero_rows() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(3))
    variables = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True],
    )

    cancellation = variables == 1.0 * variables

    assert len(cancellation) == 2
    assert cancellation.rhs == [0.0, 0.0]
    inserted = model.add_constraints(cancellation)
    assert len(inserted) == 2
    assert model.num_constraints == 2


def test_sparse_variable_to_expression_comparison_preserves_selected_rows() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(4))
    left = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True, False],
    )
    right = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, True, False, False],
    )

    model.add_constraints(left >= right * 3.0, active=[True, False, False, False])

    assert model.num_constraints == 1
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert [
        (coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ] == [(0, 1.0), (2, -3.0)]


def test_sparse_broadcast_comparison_preserves_full_rows_and_reuse() -> None:
    model = arco.Model()
    technology = arco.IndexSet(name="technology", members=range(2))
    region = arco.IndexSet(name="region", members=range(2))
    hour = arco.IndexSet(name="hour", members=range(3))
    year = arco.IndexSet(name="year", members=range(2))
    active = np.zeros((2, 2, 3, 2), dtype=bool)
    active[0, 0, 0, 0] = True
    active[1, 1, 2, 1] = True

    variables = model.add_variables(
        axes=(technology, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active,
    )
    rhs = arco.param(
        np.arange(8, dtype=float).reshape(2, 2, 2),
        axes=(technology, region, year),
    )
    comparison = variables >= rhs

    assert comparison.shape == (2, 2, 3, 2)
    assert len(comparison) == 24
    expected_rhs = [
        0.0,
        1.0,
        0.0,
        1.0,
        0.0,
        1.0,
        2.0,
        3.0,
        2.0,
        3.0,
        2.0,
        3.0,
        4.0,
        5.0,
        4.0,
        5.0,
        4.0,
        5.0,
        6.0,
        7.0,
        6.0,
        7.0,
        6.0,
        7.0,
    ]
    assert comparison.rhs == expected_rhs
    del variables, rhs
    model.add_constraints(comparison)
    model.add_constraints(comparison)

    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert model.num_constraints == 48
    assert [
        constraint.bounds.lower for constraint in snapshot.constraints[:24]
    ] == expected_rhs
    assert sorted(
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ) == [(0, 0, 1.0), (23, 1, 1.0), (24, 0, 1.0), (47, 1, 1.0)]


def test_sparse_broadcast_comparison_preserves_direct_and_scaled_rhs() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0])
    hour = arco.IndexSet(name="hour", members=[0, 1])
    cap = model.add_variables(
        axes=(i,), bounds=arco.NonNegativeFloat, active=[True], name="cap"
    )
    generation = model.add_variables(
        axes=(i, hour),
        bounds=arco.NonNegativeFloat,
        active=[True, False],
        name="generation",
    )

    direct = generation >= cap
    scaled = generation >= cap * 2.0
    assert direct.rhs == [0.0, 0.0]
    assert scaled.rhs == [0.0, 0.0]
    del cap, generation

    model.add_constraints(direct)
    model.add_constraints(scaled)

    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert [constraint.bounds.lower for constraint in snapshot.constraints] == [
        0.0,
        0.0,
        0.0,
        0.0,
    ]
    assert sorted(
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ) == [
        (0, 0, -1.0),
        (0, 1, 1.0),
        (1, 0, -1.0),
        (2, 0, -2.0),
        (2, 1, 1.0),
        (3, 0, -2.0),
    ]


def test_sparse_broadcast_variable_rhs_preserves_all_rows_and_reuse() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0])
    hour = arco.IndexSet(name="hour", members=[0, 1])
    lower = model.add_variables(
        axes=(i,), bounds=arco.NonNegativeFloat, active=[True], name="lower"
    )
    target = model.add_variables(
        axes=(i, hour),
        bounds=arco.NonNegativeFloat,
        active=[True, False],
        name="target",
    )

    comparison = target >= lower
    assert len(comparison) == 2
    del lower, target
    with pytest.raises(ValueError, match="broadcast"):
        model.add_constraints(comparison, active=[True, False, False])
    assert model.num_constraints == 0
    model.add_constraints(comparison)
    model.add_constraints(comparison)

    assert model.num_constraints == 4
    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert sorted(
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ) == [
        (0, 0, -1.0),
        (0, 1, 1.0),
        (1, 0, -1.0),
        (2, 0, -1.0),
        (2, 1, 1.0),
        (3, 0, -1.0),
    ]


def test_add_variables_active_mask_broadcasts_with_numpy_rules() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0, 1])
    r = arco.IndexSet(name="r", members=[0, 1])
    h = arco.IndexSet(name="h", members=[0, 1])

    mask = np.array([[True, False], [False, True]], dtype=bool)
    _ = model.add_variables(
        axes=(
            i,
            r,
            h,
        ),
        bounds=arco.NonNegativeFloat,
        active=mask,
    )

    snapshot = model.inspect()
    assert model.num_variables == 4
    assert _.shape == (2, 2, 2)
    assert _.dense_count == 8
    assert _.active_count == 4
    assert len(snapshot.variables) == 4
    assert all(variable.is_active for variable in snapshot.variables)


def test_sparse_variable_array_result_values_preserve_dense_shape() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0, 1, 2])
    x = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True],
        name="x",
    )
    model.minimize(x.sum())

    result = model.solve(log_to_console=False)
    values = result.value(x)

    assert values.shape == (3,)
    np.testing.assert_allclose(values[[0, 2]], np.array([0.0, 0.0]))
    assert np.isnan(values[1])


def test_large_active_mask_reports_active_count_without_dense_variable_creation() -> (
    None
):
    model = arco.Model()
    source = arco.IndexSet(name="source", members=range(64))
    sink = arco.IndexSet(name="sink", members=range(64))
    active = np.eye(64, dtype=bool)

    flow = model.add_variables(
        axes=(source, sink),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="flow",
    )
    snapshot = model.inspect()

    assert flow.shape == (64, 64)
    assert flow.dense_count == 4096
    assert flow.active_count == 64
    estimate = flow.memory_estimate()
    assert estimate["storage"] == "sparse"
    assert estimate["dense_slots"] == 4096
    assert estimate["active_slots"] == 64
    assert estimate["inactive_slots"] == 4032
    assert estimate["active_density"] == 64 / 4096
    assert estimate["linear_terms"] == 64
    assert estimate["quadratic_terms"] == 0
    assert estimate["estimated_term_bytes"] >= 64 * 8
    assert (
        estimate["estimated_dense_linear_term_bytes"] > estimate["estimated_term_bytes"]
    )
    assert estimate["estimated_inactive_linear_term_bytes"] > 0
    assert estimate["estimated_solver_coefficient_value_bytes"] == 64 * 8
    assert estimate["estimated_solver_coefficient_index_bytes"] == 64 * 8
    assert estimate["estimated_solver_variable_column_pointer_bytes"] == 65 * 8
    assert estimate["estimated_solver_sparse_matrix_bytes"] == (64 * 16) + (65 * 8)
    assert model.num_variables == 64
    assert snapshot.metadata.variables == 64
    assert snapshot.metadata.memory.sparse_matrix_bytes > 0


def test_sparse_active_mask_with_array_bounds_avoids_inactive_storage() -> None:
    model = arco.Model()
    source = arco.IndexSet(name="source", members=range(8))
    sink = arco.IndexSet(name="sink", members=range(8))
    active = np.eye(8, dtype=bool)
    upper = np.full((8, 8), 10.0)

    flow = model.add_variables(
        axes=(source, sink),
        bounds=arco.Bounds(lower=np.zeros_like(upper), upper=upper),
        active=active,
        name="flow",
    )

    estimate = flow.memory_estimate()
    assert estimate["storage"] == "sparse"
    assert flow.dense_count == 64
    assert flow.active_count == 8
    assert estimate["linear_terms"] == 8
    assert model.num_variables == 8


def test_sparse_active_mask_reconstructs_variable_names_and_bounds_on_demand() -> None:
    model = arco.Model()
    source = arco.IndexSet(name="source", members=range(3))
    sink = arco.IndexSet(name="sink", members=range(3))
    active = np.eye(3, dtype=bool)
    lower = np.zeros((3, 3))
    upper = np.arange(1.0, 10.0).reshape((3, 3))

    flow = model.add_variables(
        axes=(source, sink),
        bounds=arco.Bounds(lower=lower, upper=upper),
        active=active,
        name="flow",
    )

    diagonal = flow.variables

    assert [variable.name for variable in diagonal] == ["flow[0]", "flow[4]", "flow[8]"]
    assert [variable.bounds.upper for variable in diagonal] == [1.0, 5.0, 9.0]
    assert flow[4].name == "flow[4]"
    assert flow[4].bounds.upper == 5.0


def test_sparse_active_mask_reconstructs_run_encoded_names_after_prefix() -> None:
    model = arco.Model()
    prefix_set = arco.IndexSet(name="prefix", members=[0, 1])
    model.add_variables(axes=(prefix_set,), bounds=arco.NonNegativeFloat, name="prefix")

    i = arco.IndexSet(name="i", members=range(7))
    flow = model.add_variables(
        axes=(i,),
        bounds=arco.NonNegativeFloat,
        active=[True, True, True, False, True, True, True],
        name="flow",
    )

    assert [variable.name for variable in flow.variables] == [
        "flow[0]",
        "flow[1]",
        "flow[2]",
        "flow[4]",
        "flow[5]",
        "flow[6]",
    ]
    assert model.get_variable(name="flow[6]").name == "flow[6]"


def test_sparse_multidimensional_active_mask_uses_runs_for_name_lookup() -> None:
    model = arco.Model()
    prefix_set = arco.IndexSet(name="prefix", members=[0, 1])
    model.add_variables(axes=(prefix_set,), bounds=arco.NonNegativeFloat, name="prefix")
    source = arco.IndexSet(name="source", members=range(2))
    sink = arco.IndexSet(name="sink", members=range(4))
    active = np.array(
        [[True, True, True, False], [True, True, True, False]], dtype=bool
    )

    flow = model.add_variables(
        axes=(source, sink),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="flow",
    )

    assert flow.shape == (2, 4)
    assert [variable.name for variable in flow.variables] == [
        "flow[0]",
        "flow[1]",
        "flow[2]",
        "flow[4]",
        "flow[5]",
        "flow[6]",
    ]
    assert model.get_variable(name="flow[6]").name == "flow[6]"


def test_sparse_active_mask_with_labeled_array_bounds_reads_only_active_slots() -> None:
    model = arco.Model()
    source = arco.IndexSet(name="source", members=range(3))
    sink = arco.IndexSet(name="sink", members=range(3))
    hour = arco.IndexSet(name="hour", members=range(2))
    active = arco.param(np.eye(3, dtype=bool), axes=(source, sink))
    upper = arco.param(np.arange(1.0, 10.0).reshape((3, 3)), axes=(source, sink))

    flow = model.add_variables(
        axes=(source, sink, hour),
        bounds=arco.Bounds(lower=0, upper=upper),
        active=active,
        name="flow",
    )

    assert flow.shape == (3, 3, 2)
    assert flow.active_count == 6
    assert [variable.bounds.upper for variable in flow.variables] == [
        1.0,
        1.0,
        5.0,
        5.0,
        9.0,
        9.0,
    ]


def test_sparse_active_mask_snapshot_reconstructs_array_variable_names() -> None:
    model = arco.Model()
    source = arco.IndexSet(name="source", members=range(3))
    sink = arco.IndexSet(name="sink", members=range(3))
    active = np.eye(3, dtype=bool)

    _ = model.add_variables(
        axes=(source, sink),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="flow",
    )

    snapshot = model.inspect()

    assert [variable.name for variable in snapshot.variables] == [
        "flow[0]",
        "flow[4]",
        "flow[8]",
    ]
    assert model.get_variable(name="flow[4]").name == "flow[4]"


def test_constraint_block_names_reconstruct_without_per_row_metadata() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(3))
    x = model.add_variables(axes=(i,), bounds=arco.NonNegativeFloat, name="x")

    constraints = model.add_constraints(
        x,
        sense="ge",
        rhs=0.0,
        active=[True, False, True],
        name="limit",
    )
    snapshot = model.inspect()

    assert [constraint.name for constraint in model.constraints] == [
        "limit[0]",
        "limit[1]",
    ]
    assert [constraint.name for constraint in model.list_constraints()] == [
        "limit[0]",
        "limit[1]",
    ]
    assert [constraint.name for constraint in snapshot.constraints] == [
        "limit[0]",
        "limit[1]",
    ]
    assert constraints[1].name == "limit[1]"
    assert int(model.get_constraint(name="limit[1]")) == 1


def test_scalar_constraint_name_still_uses_explicit_metadata() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")

    _ = model.add_constraint(x >= 0.0, name="limit")

    assert model.constraints[0].name == "limit"
    assert model.inspect().constraints[0].name == "limit"
    assert int(model.get_constraint(name="limit")) == 0


def test_sparse_active_mask_reduction_counts_only_active_terms() -> None:
    model = arco.Model()
    source = arco.IndexSet(name="source", members=range(128))
    sink = arco.IndexSet(name="sink", members=range(128))
    active = np.eye(128, dtype=bool)

    flow = model.add_variables(
        axes=(source, sink),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="flow",
    )

    reduced = flow.sum(over=source)
    estimate = reduced.memory_estimate()

    assert reduced.shape == (128,)
    assert estimate["dense_slots"] == 128
    assert estimate["linear_terms"] == 128


def test_sparse_active_mask_labeled_multiply_compare_skips_inactive_rows() -> None:
    model = arco.Model()
    tech = arco.IndexSet(name="tech", members=range(4))
    region = arco.IndexSet(name="region", members=range(4))
    hour = arco.IndexSet(name="hour", members=range(3))
    active_ir = np.eye(4, dtype=bool)
    active_irh = active_ir[:, :, None]
    cf = arco.param(np.full((4, 4, 3), 0.5), axes=(tech, region, hour))

    cap = model.add_variables(
        axes=(tech, region),
        bounds=arco.NonNegativeFloat,
        active=active_ir,
        name="cap",
    )
    gen = model.add_variables(
        axes=(tech, region, hour),
        bounds=arco.NonNegativeFloat,
        active=active_irh,
        name="gen",
    )

    scaled_cap = cf * cap
    scaled_estimate = scaled_cap.memory_estimate()

    assert scaled_cap.shape == (4, 4, 3)
    assert scaled_estimate["storage"] == "sparse"
    assert scaled_estimate["dense_slots"] == 48
    assert scaled_estimate["active_slots"] == 12
    assert scaled_estimate["linear_terms"] == 12

    _ = model.add_constraints(gen <= scaled_cap, name="cap_limit")

    assert model.num_constraints == 12


def test_sparse_active_mask_chained_labeled_products_sum_active_terms() -> None:
    model = arco.Model()
    tech = arco.IndexSet(name="tech", members=range(4))
    region = arco.IndexSet(name="region", members=range(4))
    hour = arco.IndexSet(name="hour", members=range(3))
    year = arco.IndexSet(name="year", members=range(2))
    active_irt = np.eye(4, dtype=bool)[:, :, None]
    active_irth = active_irt[:, :, None, :]
    cost = arco.param(np.arange(1.0, 5.0), axes=(tech,))
    hours_weight = arco.param(np.array([2.0, 3.0, 4.0]), axes=(hour,))
    pvf = arco.param(np.array([0.95, 0.90]), axes=(year,))

    gen = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active_irth,
        name="gen",
    )

    operating_cost = pvf * cost * hours_weight * gen
    estimate = operating_cost.memory_estimate()
    objective = operating_cost.sum()

    assert estimate["storage"] == "sparse"
    assert estimate["dense_slots"] == 96
    assert estimate["active_slots"] == 24
    assert estimate["linear_terms"] == 24
    model.minimize(objective)
    snapshot = model.inspect()
    assert snapshot.objective is not None
    assert len(snapshot.objective.terms) == 24


def test_sparse_active_mask_diff_counts_ramping_terms_without_dense_storage() -> None:
    model = arco.Model()
    tech = arco.IndexSet(name="tech", members=range(4))
    region = arco.IndexSet(name="region", members=range(4))
    hour = arco.IndexSet(name="hour", members=range(5))
    year = arco.IndexSet(name="year", members=range(2))
    active_irt = np.eye(4, dtype=bool)[:, :, None]
    active_irth = active_irt[:, :, None, :]

    gen = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active_irth,
        name="gen",
    )

    ramp_delta = np.diff(gen, axis=hour)
    estimate = ramp_delta.memory_estimate()

    assert ramp_delta.shape == (4, 4, 4, 2)
    assert estimate["storage"] == "sparse"
    assert estimate["dense_slots"] == 128
    assert estimate["active_slots"] == 32
    assert estimate["linear_terms"] == 64


def test_sparse_diff_merges_holes_and_drops_zero_rows() -> None:
    model = arco.Model()
    outer = arco.IndexSet(name="outer", members=range(2))
    hour = arco.IndexSet(name="hour", members=range(4))
    inner = arco.IndexSet(name="inner", members=range(2))
    active = np.array(
        [
            [[True, False], [False, True], [True, True], [False, False]],
            [[False, True], [True, False], [False, False], [True, True]],
        ],
        dtype=bool,
    )
    gen = model.add_variables(
        axes=(outer, hour, inner),
        bounds=arco.NonNegativeFloat,
        active=active,
    )

    for axis_number, axis in enumerate((outer, hour, inner)):
        ramp_a = np.diff(gen, axis=axis)
        ramp_b = gen.diff(over=axis)
        expected_active = np.logical_or(
            np.take(active, range(active.shape[axis_number] - 1), axis=axis_number),
            np.take(active, range(1, active.shape[axis_number]), axis=axis_number),
        )
        expected_terms = np.take(
            active.astype(int), range(active.shape[axis_number] - 1), axis=axis_number
        ) + np.take(
            active.astype(int), range(1, active.shape[axis_number]), axis=axis_number
        )

        assert ramp_a.shape[axis_number] == active.shape[axis_number] - 1
        assert ramp_a.memory_estimate()["active_slots"] == int(expected_active.sum())
        assert ramp_a.memory_estimate()["linear_terms"] == int(expected_terms.sum())
        assert ramp_b.memory_estimate() == ramp_a.memory_estimate()

    zero_diff = np.diff(gen * 0.0, axis=hour)
    assert zero_diff.memory_estimate()["active_slots"] == 0


def test_sparse_diff_preserves_locations_and_signs_on_each_axis() -> None:
    active = np.array(
        [
            [[True, False], [False, True], [True, True], [False, False]],
            [[False, True], [True, False], [False, False], [True, True]],
        ],
        dtype=bool,
    )
    axes = (
        arco.IndexSet(name="outer", members=range(2)),
        arco.IndexSet(name="hour", members=range(4)),
        arco.IndexSet(name="inner", members=range(2)),
    )

    for axis_number, axis in enumerate(axes):
        model = arco.Model()
        gen = model.add_variables(
            axes=axes,
            bounds=arco.NonNegativeFloat,
            active=active,
            name="gen",
        )
        ramp = np.diff(gen, axis=axis)
        expected_active = np.logical_or(
            np.take(active, range(active.shape[axis_number] - 1), axis=axis_number),
            np.take(active, range(1, active.shape[axis_number]), axis=axis_number),
        )
        output_axes = list(axes)
        output_axes[axis_number] = axis[:-1]
        target = model.add_variables(
            axes=tuple(output_axes),
            bounds=arco.NonNegativeFloat,
            active=expected_active,
            name="target",
        )
        model.add_constraints(ramp == target, name="diff")
        snapshot = model.inspect(include_coeffs=True)
        assert snapshot.coefficients is not None

        source_ids = dict(zip(np.flatnonzero(active), map(int, gen.variables)))
        target_ids = dict(
            zip(np.flatnonzero(expected_active), map(int, target.variables))
        )
        expected_rows: list[list[tuple[int, float]]] = []
        for output_flat in np.flatnonzero(expected_active):
            coordinates = list(np.unravel_index(output_flat, expected_active.shape))
            previous_flat = np.ravel_multi_index(tuple(coordinates), active.shape)
            coordinates[axis_number] += 1
            current_flat = np.ravel_multi_index(tuple(coordinates), active.shape)
            expected_row = []
            if active.flat[current_flat]:
                expected_row.append((source_ids[current_flat], 1.0))
            if active.flat[previous_flat]:
                expected_row.append((source_ids[previous_flat], -1.0))
            expected_row.append((target_ids[output_flat], -1.0))
            expected_rows.append(expected_row)

        actual_rows: dict[int, list[tuple[int, float]]] = {}
        for coefficient in snapshot.coefficients:
            actual_rows.setdefault(coefficient.constraint_id, []).append(
                (coefficient.variable_id, coefficient.value)
            )
        assert [
            sorted(actual_rows[constraint.id]) for constraint in snapshot.constraints
        ] == [sorted(row) for row in expected_rows]
        assert ramp.index_sets[axis_number].members == list(axis.members)[1:]


def test_sparse_diff_keeps_rows_when_nonzero_terms_cancel() -> None:
    model = arco.Model()
    outer = arco.IndexSet(name="outer", members=range(2))
    hour = arco.IndexSet(name="hour", members=range(3))
    active = np.array([True, False], dtype=bool)
    gen = model.add_variables(
        axes=(outer,),
        bounds=arco.NonNegativeFloat,
        active=active,
    )
    repeated = gen * arco.param(np.ones(3), axes=(hour,))

    ramp = np.diff(repeated, axis=hour)
    estimate = ramp.memory_estimate()
    assert ramp.shape == (3 - 1, 2)
    assert estimate["active_slots"] == 2
    assert estimate["linear_terms"] == 4


def test_sparse_diff_singleton_axis_has_no_rows() -> None:
    model = arco.Model()
    hour = arco.IndexSet(name="hour", members=[0])
    gen = model.add_variables(
        axes=(hour,),
        bounds=arco.NonNegativeFloat,
        active=np.array([True], dtype=bool),
    )

    ramp = np.diff(gen, axis=hour)

    assert ramp.shape == (0,)
    assert ramp.memory_estimate()["active_slots"] == 0


def test_sparse_constraint_array_reuse_normalizes_terms() -> None:
    model = arco.Model()
    index = arco.IndexSet(name="index", members=range(3))
    variables = model.add_variables(
        axes=(index,),
        bounds=arco.NonNegativeFloat,
        active=[True, False, True],
    )
    duplicate_terms = variables == 2.0 * variables
    cancellation = variables == 1.0 * variables
    first_only = np.array([True, False, False], dtype=bool)

    model.add_constraints(duplicate_terms)
    model.add_constraints(duplicate_terms, active=first_only)
    model.add_constraints(cancellation)
    model.add_constraints(cancellation, active=first_only)

    snapshot = model.inspect(include_coeffs=True)
    assert snapshot.coefficients is not None
    assert [constraint.nnz for constraint in snapshot.constraints] == [1, 1, 1, 0, 0, 0]
    assert [
        (constraint.bounds.lower, constraint.bounds.upper)
        for constraint in snapshot.constraints
    ] == [(0.0, 0.0)] * 6
    assert sorted(
        (coefficient.constraint_id, coefficient.variable_id, coefficient.value)
        for coefficient in snapshot.coefficients
    ) == sorted([(0, 0, -1.0), (1, 1, -1.0), (2, 0, -1.0)])


def test_sparse_ramping_constraints_intersect_labeled_active_mask() -> None:
    model = arco.Model()
    tech = arco.IndexSet(name="tech", members=range(4))
    region = arco.IndexSet(name="region", members=range(4))
    hour = arco.IndexSet(name="hour", members=range(5))
    year = arco.IndexSet(name="year", members=range(2))
    hour_ramp = hour[:-1]
    active_irt = np.eye(4, dtype=bool)[:, :, None].repeat(2, axis=2)
    constraint_active = active_irt.copy()
    constraint_active[0, 0, 0] = False

    gen = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active_irt[:, :, None, :],
        name="gen",
    )
    rampup = model.add_variables(
        axes=(tech, region, hour_ramp, year),
        bounds=arco.NonNegativeFloat,
        active=active_irt[:, :, None, :],
        name="rampup",
    )

    constraints = model.add_constraints(
        rampup >= np.diff(gen, axis=hour),
        active=arco.param(constraint_active, axes=(tech, region, year)),
        name="ramping",
    )

    assert len(constraints) == 28
    assert model.num_constraints == 28


def test_sparse_active_mask_storage_balance_add_sub_stays_sparse() -> None:
    model = arco.Model()
    tech = arco.IndexSet(name="tech", members=range(4))
    region = arco.IndexSet(name="region", members=range(4))
    hour = arco.IndexSet(name="hour", members=range(5))
    year = arco.IndexSet(name="year", members=range(2))
    active = np.eye(4, dtype=bool)[:, :, None, None]

    soc = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="soc",
    )
    charge = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="charge",
    )
    gen = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="gen",
    )

    balance = soc + 0.92 * charge - gen
    estimate = balance.memory_estimate()

    assert balance.shape == (4, 4, 5, 2)
    assert estimate["storage"] == "sparse"
    assert estimate["dense_slots"] == 160
    assert estimate["active_slots"] == 40
    assert estimate["linear_terms"] == 120


def test_sparse_active_mask_rolled_storage_balance_constraints_skip_inactive_rows() -> (
    None
):
    model = arco.Model()
    tech = arco.IndexSet(name="tech", members=range(4))
    region = arco.IndexSet(name="region", members=range(4))
    hour = arco.IndexSet(name="hour", members=range(5))
    year = arco.IndexSet(name="year", members=range(2))
    active = np.eye(4, dtype=bool)[:, :, None, None]

    soc = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="soc",
    )
    charge = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="charge",
    )
    gen = model.add_variables(
        axes=(tech, region, hour, year),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="gen",
    )

    next_soc = np.roll(soc, -1, axis=hour)
    balance = soc + 0.92 * charge - gen
    rolled_estimate = next_soc.memory_estimate()

    assert rolled_estimate["storage"] == "sparse"
    assert rolled_estimate["dense_slots"] == 160
    assert rolled_estimate["active_slots"] == 40

    _ = model.add_constraints(next_soc == balance, name="storage_balance")

    assert model.num_constraints == 40


def test_reduced_alias_axes_can_relabel_for_vectorized_supply_balance() -> None:
    model = arco.Model()
    region = arco.IndexSet(name="r", members=range(4))
    region_from = region.alias("from")
    region_to = region.alias("to")
    hour = arco.IndexSet(name="h", members=range(3))
    active_routes = np.eye(4, dtype=bool)
    load = arco.param(np.full((4, 3), 10.0), axes=(region, hour))

    gen = model.add_variables(
        axes=(region, hour),
        bounds=arco.NonNegativeFloat,
        name="gen",
    )
    flow = model.add_variables(
        axes=(region_from, region_to, hour),
        bounds=arco.NonNegativeFloat,
        active=active_routes[:, :, None],
        name="flow",
    )

    imports = (flow @ region_from).relabel_axis(region_to, region)
    exports = (flow @ region_to).relabel_axis(region_from, region)
    balance = gen + 0.95 * imports - exports

    assert [axis.name for axis in imports.index_sets] == ["r", "h"]
    assert [axis.name for axis in exports.index_sets] == ["r", "h"]
    assert balance.shape == (4, 3)

    _ = model.add_constraints(balance == load, name="supply_balance")

    assert model.num_constraints == 12


def test_expression_array_memory_estimate_preserves_compact_storage() -> None:
    model = arco.Model()
    node = arco.IndexSet(name="node", members=range(128))

    x = model.add_variables(axes=(node,), bounds=arco.NonNegativeFloat, name="x")
    expr = 2.0 * x + 3.0

    variable_estimate = x.memory_estimate()
    expression_estimate = expr.memory_estimate()

    assert variable_estimate["storage"] == "compact"
    assert variable_estimate["dense_slots"] == 128
    assert variable_estimate["active_slots"] == 128
    assert variable_estimate["linear_terms"] == 128
    assert expression_estimate["storage"] == "compact"
    assert expression_estimate["dense_slots"] == 128
    assert expression_estimate["active_slots"] == 128
    assert expression_estimate["inactive_slots"] == 0
    assert expression_estimate["active_density"] == 1.0
    assert expression_estimate["linear_terms"] == 128
    assert (
        expression_estimate["estimated_term_bytes"]
        == variable_estimate["estimated_term_bytes"]
    )
    assert variable_estimate["estimated_solver_sparse_matrix_bytes"] == (128 * 16) + (
        129 * 8
    )
    assert expression_estimate["estimated_solver_sparse_matrix_bytes"] == (128 * 16) + (
        129 * 8
    )


def test_sparse_expression_sum_builds_objective_without_dense_slots() -> None:
    model = arco.Model()
    i = arco.IndexSet(name="i", members=range(4))
    h = arco.IndexSet(name="h", members=range(3))
    active = arco.param(
        np.array([True, False, True, False], dtype=bool),
        axes=(i,),
    )
    weights = arco.param(np.array([1.0, 10.0, 2.0, 20.0]), axes=(i,))

    x = model.add_variables(
        axes=(i, h),
        bounds=arco.NonNegativeFloat,
        active=active,
        name="x",
    )
    weighted = weights * x
    model.minimize(weighted.sum())

    assert x.active_count == 6
    assert weighted.memory_estimate()["storage"] == "sparse"
    assert model.inspect().objective.terms == [
        (0, 1.0),
        (1, 1.0),
        (2, 1.0),
        (3, 2.0),
        (4, 2.0),
        (5, 2.0),
    ]


def test_labeled_active_mask_rejects_duplicate_axes_through_shared_shape_contract() -> (
    None
):
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0, 1])
    mask = LabeledMask(np.eye(2, dtype=bool), axes=(i, i))

    with pytest.raises(arco.ArrayDimensionError, match="duplicate axis"):
        model.add_variables(axes=(i,), bounds=arco.NonNegativeFloat, active=mask)
