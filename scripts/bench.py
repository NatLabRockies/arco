#!/usr/bin/env python3
"""CLI benchmark runner for arco using rmon resource monitoring.

Wraps arco CLI commands with rmon to measure wall-clock duration and peak RSS.
Outputs github-action-benchmark compatible JSON.
"""

from __future__ import annotations

import argparse
import json
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

EXAMPLES_DIR = Path(__file__).parent.parent / "examples"
DEFAULT_CASES: tuple[str, ...] = (
    "generator-allocation",
    "price-taker-battery",
    "simple-electricity-market-storage",
    "capacity-expansion",
    "dense-lp",
)
DEFAULT_WORKFLOWS: tuple[str, ...] = ("validate", "run")
SUPPORTED_WORKFLOWS: tuple[str, ...] = (
    "validate",
    "print-model",
    "run",
    "compile",  # backward-compatible alias for print-model
    "solve",  # backward-compatible alias for run
)
MIN_MONITORED_RUNTIME_MS = 4_000.0
MAX_INNER_ITERATIONS = 5_000
_INNER_LOOP_RUNNER = (
    "import subprocess,sys\n"
    "iterations=int(sys.argv[1])\n"
    "cmd=sys.argv[2:]\n"
    "for _ in range(iterations):\n"
    "    rc=subprocess.run(cmd).returncode\n"
    "    if rc:\n"
    "        raise SystemExit(rc)\n"
)


@dataclass(frozen=True)
class BenchmarkCase:
    name: str
    kdl_path: Path
    workflow: str


@dataclass
class BenchmarkResult:
    case: str
    workflow: str
    median_duration_ms: float
    peak_rss_mb: float
    samples: int


def _canonical_workflow(workflow: str) -> str:
    if workflow == "compile":
        return "print-model"
    if workflow == "solve":
        return "run"
    return workflow


def discover_cases() -> list[BenchmarkCase]:
    """Discover curated, runnable KDL benchmark cases."""
    case_models: list[tuple[str, Path]] = []
    for case_name in DEFAULT_CASES:
        kdl_path = EXAMPLES_DIR / case_name / "input.kdl"
        if not kdl_path.is_file():
            print(f"Skipping missing benchmark case: {kdl_path}", file=sys.stderr)
            continue
        case_models.append((case_name, kdl_path))

    cases = [
        BenchmarkCase(name, kdl_path, workflow)
        for name, kdl_path in case_models
        for workflow in DEFAULT_WORKFLOWS
    ]
    return sorted(cases, key=lambda c: (c.name, c.workflow))


def _build_arco_args(*, workflow: str, model_path: Path) -> list[str]:
    if workflow == "validate":
        return ["validate", str(model_path)]
    if workflow == "print-model":
        return ["print-model", str(model_path)]
    if workflow == "run":
        return ["run", str(model_path), "--compact"]
    raise ValueError(f"unsupported workflow: {workflow}")


def _parse_rmon_peak_rss_mb(results_path: Path) -> float:
    if not results_path.is_file():
        return 0.0

    peak_rss_bytes = 0.0
    for line in results_path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        payload = json.loads(line)
        for entry in payload.get("results", []):
            if entry.get("resource_type") != "process":
                continue
            maximum = entry.get("maximum", {})
            rss = maximum.get("rss")
            if isinstance(rss, int | float):
                peak_rss_bytes = max(peak_rss_bytes, float(rss))

    return peak_rss_bytes / (1024.0 * 1024.0)


def _tail(text: str, *, max_lines: int = 8) -> str:
    lines = [line for line in text.strip().splitlines() if line.strip()]
    if not lines:
        return ""
    return "\n".join(lines[-max_lines:])


def _estimate_inner_iterations(arco_cmd: list[str]) -> int | None:
    started = time.perf_counter()
    try:
        result = subprocess.run(arco_cmd, capture_output=True, text=True, timeout=300)
    except (OSError, subprocess.TimeoutExpired) as exc:
        print(f"  benchmark command failed to launch: {exc}", file=sys.stderr)
        return None

    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if result.returncode != 0:
        print("  benchmark command failed during calibration", file=sys.stderr)
        stderr_tail = _tail(result.stderr)
        if stderr_tail:
            print(stderr_tail, file=sys.stderr)
        return None

    if elapsed_ms <= 0.0:
        return 1

    estimated = int(MIN_MONITORED_RUNTIME_MS / elapsed_ms) + 1
    return max(1, min(MAX_INNER_ITERATIONS, estimated))


def _build_monitored_command(
    arco_cmd: list[str], *, inner_iterations: int
) -> list[str]:
    if inner_iterations <= 1:
        return arco_cmd
    return [
        sys.executable,
        "-c",
        _INNER_LOOP_RUNNER,
        str(inner_iterations),
        *arco_cmd,
    ]


def run_benchmark(
    arco_binary: str, case: BenchmarkCase, repetitions: int
) -> BenchmarkResult | None:
    """Run benchmark for a single case with multiple repetitions."""
    durations: list[float] = []
    peak_rss: list[float] = []

    arco_cmd = [
        arco_binary,
        *_build_arco_args(workflow=case.workflow, model_path=case.kdl_path),
    ]
    inner_iterations = _estimate_inner_iterations(arco_cmd)
    if inner_iterations is None:
        return None

    if inner_iterations > 1:
        print(
            f"  using {inner_iterations} inner iterations for process-memory sampling",
            file=sys.stderr,
        )

    monitored_cmd = _build_monitored_command(
        arco_cmd, inner_iterations=inner_iterations
    )

    for repetition in range(repetitions):
        started = time.perf_counter()
        try:
            timed_result = subprocess.run(
                arco_cmd,
                capture_output=True,
                text=True,
                timeout=300,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            print(f"  arco command failed to launch: {exc}", file=sys.stderr)
            return None

        elapsed_ms = (time.perf_counter() - started) * 1000.0
        if timed_result.returncode != 0:
            print("  arco command failed", file=sys.stderr)
            stderr_tail = _tail(timed_result.stderr)
            if stderr_tail:
                print(stderr_tail, file=sys.stderr)
            return None

        with tempfile.TemporaryDirectory(prefix="arco-bench-rmon-") as output_dir:
            run_name = f"{case.name}-{case.workflow}-{repetition}"
            results_path = Path(output_dir) / f"{run_name}_results.json"
            cmd = [
                "rmon",
                "monitor-process",
                "--interval",
                "1",
                "--output",
                output_dir,
                "--name",
                run_name,
                "--",
                *monitored_cmd,
            ]
            try:
                result = subprocess.run(
                    cmd, capture_output=True, text=True, timeout=300
                )
            except (OSError, subprocess.TimeoutExpired) as exc:
                print(f"  rmon failed to launch: {exc}", file=sys.stderr)
                return None

            if result.returncode != 0:
                print("  rmon/arco command failed", file=sys.stderr)
                stderr_tail = _tail(result.stderr)
                if stderr_tail:
                    print(stderr_tail, file=sys.stderr)
                return None

            try:
                peak_rss_mb = _parse_rmon_peak_rss_mb(results_path)
            except json.JSONDecodeError as exc:
                print(f"  failed to parse rmon output JSON: {exc}", file=sys.stderr)
                return None

            durations.append(elapsed_ms)
            peak_rss.append(peak_rss_mb)

    return BenchmarkResult(
        case=case.name,
        workflow=case.workflow,
        median_duration_ms=statistics.median(durations),
        peak_rss_mb=max(peak_rss) if peak_rss else 0.0,
        samples=repetitions,
    )


def _parse_csv(value: str) -> list[str]:
    return [entry.strip() for entry in value.split(",") if entry.strip()]


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark arco CLI workflows")
    parser.add_argument("--arco-binary", default="arco", help="Path to arco binary")
    parser.add_argument(
        "--cases",
        help=(
            "Comma-separated benchmark case names "
            f"(default: {', '.join(DEFAULT_CASES)})"
        ),
    )
    parser.add_argument(
        "--workflows",
        default=",".join(DEFAULT_WORKFLOWS),
        help=(
            "Comma-separated workflows. Supported: " + ", ".join(SUPPORTED_WORKFLOWS)
        ),
    )
    parser.add_argument("--repetitions", type=int, default=10)
    parser.add_argument("--output", type=Path, default=Path("benchmark-results.json"))
    args = parser.parse_args()

    all_cases = discover_cases()

    requested_workflows = set(_parse_csv(args.workflows))
    unsupported = sorted(requested_workflows - set(SUPPORTED_WORKFLOWS))
    if unsupported:
        print("Unsupported workflows: " + ", ".join(unsupported), file=sys.stderr)
        return 2
    workflows = {_canonical_workflow(item) for item in requested_workflows}

    if args.cases:
        case_names = set(_parse_csv(args.cases))
        filtered = [
            c for c in all_cases if c.name in case_names and c.workflow in workflows
        ]
    else:
        filtered = [c for c in all_cases if c.workflow in workflows]

    if not filtered:
        print("No benchmark cases matched the selection.", file=sys.stderr)
        return 2

    results: list[BenchmarkResult] = []
    failures = 0
    for case in filtered:
        print(f"Benchmarking {case.name}/{case.workflow}...", file=sys.stderr)
        result = run_benchmark(args.arco_binary, case, args.repetitions)
        if result is None:
            failures += 1
            print("  FAILED", file=sys.stderr)
            continue

        results.append(result)
        print(
            f"  {result.median_duration_ms:.1f}ms, {result.peak_rss_mb:.1f}MiB",
            file=sys.stderr,
        )

    validate_medians = {
        r.case: r.median_duration_ms for r in results if r.workflow == "validate"
    }

    output = []
    for r in results:
        extra_lines = [
            f"workflow={r.workflow}",
            f"case={r.case}",
            f"peak_rss_mb={r.peak_rss_mb:.2f}",
            "aggregation=median",
        ]
        if r.workflow == "run" and r.case in validate_medians:
            validate_median = validate_medians[r.case]
            extra_lines.append(f"validate_median_ms={validate_median:.3f}")
            extra_lines.append(
                f"solve_estimate_ms={max(0.0, r.median_duration_ms - validate_median):.3f}"
            )

        output.append(
            {
                "name": f"{r.workflow}/{r.case}",
                "unit": "ms",
                "value": round(r.median_duration_ms, 3),
                "range": str(r.samples),
                "extra": "\n".join(extra_lines),
            }
        )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2))
    print(f"Results written to {args.output}", file=sys.stderr)

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
