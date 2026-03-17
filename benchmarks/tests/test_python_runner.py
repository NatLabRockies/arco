from __future__ import annotations

from types import SimpleNamespace

import pytest

from arco_benchmarks import python_runner


def test_read_peak_rss_bytes_linux_scales_kib(monkeypatch) -> None:
    monkeypatch.setattr(
        python_runner.resource,
        "getrusage",
        lambda who: SimpleNamespace(ru_maxrss=1234),
    )
    monkeypatch.setattr(python_runner.sys, "platform", "linux")

    assert python_runner._read_peak_rss_bytes() == 1234 * 1024


def test_run_point_reports_peak_rss_delta(monkeypatch) -> None:
    rss_values = iter([1_000_000, 1_750_000])
    clock_values = iter([10.0, 10.25])
    called: dict[str, int] = {}

    def _fake_build_arco(n: int) -> object:
        called["n"] = n
        return object()

    monkeypatch.setattr(
        python_runner,
        "_read_peak_rss_bytes",
        lambda: next(rss_values),
    )
    monkeypatch.setattr(python_runner, "perf_counter", lambda: next(clock_values))
    monkeypatch.setattr(python_runner, "_build_arco", _fake_build_arco)

    result = python_runner.run_point(tool="arco", phase="build", n=10, solver="highs")

    assert called["n"] == 10
    assert result["elapsed_seconds"] == 0.25
    assert result["peak_rss_bytes"] == 1_750_000
    assert result["peak_rss_delta_bytes"] == 750_000


def test_run_point_materialize_phase_calls_materializer(monkeypatch) -> None:
    rss_values = iter([2_000_000, 2_200_000])
    clock_values = iter([5.0, 5.4])
    calls: dict[str, bool] = {"materialize": False, "solve": False}

    monkeypatch.setattr(
        python_runner,
        "_read_peak_rss_bytes",
        lambda: next(rss_values),
    )
    monkeypatch.setattr(python_runner, "perf_counter", lambda: next(clock_values))
    monkeypatch.setattr(python_runner, "_build_arco", lambda n: object())
    monkeypatch.setattr(
        python_runner,
        "_materialize_arco",
        lambda model: calls.__setitem__("materialize", True),
        raising=False,
    )
    monkeypatch.setattr(
        python_runner,
        "_solve_arco",
        lambda model, solver: calls.__setitem__("solve", True),
    )

    result = python_runner.run_point(
        tool="arco",
        phase="materialize",
        n=10,
        solver="highs",
    )

    assert calls["materialize"] is True
    assert calls["solve"] is False
    assert result["elapsed_seconds"] == pytest.approx(0.4)
