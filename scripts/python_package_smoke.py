from __future__ import annotations

import argparse
import glob
from pathlib import Path
import subprocess
import sys
from typing import Sequence


def _parse_args(*, argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Install a built artifact with uv and validate import."
    )
    parser.add_argument(
        "--artifact-glob",
        required=True,
        help="Glob pattern for built distributions, for example 'dist/*.whl'.",
    )
    parser.add_argument(
        "--import-name",
        default="arco",
        help="Top-level import to validate after installation.",
    )
    return parser.parse_args(argv)


def _resolve_artifacts(*, pattern: str) -> list[Path]:
    artifacts = [Path(path).resolve() for path in glob.glob(pattern)]
    if not artifacts:
        raise FileNotFoundError(f"No package artifacts matched pattern: {pattern}")
    return sorted(artifacts)


def _run_uv_smoke(*, python_executable: Path, artifacts: Sequence[Path], import_name: str) -> None:
    command = [
        "uv",
        "run",
        "--no-project",
        "--isolated",
        "--python",
        str(python_executable),
    ]
    for artifact in artifacts:
        command.extend(["--with", str(artifact)])
    command.extend(
        [
            "python",
            "-c",
            f"import importlib; importlib.import_module({import_name!r})",
        ]
    )
    subprocess.check_call(command)


def main(*, argv: Sequence[str]) -> int:
    args = _parse_args(argv=argv)
    artifacts = _resolve_artifacts(pattern=args.artifact_glob)
    python_executable = Path(sys.executable)
    _run_uv_smoke(
        python_executable=python_executable,
        artifacts=artifacts,
        import_name=args.import_name,
    )
    print(f"smoke-ok import={args.import_name} artifacts={len(artifacts)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
