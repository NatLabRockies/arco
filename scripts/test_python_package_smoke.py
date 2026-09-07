from __future__ import annotations

import os
from pathlib import Path
import runpy
import sys

import pytest


_smoke_module = runpy.run_path(str(Path(__file__).with_name("python_package_smoke.py")))
_smoke_env = _smoke_module["_smoke_env"]
_build_import_code = _smoke_module["_build_import_code"]


def test_smoke_env_prepends_windows_solver_runtime_paths() -> None:
    env = {
        "PATH": "C:/Windows/System32",
        "SCIP_SYS_BUNDLED_DIR": "/c/solver/scip_install",
        "ARCO_SCIP_LIBRARY_PATH": "/c/solver/scip_install/lib",
        "LD_LIBRARY_PATH": "/c/solver/scip_install/lib:/c/solver/scip_install/bin",
        "XPRESSDIR": "D:/xpress",
    }

    smoke_env = _smoke_env(env=env, platform="win32", pathsep=";")

    path_entries = smoke_env["PATH"].split(";")
    assert path_entries[:4] == [
        "C:/solver/scip_install/bin",
        "C:/solver/scip_install/lib",
        "D:/xpress/bin",
        "D:/xpress/lib",
    ]
    assert path_entries[-1] == "C:/Windows/System32"
    assert smoke_env["ARCO_PYTHON_SMOKE_DLL_DIRS"].split(";") == path_entries[:4]


def test_smoke_env_leaves_non_windows_path_unchanged() -> None:
    env = {
        "PATH": "/usr/bin",
        "SCIP_SYS_BUNDLED_DIR": "/opt/scip",
    }

    assert _smoke_env(env=env, platform="linux", pathsep=":") == env


def test_import_code_adds_existing_windows_dll_directories(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    added_paths: list[str] = []
    handles: list[object] = []
    runtime_bin = tmp_path / "bin"
    runtime_bin.mkdir()
    runtime_lib = tmp_path / "lib"

    def add_dll_directory(path: str) -> object:
        if not Path(path).is_dir():
            raise FileNotFoundError(path)
        handle = object()
        added_paths.append(path)
        handles.append(handle)
        return handle

    monkeypatch.setattr(sys, "platform", "win32")
    monkeypatch.setattr(os, "add_dll_directory", add_dll_directory, raising=False)
    monkeypatch.setattr(os, "pathsep", ";")
    monkeypatch.setenv("ARCO_PYTHON_SMOKE_DLL_DIRS", f"{runtime_bin};{runtime_lib}")

    exec(_build_import_code(import_name="json"), {})

    assert added_paths == [str(runtime_bin)]
    assert len(handles) == 1
