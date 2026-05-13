#!/usr/bin/env python3
"""Create temporary CSV inputs for the ReEDS KDL benchmark."""

from __future__ import annotations

import argparse
import csv
import shutil
import tempfile
from pathlib import Path

KDL_FILES = ("input.kdl", "data.kdl", "model.kdl")
REGIONS = ("r1", "r2")
TECHS = ("gas", "wind")
HOURS = ("h1", "h2")
YEARS = (2030, 2035)


def write_csv(
    path: Path, header: tuple[str, ...], rows: list[tuple[object, ...]]
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(header)
        writer.writerows(rows)


def prepare_kdl_contract(output_dir: Path | None = None) -> Path:
    """Copy KDL files and generate tiny runnable CSV inputs."""

    if output_dir is None:
        output_dir = Path(tempfile.mkdtemp(prefix="arco-reeds-kdl-"))
    output_dir.mkdir(parents=True, exist_ok=True)

    source_dir = Path(__file__).resolve().parent
    for name in KDL_FILES:
        shutil.copy2(source_dir / name, output_dir / name)

    data_dir = output_dir / "data"
    write_csv(data_dir / "regions.csv", ("region",), [(r,) for r in REGIONS])
    write_csv(data_dir / "techs.csv", ("tech",), [(i,) for i in TECHS])
    write_csv(data_dir / "hours.csv", ("hour",), [(h,) for h in HOURS])
    write_csv(data_dir / "years.csv", ("year",), [(t,) for t in YEARS])
    write_csv(
        data_dir / "tech_params.csv",
        ("tech", "cost_inv", "cost_gen"),
        [("gas", 400.0, 45.0), ("wind", 900.0, 2.0)],
    )

    active = [(i, r, t) for i in TECHS for r in REGIONS for t in YEARS]
    write_csv(data_dir / "active_capacity.csv", ("tech", "region", "year"), active)
    write_csv(
        data_dir / "cap_transition.csv",
        ("tech", "region", "prev_year", "year"),
        [(i, r, 2030, 2035) for i in TECHS for r in REGIONS],
    )

    dispatch_rows = []
    for i in TECHS:
        for r in REGIONS:
            for h in HOURS:
                for t in YEARS:
                    cf = 1.0 if i == "gas" else (0.45 if h == "h1" else 0.25)
                    dispatch_rows.append((i, r, h, t, cf))
    write_csv(
        data_dir / "dispatch.csv",
        ("tech", "region", "hour", "year", "cf"),
        dispatch_rows,
    )

    write_csv(
        data_dir / "cap_init.csv",
        ("tech", "region", "cap_init"),
        [
            ("gas", "r1", 80.0),
            ("gas", "r2", 70.0),
            ("wind", "r1", 30.0),
            ("wind", "r2", 20.0),
        ],
    )
    write_csv(
        data_dir / "load.csv",
        ("region", "hour", "year", "load"),
        [
            (r, h, t, 65.0 if h == "h1" else 75.0)
            for r in REGIONS
            for h in HOURS
            for t in YEARS
        ],
    )
    return output_dir / "input.kdl"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Prepare a temporary ReEDS KDL benchmark input."
    )
    parser.add_argument("--output-dir", type=Path)
    args = parser.parse_args()
    print(prepare_kdl_contract(args.output_dir))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
