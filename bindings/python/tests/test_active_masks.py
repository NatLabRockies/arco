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
    assert estimate["storage"] == "full"
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


def test_labeled_active_mask_rejects_duplicate_axes_through_shared_shape_contract() -> (
    None
):
    model = arco.Model()
    i = arco.IndexSet(name="i", members=[0, 1])
    mask = LabeledMask(np.eye(2, dtype=bool), axes=(i, i))

    with pytest.raises(arco.ArrayDimensionError, match="duplicate axis"):
        model.add_variables(axes=(i,), bounds=arco.NonNegativeFloat, active=mask)
