#!/usr/bin/env python3
"""Benchmark runner for arco using torc resource monitoring.

Wraps arco CLI commands with torc's built-in resource monitoring to measure
wall-clock duration and peak RSS.  Outputs github-action-benchmark-compatible JSON
on stdout; diagnostics go to stderr.

Requires ``torc`` and ``torc-server`` on PATH (install with
``cargo install torc --features server-bin``).

Exit codes
  0  all benchmarks succeeded
  1  one or more benchmark cases failed
  2  invalid arguments (unsupported workflow, no matching cases)
"""

from __future__ import annotations

import argparse
import json
import shlex
import statistics
import subprocess
import sys
import tempfile
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


@dataclass(frozen=True)
class BenchmarkCase:
    """A single arco CLI benchmark case (model path + workflow)."""

    name: str
    kdl_path: Path
    workflow: str


@dataclass
class BenchmarkResult:
    """Aggregated benchmark results across repetitions."""

    case: str
    workflow: str
    median_duration_ms: float
    peak_rss_mb: float
    samples: int


def canonical_workflow(workflow: str) -> str:
    """Resolve backward-compatible workflow aliases to canonical names."""
    if workflow == "compile":
        return "print-model"
    if workflow == "solve":
        return "run"
    return workflow


def discover_cases() -> list[BenchmarkCase]:
    """Discover curated, runnable KDL benchmark cases under examples/."""
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


def build_arco_args(*, workflow: str, model_path: Path) -> list[str]:
    """Build the arco CLI argument list for a given workflow and model path."""
    if workflow == "validate":
        return ["validate", str(model_path)]
    if workflow == "print-model":
        return ["print-model", str(model_path)]
    if workflow == "run":
        return ["run", str(model_path), "--compact"]
    raise ValueError(f"unsupported workflow: {workflow}")


def tail_lines(text: str, *, max_lines: int = 8) -> str:
    """Return the last non-blank lines of *text* for diagnostic display."""
    lines = [line for line in text.strip().splitlines() if line.strip()]
    if not lines:
        return ""
    return "\n".join(lines[-max_lines:])


def parse_csv(value: str) -> list[str]:
    """Split a comma-separated string into trimmed, non-empty entries."""
    return [entry.strip() for entry in value.split(",") if entry.strip()]


def run_benchmark(
    arco_binary: str, *, case: BenchmarkCase, repetitions: int
) -> BenchmarkResult | None:
    """Run a single benchmark case with torc and return aggregated results.

    Creates a torc workflow with *repetitions* identical jobs, runs them
    sequentially, then extracts peak memory and duration from torc's result
    records.
    """
    arco_args = build_arco_args(workflow=case.workflow, model_path=case.kdl_path)
    arco_cmd_str = shlex.join([arco_binary, *arco_args])

    with tempfile.TemporaryDirectory(prefix="arco-bench-torc-") as tmpdir:
        tmp = Path(tmpdir)
        db_path = tmp / "torc.db"
        commands_path = tmp / "commands.txt"

        # Write N identical commands (one per repetition).
        commands_path.write_text("\n".join([arco_cmd_str] * repetitions) + "\n")

        exec_cmd: list[str] = [
            "torc",
            "exec",
            "--standalone",
            "-C",
            str(commands_path),
            "--name",
            f"arco-bench-{case.name}-{case.workflow}",
            "--monitor",
            "summary",
            "--max-parallel-jobs",
            "1",
            "--sample-interval-seconds",
            "1",
            "--format",
            "json",
            "--db",
            str(db_path),
            "-o",
            str(tmp),
        ]

        try:
            exec_result = subprocess.run(
                exec_cmd, capture_output=True, text=True, timeout=600
            )
        except subprocess.TimeoutExpired:
            print("  torc exec timed out after 600s", file=sys.stderr)
            return None
        except OSError as exc:
            print(f"  torc exec failed to launch: {exc}", file=sys.stderr)
            return None

        if exec_result.returncode != 0:
            print("  torc exec command failed", file=sys.stderr)
            stderr_tail = tail_lines(exec_result.stderr)
            if stderr_tail:
                print(stderr_tail, file=sys.stderr)
            return None

        # Parse the workflow_id from torc exec JSON output.
        try:
            exec_payload = json.loads(exec_result.stdout)
        except json.JSONDecodeError as exc:
            print(f"  failed to parse torc exec JSON: {exc}", file=sys.stderr)
            return None
        try:
            workflow_id = exec_payload["workflow_id"]
        except KeyError:
            print(
                "  torc exec output missing workflow_id field", file=sys.stderr
            )
            return None

        # Query results for this workflow.
        results_cmd: list[str] = [
            "torc",
            "results",
            "list",
            str(workflow_id),
            "--format",
            "json",
            "--standalone",
            "--db",
            str(db_path),
        ]
        try:
            results_result = subprocess.run(
                results_cmd, capture_output=True, text=True, timeout=30
            )
        except subprocess.TimeoutExpired:
            print("  torc results list timed out", file=sys.stderr)
            return None
        except OSError as exc:
            print(
                f"  torc results list failed to launch: {exc}", file=sys.stderr
            )
            return None

        if results_result.returncode != 0:
            print("  torc results list command failed", file=sys.stderr)
            stderr_tail = tail_lines(results_result.stderr)
            if stderr_tail:
                print(stderr_tail, file=sys.stderr)
            return None

        try:
            results_payload = json.loads(results_result.stdout)
        except json.JSONDecodeError as exc:
            print(
                f"  failed to parse torc results JSON: {exc}", file=sys.stderr
            )
            return None
        try:
            items = results_payload["items"]
        except KeyError:
            print(
                "  torc results output missing items field", file=sys.stderr
            )
            return None

        durations: list[float] = []
        peak_rss: list[float] = []

        for item in items:
            exec_min = item.get("exec_time_minutes")
            peak_bytes = item.get("peak_memory_bytes")

            if isinstance(exec_min, (int, float)):
                durations.append(float(exec_min) * 60_000.0)

            # peak_memory_bytes can be None (torc didn't collect data) or 0
            # (collected zero / no samples fired).  Treat both as 0.0 MiB.
            if isinstance(peak_bytes, (int, float)):
                peak_rss.append(
                    max(float(peak_bytes), 0.0) / (1024.0 * 1024.0)
                )
            else:
                peak_rss.append(0.0)

        if not durations:
            print(
                f"  no duration data from {len(items)} results", file=sys.stderr
            )
            return None

        if len(durations) < repetitions:
            print(
                f"  warning: expected {repetitions} results, got {len(durations)}",
                file=sys.stderr,
            )

        return BenchmarkResult(
            case=case.name,
            workflow=case.workflow,
            median_duration_ms=statistics.median(durations),
            peak_rss_mb=max(peak_rss) if peak_rss else 0.0,
            samples=len(durations),
        )


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
            "Comma-separated workflows. Supported: "
            + ", ".join(SUPPORTED_WORKFLOWS)
        ),
    )
    parser.add_argument("--repetitions", type=int, default=10)
    parser.add_argument(
        "--output", type=Path, default=Path("benchmark-results.json")
    )
    args = parser.parse_args()

    all_cases = discover_cases()

    requested_workflows = set(parse_csv(args.workflows))
    unsupported = sorted(requested_workflows - set(SUPPORTED_WORKFLOWS))
    if unsupported:
        print("Unsupported workflows: " + ", ".join(unsupported), file=sys.stderr)
        return 2
    workflows = {canonical_workflow(item) for item in requested_workflows}

    if args.cases:
        case_names = set(parse_csv(args.cases))
        filtered = [
            c
            for c in all_cases
            if c.name in case_names and c.workflow in workflows
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
        result = run_benchmark(
            args.arco_binary, case=case, repetitions=args.repetitions
        )
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
        r.case: r.median_duration_ms
        for r in results
        if r.workflow == "validate"
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
            extra_lines.append(
                f"validate_median_ms={validate_median:.3f}"
            )
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
