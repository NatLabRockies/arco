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


@dataclass(frozen=True)
class BenchmarkCase:
    name: str
    kdl_path: Path
    command: str  # "compile", "solve", "validate"


@dataclass
class BenchmarkResult:
    case: str
    command: str
    duration_ms: float
    peak_rss_mb: float
    samples: int


def discover_cases() -> list[BenchmarkCase]:
    """Discover available KDL example cases."""
    cases = []
    for kdl_file in EXAMPLES_DIR.rglob("*.kdl"):
        name = kdl_file.parent.name
        cases.append(BenchmarkCase(name, kdl_file, "compile"))
        cases.append(BenchmarkCase(name, kdl_file, "solve"))
    return sorted(cases, key=lambda c: (c.name, c.command))


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
            case.command,
            str(case.kdl_path),
        ]
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            if result.returncode != 0:
                return None
            data = json.loads(result.stdout)
            durations.append(data.get("duration_ms", 0))
            peak_rss.append(data.get("peak_rss_mb", 0))
        except (subprocess.TimeoutExpired, json.JSONDecodeError, KeyError):
            return None

    return BenchmarkResult(
        case=case.name,
        command=case.command,
        duration_ms=sum(durations) / len(durations),
        peak_rss_mb=max(peak_rss),
        samples=repetitions,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark arco CLI workflows")
    parser.add_argument("--arco-binary", default="arco", help="Path to arco binary")
    parser.add_argument("--cases", help="Comma-separated case names (default: all)")
    parser.add_argument(
        "--workflows", default="compile,solve", help="Comma-separated commands"
    )
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--output", type=Path, default=Path("benchmark-results.json"))
    args = parser.parse_args()

    all_cases = discover_cases()
    workflows = set(args.workflows.split(","))

    if args.cases:
        case_names = set(args.cases.split(","))
        filtered = [
            c for c in all_cases if c.name in case_names and c.command in workflows
        ]
    else:
        filtered = [c for c in all_cases if c.command in workflows]

    results = []
    for case in filtered:
        print(f"Benchmarking {case.name}/{case.command}...", file=sys.stderr)
        result = run_benchmark(args.arco_binary, case, args.repetitions)
        if result:
            results.append(result)
            print(
                f"  {result.duration_ms:.1f}ms, {result.peak_rss_mb:.1f}MiB",
                file=sys.stderr,
            )
        else:
            print("  FAILED", file=sys.stderr)

    output = [
        {
            "name": f"{r.command}/{r.case}",
            "unit": "ms",
            "value": round(r.duration_ms, 3),
            "range": str(r.samples),
            "extra": f"command={r.command}\ncase={r.case}\npeak_rss_mb={r.peak_rss_mb:.2f}",
        }
        for r in results
    ]
    args.output.write_text(json.dumps(output, indent=2))
    print(f"Results written to {args.output}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
