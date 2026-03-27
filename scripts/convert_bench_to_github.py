"""Convert arco-bench JSONL output to github-action-benchmark format."""

from __future__ import annotations

import json
from collections import defaultdict
from statistics import median
import sys


def convert(input_path: str, output_path: str) -> None:
    with open(input_path) as f:
        records = [json.loads(line) for line in f if line.strip()]

    durations_by_name: dict[str, list[float]] = defaultdict(list)
    extra_by_name: dict[str, str] = {}
    for r in records:
        if r.get("stage") != "total":
            continue

        benchmark_name = f"{r['scenario']}/{r['case_name']}"
        durations_by_name[benchmark_name].append(float(r["duration_ms"]))
        extra_by_name[benchmark_name] = f"vars={r['variables']} cons={r['constraints']}"

    results = [
        {
            "name": name,
            "unit": "ms",
            "value": round(median(durations), 3),
            "extra": extra_by_name[name],
        }
        for name, durations in sorted(durations_by_name.items())
    ]

    if not results:
        print("Error: no benchmark results produced", file=sys.stderr)
        raise SystemExit(1)

    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)
        f.write("\n")

    total_records = sum(len(durations) for durations in durations_by_name.values())
    print(
        f"Converted {len(records)} records "
        f"({total_records} total-stage samples) -> {len(results)} benchmarks"
    )


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.jsonl> <output.json>", file=sys.stderr)
        raise SystemExit(1)
    convert(sys.argv[1], sys.argv[2])
