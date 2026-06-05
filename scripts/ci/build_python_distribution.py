from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
from typing import Sequence

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.sync_python_licenses import sync_python_licenses  # noqa: E402


def build_wheel(
    *, source_dir: Path, out_dir: Path, python_interpreter: str, wheel_features: str
) -> None:
    sync_python_licenses(repo_root=source_dir)
    command = [
        "uv",
        "run",
        "--project",
        str(source_dir / "bindings" / "python"),
        "--with",
        "maturin",
        "maturin",
        "build",
        "--release",
        "--manifest-path",
        str(source_dir / "bindings" / "python" / "Cargo.toml"),
        "-i",
        python_interpreter,
        "--compatibility",
        "pypi",
        "--out",
        str(out_dir),
    ]
    if wheel_features:
        command.extend(["--features", wheel_features])
    subprocess.check_call(command)


def build_sdist(*, source_dir: Path, out_dir: Path) -> None:
    sync_python_licenses(repo_root=source_dir)
    command = [
        "uv",
        "run",
        "--project",
        str(source_dir / "bindings" / "python"),
        "--with",
        "maturin",
        "maturin",
        "sdist",
        "--manifest-path",
        str(source_dir / "bindings" / "python" / "Cargo.toml"),
        "--out",
        str(out_dir),
    ]
    subprocess.check_call(command)


def _existing_path(*, path: Path, name: str) -> Path:
    resolved = path.resolve()
    if not resolved.exists():
        raise FileNotFoundError(f"{name} does not exist: {resolved.as_posix()}")
    return resolved


def _parse_args(*, argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build Arco Python distributions from a selected source tree."
    )
    parser.add_argument("--build", choices=("wheel", "sdist"), required=True)
    parser.add_argument(
        "--source-dir",
        type=Path,
        default=Path(os.environ.get("PYTHON_WHEEL_SOURCE_DIR", ".")),
        help="Repository root containing the Python package source to build.",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path(os.environ.get("PYTHON_WHEEL_OUT_DIR", "dist")),
        help="Directory for built distributions.",
    )
    parser.add_argument(
        "--python-interpreter",
        default=os.environ.get("PYTHON_WHEEL_INTERPRETER", "python3"),
        help="Python interpreter argument passed to maturin for wheel builds.",
    )
    parser.add_argument(
        "--wheel-features",
        default=os.environ.get("PYTHON_WHEEL_FEATURES", ""),
        help="Optional comma-separated Cargo feature list passed to maturin wheel builds.",
    )
    return parser.parse_args(argv)


def main(*, argv: Sequence[str]) -> int:
    args = _parse_args(argv=argv)
    source_dir = _existing_path(path=args.source_dir, name="source directory")
    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    if args.build == "wheel":
        build_wheel(
            source_dir=source_dir,
            out_dir=out_dir,
            python_interpreter=args.python_interpreter,
            wheel_features=args.wheel_features,
        )
    else:
        build_sdist(source_dir=source_dir, out_dir=out_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
