"""Convert arco-bench JSONL output to github-action-benchmark format."""

from __future__ import annotations

import json
import sys


def convert(input_path: str, output_path: str) -> None:
    with open(input_path) as f:
        records = [json.loads(line) for line in f if line.strip()]

    results = []
    for r in records:
        if r.get("stage") != "total":
            continue
        results.append(
            {
                "name": f"{r['scenario']}/{r['case_name']}",
                "unit": "ms",
                "value": round(r["duration_ms"], 3),
                "extra": f"vars={r['variables']} cons={r['constraints']}",
            }
        )

    if not results:
        print("Error: no benchmark results produced", file=sys.stderr)
        raise SystemExit(1)

    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)
        f.write("\n")

    print(f"Converted {len(records)} records -> {len(results)} benchmarks")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.jsonl> <output.json>", file=sys.stderr)
        raise SystemExit(1)
    convert(sys.argv[1], sys.argv[2])
