from __future__ import annotations

import importlib.util
import sys
import types
from pathlib import Path


def load_formulation() -> types.ModuleType:
    example_dir = Path(__file__).resolve().parent
    sys.path.insert(0, str(example_dir))
    sys.modules.setdefault("arco", types.SimpleNamespace())
    spec = importlib.util.spec_from_file_location(
        "reeds_benchmark_formulation", example_dir / "formulation.py"
    )
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_ru_maxrss_to_mb_uses_platform_units() -> None:
    formulation = load_formulation()

    assert formulation._ru_maxrss_to_mb(2048.0, "linux") == 2.0
    assert formulation._ru_maxrss_to_mb(2.0 * 1024.0 * 1024.0, "darwin") == 2.0


def test_format_rss_mb_reports_unsupported_measurements_explicitly() -> None:
    formulation = load_formulation()

    assert formulation._format_rss_mb(None) == "n/a"
    assert formulation._format_rss_mb(12.25) == "12.2 MB"
