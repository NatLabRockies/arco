from __future__ import annotations

import argparse
from pathlib import Path

from arco_benchmarks.github_action import run_benchmark_suite


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run arco-bench under resource_monitor and emit benchmark-action JSON"
    )
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--artifacts-dir", type=Path, required=True)
    parser.add_argument(
        "--scenarios",
        default="model-build,kdl-compile",
        help="Comma-separated arco-bench scenarios to execute",
    )
    parser.add_argument(
        "--cases",
        default="100,1000,10000,100000",
        help="Comma-separated model-build variable counts",
    )
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--sample-interval-seconds", type=float, default=1.0)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    scenarios = [scenario for scenario in args.scenarios.split(",") if scenario]
    model_build_cases = [int(value) for value in args.cases.split(",") if value]
    result = run_benchmark_suite(
        binary_path=args.binary,
        repo_root=args.repo_root,
        artifacts_dir=args.artifacts_dir,
        scenarios=scenarios,
        model_build_cases=model_build_cases,
        repetitions=args.repetitions,
        sample_interval_seconds=args.sample_interval_seconds,
    )
    print(f"benchmark-action: {result.benchmark_action_output_path}")
    print(f"combined-jsonl: {result.combined_jsonl_output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
