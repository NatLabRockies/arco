from __future__ import annotations

import os

import pytest

import arco


def build_model() -> arco.Model:
    model = arco.Model()
    x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
    model.add_constraint(x >= 1.0, name="demand")
    model.minimize(2.0 * x)
    return model


def build_infeasible_model() -> arco.Model:
    model = arco.Model()
    x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=0.0))
    model.add_constraint(x >= 1.0, name="demand")
    model.minimize(x)
    return model


def solve_xpress_or_skip(model: arco.Model, solver: arco.Xpress) -> arco.SolveResult:
    runtime_dir = XPRESS_RUNTIME_INFO.get("runtime_dir")
    if runtime_dir:
        os.environ.setdefault("XPRESSDIR", str(runtime_dir))
    try:
        return model.solve(solver=solver)
    except (
        arco.SolverNotAvailableError
    ) as exc:  # pragma: no cover - environment dependent
        message = str(exc)
        if "Xpress license initialization failed" in message:
            pytest.skip(message)
        raise


XPRESS_RUNTIME_INFO = arco.solver_runtime_info(family="xpress")
HAS_XPRESS_RUNTIME = bool(XPRESS_RUNTIME_INFO.get("runtime_dir"))
HAS_XPRESS_BACKEND = bool(XPRESS_RUNTIME_INFO.get("backend_enabled"))


@pytest.mark.skipif(
    not HAS_XPRESS_RUNTIME,
    reason="local Xpress runtime not available",
)
def test_xpress_runtime_info_exposes_license_contract() -> None:
    info = XPRESS_RUNTIME_INFO
    assert info["family"] == "xpress"
    assert info["requires_license"] is True
    assert info["license_env_var"] == "XPAUTH_PATH"
    assert info["runtime_env_var"] == "XPRESSDIR"


@pytest.mark.skipif(
    (not HAS_XPRESS_RUNTIME) or (not HAS_XPRESS_BACKEND),
    reason="xpress runtime/backend not available in this build",
)
def test_xpress_solver_object_solves_model() -> None:
    runtime_dir = XPRESS_RUNTIME_INFO.get("runtime_dir")
    if runtime_dir:
        os.environ.setdefault("XPRESSDIR", str(runtime_dir))

    try:
        result = build_model().solve(solver=arco.Xpress(log_to_console=False))
    except (
        arco.SolverNotAvailableError
    ) as exc:  # pragma: no cover - environment dependent
        message = str(exc)
        if "Xpress license initialization failed" in message:
            pytest.skip(message)
        raise

    assert result.is_optimal()
    assert result.objective_value == pytest.approx(2.0)


@pytest.mark.skipif(
    (not HAS_XPRESS_RUNTIME) or (not HAS_XPRESS_BACKEND),
    reason="xpress runtime/backend not available in this build",
)
def test_xpress_solver_selection_family_solves_model() -> None:
    runtime_dir = XPRESS_RUNTIME_INFO.get("runtime_dir")
    if runtime_dir:
        os.environ.setdefault("XPRESSDIR", str(runtime_dir))

    try:
        result = build_model().solve(solver=arco.SolverSelection.family("xpress"))
    except (
        arco.SolverNotAvailableError
    ) as exc:  # pragma: no cover - environment dependent
        message = str(exc)
        if "Xpress license initialization failed" in message:
            pytest.skip(message)
        raise

    assert result.is_optimal()
    assert result.objective_value == pytest.approx(2.0)


@pytest.mark.skipif(
    (not HAS_XPRESS_RUNTIME) or (not HAS_XPRESS_BACKEND),
    reason="xpress runtime/backend not available in this build",
)
def test_xpress_solves_with_selected_lp_algorithms() -> None:
    runtime_dir = XPRESS_RUNTIME_INFO.get("runtime_dir")
    if runtime_dir:
        os.environ.setdefault("XPRESSDIR", str(runtime_dir))

    for algorithm in ("primal", "dual", "barrier"):
        solver = arco.Xpress(
            log_to_console=False,
            lp_algorithm={
                "primal": arco.LpAlgorithm.PRIMAL_SIMPLEX,
                "dual": arco.LpAlgorithm.DUAL_SIMPLEX,
                "barrier": arco.LpAlgorithm.BARRIER,
            }[algorithm],
        )
        try:
            result = build_model().solve(solver=solver)
        except arco.SolverNotAvailableError as exc:  # pragma: no cover
            message = str(exc)
            if "Xpress license initialization failed" in message:
                pytest.skip(message)
            raise

        assert result.is_optimal()
        assert result.objective_value == pytest.approx(2.0)


@pytest.mark.skipif(
    (not HAS_XPRESS_RUNTIME) or (not HAS_XPRESS_BACKEND),
    reason="xpress runtime/backend not available in this build",
)
def test_xpress_consuming_solve_releases_model_at_native_handoff() -> None:
    model = build_model()
    result = solve_xpress_or_skip(
        model,
        arco.Xpress(
            log_to_console=False,
            parameters={
                "arco.consume_model": "true",
                "arco.extract_solution": "false",
                "arco.fingerprint": "false",
            },
        ),
    )

    assert result.is_optimal()
    assert result.objective_value == pytest.approx(2.0)
    assert model.num_variables == 0
    assert model.num_constraints == 0


@pytest.mark.skipif(
    (not HAS_XPRESS_RUNTIME) or (not HAS_XPRESS_BACKEND),
    reason="xpress runtime/backend not available in this build",
)
def test_xpress_nonconsuming_solve_keeps_model_available() -> None:
    model = build_model()
    result = solve_xpress_or_skip(model, arco.Xpress(log_to_console=False))

    assert result.is_optimal()
    assert model.num_variables == 1
    assert model.num_constraints == 1


@pytest.mark.skipif(
    (not HAS_XPRESS_RUNTIME) or (not HAS_XPRESS_BACKEND),
    reason="xpress runtime/backend not available in this build",
)
def test_xpress_consuming_infeasible_solve_leaves_model_consumed() -> None:
    model = build_infeasible_model()
    result = solve_xpress_or_skip(
        model,
        arco.Xpress(
            log_to_console=False,
            parameters={"arco.consume_model": "true"},
        ),
    )

    assert result.status == arco.SolutionStatus.INFEASIBLE
    assert model.num_variables == 0
    assert model.num_constraints == 0


@pytest.mark.skipif(
    not HAS_XPRESS_BACKEND,
    reason="xpress backend not available in this build",
)
def test_xpress_rejects_unsupported_lp_algorithm_before_runtime_setup() -> None:
    solver = arco.Xpress(
        log_to_console=False,
        lp_algorithm=arco.LpAlgorithm.PRIMAL_DUAL_FIRST_ORDER,
    )

    with pytest.raises(
        arco.SolverInvalidSettingError,
        match="primal_dual_first_order.*not supported by the Xpress backend",
    ):
        build_model().solve(solver=solver)


def test_xpress_constructor_fails_fast_when_backend_disabled() -> None:
    if HAS_XPRESS_BACKEND:
        pytest.skip("xpress backend enabled in this build")

    with pytest.raises(
        arco.SolverNotAvailableError, match="built without the xpress feature"
    ):
        build_model().solve(solver=arco.Xpress(log_to_console=False))


@pytest.mark.skipif(
    (not HAS_XPRESS_BACKEND) or HAS_XPRESS_RUNTIME,
    reason="requires an Xpress backend without a local runtime",
)
def test_xpress_consuming_preparation_failure_preserves_previous_solution() -> None:
    model = build_model()
    previous_result = model.solve(solver=arco.HiGHS(log_to_console=False))

    with pytest.raises(arco.SolverNotAvailableError):
        model.solve(
            solver=arco.Xpress(
                log_to_console=False,
                parameters={"arco.consume_model": "true"},
            )
        )

    assert model.num_variables == 1
    assert model.num_constraints == 1
    assert model._last_solution is previous_result
