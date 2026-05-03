#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

python3 - "$ROOT" <<'PY'
import re
import statistics
import subprocess
import sys

root = sys.argv[1]
runs = 3
results = []

for _ in range(runs):
    completed = subprocess.run(
        ["sentrux", "check"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    text = completed.stdout + completed.stderr
    quality = re.search(r"^Quality:\s+(\d+)$", text, re.M)
    unresolved = re.search(r"^\[resolve\]\s+\d+\s+resolved,\s+(\d+)\s+unresolved", text, re.M)
    import_edges = re.search(
        r"^\[build_graphs\].*?\|\s+(\d+)\s+import,\s+\d+\s+call,\s+\d+\s+inherit\s+edges$",
        text,
        re.M,
    )
    if not quality or not unresolved or not import_edges:
        print(text, file=sys.stderr)
        raise SystemExit(completed.returncode or 1)
    results.append(
        {
            "quality": int(quality.group(1)),
            "unresolved": int(unresolved.group(1)),
            "import_edges": int(import_edges.group(1)),
        }
    )
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)

for key in ("quality", "unresolved", "import_edges"):
    values = [result[key] for result in results]
    print(f"METRIC {key}={int(statistics.median(values))}")
PY
