from __future__ import annotations

from collections.abc import Callable
import importlib.util
from pathlib import Path


def _load_smoke_env() -> Callable[..., dict[str, str]]:
    script = Path(__file__).with_name("python_package_smoke.py")
    spec = importlib.util.spec_from_file_location("python_package_smoke", script)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {script}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module._smoke_env


_smoke_env = _load_smoke_env()


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


def test_smoke_env_leaves_non_windows_path_unchanged() -> None:
    env = {
        "PATH": "/usr/bin",
        "SCIP_SYS_BUNDLED_DIR": "/opt/scip",
    }

    assert _smoke_env(env=env, platform="linux", pathsep=":") == env
