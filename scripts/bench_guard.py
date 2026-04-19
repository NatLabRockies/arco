#!/usr/bin/env python3
"""Benchmark regression guard with basic noise-aware heuristics.

Consumes:
- current benchmark JSON from scripts/bench.py (list of {name,value,range,...})
- historical benchmark JSON produced by github-action-benchmark

Policy:
- compares only benchmarks present in both current and historical latest baseline
- enforces ratio threshold only when historical sample count >= min baseline samples
- requires minimum absolute delta to avoid tripping on tiny-ms noise
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Metric:
    name: str
    value: float
    samples: int


def _read_json(path: Path) -> Any:
    return json.loads(path.read_text())


def _parse_current_metrics(path: Path) -> dict[str, Metric]:
    payload = _read_json(path)
    if not isinstance(payload, list):
        raise ValueError(f"current payload must be a list: {path}")

    metrics: dict[str, Metric] = {}
    for entry in payload:
        if not isinstance(entry, dict):
            continue
        name = entry.get("name")
        value = entry.get("value")
        samples = entry.get("range", "0")
        if not isinstance(name, str) or not isinstance(value, (int, float)):
            continue
        try:
            sample_count = int(samples)
        except (TypeError, ValueError):
            sample_count = 0
        metrics[name] = Metric(name=name, value=float(value), samples=sample_count)
    return metrics


def _find_latest_baseline_entry(payload: Any) -> dict[str, Any] | None:
    if not isinstance(payload, dict):
        return None

    entries = payload.get("entries")
    if not isinstance(entries, list) or not entries:
        return None

    for entry in reversed(entries):
        if isinstance(entry, dict) and isinstance(entry.get("benchmarks"), list):
            return entry
    return None


def _parse_baseline_metrics(path: Path) -> dict[str, Metric]:
    payload = _read_json(path)
    latest = _find_latest_baseline_entry(payload)
    if latest is None:
        return {}

    metrics: dict[str, Metric] = {}
    benchmarks = latest.get("benchmarks", [])
    for bench in benchmarks:
        if not isinstance(bench, dict):
            continue
        name = bench.get("name")
        value = bench.get("value")
        samples = bench.get("range", "0")
        if not isinstance(name, str) or not isinstance(value, (int, float)):
            continue
        try:
            sample_count = int(samples)
        except (TypeError, ValueError):
            sample_count = 0
        metrics[name] = Metric(name=name, value=float(value), samples=sample_count)
    return metrics


def main() -> int:
    parser = argparse.ArgumentParser(description="Arco benchmark regression guard")
    parser.add_argument("--current", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--ratio-threshold", type=float, default=1.25)
    parser.add_argument("--absolute-delta-ms", type=float, default=1.0)
    parser.add_argument("--min-baseline-samples", type=int, default=10)
    args = parser.parse_args()

    current = _parse_current_metrics(args.current)
    if not current:
        print("bench-guard: no current metrics found; skip")
        return 0

    if not args.baseline.is_file():
        print(f"bench-guard: baseline file missing ({args.baseline}); skip")
        return 0

    baseline = _parse_baseline_metrics(args.baseline)
    if not baseline:
        print("bench-guard: no baseline metrics found; skip")
        return 0

    failures: list[str] = []
    skipped: list[str] = []
    compared = 0

    for name, current_metric in sorted(current.items()):
        baseline_metric = baseline.get(name)
        if baseline_metric is None:
            skipped.append(f"{name}:missing-baseline")
            continue
        if baseline_metric.samples < args.min_baseline_samples:
            skipped.append(
                f"{name}:insufficient-baseline-samples({baseline_metric.samples})"
            )
            continue
        if baseline_metric.value <= 0.0:
            skipped.append(f"{name}:non-positive-baseline({baseline_metric.value:.3f})")
            continue

        compared += 1
        ratio = current_metric.value / baseline_metric.value
        delta = current_metric.value - baseline_metric.value

        if ratio > args.ratio_threshold and delta >= args.absolute_delta_ms:
            failures.append(
                f"{name}: current={current_metric.value:.3f}ms baseline={baseline_metric.value:.3f}ms "
                f"ratio={ratio:.3f} delta={delta:.3f}ms"
            )

    print(f"bench-guard: compared={compared} skipped={len(skipped)} failures={len(failures)}")
    if skipped:
        print("bench-guard: skipped -> " + ", ".join(skipped))

    if failures:
        print("bench-guard: regressions detected")
        for failure in failures:
            print("  - " + failure)
        return 1

    print("bench-guard: pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
