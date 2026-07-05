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


def test_benchmark_environment_prepends_binary_directory(
    tmp_path: Path, monkeypatch
) -> None:
    bench = load_bench_module()
    binary_dir = tmp_path / "bin"
    binary_dir.mkdir()
    binary = binary_dir / "arco"
    binary.write_text("")
    binary.chmod(0o755)
    monkeypatch.setenv("LD_LIBRARY_PATH", "/existing/lib")
    monkeypatch.delenv("DYLD_LIBRARY_PATH", raising=False)

    env = bench.benchmark_environment(
        arco_binary=str(binary), config_root=tmp_path / "config"
    )

    assert env["LD_LIBRARY_PATH"].split(os.pathsep)[:2] == [
        str(binary_dir),
        "/existing/lib",
    ]
    assert env["DYLD_LIBRARY_PATH"] == str(binary_dir)


def test_shell_environment_prefix_quotes_runtime_assignments(tmp_path: Path) -> None:
    bench = load_bench_module()
    env = {
        "ARCO_CONFIG_DIR": str(tmp_path / "user config"),
        "ARCO_PROJECT_CONFIG_DIR": str(tmp_path / "project config"),
        "LD_LIBRARY_PATH": str(tmp_path / "lib dir"),
    }

    prefix = bench.shell_environment_prefix(env)

    assert f"ARCO_CONFIG_DIR='{tmp_path / 'user config'}'" in prefix
    assert f"ARCO_PROJECT_CONFIG_DIR='{tmp_path / 'project config'}'" in prefix
    assert f"LD_LIBRARY_PATH='{tmp_path / 'lib dir'}'" in prefix


def test_parse_torc_results_items_accepts_current_and_legacy_shapes() -> None:
    bench = load_bench_module()
    items = [{"exec_time_minutes": 0.1}]

    assert bench.parse_torc_results_items({"items": items}) == items
    assert bench.parse_torc_results_items({"results": items}) == items
    assert bench.parse_torc_results_items(items) == items
    assert bench.parse_torc_results_items({"data": items}) is None
