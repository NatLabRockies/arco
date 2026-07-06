from __future__ import annotations

import argparse
import glob
import os
from pathlib import Path
import subprocess
import sys
from typing import Mapping, Sequence

_WINDOWS_DLL_DIRS_ENV = "ARCO_PYTHON_SMOKE_DLL_DIRS"


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


def _normalize_windows_path(path: str) -> str:
    if len(path) >= 3 and path[0] == "/" and path[2] == "/":
        drive = path[1]
        if drive.isalpha():
            return f"{drive.upper()}:{path[2:]}"
    return path


def _split_runtime_path_list(value: str) -> list[str]:
    return [part for part in value.split(":") if part]


def _runtime_path_candidates(*, env: Mapping[str, str]) -> list[str]:
    candidates: list[str] = []

    for name, value in env.items():
        if name == "SCIP_SYS_BUNDLED_DIR" or name.startswith("SCIP_SYS_BUNDLED_DIR_"):
            root = Path(value)
            candidates.extend([str(root / "bin"), str(root / "lib")])
        elif name == "ARCO_SCIP_LIBRARY_PATH" or name.startswith(
            "ARCO_SCIP_LIBRARY_PATH_"
        ):
            library_path = Path(value)
            candidates.append(str(library_path))
            if library_path.name == "lib":
                candidates.append(str(library_path.parent / "bin"))

    if xpress_dir := env.get("XPRESSDIR"):
        root = Path(xpress_dir)
        candidates.extend([str(root / "bin"), str(root / "lib")])

    if not candidates:
        for name in ("LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH", "LIBRARY_PATH"):
            candidates.extend(_split_runtime_path_list(env.get(name, "")))

    return candidates


def _smoke_env(
    *, env: Mapping[str, str], platform: str = sys.platform, pathsep: str = os.pathsep
) -> dict[str, str]:
    smoke_env = dict(env)
    if platform != "win32":
        return smoke_env

    runtime_paths: list[str] = []
    seen: set[str] = set()
    for candidate in _runtime_path_candidates(env=env):
        normalized = _normalize_windows_path(candidate)
        key = normalized.lower()
        if key in seen:
            continue
        seen.add(key)
        runtime_paths.append(normalized)

    if runtime_paths:
        existing_path = smoke_env.get("PATH", "")
        smoke_env["PATH"] = pathsep.join(
            [*runtime_paths, *([existing_path] if existing_path else [])]
        )
        smoke_env[_WINDOWS_DLL_DIRS_ENV] = pathsep.join(runtime_paths)
    return smoke_env


def _build_import_code(*, import_name: str) -> str:
    return (
        "import importlib\n"
        "import os\n"
        "import sys\n"
        "dll_directory_handles = []\n"
        "if sys.platform == 'win32':\n"
        f"    dll_dirs = os.environ.get({_WINDOWS_DLL_DIRS_ENV!r}, '')\n"
        "    add_dll_directory = getattr(os, 'add_dll_directory', None)\n"
        "    if add_dll_directory is not None:\n"
        "        for dll_dir in dll_dirs.split(os.pathsep):\n"
        "            if dll_dir:\n"
        "                dll_directory_handles.append(add_dll_directory(dll_dir))\n"
        f"importlib.import_module({import_name!r})\n"
    )


def _run_uv_smoke(
    *, python_executable: Path, artifacts: Sequence[Path], import_name: str
) -> None:
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
            _build_import_code(import_name=import_name),
        ]
    )
    subprocess.check_call(command, env=_smoke_env(env=os.environ))


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
