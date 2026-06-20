from __future__ import annotations

import importlib.util
import os
import sys
from pathlib import Path


def load_bench_module():
    module_path = Path(__file__).with_name("bench.py")
    spec = importlib.util.spec_from_file_location("arco_bench", module_path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load {module_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_isolated_solver_config_environment_uses_empty_temp_dirs(
    tmp_path: Path, monkeypatch
) -> None:
    bench = load_bench_module()
    monkeypatch.setenv("ARCO_CONFIG_DIR", "/home/example/.config/arco")
    monkeypatch.setenv("ARCO_PROJECT_CONFIG_DIR", "/repo/.arco")
    monkeypatch.setenv("ARCO_KEEP_ME", "1")

    env = bench.isolated_solver_config_environment(tmp_path)

    user_config = tmp_path / "user"
    project_config = tmp_path / "project"
    assert env["ARCO_CONFIG_DIR"] == str(user_config)
    assert env["ARCO_PROJECT_CONFIG_DIR"] == str(project_config)
    assert env["ARCO_KEEP_ME"] == "1"
    assert user_config.is_dir()
    assert project_config.is_dir()
    assert os.environ["ARCO_CONFIG_DIR"] == "/home/example/.config/arco"
