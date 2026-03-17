from __future__ import annotations

import json
from pathlib import Path

from arco_benchmarks.export_results import _merge_with_memory, load_raw_results


def _write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload), encoding="utf-8")


def test_load_raw_results_reads_elapsed_seconds(tmp_path: Path) -> None:
    _write_json(
        tmp_path / "arco_build_n200.json",
        {
            "tool": "arco",
            "phase": "build",
            "n": 200,
            "elapsed_seconds": 0.25,
            "peak_rss_delta_bytes": 100_000_000,
        },
    )

    results = load_raw_results(tmp_path)

    assert results[("arco", "build", 200)]["wall_time_seconds"] == 0.25
    assert results[("arco", "build", 200)]["peak_memory_gb"] == 0.1


def test_merge_prefers_raw_memory_over_torc(tmp_path: Path) -> None:
    _write_json(
        tmp_path / "linopy_build_n400.json",
        {
            "tool": "linopy",
            "phase": "build",
            "n": 400,
            "elapsed_seconds": 1.75,
            "peak_rss_delta_bytes": 250_000_000,
        },
    )
    raw = load_raw_results(tmp_path)

    merged = _merge_with_memory(
        raw_rows=raw,
        memory_rows=[
            {
                "tool": "linopy",
                "phase": "build",
                "n": 400,
                "num_variables": 320_000,
                "peak_memory_gb": 0.99,
            }
        ],
    )

    assert len(merged) == 1
    assert merged[0]["wall_time_seconds"] == 1.75
    assert merged[0]["peak_memory_gb"] == 0.25
    assert merged[0]["memory_source"] == "raw"


def test_merge_falls_back_to_torc_memory(tmp_path: Path) -> None:
    _write_json(
        tmp_path / "pyomo_build_n500.json",
        {
            "tool": "pyomo",
            "phase": "build",
            "n": 500,
            "elapsed_seconds": 2.1,
        },
    )
    raw = load_raw_results(tmp_path)

    merged = _merge_with_memory(
        raw_rows=raw,
        memory_rows=[
            {
                "tool": "pyomo",
                "phase": "build",
                "n": 500,
                "num_variables": 500_000,
                "peak_memory_gb": 0.7,
            }
        ],
    )

    assert len(merged) == 1
    assert merged[0]["peak_memory_gb"] == 0.7
    assert merged[0]["memory_source"] == "torc"
