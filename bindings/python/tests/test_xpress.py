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


XPRESS_RUNTIME_INFO = arco.solver_runtime_info(family="xpress")
HAS_XPRESS_RUNTIME = bool(XPRESS_RUNTIME_INFO.get("runtime_dir"))


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
    not HAS_XPRESS_RUNTIME,
    reason="local Xpress runtime not available",
)
def test_xpress_solver_object_solves_model() -> None:
    runtime_dir = XPRESS_RUNTIME_INFO.get("runtime_dir")
    if runtime_dir:
        os.environ.setdefault("XPRESSDIR", str(runtime_dir))

    try:
        result = build_model().solve(solver=arco.Xpress(log_to_console=False))
    except Exception as exc:  # pragma: no cover - environment dependent
        message = str(exc)
        if (
            "Xpress license initialization failed" in message
            or "Xpress model-view backend is not enabled" in message
        ):
            pytest.skip(message)
        raise

    assert result.is_optimal()
    assert result.objective_value == pytest.approx(2.0)


@pytest.mark.skipif(
    not HAS_XPRESS_RUNTIME,
    reason="local Xpress runtime not available",
)
def test_xpress_solver_selection_family_solves_model() -> None:
    runtime_dir = XPRESS_RUNTIME_INFO.get("runtime_dir")
    if runtime_dir:
        os.environ.setdefault("XPRESSDIR", str(runtime_dir))

    try:
        result = build_model().solve(solver=arco.SolverSelection.family("xpress"))
    except Exception as exc:  # pragma: no cover - environment dependent
        message = str(exc)
        if (
            "Xpress license initialization failed" in message
            or "Xpress model-view backend is not enabled" in message
        ):
            pytest.skip(message)
        raise

    assert result.is_optimal()
    assert result.objective_value == pytest.approx(2.0)
