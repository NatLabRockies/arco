from __future__ import annotations

import os
from pathlib import Path

import pytest

import arco


def _local_xpress_dir() -> str | None:
    configured = os.environ.get("XPRESSDIR")
    if configured and Path(configured).exists():
        return configured

    candidates = [
        Path.home() / "User Apps" / "FICO Xpress" / "xpressmp",
        Path.home() / "opt" / "xpressmp",
        Path("/Applications/FICO Xpress/xpressmp"),
        Path("/Volumes/FICO Xpress Installer/FICO Xpress/xpressmp"),
        Path("/opt/xpressmp"),
        Path("/Library/xpressmp"),
        Path("C:/xpressmp"),
    ]

    user_profile = os.environ.get("USERPROFILE")
    if user_profile:
        candidates.append(
            Path(user_profile) / "AppData" / "Local" / "FICO Xpress" / "xpressmp"
        )
    program_files = os.environ.get("ProgramFiles")
    if program_files:
        candidates.append(Path(program_files) / "FICO Xpress" / "xpressmp")
    program_files_x86 = os.environ.get("ProgramFiles(x86)")
    if program_files_x86:
        candidates.append(Path(program_files_x86) / "FICO Xpress" / "xpressmp")
    for candidate in candidates:
        if candidate.exists():
            return str(candidate)

    return None


def _build_model() -> arco.Model:
    model = arco.Model()
    x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
    model.add_constraint(x >= 1.0, name="demand")
    model.minimize(2.0 * x)
    return model


@pytest.mark.skipif(
    _local_xpress_dir() is None, reason="local Xpress SDK not available"
)
def test_xpress_solver_object_solves_model() -> None:
    os.environ.setdefault("XPRESSDIR", _local_xpress_dir() or "")

    try:
        result = _build_model().solve(solver=arco.Xpress(log_to_console=False))
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
    _local_xpress_dir() is None, reason="local Xpress SDK not available"
)
def test_xpress_solver_selection_family_solves_model() -> None:
    os.environ.setdefault("XPRESSDIR", _local_xpress_dir() or "")

    try:
        result = _build_model().solve(solver=arco.SolverSelection.family("xpress"))
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
