from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any


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


def test_build_only_text_output_reports_unavailable_memory_explicitly() -> None:
    result = run_formulation("--size", "small", "--build-only")

    assert "reeds-benchmark built:" in result.stdout
    assert "peak=" in result.stdout
    assert "peak=0.0 MB" not in result.stdout
