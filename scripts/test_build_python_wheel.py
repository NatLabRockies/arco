from __future__ import annotations

import os
from pathlib import Path
import stat
import subprocess
import sys

import pytest


_SCRIPT = Path(__file__).with_name("build_python_wheel.sh")


@pytest.mark.skipif(sys.platform == "win32", reason="requires Bash")
@pytest.mark.parametrize(
    ("compatibility", "expected_compatibility"),
    [("manylinux_2_28", "manylinux_2_28"), (None, "pypi")],
)
def test_build_wheel_passes_compatibility_to_maturin(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    compatibility: str | None,
    expected_compatibility: str,
) -> None:
    args_file = tmp_path / "uv-args"
    fake_uv = tmp_path / "uv"
    fake_uv.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        'printf "%s\\0" "$@" > "$UV_ARGS_FILE"\n',
        encoding="utf-8",
    )
    fake_uv.chmod(fake_uv.stat().st_mode | stat.S_IXUSR)

    monkeypatch.setenv("UV_ARGS_FILE", str(args_file))
    monkeypatch.setenv("PATH", f"{tmp_path}{os.pathsep}{os.environ['PATH']}")
    if compatibility is None:
        monkeypatch.delenv("PYTHON_WHEEL_COMPATIBILITY", raising=False)
    else:
        monkeypatch.setenv("PYTHON_WHEEL_COMPATIBILITY", compatibility)

    subprocess.run(["bash", str(_SCRIPT)], check=True, cwd=_SCRIPT.parent.parent)

    args = args_file.read_bytes().rstrip(b"\0").decode("utf-8").split("\0")
    compatibility_index = args.index("--compatibility")
    assert args[compatibility_index + 1] == expected_compatibility
