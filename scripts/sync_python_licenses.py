from __future__ import annotations

from pathlib import Path
import shutil
import sys
from typing import Sequence


LICENSE_FILENAMES: tuple[str, ...] = ("BSD-3-Clause.txt", "HiGHS-MIT.txt")


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _sync_python_licenses(*, repo_root: Path) -> list[Path]:
    source_dir = repo_root / "licenses"
    destination_dir = repo_root / "bindings" / "python" / "licenses"
    destination_dir.mkdir(parents=True, exist_ok=True)

    copied: list[Path] = []
    for filename in LICENSE_FILENAMES:
        source = source_dir / filename
        if not source.is_file():
            raise FileNotFoundError(
                f"Required license file is missing: {source.as_posix()}"
            )
        destination = destination_dir / filename
        shutil.copy2(source, destination)
        copied.append(destination)
    return copied


def main(*, argv: Sequence[str]) -> int:
    if argv:
        raise ValueError("This script does not accept positional arguments.")
    copied = _sync_python_licenses(repo_root=_repo_root())
    for path in copied:
        print(f"synced-license {path.as_posix()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
