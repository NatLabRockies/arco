from __future__ import annotations

from collections.abc import Callable
import importlib.util
import os
from pathlib import Path
import sys
from types import ModuleType


def _load_smoke_module() -> ModuleType:
    script = Path(__file__).with_name("python_package_smoke.py")
    spec = importlib.util.spec_from_file_location("python_package_smoke", script)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {script}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_smoke_module = _load_smoke_module()
_smoke_env: Callable[..., dict[str, str]] = _smoke_module._smoke_env


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


def test_import_code_adds_windows_dll_directories(monkeypatch) -> None:
    added_paths: list[str] = []
    handles: list[object] = []

    def add_dll_directory(path: str) -> object:
        handle = object()
        added_paths.append(path)
        handles.append(handle)
        return handle

    monkeypatch.setattr(sys, "platform", "win32")
    monkeypatch.setattr(os, "add_dll_directory", add_dll_directory, raising=False)
    monkeypatch.setattr(os, "pathsep", ";")
    monkeypatch.setenv("ARCO_PYTHON_SMOKE_DLL_DIRS", "C:/solver/bin;C:/solver/lib")

    exec(_smoke_module._build_import_code(import_name="json"), {})

    assert added_paths == ["C:/solver/bin", "C:/solver/lib"]
    assert len(handles) == 2
