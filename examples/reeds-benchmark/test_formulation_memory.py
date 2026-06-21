from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import types
from pathlib import Path
from typing import Any


def load_formulation() -> types.ModuleType:
    example_dir = Path(__file__).resolve().parent
    sys.path.insert(0, str(example_dir))
    sys.modules.setdefault("arco", types.SimpleNamespace())
    spec = importlib.util.spec_from_file_location(
        "reeds_benchmark_formulation", example_dir / "formulation.py"
    )
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run_formulation(*args: str) -> subprocess.CompletedProcess[str]:
    repo_root = Path(__file__).resolve().parents[2]
    script = repo_root / "examples" / "reeds-benchmark" / "formulation.py"
    return subprocess.run(
        ["uv", "run", str(script), *args],
        check=True,
        cwd=repo_root,
        capture_output=True,
        text=True,
    )


def test_build_profile_json_reports_memory_without_zero_fallbacks() -> None:
    result = run_formulation(
        "--size",
        "small",
        "--build-only",
        "--json",
        "--profile-build",
        "--profile-matrix",
    )

    payload: dict[str, Any] = json.loads(result.stdout)

    assert payload["solved"] is False
    peak_rss_mb = payload["peak_rss_mb"]
    assert peak_rss_mb is None or peak_rss_mb > 0.0

    build_profile = payload["build_profile"]
    assert build_profile
    assert {"stage", "seconds", "rss_mb", "delta_rss_mb"} <= build_profile[0].keys()
    for stage in build_profile:
        rss_mb = stage["rss_mb"]
        assert rss_mb is None or rss_mb > 0.0

    matrix_profile = payload["matrix_profile"]
    assert matrix_profile["num_variables"] > 0
    assert matrix_profile["num_constraints"] > 0
    assert matrix_profile["num_coefficients"] > 0
    assert "column_nnz_buckets" in matrix_profile
    assert "top_columns" in matrix_profile


def test_build_only_text_output_reports_unavailable_memory_explicitly() -> None:
    result = run_formulation("--size", "small", "--build-only")

    assert "reeds-benchmark built:" in result.stdout
    assert "peak=" in result.stdout
    assert "peak=0.0 MB" not in result.stdout


def test_ru_maxrss_to_mb_uses_platform_units() -> None:
    formulation = load_formulation()

    assert formulation._ru_maxrss_to_mb(2048.0, "linux") == 2.0
    assert formulation._ru_maxrss_to_mb(2.0 * 1024.0 * 1024.0, "darwin") == 2.0


def test_format_rss_mb_reports_unsupported_measurements_explicitly() -> None:
    formulation = load_formulation()

    assert formulation._format_rss_mb(None) == "n/a"
    assert formulation._format_rss_mb(12.25) == "12.2 MB"
