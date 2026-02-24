from __future__ import annotations

import json
import re
import shutil
import subprocess
from pathlib import Path
from typing import Any

import pandas as pd

from arco_benchmarks.model import num_variables

JOB_PATTERN = re.compile(r"bench_(?P<tool>[a-z]+)_(?P<phase>[a-z]+)_n(?P<n>\d+)")


def _first_number(row: dict[str, Any], names: tuple[str, ...]) -> float | None:
    for name in names:
        value = row.get(name)
        if isinstance(value, (int, float)):
            return float(value)
    return None


def _extract_items(payload: Any) -> list[dict[str, Any]]:
    if isinstance(payload, list):
        return [x for x in payload if isinstance(x, dict)]
    if isinstance(payload, dict):
        for key in ("results", "workflows", "jobs", "data", "items", "rows"):
            value = payload.get(key)
            if isinstance(value, list):
                return [x for x in value if isinstance(x, dict)]
        return [payload]
    return []


def normalize_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    normalized: list[dict[str, Any]] = []
    for row in rows:
        job_name = str(row.get("job_name") or row.get("name") or "")
        match = JOB_PATTERN.match(job_name)
        if match is None:
            continue

        n = int(match.group("n"))
        peak_memory_gb = _first_number(
            row,
            (
                "peak_memory_gb",
                "peak_memory",
                "max_memory_gb",
                "memory_gb",
            ),
        )
        if peak_memory_gb is None:
            peak_memory_bytes = _first_number(
                row, ("peak_memory_bytes", "memory_bytes")
            )
            peak_memory_gb = (
                (peak_memory_bytes / 1.0e9) if peak_memory_bytes is not None else None
            )

        wall_time_seconds = _first_number(
            row,
            (
                "duration_seconds",
                "wall_time_seconds",
                "runtime_seconds",
                "execution_time",
                "duration",
            ),
        )
        if wall_time_seconds is None:
            exec_minutes = _first_number(row, ("exec_time_minutes",))
            if exec_minutes is not None:
                wall_time_seconds = exec_minutes * 60.0

        normalized.append(
            {
                "tool": match.group("tool"),
                "phase": match.group("phase"),
                "n": n,
                "num_variables": num_variables(n),
                "wall_time_seconds": wall_time_seconds,
                "peak_memory_gb": peak_memory_gb,
            }
        )
    return normalized


def _get_latest_workflow_id() -> int:
    torc = shutil.which("torc") or f"{Path.home()}/.local/bin/torc"
    proc = subprocess.run(
        [torc, "-f", "json", "workflows", "list"],
        check=True,
        capture_output=True,
        text=True,
    )
    payload = json.loads(proc.stdout)
    items = _extract_items(payload)
    if not items:
        raise RuntimeError("No workflows found in Torc")
    latest = max(items, key=lambda x: int(x.get("id", 0)))
    return int(latest["id"])


def export_results(*, workflow_id: int | None, out_path: Path) -> Path:
    torc = shutil.which("torc") or f"{Path.home()}/.local/bin/torc"
    workflow = workflow_id if workflow_id is not None else _get_latest_workflow_id()
    jobs_proc = subprocess.run(
        [torc, "-f", "json", "jobs", "list", str(workflow)],
        check=True,
        capture_output=True,
        text=True,
    )
    job_rows = _extract_items(json.loads(jobs_proc.stdout))
    job_names = {
        int(row["id"]): str(row.get("name", "")) for row in job_rows if "id" in row
    }

    proc = subprocess.run(
        [torc, "-f", "json", "results", "list", str(workflow)],
        check=True,
        capture_output=True,
        text=True,
    )
    payload = json.loads(proc.stdout)
    rows = _extract_items(payload)
    for row in rows:
        job_id = row.get("job_id")
        if isinstance(job_id, int) and job_id in job_names:
            row["job_name"] = job_names[job_id]

    normalized = normalize_rows(rows)
    if not normalized:
        raise RuntimeError("No benchmark rows found in Torc results")

    df = pd.DataFrame(normalized)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    df.sort_values(["phase", "tool", "n"]).to_csv(out_path, index=False)
    return out_path
