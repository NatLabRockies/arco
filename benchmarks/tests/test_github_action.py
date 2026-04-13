from __future__ import annotations

import json
from pathlib import Path

from arco_benchmarks.github_action import (
    KDL_COMPILE_CASES,
    BenchmarkSummary,
    benchmark_action_entries,
    build_targets,
    summarize_benchmark_run,
)


def _write_jsonl(path: Path, rows: list[dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "\n".join(json.dumps(row) for row in rows) + "\n",
        encoding="utf-8",
    )


def test_build_targets_expands_requested_scenarios() -> None:
    targets = build_targets(
        scenarios=("model-build", "kdl-compile"),
        model_build_cases=(100, 1000),
    )

    assert [target.case_name for target in targets[:2]] == ["vars_100", "vars_1000"]
    assert [target.case_name for target in targets[2:]] == list(KDL_COMPILE_CASES)


def test_summarize_benchmark_run_prefers_resource_monitor_peak_rss(tmp_path: Path) -> None:
    bench_path = tmp_path / "bench.jsonl"
    monitor_path = tmp_path / "monitor.jsonl"
    _write_jsonl(
        bench_path,
        [
            {
                "scenario": "model-build",
                "case_name": "vars_100",
                "variables": 100,
                "constraints": 1,
                "stage": "total",
                "duration_ms": 10.0,
                "rss_before_bytes": 1_000,
                "rss_after_bytes": 2_000,
            },
            {
                "scenario": "model-build",
                "case_name": "vars_100",
                "variables": 100,
                "constraints": 1,
                "stage": "total",
                "duration_ms": 14.0,
                "rss_before_bytes": 2_000,
                "rss_after_bytes": 3_000,
            },
        ],
    )
    _write_jsonl(
        monitor_path,
        [
            {"hostname": "ci", "results": []},
            {
                "hostname": "ci",
                "results": [
                    {
                        "process_key": "arco-bench",
                        "num_samples": 5,
                        "average": {"cpu_percent": 83.5},
                        "minimum": {"rss": 1_000_000},
                        "maximum": {"rss": 9_000_000},
                    }
                ],
            },
        ],
    )

    summary = summarize_benchmark_run(
        bench_output_path=bench_path,
        monitor_output_path=monitor_path,
    )

    assert summary.duration_ms_samples == (10.0, 14.0)
    assert summary.peak_rss_bytes == 9_000_000
    assert summary.peak_rss_source == "resource_monitor"
    assert summary.avg_cpu_percent == 83.5
    assert summary.monitor_samples == 5


def test_summarize_benchmark_run_falls_back_to_arco_bench_peak_rss(tmp_path: Path) -> None:
    bench_path = tmp_path / "bench.jsonl"
    monitor_path = tmp_path / "monitor.jsonl"
    _write_jsonl(
        bench_path,
        [
            {
                "scenario": "kdl-compile",
                "case_name": "unit-commitment",
                "variables": 200,
                "constraints": 20,
                "stage": "parse",
                "duration_ms": 2.0,
                "rss_before_bytes": 5_000,
                "rss_after_bytes": 7_000,
            },
            {
                "scenario": "kdl-compile",
                "case_name": "unit-commitment",
                "variables": 200,
                "constraints": 20,
                "stage": "total",
                "duration_ms": 8.0,
                "rss_before_bytes": 7_000,
                "rss_after_bytes": 11_000,
            },
        ],
    )
    _write_jsonl(
        monitor_path,
        [
            {"hostname": "ci", "results": []},
            {"hostname": "ci", "results": []},
        ],
    )

    summary = summarize_benchmark_run(
        bench_output_path=bench_path,
        monitor_output_path=monitor_path,
    )

    assert summary.peak_rss_bytes == 11_000
    assert summary.peak_rss_source == "arco_bench"
    assert summary.monitor_samples == 0


def test_benchmark_action_entries_emit_duration_and_memory() -> None:
    summary = BenchmarkSummary(
        scenario="model-build",
        case_name="vars_1000",
        variables=1000,
        constraints=10,
        duration_ms_samples=(10.0, 14.0, 11.0),
        peak_rss_bytes=8 * 1024 * 1024,
        peak_rss_source="resource_monitor",
        avg_cpu_percent=77.25,
        monitor_samples=3,
    )

    entries = benchmark_action_entries(summary)

    assert [entry["name"] for entry in entries] == [
        "model-build/vars_1000 duration",
        "model-build/vars_1000 peak-rss",
    ]
    assert entries[0]["value"] == 11.0
    assert entries[0]["range"] == "4.000"
    assert entries[1]["value"] == 8.0
    assert "peak_rss_source=resource_monitor" in str(entries[1]["extra"])
