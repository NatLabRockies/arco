#!/usr/bin/env python3
"""CLI benchmark runner for arco using rmon resource monitoring.

Wraps arco CLI commands with rmon to measure duration and peak memory.
Outputs github-action-benchmark compatible JSON.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

EXAMPLES_DIR = Path(__file__).parent.parent / "examples"
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
    name: str
    kdl_path: Path
    workflow: str


@dataclass
class BenchmarkResult:
    case: str
    workflow: str
    duration_ms: float
    peak_rss_mb: float
    samples: int


def discover_cases() -> list[BenchmarkCase]:
    """Discover available KDL example cases."""
    cases = []
    for kdl_file in EXAMPLES_DIR.rglob("*.kdl"):
        name = kdl_file.parent.name
        cases.append(BenchmarkCase(name, kdl_file, "validate"))
        cases.append(BenchmarkCase(name, kdl_file, "run"))
    return sorted(cases, key=lambda c: (c.name, c.workflow))


def _build_arco_args(*, workflow: str, model_path: Path) -> list[str]:
    if workflow in {"validate"}:
        return ["validate", str(model_path)]
    if workflow in {"print-model", "compile"}:
        return ["print-model", str(model_path)]
    if workflow in {"run", "solve"}:
        return ["run", str(model_path), "--compact"]
    raise ValueError(f"unsupported workflow: {workflow}")


def run_benchmark(
    arco_binary: str, case: BenchmarkCase, repetitions: int
) -> BenchmarkResult | None:
    """Run benchmark for a single case with multiple repetitions."""
    durations = []
    peak_rss = []

    for _ in range(repetitions):
        cmd = [
            "rmon",
            "monitor-process",
            "--format",
            "json",
            "--",
            arco_binary,
            *_build_arco_args(workflow=case.workflow, model_path=case.kdl_path),
        ]
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            if result.returncode != 0:
                return None
            data = json.loads(result.stdout)
            durations.append(data.get("duration_ms", 0))
            peak_rss.append(data.get("peak_rss_mb", 0))
        except (
            OSError,
            subprocess.TimeoutExpired,
            json.JSONDecodeError,
            KeyError,
            ValueError,
        ):
            return None

    return BenchmarkResult(
        case=case.name,
        workflow=case.workflow,
        duration_ms=sum(durations) / len(durations),
        peak_rss_mb=max(peak_rss),
        samples=repetitions,
    )


def _parse_csv(value: str) -> list[str]:
    return [entry.strip() for entry in value.split(",") if entry.strip()]


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark arco CLI workflows")
    parser.add_argument("--arco-binary", default="arco", help="Path to arco binary")
    parser.add_argument("--cases", help="Comma-separated case names (default: all)")
    parser.add_argument(
        "--workflows",
        default=",".join(DEFAULT_WORKFLOWS),
        help=(
            "Comma-separated workflows. Supported: "
            + ", ".join(SUPPORTED_WORKFLOWS)
        ),
    )
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--output", type=Path, default=Path("benchmark-results.json"))
    args = parser.parse_args()

    all_cases = discover_cases()
    workflows = set(_parse_csv(args.workflows))
    unsupported = sorted(workflows - set(SUPPORTED_WORKFLOWS))
    if unsupported:
        print(
            "Unsupported workflows: " + ", ".join(unsupported),
            file=sys.stderr,
        )
        return 2

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

    results = []
    failures = 0
    for case in filtered:
        print(f"Benchmarking {case.name}/{case.workflow}...", file=sys.stderr)
        result = run_benchmark(args.arco_binary, case, args.repetitions)
        if result:
            results.append(result)
            print(
                f"  {result.duration_ms:.1f}ms, {result.peak_rss_mb:.1f}MiB",
                file=sys.stderr,
            )
        else:
            failures += 1
            print("  FAILED", file=sys.stderr)

    output = [
        {
            "name": f"{r.workflow}/{r.case}",
            "unit": "ms",
            "value": round(r.duration_ms, 3),
            "range": str(r.samples),
            "extra": (
                f"workflow={r.workflow}\n"
                f"case={r.case}\n"
                f"peak_rss_mb={r.peak_rss_mb:.2f}"
            ),
        }
        for r in results
    ]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2))
    print(f"Results written to {args.output}", file=sys.stderr)

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
