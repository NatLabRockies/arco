from __future__ import annotations

import builtins
import enum
import os
import subprocess
import sys

import numpy as np
import pytest

import arco


def test_scalar_model_api_contract_solves_with_named_objects() -> None:
    model = arco.Model()
    x = model.add_variable(
        bounds=arco.Bounds(lower=1.0, upper=float("inf")),
        name="x",
    )
    y = model.add_variable(
        bounds=arco.Bounds(lower=2.0, upper=float("inf")),
        name="y",
    )

    constraint = model.add_constraint(x + y >= 5.0, name="demand")
    model.minimize(3.0 * x + 2.0 * y, name="total_cost")

    result = model.solve(log_to_console=False)

    assert result.is_optimal()
    assert constraint.name == "demand"
    assert round(result.objective_value, 6) == 11.0
    assert round(result.value(x), 6) == 1.0
    assert round(result.value(y), 6) == 4.0


def test_scalar_api_contract_requires_keyword_model_constructor_configuration() -> None:
    simplify_level = arco.SimplifyLevel.NONE
    solver = arco.HiGHS()

    try:
        arco.Model(arco.SimplifyLevel.NONE)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional simplify_level in Model() to fail")

    try:
        arco.Model(arco.HiGHS())
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional solver in Model() to fail")

    try:
        arco.Model(arco.SimplifyLevel.NONE, arco.HiGHS())
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional simplify_level/solver in Model() to fail"
        )

    try:
        arco.Model(simplify_level)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional simplify_level variable in Model() to fail"
        )

    try:
        arco.Model(solver)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional solver variable in Model() to fail")

    try:
        arco.Model(simplify_level, solver)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional simplify_level/solver variables in Model() to fail"
        )

    try:
        arco.Model(arco.SimplifyLevel.NONE, solver)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional mixed simplify_level literal/solver variable to fail"
        )

    try:
        arco.Model(simplify_level, arco.HiGHS())
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional mixed simplify_level variable/solver literal to fail"
        )


def test_indexed_model_api_contract_solves_with_array_result_access() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    demand = arco.param(
        np.array([3.0, 5.0]),
        axes=(plant,),
        name="demand",
    )
    cost = arco.param(
        np.array([2.0, 1.0]),
        axes=(plant,),
        name="cost",
    )

    output = model.add_variables(
        axes=(plant,),
        bounds=arco.NonNegativeFloat,
        name="output",
    )
    constraints = model.add_constraints(output >= demand, name="meet_demand")
    model.minimize((cost * output).sum(), name="total_cost")

    snapshot = model.inspect(include_coeffs=True)
    result = model.solve(log_to_console=False)

    assert snapshot.metadata.variables == 2
    assert snapshot.metadata.constraints == 2
    assert snapshot.metadata.coefficients == 2
    assert snapshot.metadata.memory.coefficient_value_bytes == 16
    assert snapshot.metadata.memory.sparse_matrix_bytes >= 16
    assert result.is_optimal()
    assert [constraint.name for constraint in constraints] == [
        "meet_demand[0]",
        "meet_demand[1]",
    ]
    assert round(result.objective_value, 6) == 11.0
    np.testing.assert_allclose(result.value(output), np.array([3.0, 5.0]))


def test_model_matrix_profile_reports_column_density_without_sparse_export() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    y = model.add_variable(bounds=arco.NonNegativeFloat, name="y")
    z = model.add_variable(bounds=arco.NonNegativeFloat, name="z")
    model.add_constraint(x >= 1.0, name="x_min")
    model.add_constraint(x + y >= 2.0, name="xy_min")
    model.add_constraint(x <= 4.0, name="x_max")
    model.minimize(x + y + z, name="cost")

    profile = model.matrix_profile(top_n=2, dense_threshold=2)
    buckets = profile["column_nnz_buckets"]
    top_columns = profile["top_columns"]

    assert profile["num_variables"] == 3
    assert profile["num_constraints"] == 3
    assert profile["num_coefficients"] == 4
    assert profile["max_column_nnz"] == 3
    assert profile["min_nonzero_column_nnz"] == 1
    assert profile["dense_columns"] == 1
    assert buckets["eq_0"] == 1
    assert buckets["eq_1"] == 1
    assert top_columns[0] == {"variable_id": int(x), "name": "x", "nnz": 3}
    assert top_columns[1] == {"variable_id": int(y), "name": "y", "nnz": 1}


def test_model_reserve_preserves_build_and_solve_behavior() -> None:
    model = arco.Model()
    model.reserve(num_variables=4, num_constraints=3)

    x = model.add_variable(bounds=arco.NonNegativeFloat)
    y = model.add_variable(bounds=arco.NonNegativeFloat)
    model.add_constraint(x + y >= 3.0)
    model.add_constraint(x <= 5.0)
    model.minimize(x + 2.0 * y)

    result = model.solve(log_to_console=False)

    assert model.num_variables == 2
    assert model.num_constraints == 2
    assert result.is_optimal()
    assert round(result.objective_value, 6) == 3.0


def test_model_can_append_objective_terms_incrementally() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    y = model.add_variable(bounds=arco.NonNegativeFloat, name="y")
    model.add_constraint(x >= 1.0)
    model.add_constraint(y >= 2.0)

    model.minimize(x)
    model.add_objective_terms(2.0 * y)

    result = model.solve(log_to_console=False)

    assert result.is_optimal()
    assert round(result.objective_value, 6) == 5.0


def test_model_rejects_appended_objective_terms_without_objective_sense() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")

    with pytest.raises(arco.ObjectiveMissingError):
        model.add_objective_terms(x)


def test_debug_api_contract_exposes_constraint_slack_and_dual() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="total_cost")

    result = model.solve(log_to_console=False)

    assert result.is_optimal()
    assert round(result.value(x), 6) == 1.0
    assert round(result.slack(lower_bound), 6) == 0.0
    assert round(result.dual(lower_bound), 6) == 1.0


def test_debug_api_contract_uses_named_result_accessors_only() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")

    result = model.solve(log_to_console=False)

    assert round(result.reduced_cost(x), 6) == 1.0
    assert not hasattr(result, "get_value")
    assert not hasattr(result, "get_dual")
    assert not hasattr(result, "get_slack")
    assert not hasattr(result, "get_reduced_cost")
    assert not hasattr(result, "primal")
    assert not hasattr(result, "variable_dual")
    assert not hasattr(result, "constraint_dual")


def test_debug_api_contract_keeps_raw_result_vector_access_in_expert_path() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")

    result = model.solve(log_to_console=False)

    primal = result.get_primal(index=int(x))
    constraint_dual = result.get_constraint_dual(index=int(lower_bound))
    variable_dual = result.get_variable_dual(index=int(x))

    assert round(primal, 6) == round(result.value(x), 6) == 1.0
    assert round(constraint_dual, 6) == round(result.dual(lower_bound), 6) == 1.0
    assert isinstance(variable_dual, float)
    assert round(variable_dual, 6) == 0.0
    assert round(result.reduced_cost(x), 6) == 0.0


def test_debug_api_contract_keeps_raw_result_vector_properties_in_expert_path() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")

    result = model.solve(log_to_console=False)

    primal_values = result.primal_values
    constraint_duals = result.constraint_duals
    variable_duals = result.variable_duals

    assert result.num_primal_values() == len(primal_values) == 1
    assert result.num_constraint_duals() == len(constraint_duals) == 1
    assert result.num_variable_duals() == len(variable_duals) == 1
    assert round(primal_values[int(x)], 6) == round(result.get_primal(index=int(x)), 6)
    assert round(constraint_duals[int(lower_bound)], 6) == round(
        result.get_constraint_dual(index=int(lower_bound)), 6
    )
    assert round(variable_duals[int(x)], 6) == round(
        result.get_variable_dual(index=int(x)), 6
    )


def test_objective_only_solve_can_skip_solution_vectors() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(2.0 * x, name="cost")

    result = model.solve(
        solver=arco.HiGHS(
            log_to_console=False,
            parameters={
                "arco.extract_solution": "false",
                "arco.fingerprint": "false",
            },
        )
    )

    assert result.is_optimal()
    assert round(result.objective_value, 6) == 2.0
    assert result.num_primal_values() == 0
    assert result.num_variable_duals() == 0
    assert result.num_constraint_duals() == 0


def test_consuming_solve_releases_model_after_handoff() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(2.0 * x, name="cost")

    result = model.solve(
        solver=arco.HiGHS(
            log_to_console=False,
            parameters={
                "arco.consume_model": "true",
                "arco.extract_solution": "false",
                "arco.fingerprint": "false",
            },
        )
    )

    assert result.is_optimal()
    assert round(result.objective_value, 6) == 2.0
    assert result.num_primal_values() == 0
    assert model.num_variables == 0
    assert model.num_constraints == 0


def test_consuming_infeasible_highs_solve_releases_model_after_handoff() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=0.0), name="x")
    model.add_constraint(x >= 1.0, name="infeasible")
    model.minimize(x)

    result = model.solve(
        solver=arco.HiGHS(
            log_to_console=False,
            parameters={"arco.consume_model": "true"},
        )
    )

    assert result.status == arco.SolutionStatus.INFEASIBLE
    assert model.num_variables == 0
    assert model.num_constraints == 0


def test_nonconsuming_highs_solve_keeps_model_available() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(2.0 * x)

    result = model.solve(solver=arco.HiGHS(log_to_console=False))

    assert result.is_optimal()
    assert round(result.objective_value, 6) == 2.0
    assert model.num_variables == 1
    assert model.num_constraints == 1


def test_debug_api_contract_requires_keyword_solve_configuration() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    solver = arco.HiGHS()
    log_to_console = False

    try:
        model.solve(arco.HiGHS())
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional solver argument to solve() to fail")

    try:
        model.solve(False)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional log_to_console argument to solve() to fail"
        )

    try:
        model.solve(1.0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional solve configuration to fail")

    try:
        model.solve(arco.HiGHS(), False)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected combined positional solve config to fail")

    try:
        model.solve(solver)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional solve variable argument to fail")

    try:
        model.solve(solver, log_to_console)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional solve variable arguments to fail"
        )

    try:
        model.solve(arco.HiGHS(), log_to_console)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional solve literal/variable arguments to fail"
        )

    try:
        model.solve(solver, False)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional solve variable/literal arguments to fail"
        )


def test_debug_api_contract_requires_keyword_inspect_configuration() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    include_coeffs = True
    payload = [0]

    try:
        model.inspect(True)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional include_coeffs argument to inspect() to fail"
        )

    try:
        model.inspect([0])
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional inspect configuration payload to fail"
        )

    try:
        model.inspect(include_coeffs)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional inspect variable argument to fail")

    try:
        model.inspect(payload)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional inspect payload variable to fail")


def test_model_api_contract_removes_beginner_name_metadata_shortcuts() -> None:
    model = arco.Model()

    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    constraint = model.add_constraint(x >= 1.0, name="minimum")
    model.minimize(x, name="cost")

    assert model.get_variable(name="x").name == "x"
    assert model.get_constraint(name="minimum").name == constraint.name
    assert not hasattr(model, "get_variable_by_name")
    assert not hasattr(model, "get_constraint_by_name")
    assert not hasattr(model, "get_variable_name")
    assert not hasattr(model, "get_constraint_name")
    assert hasattr(model, "get_variable_metadata")
    assert not hasattr(model, "get_constraint_metadata")
    assert not hasattr(model, "set_variable_metadata")
    assert not hasattr(model, "set_constraint_metadata")
    assert not hasattr(model, "get_objective_name")
    assert not hasattr(model, "set_objective_name")
    assert not hasattr(model, "get_columns")
    assert not hasattr(model, "export_arrow")


def test_scalar_model_api_contract_round_trips_variable_metadata() -> None:
    model = arco.Model()
    metadata = {"role": "output", "tags": ["primary", "tracked"]}

    x = model.add_variable(
        bounds=arco.Bounds(lower=1.0, upper=float("inf")),
        name="x",
        metadata=metadata,
    )
    y = model.add_variable(bounds=arco.Bounds(lower=2.0, upper=float("inf")), name="y")

    snapshot = model.inspect()

    assert model.get_variable_metadata(x) == metadata
    assert model.get_variable_metadata(y) is None
    assert snapshot.variables[0].metadata == metadata
    assert snapshot.variables[1].metadata is None


def test_debug_api_contract_exposes_named_lookup_error_codes() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    constraint = model.add_constraint(x >= 1.0, name="minimum")
    model.minimize(x, name="cost")
    codes = arco.diagnostic_codes()

    assert model.get_variable(name="x").name == "x"
    assert model.get_constraint(name="minimum").name == constraint.name

    try:
        model.get_variable(name="missing")
    except arco.VariableNotFoundError as exc:
        assert exc.code == codes["VARIABLE_NOT_FOUND"]
        assert arco.error_code(exc) == codes["VARIABLE_NOT_FOUND"]
    else:  # pragma: no cover
        raise AssertionError("expected missing variable lookup to fail")

    try:
        model.get_constraint(name="missing")
    except arco.ConstraintNotFoundError as exc:
        assert exc.code == codes["CONSTRAINT_NOT_FOUND"]
        assert arco.error_code(exc) == codes["CONSTRAINT_NOT_FOUND"]
    else:  # pragma: no cover
        raise AssertionError("expected missing constraint lookup to fail")


def test_debug_api_contract_requires_keyword_named_lookups() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="minimum")
    model.minimize(x, name="cost")
    variable_name = "x"
    constraint_name = "minimum"
    extra_arg = "extra"

    try:
        model.get_variable("x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional get_variable lookup to fail")

    try:
        model.get_constraint("minimum")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional get_constraint lookup to fail")

    try:
        model.get_variable("x", "extra")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected combined positional get_variable misuse to fail")

    try:
        model.get_constraint("minimum", "extra")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional get_constraint misuse to fail"
        )

    try:
        model.get_variable(variable_name, extra_arg)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional get_variable variable misuse to fail"
        )

    try:
        model.get_constraint(constraint_name, extra_arg)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional get_constraint variable misuse to fail"
        )

    try:
        model.get_variable("x", extra_arg)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional get_variable literal/variable misuse to fail"
        )

    try:
        model.get_constraint("minimum", extra_arg)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected combined positional get_constraint literal/variable misuse to fail"
        )

    try:
        model.get_variable(variable_name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional variable-name lookup to fail")

    try:
        model.get_constraint(constraint_name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional constraint-name lookup to fail")


def test_debug_api_contract_exposes_stable_python_error_codes() -> None:
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    codes = arco.diagnostic_codes()

    try:
        arco.param(np.array([1.0, 2.0, 3.0]), axes=(plant,))
    except arco.ArrayShapeMismatchError as exc:
        assert exc.code == codes["ARRAY_SHAPE_MISMATCH"]
        assert arco.error_code(exc) == codes["ARRAY_SHAPE_MISMATCH"]
    else:  # pragma: no cover
        raise AssertionError("expected parameter shape validation to fail")


def test_indexed_api_contract_exposes_array_dimension_error_code() -> None:
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    codes = arco.diagnostic_codes()

    try:
        arco.param(np.array([[1.0, 2.0], [3.0, 4.0]]), axes=(plant, plant))
    except arco.ArrayDimensionError as exc:
        assert exc.code == codes["ARRAY_DIMENSION"]
        assert arco.error_code(exc) == codes["ARRAY_DIMENSION"]
    else:  # pragma: no cover
        raise AssertionError("expected duplicate-axis validation to fail")


def test_indexed_api_contract_exposes_array_type_error_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.param(np.array([1.0]), axes=("plant",))
    except arco.ArrayTypeError as exc:
        assert exc.code == codes["ARRAY_TYPE"]
        assert arco.error_code(exc) == codes["ARRAY_TYPE"]
    else:  # pragma: no cover
        raise AssertionError("expected non-IndexSet axis validation to fail")


def test_indexed_api_contract_exposes_array_index_error_code() -> None:
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    hour = arco.IndexSet(name="hour", members=[1, 2])
    demand = arco.param(np.array([1.0, 2.0]), axes=(plant,))
    codes = arco.diagnostic_codes()

    try:
        demand.sum(over=hour)
    except arco.ArrayIndexError as exc:
        assert exc.code == codes["ARRAY_INDEX"]
        assert arco.error_code(exc) == codes["ARRAY_INDEX"]
    else:  # pragma: no cover
        raise AssertionError("expected missing-axis lookup to fail")


def test_indexed_api_contract_exposes_array_getitem_index_code() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    output = model.add_variables(
        axes=(plant,),
        bounds=arco.NonNegativeFloat,
        name="output",
    )
    exprs = output + 1.0
    constraints = model.add_constraints(output, sense="ge", rhs=0.0)
    codes = arco.diagnostic_codes()

    for array in (output, exprs, constraints):
        with pytest.raises(arco.ArrayIndexError) as exc:
            _ = array[99]
        assert exc.value.code == codes["ARRAY_INDEX"]
        assert arco.error_code(exc.value) == codes["ARRAY_INDEX"]


def test_indexed_api_contract_exposes_array_comparison_type_code() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    output = model.add_variables(
        axes=(plant,),
        bounds=arco.NonNegativeFloat,
        name="output",
    )
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.ArrayTypeError) as exc:
        _ = output >= object()
    assert exc.value.code == codes["ARRAY_TYPE"]
    assert arco.error_code(exc.value) == codes["ARRAY_TYPE"]


def test_indexed_api_contract_exposes_index_set_empty_error_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.IndexSet(name="empty", members=[])
    except arco.IndexSetEmptyError as exc:
        assert exc.code == codes["INDEX_SET_EMPTY"]
        assert arco.error_code(exc) == codes["INDEX_SET_EMPTY"]
    else:  # pragma: no cover
        raise AssertionError("expected empty IndexSet validation to fail")


def test_indexed_api_contract_exposes_index_set_argument_error_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.IndexSet(name="bad")
    except arco.IndexSetArgumentError as exc:
        assert exc.code == codes["INDEX_SET_ARGUMENT"]
        assert arco.error_code(exc) == codes["INDEX_SET_ARGUMENT"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid IndexSet arguments to fail")


def test_indexed_api_contract_exposes_index_set_index_error_code() -> None:
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.IndexSetIndexError) as exc:
        _ = plant[99]
    assert exc.value.code == codes["INDEX_SET_INDEX"]
    assert arco.error_code(exc.value) == codes["INDEX_SET_INDEX"]
    assert list(plant) == plant.members


def test_indexed_api_contract_accepts_legacy_positional_index_set_name() -> None:
    set_name = "plant"
    size = 3
    members = ["north", "south"]

    plant = arco.IndexSet("plant", members=["north", "south"])
    assert plant.name == "plant"
    assert plant.members == ["north", "south"]

    plant_from_variable = arco.IndexSet(set_name, members=["north", "south"])
    assert plant_from_variable.name == set_name

    try:
        arco.IndexSet(name="plant", members=["north", "south"], size=2)
    except arco.IndexSetArgumentError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected simultaneous IndexSet size/members to fail")

    try:
        arco.IndexSet("plant", ["north", "south"])
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional IndexSet members to fail")

    try:
        arco.IndexSet("plant", members)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional IndexSet variable members to fail")

    try:
        arco.IndexSet("T", 3)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional IndexSet size to fail")

    try:
        arco.IndexSet("T", size)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional IndexSet variable size to fail")


def test_scalar_api_contract_requires_keyword_variable_bounds() -> None:
    model = arco.Model()

    try:
        model.add_variable(arco.NonNegativeFloat, name="x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional add_variable bounds to fail")

    try:
        model.add_variable(arco.NonNegativeFloat, True)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional add_variable arguments to fail")


def test_scalar_api_contract_accepts_legacy_positional_bounds_fields() -> None:
    bounds = arco.Bounds(0.0, 1.0)

    assert bounds.lower == 0.0
    assert bounds.upper == 1.0


def test_indexed_api_contract_exposes_index_set_type_error_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.IndexSet(name="bad", members=[object()])
    except arco.IndexSetTypeError as exc:
        assert exc.code == codes["INDEX_SET_TYPE"]
        assert arco.error_code(exc) == codes["INDEX_SET_TYPE"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid IndexSet member type to fail")


def test_scalar_api_contract_exposes_bounds_invalid_error_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.Bounds(lower=10.0, upper=0.0)
    except arco.BoundsInvalidError as exc:
        assert exc.code == codes["BOUNDS_INVALID"]
        assert arco.error_code(exc) == codes["BOUNDS_INVALID"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid Bounds validation to fail")


def test_scalar_api_contract_exposes_bounds_missing_error_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.Bounds(lower=0.0)
    except arco.BoundsInvalidError as exc:
        assert exc.code == codes["BOUNDS_INVALID"]
        assert arco.error_code(exc) == codes["BOUNDS_INVALID"]
    else:  # pragma: no cover
        raise AssertionError("expected incomplete Bounds validation to fail")


def test_scalar_api_contract_exposes_bounds_type_error_code() -> None:
    model = arco.Model()
    codes = arco.diagnostic_codes()

    try:
        model.add_variable(bounds=object())
    except arco.BoundsInvalidError as exc:
        assert exc.code == codes["BOUNDS_INVALID"]
        assert arco.error_code(exc) == codes["BOUNDS_INVALID"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid bounds object to fail")


def test_scalar_api_contract_exposes_constraint_bounds_missing_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    codes = arco.diagnostic_codes()

    try:
        model.add_constraint(x + 1.0, name="missing_bounds")
    except arco.ConstraintBoundsMissingError as exc:
        assert exc.code == codes["CONSTRAINT_BOUNDS_MISSING"]
        assert arco.error_code(exc) == codes["CONSTRAINT_BOUNDS_MISSING"]
    else:  # pragma: no cover
        raise AssertionError("expected bare expression constraint to require bounds")


def test_scalar_api_contract_exposes_constraint_type_code() -> None:
    model = arco.Model()
    codes = arco.diagnostic_codes()

    try:
        model.add_constraint(object(), name="bad_constraint")
    except arco.ConstraintTypeError as exc:
        assert exc.code == codes["CONSTRAINT_TYPE"]
        assert arco.error_code(exc) == codes["CONSTRAINT_TYPE"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid constraint expression type to fail")


def test_indexed_api_contract_exposes_axes_type_code() -> None:
    model = arco.Model()
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.ArrayTypeError) as exc:
        model.add_variables(axes=("not-an-index-set",), bounds=arco.NonNegativeFloat)

    assert exc.value.code == codes["ARRAY_TYPE"]
    assert arco.error_code(exc.value) == codes["ARRAY_TYPE"]


def test_debug_api_contract_exposes_constraint_invalid_id_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    codes = arco.diagnostic_codes()

    try:
        model.set_coefficient(var_idx=int(x), constraint_idx=999, coeff=1.0)
    except arco.ConstraintInvalidIdError as exc:
        assert exc.code == codes["CONSTRAINT_INVALID_ID"]
        assert arco.error_code(exc) == codes["CONSTRAINT_INVALID_ID"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid constraint id to fail")


def test_scalar_api_contract_exposes_model_binary_bounds_code() -> None:
    model = arco.Model()
    codes = arco.diagnostic_codes()

    try:
        model.add_variable(bounds=arco.Bounds(lower=0.0, upper=2.0), is_binary=True)
    except arco.ModelBinaryBoundsError as exc:
        assert exc.code == codes["MODEL_BINARY_BOUNDS"]
        assert arco.error_code(exc) == codes["MODEL_BINARY_BOUNDS"]
    else:  # pragma: no cover
        raise AssertionError("expected binary variable bounds validation to fail")


def test_indexed_api_contract_exposes_constraint_sense_code() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    demand = arco.param(np.array([1.0, 2.0]), axes=(plant,))
    output = model.add_variables(
        axes=(plant,),
        bounds=arco.NonNegativeFloat,
        name="output",
    )
    comparison = output >= demand
    codes = arco.diagnostic_codes()

    try:
        model.add_constraints(comparison, rhs=demand, name="bad_constraints")
    except arco.ConstraintSenseError as exc:
        assert exc.code == codes["CONSTRAINT_SENSE"]
        assert arco.error_code(exc) == codes["CONSTRAINT_SENSE"]
    else:  # pragma: no cover
        raise AssertionError("expected comparison array to reject extra rhs")


def test_indexed_api_contract_exposes_invalid_constraint_sense_code() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    output = model.add_variables(
        axes=(plant,),
        bounds=arco.NonNegativeFloat,
        name="output",
    )
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.ConstraintSenseError) as exc:
        model.add_constraints(output, sense="greaterish", rhs=0.0)

    assert exc.value.code == codes["CONSTRAINT_SENSE"]
    assert arco.error_code(exc.value) == codes["CONSTRAINT_SENSE"]


def test_debug_api_contract_exposes_solver_setting_error_codes() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.Solver(threads=0)
    except arco.SolverInvalidSettingError as exc:
        assert exc.code == codes["SOLVER_INVALID_SETTING"]
        assert arco.error_code(exc) == codes["SOLVER_INVALID_SETTING"]
    else:  # pragma: no cover
        raise AssertionError("expected solver setting validation to fail")


def test_solver_api_exposes_solver_independent_lp_algorithm() -> None:
    solver = arco.HiGHS(lp_algorithm=arco.LpAlgorithm.DUAL_SIMPLEX)
    assert solver.lp_algorithm == arco.LpAlgorithm.DUAL_SIMPLEX

    updated = solver.copy(update={"lp_algorithm": arco.LpAlgorithm.BARRIER})
    assert updated.lp_algorithm == arco.LpAlgorithm.BARRIER
    assert solver.lp_algorithm == arco.LpAlgorithm.DUAL_SIMPLEX

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0)
    model.minimize(x)
    result = model.solve(
        lp_algorithm=arco.LpAlgorithm.PRIMAL_SIMPLEX,
        log_to_console=False,
    )
    assert result.is_optimal()


def test_debug_api_contract_rejects_xpress_unsupported_verbosity() -> None:
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.SolverInvalidSettingError) as constructor_exc:
        arco.Xpress(verbosity=1)
    assert constructor_exc.value.code == codes["SOLVER_INVALID_SETTING"]
    assert arco.error_code(constructor_exc.value) == codes["SOLVER_INVALID_SETTING"]

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    selection = arco.SolverSelection.family("xpress")

    with pytest.raises(arco.SolverInvalidSettingError) as solve_exc:
        model.solve(solver=selection, verbosity=1, log_to_console=False)
    assert solve_exc.value.code == codes["SOLVER_INVALID_SETTING"]
    assert arco.error_code(solve_exc.value) == codes["SOLVER_INVALID_SETTING"]


def test_debug_api_contract_rejects_primal_start_with_solver_setting_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    codes = arco.diagnostic_codes()

    try:
        model.solve(primal_start=[(int(x), 0.0)], log_to_console=False)
    except arco.SolverInvalidSettingError as exc:
        assert exc.code == codes["SOLVER_INVALID_SETTING"]
        assert arco.error_code(exc) == codes["SOLVER_INVALID_SETTING"]
        assert "primal_start is not supported" in str(exc)
    else:  # pragma: no cover
        raise AssertionError("expected unsupported primal_start to fail")


def test_debug_api_contract_exposes_solver_type_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    codes = arco.diagnostic_codes()

    try:
        model.solve(solver=object(), log_to_console=False)
    except arco.SolverTypeError as exc:
        assert exc.code == codes["SOLVER_TYPE"]
        assert arco.error_code(exc) == codes["SOLVER_TYPE"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid solver object to fail")


def test_debug_api_contract_exposes_solver_index_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()

    try:
        result.get_primal(index=999)
    except arco.SolverIndexError as exc:
        assert exc.code == codes["SOLVER_INDEX"]
        assert arco.error_code(exc) == codes["SOLVER_INDEX"]
    else:  # pragma: no cover
        raise AssertionError("expected result index validation to fail")


def test_debug_api_contract_exposes_constraint_dual_index_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()

    try:
        result.get_constraint_dual(index=999)
    except arco.SolverIndexError as exc:
        assert exc.code == codes["SOLVER_INDEX"]
        assert arco.error_code(exc) == codes["SOLVER_INDEX"]
    else:  # pragma: no cover
        raise AssertionError("expected constraint dual index validation to fail")


def test_debug_api_contract_exposes_variable_dual_index_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()

    try:
        result.get_variable_dual(index=999)
    except arco.SolverIndexError as exc:
        assert exc.code == codes["SOLVER_INDEX"]
        assert arco.error_code(exc) == codes["SOLVER_INDEX"]
    else:  # pragma: no cover
        raise AssertionError("expected variable dual index validation to fail")


def test_debug_api_contract_exposes_raw_getter_negative_index_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=-1)
        except arco.SolverIndexError as exc:
            assert exc.code == codes["SOLVER_INDEX"]
            assert arco.error_code(exc) == codes["SOLVER_INDEX"]
        else:  # pragma: no cover
            raise AssertionError(
                "expected raw getter negative-index validation to fail"
            )


def test_debug_api_contract_exposes_raw_getter_huge_integer_index_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    huge_index = 10**200

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=huge_index)
        except arco.SolverIndexError as exc:
            assert exc.code == codes["SOLVER_INDEX"]
            assert arco.error_code(exc) == codes["SOLVER_INDEX"]
        else:  # pragma: no cover
            raise AssertionError("expected raw getter huge-index validation to fail")


def test_debug_api_contract_accepts_numpy_integer_raw_getter_indices() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)

    np_index_variants = (np.int32(0), np.int64(0), np.uint32(0), np.uint64(0))
    for index in np_index_variants:
        assert round(result.get_primal(index=index), 6) == round(
            result.get_primal(index=0), 6
        )
        assert round(result.get_constraint_dual(index=index), 6) == round(
            result.get_constraint_dual(index=int(lower_bound)), 6
        )
        assert round(result.get_variable_dual(index=index), 6) == round(
            result.get_variable_dual(index=0), 6
        )


def test_debug_api_contract_accepts_int_enum_raw_getter_indices() -> None:
    class Index(enum.IntEnum):
        ZERO = 0

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)

    index = Index.ZERO
    assert round(result.get_primal(index=index), 6) == round(
        result.get_primal(index=0), 6
    )
    assert round(result.get_constraint_dual(index=index), 6) == round(
        result.get_constraint_dual(index=int(lower_bound)), 6
    )
    assert round(result.get_variable_dual(index=index), 6) == round(
        result.get_variable_dual(index=0), 6
    )


def test_debug_api_contract_accepts_int_enum_index_protocol_returns() -> None:
    class Index(enum.IntEnum):
        ZERO = 0

    class IntEnumIndexLike:
        def __index__(self) -> Index:
            return Index.ZERO

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    index_like = IntEnumIndexLike()

    assert round(result.get_primal(index=index_like), 6) == round(
        result.get_primal(index=0), 6
    )
    assert round(result.get_constraint_dual(index=index_like), 6) == round(
        result.get_constraint_dual(index=int(lower_bound)), 6
    )
    assert round(result.get_variable_dual(index=index_like), 6) == round(
        result.get_variable_dual(index=0), 6
    )


def test_debug_api_contract_accepts_int_subclass_raw_getter_indices() -> None:
    class IntSubclass(int):
        pass

    class IntSubclassIndexLike:
        def __index__(self) -> IntSubclass:
            return IntSubclass(0)

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    direct_index = IntSubclass(0)
    protocol_index = IntSubclassIndexLike()

    for index in (direct_index, protocol_index):
        assert round(result.get_primal(index=index), 6) == round(
            result.get_primal(index=0), 6
        )
        assert round(result.get_constraint_dual(index=index), 6) == round(
            result.get_constraint_dual(index=int(lower_bound)), 6
        )
        assert round(result.get_variable_dual(index=index), 6) == round(
            result.get_variable_dual(index=0), 6
        )


def test_debug_api_contract_accepts_index_protocol_objects_for_raw_getters() -> None:
    class IndexLike:
        def __init__(self, value: int) -> None:
            self._value = value

        def __index__(self) -> int:
            return self._value

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    lower_bound = model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)

    index_like_zero = IndexLike(0)
    index_like_constraint = IndexLike(int(lower_bound))
    assert round(result.get_primal(index=index_like_zero), 6) == round(
        result.get_primal(index=0), 6
    )
    assert round(result.get_constraint_dual(index=index_like_constraint), 6) == round(
        result.get_constraint_dual(index=int(lower_bound)), 6
    )
    assert round(result.get_variable_dual(index=index_like_zero), 6) == round(
        result.get_variable_dual(index=0), 6
    )


def test_debug_api_contract_rejects_malformed_index_protocol_returns() -> None:
    class BrokenIndexLike:
        def __index__(self) -> float:
            return 1.25

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    invalid_index = BrokenIndexLike()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=invalid_index)
        except arco.SolverTypeError as exc:
            assert exc.code == codes["SOLVER_TYPE"]
            assert arco.error_code(exc) == codes["SOLVER_TYPE"]
            assert "index must be an integer" in str(exc)
        else:  # pragma: no cover
            raise AssertionError("expected malformed __index__ return to fail")


def test_debug_api_contract_rejects_bool_index_protocol_returns() -> None:
    class BoolIndexLike:
        def __index__(self) -> bool:
            return True

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    invalid_index = BoolIndexLike()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=invalid_index)
        except arco.SolverTypeError as exc:
            assert exc.code == codes["SOLVER_TYPE"]
            assert arco.error_code(exc) == codes["SOLVER_TYPE"]
            assert "index must be an integer" in str(exc)
        else:  # pragma: no cover
            raise AssertionError("expected bool __index__ return to fail")


def test_debug_api_contract_maps_huge_index_protocol_returns_to_index_code() -> None:
    class HugeIndexLike:
        def __index__(self) -> int:
            return 10**200

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    huge_index = HugeIndexLike()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=huge_index)
        except arco.SolverIndexError as exc:
            assert exc.code == codes["SOLVER_INDEX"]
            assert arco.error_code(exc) == codes["SOLVER_INDEX"]
        else:  # pragma: no cover
            raise AssertionError(
                "expected huge __index__ return to fail as index error"
            )


def test_debug_api_contract_maps_negative_index_protocol_returns_to_index_code() -> (
    None
):
    class NegativeIndexLike:
        def __index__(self) -> int:
            return -1

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    negative_index = NegativeIndexLike()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=negative_index)
        except arco.SolverIndexError as exc:
            assert exc.code == codes["SOLVER_INDEX"]
            assert arco.error_code(exc) == codes["SOLVER_INDEX"]
        else:  # pragma: no cover
            raise AssertionError(
                "expected negative __index__ return to fail as index error"
            )


def test_debug_api_contract_maps_huge_negative_index_protocol_returns_to_index_code() -> (
    None
):
    class HugeNegativeIndexLike:
        def __index__(self) -> int:
            return -(10**200)

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    huge_negative_index = HugeNegativeIndexLike()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=huge_negative_index)
        except arco.SolverIndexError as exc:
            assert exc.code == codes["SOLVER_INDEX"]
            assert arco.error_code(exc) == codes["SOLVER_INDEX"]
        else:  # pragma: no cover
            raise AssertionError(
                "expected huge negative __index__ return to fail as index error"
            )


def test_debug_api_contract_maps_index_protocol_raiser_to_type_code() -> None:
    class RaisingIndexLike:
        def __index__(self) -> int:
            raise RuntimeError("boom")

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    raising_index = RaisingIndexLike()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=raising_index)
        except arco.SolverTypeError as exc:
            assert exc.code == codes["SOLVER_TYPE"]
            assert arco.error_code(exc) == codes["SOLVER_TYPE"]
            assert "index must be an integer" in str(exc)
        else:  # pragma: no cover
            raise AssertionError("expected raising __index__ to fail as type error")


def test_debug_api_contract_rejects_numpy_scalar_index_protocol_returns() -> None:
    class NumpyScalarIndexLike:
        def __index__(self) -> np.integer:
            return np.int64(1)

    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()
    invalid_index = NumpyScalarIndexLike()

    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        try:
            getter(index=invalid_index)
        except arco.SolverTypeError as exc:
            assert exc.code == codes["SOLVER_TYPE"]
            assert arco.error_code(exc) == codes["SOLVER_TYPE"]
            assert "index must be an integer" in str(exc)
        else:  # pragma: no cover
            raise AssertionError(
                "expected numpy-scalar __index__ return to fail as type error"
            )


def test_debug_api_contract_exposes_raw_getter_index_type_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()

    invalid_indices = ("not-an-index", None, 1.25, object(), True, False)
    for getter in (
        result.get_primal,
        result.get_constraint_dual,
        result.get_variable_dual,
    ):
        for invalid_index in invalid_indices:
            try:
                getter(index=invalid_index)
            except arco.SolverTypeError as exc:
                assert exc.code == codes["SOLVER_TYPE"]
                assert arco.error_code(exc) == codes["SOLVER_TYPE"]
                assert "index must be an integer" in str(exc)
            else:  # pragma: no cover
                raise AssertionError(
                    "expected raw getter index type validation to fail"
                )


def test_debug_api_contract_requires_keyword_raw_getter_index() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.add_constraint(x >= 1.0, name="lower_bound")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    index = 0

    try:
        result.get_primal(0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional get_primal index to fail")

    try:
        result.get_constraint_dual(0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional get_constraint_dual index to fail")

    try:
        result.get_variable_dual(0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional get_variable_dual index to fail")

    try:
        result.get_primal(index)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional variable get_primal index to fail")

    try:
        result.get_constraint_dual(index)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional variable get_constraint_dual index to fail"
        )

    try:
        result.get_variable_dual(index)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional variable get_variable_dual index to fail"
        )


def test_debug_api_contract_exposes_result_value_type_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x, name="cost")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()

    try:
        result.value(object())
    except arco.SolverTypeError as exc:
        assert exc.code == codes["SOLVER_TYPE"]
        assert arco.error_code(exc) == codes["SOLVER_TYPE"]
    else:  # pragma: no cover
        raise AssertionError("expected result value type validation to fail")


def test_debug_api_contract_exposes_result_object_index_code() -> None:
    solved = arco.Model()
    x = solved.add_variable(bounds=arco.NonNegativeFloat, name="x")
    solved.minimize(x, name="cost")
    result = solved.solve(log_to_console=False)

    source = arco.Model()
    y = source.add_variable(bounds=arco.NonNegativeFloat, name="y")
    constraint = source.add_constraint(y >= 1.0, name="foreign")
    codes = arco.diagnostic_codes()

    try:
        result.dual(constraint)
    except arco.SolverIndexError as exc:
        assert exc.code == codes["SOLVER_INDEX"]
        assert arco.error_code(exc) == codes["SOLVER_INDEX"]
    else:  # pragma: no cover
        raise AssertionError("expected result object index validation to fail")


def test_debug_api_contract_exposes_solver_status_class_codes() -> None:
    codes = arco.diagnostic_codes()

    assert arco.SolverInfeasibleError.code == codes["SOLVER_INFEASIBLE"]
    assert arco.SolverUnboundedError.code == codes["SOLVER_UNBOUNDED"]
    assert arco.SolverTimeLimitError.code == codes["SOLVER_TIME_LIMIT"]
    assert arco.SolverIterationLimitError.code == codes["SOLVER_ITERATION_LIMIT"]
    assert arco.SolverInternalError.code == codes["SOLVER_INTERNAL"]
    assert arco.SolverNotAvailableError.code == codes["SOLVER_NOT_AVAILABLE"]


def test_debug_api_contract_exposes_solver_unavailable_error_code() -> None:
    codes = arco.diagnostic_codes()
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    model.minimize(x)

    with pytest.raises(arco.SolverNotAvailableError) as exc:
        model.solve(
            solver=arco.SolverSelection.family("definitely_missing_solver"),
            log_to_console=False,
        )

    assert exc.value.code == codes["SOLVER_NOT_AVAILABLE"]
    assert arco.error_code(exc.value) == codes["SOLVER_NOT_AVAILABLE"]
    assert "not available" in str(exc.value)


def test_debug_api_contract_exposes_remaining_python_exception_class_codes() -> None:
    codes = arco.diagnostic_codes()

    assert arco.ArrayOverflowError.code == codes["ARRAY_OVERFLOW"]
    assert arco.BlockArtifactError.code == codes["BLOCK_ARTIFACT_IO"]
    assert arco.CscContiguityError.code == codes["CSC_CONTIGUITY"]
    assert arco.CscDimensionError.code == codes["CSC_DIMENSION"]
    assert arco.CscDtypeError.code == codes["CSC_DTYPE"]
    assert arco.CscInvalidDataError.code == codes["CSC_INVALID_DATA"]
    assert arco.CscNegativeIndexError.code == codes["CSC_NEGATIVE_INDEX"]
    assert arco.DependencyMissingError.code == codes["DEPENDENCY_MISSING"]
    assert arco.ExprTypeError.code == codes["EXPR_TYPE"]
    assert arco.LoggingConfigError.code == codes["LOGGING_CONFIG"]
    assert arco.LoggingIoError.code == codes["LOGGING_IO"]
    assert arco.MetadataConversionError.code == codes["METADATA_CONVERSION"]
    assert arco.ObjectiveIndexError.code == codes["OBJECTIVE_INDEX"]


def test_debug_api_contract_exposes_logging_config_error_code() -> None:
    script = """
import arco

codes = arco.diagnostic_codes()
try:
    arco.enable_logging(level="arco==debug")
except arco.LoggingConfigError as exc:
    assert exc.code == codes["LOGGING_CONFIG"]
    assert arco.error_code(exc) == codes["LOGGING_CONFIG"]
    assert "Invalid log filter" in str(exc)
else:
    raise AssertionError("expected invalid log filter to fail")
"""
    result = subprocess.run(
        [sys.executable, "-c", script],
        check=False,
        env={**os.environ, "ARCO_LOG_FORMAT": "pretty"},
        text=True,
        capture_output=True,
    )
    assert result.returncode == 0, result.stderr


def test_debug_api_contract_exposes_logging_io_error_code(tmp_path) -> None:
    script = """
import arco

codes = arco.diagnostic_codes()
try:
    arco.enable_logging(level="off")
except arco.LoggingIoError as exc:
    assert exc.code == codes["LOGGING_IO"]
    assert arco.error_code(exc) == codes["LOGGING_IO"]
    assert "Failed to open log file" in str(exc)
else:
    raise AssertionError("expected log file open to fail")
"""
    result = subprocess.run(
        [sys.executable, "-c", script],
        check=False,
        env={
            **os.environ,
            "ARCO_LOG_FILE": str(tmp_path),
            "ARCO_LOG_FORMAT": "pretty",
        },
        text=True,
        capture_output=True,
    )
    assert result.returncode == 0, result.stderr


def test_param_reports_missing_numpy_as_dependency_error(monkeypatch) -> None:
    original_import = builtins.__import__
    codes = arco.diagnostic_codes()

    def fail_numpy_import(name, globals=None, locals=None, fromlist=(), level=0):
        if name == "numpy":
            raise ModuleNotFoundError("No module named 'numpy'", name="numpy")
        return original_import(name, globals, locals, fromlist, level)

    monkeypatch.setattr(builtins, "__import__", fail_numpy_import)

    with pytest.raises(arco.DependencyMissingError) as exc:
        arco.param([1.0], axes=(arco.IndexSet(name="i", members=["a"]),))

    assert exc.value.code == codes["DEPENDENCY_MISSING"]
    assert arco.error_code(exc.value) == codes["DEPENDENCY_MISSING"]
    assert "numpy" in str(exc.value)


def test_debug_api_contract_exposes_objective_missing_error_code() -> None:
    codes = arco.diagnostic_codes()
    model = arco.Model()
    model.add_variable(bounds=arco.NonNegativeFloat, name="x")

    try:
        model.solve(log_to_console=False)
    except arco.ObjectiveMissingError as exc:
        assert exc.code == codes["OBJECTIVE_MISSING"]
        assert arco.error_code(exc) == codes["OBJECTIVE_MISSING"]
    else:  # pragma: no cover
        raise AssertionError("expected solve without objective to fail")


def test_debug_api_contract_exposes_variable_invalid_id_code() -> None:
    model = arco.Model()
    codes = arco.diagnostic_codes()

    try:
        model.deactivate_variable(var_id=999)
    except arco.VariableInvalidIdError as exc:
        assert exc.code == codes["VARIABLE_INVALID_ID"]
        assert arco.error_code(exc) == codes["VARIABLE_INVALID_ID"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid variable id to fail")


def test_debug_api_contract_exposes_objective_already_set_code() -> None:
    codes = arco.diagnostic_codes()

    assert arco.ObjectiveAlreadySetError.code == codes["OBJECTIVE_ALREADY_SET"]


def test_debug_api_contract_exposes_slack_invalid_penalty_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    constraint = model.add_constraint(x >= 1.0, name="minimum")
    model.minimize(x, name="cost")
    codes = arco.diagnostic_codes()

    try:
        model.add_slack(constraint, bound="lower", penalty=-1.0)
    except arco.SlackInvalidPenaltyError as exc:
        assert exc.code == codes["SLACK_INVALID_PENALTY"]
        assert arco.error_code(exc) == codes["SLACK_INVALID_PENALTY"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid slack penalty to fail")


def test_debug_api_contract_exposes_slack_penalty_shape_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    c1 = model.add_constraint(x >= 1.0, name="minimum")
    c2 = model.add_constraint(x <= 2.0, name="maximum")
    model.minimize(x, name="cost")
    codes = arco.diagnostic_codes()

    try:
        model.add_slacks([c1, c2], bound="lower", penalty=[1.0])
    except arco.SlackInvalidPenaltyError as exc:
        assert exc.code == codes["SLACK_INVALID_PENALTY"]
        assert arco.error_code(exc) == codes["SLACK_INVALID_PENALTY"]
    else:  # pragma: no cover
        raise AssertionError("expected slack penalty shape validation to fail")


def test_debug_api_contract_exposes_slack_bound_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    constraint = model.add_constraint(x >= 1.0, name="minimum")
    model.minimize(x, name="cost")
    codes = arco.diagnostic_codes()

    try:
        model.add_slack(constraint, bound="middle", penalty=1.0)
    except arco.SlackBoundError as exc:
        assert exc.code == codes["SLACK_BOUND"]
        assert arco.error_code(exc) == codes["SLACK_BOUND"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid slack bound to fail")


def test_debug_api_contract_exposes_slack_value_unavailable_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    constraint = model.add_constraint(x >= 1.0, name="minimum")
    model.minimize(x, name="cost")
    slack = model.add_slack(constraint, bound="lower", penalty=1.0)
    codes = arco.diagnostic_codes()

    try:
        _ = slack.value
    except arco.SlackValueUnavailableError as exc:
        assert exc.code == codes["SLACK_VALUE_UNAVAILABLE"]
        assert arco.error_code(exc) == codes["SLACK_VALUE_UNAVAILABLE"]
    else:  # pragma: no cover
        raise AssertionError("expected slack value before solve to fail")


def test_scalar_api_contract_exposes_expr_division_by_zero_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    codes = arco.diagnostic_codes()

    try:
        x / 0.0
    except arco.ExprDivisionByZeroError as exc:
        assert exc.code == codes["EXPR_DIVISION_BY_ZERO"]
        assert arco.error_code(exc) == codes["EXPR_DIVISION_BY_ZERO"]
    else:  # pragma: no cover
        raise AssertionError("expected division by zero to fail")


def test_scalar_api_contract_exposes_expr_constant_offset_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    codes = arco.diagnostic_codes()

    try:
        int(x + 1.0)
    except arco.ExprConstantOffsetError as exc:
        assert exc.code == codes["EXPR_CONSTANT_OFFSET"]
        assert arco.error_code(exc) == codes["EXPR_CONSTANT_OFFSET"]
    else:  # pragma: no cover
        raise AssertionError("expected constant-offset expression conversion to fail")


def test_scalar_api_contract_exposes_expr_not_single_variable_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    y = model.add_variable(bounds=arco.NonNegativeFloat, name="y")
    codes = arco.diagnostic_codes()

    try:
        int(x + y)
    except arco.ExprNotSingleVariableError as exc:
        assert exc.code == codes["EXPR_NOT_SINGLE_VARIABLE"]
        assert arco.error_code(exc) == codes["EXPR_NOT_SINGLE_VARIABLE"]
    else:  # pragma: no cover
        raise AssertionError("expected multi-variable expression conversion to fail")


def test_scalar_api_contract_exposes_expr_coefficient_code() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    codes = arco.diagnostic_codes()

    try:
        int(2.0 * x)
    except arco.ExprCoefficientError as exc:
        assert exc.code == codes["EXPR_COEFFICIENT"]
        assert arco.error_code(exc) == codes["EXPR_COEFFICIENT"]
    else:  # pragma: no cover
        raise AssertionError("expected non-unit coefficient conversion to fail")


def test_scalar_api_contract_exposes_expr_type_code_for_bad_operands() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.ExprTypeError) as multiply_exc:
        _ = x * object()
    assert multiply_exc.value.code == codes["EXPR_TYPE"]
    assert arco.error_code(multiply_exc.value) == codes["EXPR_TYPE"]

    with pytest.raises(arco.ExprTypeError) as compare_exc:
        _ = x >= object()
    assert compare_exc.value.code == codes["EXPR_TYPE"]
    assert arco.error_code(compare_exc.value) == codes["EXPR_TYPE"]


def test_debug_api_contract_exposes_cli_diagnostic_codes() -> None:
    codes = arco.diagnostic_codes()

    assert codes["ALGEBRA_PARSE_ERROR"] == "arco::algebra::parse_error"
    assert codes["BOUNDS_INVALID"] == "arco::bounds::invalid"
    assert codes["CONFIG_IO"] == "arco::config::io"
    assert codes["CONFIG_MISSING_DIRECTORY"] == "arco::config::missing_directory"
    assert (
        codes["CONFIG_MISSING_PROJECT_DIRECTORY"]
        == "arco::config::missing_project_directory"
    )
    assert (
        codes["CONFIG_SECRET_REFERENCE_REQUIRED"]
        == "arco::config::secret_reference_required"
    )
    assert codes["CONFIG_SELECTION"] == "arco::config::selection"
    assert codes["CONFIG_TOML"] == "arco::config::toml"
    assert codes["ARRAY_DIMENSION"] == "arco::array::dimension"
    assert codes["ARRAY_INDEX"] == "arco::array::index"
    assert codes["ARRAY_OVERFLOW"] == "arco::array::overflow"
    assert codes["COMPILE_CSV"] == "arco::compile::csv"
    assert (
        codes["COMPILE_EMPTY_TUPLE_REDUCTION"] == "arco::compile::empty_tuple_reduction"
    )
    assert (
        codes["COMPILE_INVALID_CONSTRAINT_FILTER"]
        == "arco::compile::invalid_constraint_filter"
    )
    assert codes["COMPILE_INVALID_FORMULATION"] == "arco::compile::invalid_formulation"
    assert codes["COMPILE_MISSING_COLUMN"] == "arco::compile::missing_column"
    assert codes["COMPILE_INVALID_NUMBER"] == "arco::compile::invalid_number"
    assert codes["COMPILE_MISSING_ASSET"] == "arco::compile::missing_asset"
    assert codes["COMPILE_MISSING_DATA"] == "arco::compile::missing_data"
    assert codes["COMPILE_MISSING_DATA_POINT"] == "arco::compile::missing_data_point"
    assert codes["COMPILE_MISSING_DECLARATION"] == "arco::compile::missing_declaration"
    assert codes["COMPILE_MISSING_PARAMETER"] == "arco::compile::missing_parameter"
    assert codes["COMPILE_MISSING_SCENARIO"] == "arco::compile::missing_scenario"
    assert codes["ARRAY_TYPE"] == "arco::array::type"
    assert codes["BLOCK_ARTIFACT_IO"] == "arco::block::artifact_io"
    assert codes["BLOCK_RESULT"] == "arco::block::result"
    assert codes["CONSTRAINT_BOUNDS_MISSING"] == "arco::constraint::bounds_missing"
    assert codes["CONSTRAINT_INVALID_BOUNDS"] == "arco::constraint::invalid_bounds"
    assert codes["CONSTRAINT_INVALID_ID"] == "arco::constraint::invalid_id"
    assert codes["CONSTRAINT_SENSE"] == "arco::constraint::sense"
    assert codes["CONSTRAINT_TYPE"] == "arco::constraint::type"
    assert codes["CSC_CONTIGUITY"] == "arco::csc::contiguity"
    assert codes["CSC_DIMENSION"] == "arco::csc::dimension"
    assert codes["CSC_DTYPE"] == "arco::csc::dtype"
    assert codes["CSC_INVALID_DATA"] == "arco::csc::invalid_data"
    assert codes["CSC_NEGATIVE_INDEX"] == "arco::csc::negative_index"
    assert (
        codes["DRIVER_BACKEND_NOT_AVAILABLE"] == "arco::driver::backend_not_available"
    )
    assert codes["DRIVER_INSPECT_FORMAT"] == "arco::driver::inspect_format"
    assert codes["DRIVER_JSON"] == "arco::driver::json"
    assert codes["EXPR_COEFFICIENT"] == "arco::expr::coefficient"
    assert codes["EXPR_CONSTANT_OFFSET"] == "arco::expr::constant_offset"
    assert codes["EXPR_DIVISION_BY_ZERO"] == "arco::expr::division_by_zero"
    assert codes["EXPR_NOT_SINGLE_VARIABLE"] == "arco::expr::not_single_variable"
    assert codes["EXPR_TYPE"] == "arco::expr::type"
    assert codes["INDEX_SET_ARGUMENT"] == "arco::index_set::argument"
    assert codes["INDEX_SET_EMPTY"] == "arco::index_set::empty"
    assert codes["INDEX_SET_INDEX"] == "arco::index_set::index"
    assert codes["INDEX_SET_TYPE"] == "arco::index_set::type"
    assert codes["MODEL_BINARY_BOUNDS"] == "arco::model::binary_bounds"
    assert codes["OBJECTIVE_ALREADY_SET"] == "arco::objective::already_set"
    assert codes["OBJECTIVE_INDEX"] == "arco::objective::index"
    assert codes["OBJECTIVE_MISSING"] == "arco::objective::missing"
    assert codes["SEMANTIC_CSV"] == "arco::semantic::csv"
    assert (
        codes["SEMANTIC_AMBIGUOUS_TUPLE_SUBSET_INDEX"]
        == "arco::semantic::ambiguous_tuple_subset_index"
    )
    assert (
        codes["SEMANTIC_DUPLICATE_DATA_BINDING"]
        == "arco::semantic::duplicate_data_binding"
    )
    assert (
        codes["SEMANTIC_DUPLICATE_DECLARATION"]
        == "arco::semantic::duplicate_declaration"
    )
    assert (
        codes["SEMANTIC_DUPLICATE_MODEL_DECLARATION"]
        == "arco::semantic::duplicate_model_declaration"
    )
    assert (
        codes["SEMANTIC_DUPLICATE_TUPLE_ROWS"] == "arco::semantic::duplicate_tuple_rows"
    )
    assert codes["SEMANTIC_EXPRESSION_CYCLE"] == "arco::semantic::expression_cycle"
    assert codes["SEMANTIC_MISSING_CELL"] == "arco::semantic::missing_cell"
    assert codes["SEMANTIC_MISSING_COLUMN"] == "arco::semantic::missing_column"
    assert (
        codes["SEMANTIC_MISSING_DECLARATION"] == "arco::semantic::missing_declaration"
    )
    assert (
        codes["SEMANTIC_MISSING_INITIAL_BOUNDARY"]
        == "arco::semantic::missing_initial_boundary"
    )
    assert codes["SEMANTIC_MISSING_MODEL"] == "arco::semantic::missing_model"
    assert codes["SEMANTIC_MISSING_MODEL_USE"] == "arco::semantic::missing_model_use"
    assert codes["SEMANTIC_MISSING_SCENARIO"] == "arco::semantic::missing_scenario"
    assert codes["SEMANTIC_SCENARIO_COUNT"] == "arco::semantic::scenario_count"
    assert (
        codes["SEMANTIC_TUPLE_SET_SCHEMA_MISMATCH"]
        == "arco::semantic::tuple_set_schema_mismatch"
    )
    assert (
        codes["SEMANTIC_TUPLE_SUBSET_DOMAIN_MISMATCH"]
        == "arco::semantic::tuple_subset_domain_mismatch"
    )
    assert (
        codes["SEMANTIC_UNKNOWN_SCENARIO_DATA_BINDING"]
        == "arco::semantic::unknown_scenario_data_binding"
    )
    assert (
        codes["SEMANTIC_UNRESOLVED_FILTER_IDENTIFIER"]
        == "arco::semantic::unresolved_filter_identifier"
    )
    assert (
        codes["SEMANTIC_UNRESOLVED_RULE_SET_FILTER_IDENTIFIER"]
        == "arco::semantic::unresolved_rule_set_filter_identifier"
    )
    assert codes["SLACK_BOUND"] == "arco::slack::bound"
    assert codes["SLACK_INVALID_PENALTY"] == "arco::slack::invalid_penalty"
    assert codes["SOLVER_INFEASIBLE"] == "arco::solver::infeasible"
    assert codes["SOLVER_INDEX"] == "arco::solver::index"
    assert codes["SOLVER_INTERNAL"] == "arco::solver::internal"
    assert codes["SOLVER_ITERATION_LIMIT"] == "arco::solver::iteration_limit"
    assert codes["SOLVER_INVALID_SETTING"] == "arco::solver::invalid_setting"
    assert codes["SOLVER_MODEL_SIZE_LIMIT"] == "arco::solver::model_size_limit"
    assert codes["SOLVER_NOT_AVAILABLE"] == "arco::solver::not_available"
    assert codes["SOLVER_TIME_LIMIT"] == "arco::solver::time_limit"
    assert codes["SOLVER_TYPE"] == "arco::solver::type"
    assert codes["SOLVER_UNBOUNDED"] == "arco::solver::unbounded"
    assert codes["SOURCE_INVALID_ALGEBRA"] == "arco::source::invalid_algebra"
    assert codes["SOURCE_INVALID_INCLUDE"] == "arco::source::invalid_include"
    assert codes["SOURCE_INVALID_VALUE"] == "arco::source::invalid_value"
    assert codes["SOURCE_IO"] == "arco::source::io"
    assert codes["SOURCE_KDL"] == "arco::source::kdl"
    assert codes["SOURCE_MISSING_ARGUMENT"] == "arco::source::missing_argument"
    assert codes["SOURCE_MISSING_NODE"] == "arco::source::missing_node"
    assert codes["SOURCE_MISSING_PROPERTY"] == "arco::source::missing_property"
    assert (
        codes["SOURCE_UNSUPPORTED_DECLARATION"]
        == "arco::source::unsupported_declaration"
    )
    assert codes["VARIABLE_INVALID_ID"] == "arco::variable::invalid_id"


def test_api_contract_rejects_ambiguous_param_positional_shape() -> None:
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    values = np.array([1.0, 2.0])
    param_name = "cost"

    try:
        arco.param("cost", np.array([1.0, 2.0]), plant)
    except (TypeError, arco.ArrayDimensionError, arco.ArrayTypeError):
        pass
    else:  # pragma: no cover
        raise AssertionError("expected ambiguous positional param signature to fail")

    try:
        arco.param(param_name, values, plant)
    except (TypeError, arco.ArrayDimensionError, arco.ArrayTypeError):
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected ambiguous positional param signature using variables to fail"
        )


def test_api_contract_accepts_legacy_positional_param_axes() -> None:
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    values = np.array([1.0, 2.0])
    axes = (plant,)

    param_from_tuple = arco.param(np.array([1.0, 2.0]), (plant,))
    assert param_from_tuple.axes == axes

    param_from_axis = arco.param(values, plant)
    assert param_from_axis.axes == axes

    try:
        arco.param(np.array([1.0, 2.0]), (plant,), "demand")
    except (TypeError, arco.ArrayDimensionError, arco.ArrayTypeError):
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional param name usage to fail")

    try:
        arco.param(values, axes, axes=axes)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected duplicate positional/keyword axes to fail")


def test_api_contract_rejects_positional_variable_constructors() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    axes = (plant,)

    try:
        model.add_variable(arco.NonNegativeFloat, False, False, "x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional add_variable signature to fail")

    try:
        model.add_variables((plant,), arco.NonNegativeFloat, False, False, None, "x")
    except (TypeError, arco.ArrayTypeError):
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional add_variables signature to fail")

    x = model.add_variables((plant,), bounds=arco.NonNegativeFloat, name="x")
    assert x.shape == (2,)

    y = model.add_variables(axes, bounds=arco.NonNegativeFloat, name="y")
    assert y.shape == (2,)

    try:
        model.add_variables(axes)
    except TypeError as exc:
        assert "bounds" in str(exc)
    else:  # pragma: no cover
        raise AssertionError("expected missing bounds to fail")


def test_api_contract_rejects_positional_constraint_name_order() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    constraint_name = "minimum"

    try:
        model.add_constraint("minimum", x >= 1.0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected constraint(name, expr) positional form to fail")

    try:
        model.add_constraint(constraint_name, x >= 1.0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected constraint(name_var, expr) positional form to fail"
        )


def test_api_contract_requires_keyword_constraint_configuration() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    constraint_name = "minimum"
    bounds = arco.Bounds(lower=0.0, upper=10.0)

    try:
        model.add_constraint(x, arco.Bounds(lower=0.0, upper=10.0))
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional constraint bounds to fail")

    try:
        model.add_constraint(x >= 1.0, "minimum")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional constraint name to fail")

    try:
        model.add_constraint(x >= 1.0, None, "minimum")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional constraint bounds/name to fail")

    try:
        model.add_constraint(x, bounds)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional constraint bounds variable to fail")

    try:
        model.add_constraint(x >= 1.0, constraint_name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional constraint name variable to fail")

    try:
        model.add_constraint(x >= 1.0, bounds, constraint_name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional constraint bounds/name variables to fail"
        )

    try:
        model.add_constraint(
            x >= 1.0,
            arco.Bounds(lower=0.0, upper=10.0),
            constraint_name,
        )
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional constraint literal-bounds/name-variable to fail"
        )


def test_api_contract_requires_keyword_add_constraints_configuration() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    output = model.add_variables(
        axes=(plant,),
        bounds=arco.NonNegativeFloat,
        name="output",
    )
    sense = "ge"
    rhs = 0.0
    name = "demand"
    active = True

    try:
        model.add_constraints(output, "ge", 0.0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional add_constraints sense/rhs to fail")

    try:
        model.add_constraints(output, "ge", 0.0, True)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional add_constraints active to fail")

    try:
        model.add_constraints(output, "ge", 0.0, None, "demand")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional add_constraints name to fail")

    try:
        model.add_constraints(output, sense, rhs)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional add_constraints variable args to fail"
        )

    try:
        model.add_constraints(output, sense, rhs, active)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional add_constraints variable active to fail"
        )

    try:
        model.add_constraints(output, sense, rhs, active, name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional add_constraints variable active/name to fail"
        )

    try:
        model.add_constraints(output, sense, rhs, None, name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected positional add_constraints variable name to fail"
        )


def test_api_contract_rejects_positional_objective_name_order() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    objective_name = "cost"
    profit_name = "profit"

    try:
        model.minimize("cost", x)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected minimize(name, expr) positional form to fail")

    try:
        model.maximize("profit", x)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected maximize(name, expr) positional form to fail")

    try:
        model.minimize(x, "cost")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional minimize name argument to fail")

    try:
        model.maximize(x, "profit")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional maximize name argument to fail")

    try:
        model.minimize(objective_name, x)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected minimize(name_var, expr) positional form to fail"
        )

    try:
        model.maximize(profit_name, x)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected maximize(name_var, expr) positional form to fail"
        )

    try:
        model.minimize(x, objective_name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional minimize variable name to fail")

    try:
        model.maximize(x, profit_name)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected positional maximize variable name to fail")


def test_api_contract_keeps_legacy_constructor_aliases_off_beginner_surface() -> None:
    model = arco.Model()

    assert not hasattr(arco, "Set")
    assert not hasattr(model, "variable")
    assert not hasattr(model, "add_control")
    assert not hasattr(model, "control")
    assert not hasattr(model, "constraint")


def test_api_contract_rejects_lower_upper_variable_shortcuts() -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    lower = 0.0
    upper = 1.0

    try:
        model.add_variable(lower=0.0, upper=1.0, name="x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected lower/upper shortcut on add_variable to fail")

    try:
        model.add_variables(axes=(plant,), lower=0.0, upper=1.0, name="x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected lower/upper shortcut on add_variables to fail")

    try:
        model.add_variable(lower=lower, upper=upper, name="x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected variable lower/upper shortcut on add_variable to fail"
        )

    try:
        model.add_variables(axes=(plant,), lower=lower, upper=upper, name="x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected variable lower/upper shortcut on add_variables to fail"
        )
