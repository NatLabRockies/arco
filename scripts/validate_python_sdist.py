from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
from typing import Sequence


def _parse_args(*, argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Verify that a Python source distribution has a valid Cargo workspace."
    )
    parser.add_argument("--artifact", required=True, type=Path)
    return parser.parse_args(argv)


def _extract_sdist(*, artifact: Path, destination: Path) -> Path:
    with tarfile.open(artifact) as archive:
        members = archive.getmembers()
        top_levels = {member.name.split("/", maxsplit=1)[0] for member in members}
        if len(top_levels) != 1:
            raise ValueError(f"Expected one top-level directory in {artifact}")
        archive.extractall(destination)
    return destination / top_levels.pop()


def _validate_cargo_workspace(*, source_root: Path) -> None:
    subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        check=True,
        cwd=source_root,
        stdout=subprocess.DEVNULL,
    )


def main(*, argv: Sequence[str]) -> int:
    args = _parse_args(argv=argv)
    artifact = args.artifact.resolve()
    if not artifact.is_file():
        raise FileNotFoundError(f"Source distribution does not exist: {artifact}")

    with tempfile.TemporaryDirectory(prefix="arco-python-sdist-") as temporary_dir:
        source_root = _extract_sdist(artifact=artifact, destination=Path(temporary_dir))
        _validate_cargo_workspace(source_root=source_root)

    print(f"sdist-ok artifact={artifact}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
