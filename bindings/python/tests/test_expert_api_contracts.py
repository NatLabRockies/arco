from __future__ import annotations

import math

import numpy as np

import arco


def test_expert_csc_import_export_and_raw_result_accessors() -> None:
    model = arco.Model.from_csc(
        num_constraints=1,
        num_variables=1,
        col_ptrs=[0, 1],
        row_indices=[0],
        values=[1.0],
        var_lower=[0.0],
        var_upper=[math.inf],
        con_lower=[1.0],
        con_upper=[math.inf],
        is_integer=[False],
    )
    model.set_objective(sense=arco.Sense.MINIMIZE, terms=[(0, 1.0)], name="min_x")

    snapshot = model.inspect(include_coeffs=True)
    result = model.solve(log_to_console=False)

    assert snapshot.metadata.variables == 1
    assert snapshot.metadata.constraints == 1
    assert snapshot.metadata.coefficients == 1
    assert snapshot.metadata.memory.coefficient_value_bytes == 8
    assert snapshot.metadata.memory.sparse_matrix_bytes >= 8
    assert result.is_optimal()
    assert result.get_primal(index=0) == 1.0
    assert result.primal_values == [1.0]
    assert result.variable_duals == [0.0]
    assert result.constraint_duals == [1.0]
    assert result.num_primal_values() == 1
    assert result.num_variable_duals() == 1
    assert result.num_constraint_duals() == 1
    assert len(result.primal_values) == model.num_variables
    assert len(result.variable_duals) == model.num_variables
    assert len(result.constraint_duals) == model.num_constraints
    assert result.get_constraint_dual(index=0) == 1.0

    assert model.export_csc() == {
        "col_ptrs": [0, 1],
        "row_indices": [0],
        "values": [1.0],
        "shape": (1, 1),
    }
    assert model.export_crs() == {
        "row_ptrs": [0, 1],
        "col_indices": [0],
        "values": [1.0],
        "shape": (1, 1),
    }
    assert model.export_coo() == {
        "rows": [0],
        "cols": [0],
        "values": [1.0],
        "shape": (1, 1),
    }


def test_expert_raw_objective_requires_keywords() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")

    try:
        model.set_objective(arco.Sense.MINIMIZE, [(int(x), 1.0)])
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected raw objective positional arguments to fail")

    model.set_objective(
        sense=arco.Sense.MINIMIZE,
        terms=[(int(x), 1.0)],
        name="min_x",
    )


def test_expert_from_csc_requires_keywords() -> None:
    try:
        arco.Model.from_csc(
            1,
            1,
            [0, 1],
            [0],
            [1.0],
            [0.0],
            [math.inf],
            [1.0],
            [math.inf],
            [False],
        )
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected from_csc positional arguments to fail")

    model = arco.Model.from_csc(
        num_constraints=1,
        num_variables=1,
        col_ptrs=[0, 1],
        row_indices=[0],
        values=[1.0],
        var_lower=[0.0],
        var_upper=[math.inf],
        con_lower=[1.0],
        con_upper=[math.inf],
        is_integer=[False],
    )
    snapshot = model.inspect()
    assert snapshot.metadata.variables == 1
    assert snapshot.metadata.constraints == 1


def test_expert_from_csc_negative_index_uses_typed_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.Model.from_csc(
            num_constraints=1,
            num_variables=1,
            col_ptrs=[0, 1],
            row_indices=[-1],
            values=[1.0],
            var_lower=[0.0],
            var_upper=[math.inf],
            con_lower=[1.0],
            con_upper=[math.inf],
            is_integer=[False],
        )
    except arco.CscNegativeIndexError as exc:
        assert exc.code == codes["CSC_NEGATIVE_INDEX"]
        assert arco.error_code(exc) == codes["CSC_NEGATIVE_INDEX"]
    else:  # pragma: no cover
        raise AssertionError("expected negative CSC row index to fail")


def test_expert_from_csc_invalid_dtype_uses_typed_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.Model.from_csc(
            num_constraints=1,
            num_variables=1,
            col_ptrs=[0, 1],
            row_indices=[0],
            values=["not-a-float"],
            var_lower=[0.0],
            var_upper=[math.inf],
            con_lower=[1.0],
            con_upper=[math.inf],
            is_integer=[False],
        )
    except arco.CscDtypeError as exc:
        assert exc.code == codes["CSC_DTYPE"]
        assert arco.error_code(exc) == codes["CSC_DTYPE"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid CSC dtype payload to fail")


def test_expert_from_csc_nested_numpy_values_use_typed_dimension_code() -> None:
    codes = arco.diagnostic_codes()

    try:
        arco.Model.from_csc(
            num_constraints=1,
            num_variables=1,
            col_ptrs=[0, 1],
            row_indices=[0],
            values=np.array([[1.0]], dtype=np.float64),
            var_lower=[0.0],
            var_upper=[math.inf],
            con_lower=[1.0],
            con_upper=[math.inf],
            is_integer=[False],
        )
    except arco.CscDimensionError as exc:
        assert exc.code == codes["CSC_DIMENSION"]
        assert arco.error_code(exc) == codes["CSC_DIMENSION"]
    else:  # pragma: no cover
        raise AssertionError("expected nested CSC values payload to fail")


def test_expert_from_csc_noncontiguous_values_use_typed_contiguity_code() -> None:
    codes = arco.diagnostic_codes()
    noncontiguous_values = np.array([1.0, 999.0, 1.0], dtype=np.float64)[::2]

    try:
        arco.Model.from_csc(
            num_constraints=1,
            num_variables=1,
            col_ptrs=[0, 1],
            row_indices=[0],
            values=noncontiguous_values,
            var_lower=[0.0],
            var_upper=[math.inf],
            con_lower=[1.0],
            con_upper=[math.inf],
            is_integer=[False],
        )
    except arco.CscContiguityError as exc:
        assert exc.code == codes["CSC_CONTIGUITY"]
        assert arco.error_code(exc) == codes["CSC_CONTIGUITY"]
    else:  # pragma: no cover
        raise AssertionError("expected mismatched CSC nnz payload to fail")


def test_expert_raw_name_setters_require_keywords_and_validate_indices() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat)
    row = model.add_constraint(x >= 0.0)

    try:
        model.set_variable_name(0, "x")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected raw variable-name positional arguments to fail")

    try:
        model.set_constraint_name(0, "limit")
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected raw constraint-name positional arguments to fail"
        )

    model.set_variable_name(index=0, name="x")
    model.set_constraint_name(index=int(row), name="limit")

    snapshot = model.inspect()
    assert snapshot.variables[0].name == "x"
    assert snapshot.constraints[0].name == "limit"

    try:
        model.set_variable_name(index=999, name="missing")
    except arco.VariableInvalidIdError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected invalid raw variable name index to fail")

    try:
        model.set_constraint_name(index=999, name="missing")
    except arco.ConstraintInvalidIdError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected invalid raw constraint name index to fail")


def test_expert_set_coefficient_requires_keywords_and_validates_indices() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    row = model.add_constraint(x >= 0.0, name="limit")
    codes = arco.diagnostic_codes()

    try:
        model.set_coefficient(int(x), int(row), 2.0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected raw coefficient positional arguments to fail")

    model.set_coefficient(var_idx=int(x), constraint_idx=int(row), coeff=2.0)

    try:
        model.set_coefficient(var_idx=999, constraint_idx=int(row), coeff=2.0)
    except arco.VariableInvalidIdError as exc:
        assert exc.code == codes["VARIABLE_INVALID_ID"]
        assert arco.error_code(exc) == codes["VARIABLE_INVALID_ID"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid variable index to fail")

    try:
        model.set_coefficient(var_idx=int(x), constraint_idx=999, coeff=2.0)
    except arco.ConstraintInvalidIdError as exc:
        assert exc.code == codes["CONSTRAINT_INVALID_ID"]
        assert arco.error_code(exc) == codes["CONSTRAINT_INVALID_ID"]
    else:  # pragma: no cover
        raise AssertionError("expected invalid constraint index to fail")


def test_expert_raw_objective_invalid_variable_uses_objective_index_error() -> None:
    model = arco.Model()
    model.add_variable(bounds=arco.NonNegativeFloat, name="x")
    codes = arco.diagnostic_codes()

    try:
        model.set_objective(sense=arco.Sense.MINIMIZE, terms=[(999, 1.0)])
    except arco.ObjectiveIndexError as exc:
        assert exc.code == codes["OBJECTIVE_INDEX"]
        assert arco.error_code(exc) == codes["OBJECTIVE_INDEX"]
        assert "999" in str(exc)
    else:  # pragma: no cover
        raise AssertionError("expected invalid raw objective index to fail")


def test_expert_csc_import_preserves_float64_values() -> None:
    value = 0.123456789123
    model = arco.Model.from_csc(
        num_constraints=1,
        num_variables=1,
        col_ptrs=[0, 1],
        row_indices=[0],
        values=np.array([value], dtype=np.float64),
        var_lower=np.array([0.0], dtype=np.float64),
        var_upper=np.array([1.0], dtype=np.float64),
        con_lower=np.array([0.0], dtype=np.float64),
        con_upper=np.array([1.0], dtype=np.float64),
        is_integer=[False],
    )

    assert model.export_csc()["values"] == [value]


def test_expert_raw_result_getters_require_keywords_and_validate_indices() -> None:
    model = arco.Model.from_csc(
        num_constraints=1,
        num_variables=1,
        col_ptrs=[0, 1],
        row_indices=[0],
        values=[1.0],
        var_lower=[0.0],
        var_upper=[math.inf],
        con_lower=[1.0],
        con_upper=[math.inf],
        is_integer=[False],
    )
    model.set_objective(sense=arco.Sense.MINIMIZE, terms=[(0, 1.0)], name="min_x")
    result = model.solve(log_to_console=False)
    codes = arco.diagnostic_codes()

    try:
        result.get_primal(0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected raw primal positional arguments to fail")

    try:
        result.get_variable_dual(0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected raw variable-dual positional arguments to fail")

    try:
        result.get_constraint_dual(0)
    except TypeError:
        pass
    else:  # pragma: no cover
        raise AssertionError(
            "expected raw constraint-dual positional arguments to fail"
        )

    assert result.get_primal(index=0) == 1.0
    assert result.get_variable_dual(index=0) == 0.0
    assert result.get_constraint_dual(index=0) == 1.0

    for getter in (
        result.get_primal,
        result.get_variable_dual,
        result.get_constraint_dual,
    ):
        try:
            getter(index=999)
        except arco.SolverIndexError as exc:
            assert exc.code == codes["SOLVER_INDEX"]
            assert arco.error_code(exc) == codes["SOLVER_INDEX"]
        else:  # pragma: no cover
            raise AssertionError("expected out-of-range expert result index to fail")
