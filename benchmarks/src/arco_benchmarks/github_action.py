from __future__ import annotations

from dataclasses import dataclass
import json
from pathlib import Path
import statistics
import subprocess
import time
from typing import Any, Sequence

KDL_COMPILE_CASES: tuple[str, ...] = (
    "capacity-expansion",
    "generator-allocation",
    "price-taker-battery",
    "simple-electricity-market-storage",
    "unit-commitment",
)


@dataclass(frozen=True)
class BenchmarkTarget:
    scenario: str
    case_name: str
    variables: int | None = None

    @property
    def slug(self) -> str:
        safe_case = self.case_name.replace("/", "-")
        return f"{self.scenario}_{safe_case}"


@dataclass(frozen=True)
class MonitorSummary:
    peak_rss_bytes: int | None
    avg_cpu_percent: float | None
    num_samples: int


@dataclass(frozen=True)
class BenchmarkSummary:
    scenario: str
    case_name: str
    variables: int
    constraints: int
    duration_ms_samples: tuple[float, ...]
    peak_rss_bytes: int | None
    peak_rss_source: str
    avg_cpu_percent: float | None
    monitor_samples: int


@dataclass(frozen=True)
class BenchmarkCollectionResult:
    benchmark_action_output_path: Path
    combined_jsonl_output_path: Path
    monitor_output_dir: Path
    summaries: tuple[BenchmarkSummary, ...]


@dataclass(frozen=True)
class MonitoredInvocationResult:
    bench_output_path: Path
    monitor_output_path: Path


def build_targets(*, scenarios: Sequence[str], model_build_cases: Sequence[int]) -> list[BenchmarkTarget]:
    targets: list[BenchmarkTarget] = []
    for scenario in scenarios:
        if scenario == "model-build":
            for variables in model_build_cases:
                targets.append(
                    BenchmarkTarget(
                        scenario=scenario,
                        case_name=f"vars_{variables}",
                        variables=variables,
                    )
                )
            continue
        if scenario == "kdl-compile":
            for case_name in KDL_COMPILE_CASES:
                targets.append(BenchmarkTarget(scenario=scenario, case_name=case_name))
            continue
        raise ValueError(f"Unsupported scenario for GitHub benchmark workflow: {scenario}")
    return targets


def run_benchmark_suite(
    *,
    binary_path: Path,
    repo_root: Path,
    artifacts_dir: Path,
    scenarios: Sequence[str],
    model_build_cases: Sequence[int],
    repetitions: int,
    sample_interval_seconds: float,
) -> BenchmarkCollectionResult:
    if repetitions <= 0:
        raise ValueError("repetitions must be positive")
    if sample_interval_seconds <= 0:
        raise ValueError("sample_interval_seconds must be positive")

    resolved_binary = binary_path.resolve()
    resolved_repo_root = repo_root.resolve()
    resolved_artifacts_dir = artifacts_dir.resolve()
    monitor_output_dir = resolved_artifacts_dir / "resource-monitor"
    bench_output_dir = resolved_artifacts_dir / "bench-jsonl"
    benchmark_action_output_path = resolved_artifacts_dir / "benchmark-results.json"
    combined_jsonl_output_path = resolved_artifacts_dir / "bench-results.jsonl"

    monitor_output_dir.mkdir(parents=True, exist_ok=True)
    bench_output_dir.mkdir(parents=True, exist_ok=True)

    targets = build_targets(scenarios=scenarios, model_build_cases=model_build_cases)
    invocation_results = [
        _run_target(
            target=target,
            binary_path=resolved_binary,
            repo_root=resolved_repo_root,
            bench_output_dir=bench_output_dir,
            monitor_output_dir=monitor_output_dir,
            repetitions=repetitions,
            sample_interval_seconds=sample_interval_seconds,
        )
        for target in targets
    ]

    summaries = tuple(
        summarize_benchmark_run(
            bench_output_path=result.bench_output_path,
            monitor_output_path=result.monitor_output_path,
        )
        for result in invocation_results
    )
    entries = [
        entry
        for summary in summaries
        for entry in benchmark_action_entries(summary)
    ]
    benchmark_action_output_path.write_text(
        json.dumps(entries, indent=2) + "\n",
        encoding="utf-8",
    )

    _write_combined_jsonl(
        output_path=combined_jsonl_output_path,
        bench_output_paths=[result.bench_output_path for result in invocation_results],
    )

    return BenchmarkCollectionResult(
        benchmark_action_output_path=benchmark_action_output_path,
        combined_jsonl_output_path=combined_jsonl_output_path,
        monitor_output_dir=monitor_output_dir,
        summaries=summaries,
    )


def summarize_benchmark_run(*, bench_output_path: Path, monitor_output_path: Path) -> BenchmarkSummary:
    records = _load_jsonl_records(bench_output_path)
    total_records = [record for record in records if record.get("stage") == "total"]
    if not total_records:
        raise ValueError(f"No total-stage rows found in {bench_output_path}")

    duration_ms_samples = tuple(float(record["duration_ms"]) for record in total_records)
    first_record = total_records[0]
    internal_peak_rss_bytes = _peak_rss_from_records(records)
    monitor_summary = load_monitor_summary(monitor_output_path)
    peak_rss_bytes = monitor_summary.peak_rss_bytes or internal_peak_rss_bytes
    peak_rss_source = "resource_monitor" if monitor_summary.peak_rss_bytes is not None else "arco_bench"

    return BenchmarkSummary(
        scenario=str(first_record["scenario"]),
        case_name=str(first_record["case_name"]),
        variables=int(first_record["variables"]),
        constraints=int(first_record["constraints"]),
        duration_ms_samples=duration_ms_samples,
        peak_rss_bytes=peak_rss_bytes,
        peak_rss_source=peak_rss_source,
        avg_cpu_percent=monitor_summary.avg_cpu_percent,
        monitor_samples=monitor_summary.num_samples,
    )


def load_monitor_summary(path: Path) -> MonitorSummary:
    lines = [line for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(lines) < 2:
        return MonitorSummary(peak_rss_bytes=None, avg_cpu_percent=None, num_samples=0)

    process_payload = json.loads(lines[1])
    process_results = process_payload.get("results")
    if not isinstance(process_results, list) or not process_results:
        return MonitorSummary(peak_rss_bytes=None, avg_cpu_percent=None, num_samples=0)

    process_row = process_results[0]
    maximum = process_row.get("maximum")
    average = process_row.get("average")
    peak_rss = None
    avg_cpu = None
    if isinstance(maximum, dict) and isinstance(maximum.get("rss"), (int, float)):
        peak_rss = int(maximum["rss"])
    if isinstance(average, dict) and isinstance(average.get("cpu_percent"), (int, float)):
        avg_cpu = float(average["cpu_percent"])

    num_samples = process_row.get("num_samples")
    return MonitorSummary(
        peak_rss_bytes=peak_rss,
        avg_cpu_percent=avg_cpu,
        num_samples=int(num_samples) if isinstance(num_samples, int) else 0,
    )


def benchmark_action_entries(summary: BenchmarkSummary) -> list[dict[str, object]]:
    duration_value = round(statistics.median(summary.duration_ms_samples), 3)
    duration_range = round(
        max(summary.duration_ms_samples) - min(summary.duration_ms_samples),
        3,
    )
    extra_lines = [
        f"scenario={summary.scenario}",
        f"case={summary.case_name}",
        f"variables={summary.variables}",
        f"constraints={summary.constraints}",
        f"samples={len(summary.duration_ms_samples)}",
        f"peak_rss_source={summary.peak_rss_source}",
    ]
    if summary.avg_cpu_percent is not None:
        extra_lines.append(f"avg_cpu_percent={summary.avg_cpu_percent:.3f}")
    if summary.monitor_samples > 0:
        extra_lines.append(f"monitor_samples={summary.monitor_samples}")

    entries: list[dict[str, object]] = [
        {
            "name": f"{summary.scenario}/{summary.case_name} duration",
            "unit": "ms",
            "value": duration_value,
            "range": f"{duration_range:.3f}",
            "extra": "\n".join(extra_lines),
        }
    ]
    if summary.peak_rss_bytes is not None:
        entries.append(
            {
                "name": f"{summary.scenario}/{summary.case_name} peak-rss",
                "unit": "MiB",
                "value": round(summary.peak_rss_bytes / (1024 * 1024), 3),
                "extra": "\n".join(extra_lines),
            }
        )
    return entries


def _run_target(
    *,
    target: BenchmarkTarget,
    binary_path: Path,
    repo_root: Path,
    bench_output_dir: Path,
    monitor_output_dir: Path,
    repetitions: int,
    sample_interval_seconds: float,
) -> MonitoredInvocationResult:
    bench_output_path = bench_output_dir / f"{target.slug}.jsonl"
    monitor_output_path = monitor_output_dir / f"{target.slug}.jsonl"
    stdout_path = monitor_output_dir / f"{target.slug}.stdout.log"
    stderr_path = monitor_output_dir / f"{target.slug}.stderr.log"
    command = _build_command(
        binary_path=binary_path,
        target=target,
        repetitions=repetitions,
        bench_output_path=bench_output_path,
    )
    _monitor_subprocess(
        command=command,
        repo_root=repo_root,
        sample_interval_seconds=sample_interval_seconds,
        stdout_path=stdout_path,
        stderr_path=stderr_path,
        monitor_output_path=monitor_output_path,
    )
    return MonitoredInvocationResult(
        bench_output_path=bench_output_path,
        monitor_output_path=monitor_output_path,
    )


def _build_command(
    *,
    binary_path: Path,
    target: BenchmarkTarget,
    repetitions: int,
    bench_output_path: Path,
) -> list[str]:
    command = [
        str(binary_path),
        "run",
        "--scenario",
        target.scenario,
        "--repetitions",
        str(repetitions),
        "--output",
        str(bench_output_path),
        "--format",
        "json",
    ]
    if target.scenario == "model-build":
        if target.variables is None:
            raise ValueError("model-build target requires variables")
        command.extend(["--variables", str(target.variables)])
    else:
        command.extend(["--case", target.case_name])
    return command


def _monitor_subprocess(
    *,
    command: Sequence[str],
    repo_root: Path,
    sample_interval_seconds: float,
    stdout_path: Path,
    stderr_path: Path,
    monitor_output_path: Path,
) -> None:
    from rmon.models import ComputeNodeResourceStatConfig
    from rmon.resource_stat_aggregator import ResourceStatAggregator
    from rmon.resource_stat_collector import ResourceStatCollector

    config = ComputeNodeResourceStatConfig(
        cpu=False,
        disk=False,
        memory=False,
        network=False,
        process=True,
        include_child_processes=True,
        recurse_child_processes=True,
        monitor_type="aggregation",
        make_plots=False,
        interval=sample_interval_seconds,
    )
    collector = ResourceStatCollector()
    seed_stats = collector.get_stats(ComputeNodeResourceStatConfig.all_enabled(), pids={})
    aggregator = ResourceStatAggregator(config, seed_stats)

    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    process_key = Path(command[0]).name
    with stdout_path.open("w", encoding="utf-8") as stdout_handle, stderr_path.open(
        "w", encoding="utf-8"
    ) as stderr_handle:
        with subprocess.Popen(
            list(command),
            cwd=repo_root,
            stdout=stdout_handle,
            stderr=stderr_handle,
            text=True,
        ) as process:
            pids = {process_key: process.pid}
            while True:
                stats = collector.get_stats(config, pids=pids)
                aggregator.update_stats(stats)
                if process.poll() is not None:
                    break
                time.sleep(sample_interval_seconds)
            return_code = process.wait()

    system_results = aggregator.finalize_system_stats()
    process_results = aggregator.finalize_process_stats(pids.keys())
    collector.clear_cache()
    monitor_output_path.write_text(
        system_results.model_dump_json() + "\n" + process_results.model_dump_json() + "\n",
        encoding="utf-8",
    )

    if return_code != 0:
        raise subprocess.CalledProcessError(return_code, list(command))


def _write_combined_jsonl(*, output_path: Path, bench_output_paths: Sequence[Path]) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8") as handle:
        for bench_output_path in bench_output_paths:
            for line in bench_output_path.read_text(encoding="utf-8").splitlines():
                if line.strip():
                    handle.write(line)
                    handle.write("\n")


def _load_jsonl_records(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def _peak_rss_from_records(records: Sequence[dict[str, Any]]) -> int | None:
    peaks = [
        int(value)
        for record in records
        for key in ("rss_before_bytes", "rss_after_bytes")
        for value in [record.get(key)]
        if isinstance(value, int)
    ]
    return max(peaks) if peaks else None
