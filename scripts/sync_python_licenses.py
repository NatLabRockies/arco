from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import sys
from typing import Sequence


LICENSE_FILENAMES: tuple[str, ...] = ("BSD-3-Clause.txt", "HiGHS-MIT.txt")


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def sync_python_licenses(*, repo_root: Path) -> list[Path]:
    resolved_repo_root = repo_root.resolve()
    source_dir = resolved_repo_root / "licenses"
    python_root = resolved_repo_root / "bindings" / "python"
    license_dir = python_root / "licenses"
    license_dir.mkdir(parents=True, exist_ok=True)

    copied: list[Path] = []
    for filename in LICENSE_FILENAMES:
        source = source_dir / filename
        if not source.is_file():
            raise FileNotFoundError(
                f"Required license file is missing: {source.as_posix()}"
            )
        destination = license_dir / filename
        shutil.copy2(source, destination)
        copied.append(destination)

        if filename == "BSD-3-Clause.txt":
            root_destination = python_root / filename
            shutil.copy2(source, root_destination)
            copied.append(root_destination)
    return copied


def _parse_args(*, argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Sync license files into the Python package tree."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=_repo_root(),
        help="Repository root whose bindings/python package should receive licenses.",
    )
    return parser.parse_args(argv)


def main(*, argv: Sequence[str]) -> int:
    args = _parse_args(argv=argv)
    copied = sync_python_licenses(repo_root=args.repo_root)
    for path in copied:
        print(f"synced-license {path.as_posix()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
