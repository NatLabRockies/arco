from __future__ import annotations

import os
import stat
import subprocess
from pathlib import Path

import pytest

SCRIPT = Path(__file__).with_name("check-kdl-overlay.sh")


@pytest.fixture
def overlay_repo(tmp_path: Path) -> tuple[Path, Path]:
    repo = tmp_path / "repo"
    grammar = repo / "tools" / "tree-sitter-arco-kdl"
    grammar.mkdir(parents=True)
    (grammar / "grammar.js").write_text("module.exports = grammar({});\n")
    subprocess.run(["git", "init", "--quiet", repo], check=True)

    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    parser = bin_dir / "tree-sitter"
    parser.write_text(
        """#!/usr/bin/env python3
import os
from pathlib import Path
import sys

Path(os.environ["PARSER_LOG"]).write_text(
    f"cwd={Path.cwd()}\\nargs={sys.argv[1:]!r}\\n"
)
mode = os.environ["PARSER_MODE"]
if mode == "crash":
    print("parser crashed", file=sys.stderr)
    raise SystemExit(2)
if mode == "syntax-error":
    print("(source_file (ERROR))")
    raise SystemExit(1)
else:
    print("(source_file)")
"""
    )
    parser.chmod(parser.stat().st_mode | stat.S_IXUSR)
    return repo, bin_dir


def run_check(
    repo: Path, bin_dir: Path, *, mode: str
) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.update(
        {
            "PARSER_LOG": str(repo.parent / "parser.log"),
            "PARSER_MODE": mode,
            "PATH": f"{bin_dir}{os.pathsep}{env['PATH']}",
        }
    )
    return subprocess.run(
        [SCRIPT], cwd=repo, env=env, text=True, capture_output=True, check=False
    )


def add_tracked_file(repo: Path, relative_path: str) -> Path:
    path = repo / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("model {}\n")
    subprocess.run(["git", "-C", repo, "add", relative_path], check=True)
    return path


def test_parser_command_failure_cannot_pass_with_empty_stdout(
    overlay_repo: tuple[Path, Path],
) -> None:
    repo, bin_dir = overlay_repo
    add_tracked_file(repo, "valid.kdl")

    result = run_check(repo, bin_dir, mode="crash")

    assert result.returncode != 0
    assert "parser crashed" in result.stderr
    assert "valid.kdl" in result.stderr


def test_parser_error_node_fails_the_check(
    overlay_repo: tuple[Path, Path],
) -> None:
    repo, bin_dir = overlay_repo
    add_tracked_file(repo, "invalid.kdl")

    result = run_check(repo, bin_dir, mode="syntax-error")

    assert result.returncode != 0
    assert "invalid.kdl" in result.stderr
    assert "(ERROR" in result.stdout


def test_only_tracked_non_rejected_files_are_parsed_nul_safely(
    overlay_repo: tuple[Path, Path],
) -> None:
    repo, bin_dir = overlay_repo
    tracked = add_tracked_file(repo, "fixtures/file with spaces.kdl")
    add_tracked_file(repo, "crates/arco-kdl/tests/fixtures/rejects_example.kdl")
    (repo / "untracked.kdl").write_text("invalid {}\n")

    result = run_check(repo, bin_dir, mode="valid")

    assert result.returncode == 0, result.stderr
    assert "checked=1 skipped=1 failures=0" in result.stdout
    parser_log = (repo.parent / "parser.log").read_text()
    assert f"cwd={repo / 'tools/tree-sitter-arco-kdl'}" in parser_log
    assert repr(str(tracked)) in parser_log
    assert "untracked.kdl" not in parser_log
    assert "'--quiet'" in parser_log
    assert "'-p'" not in parser_log
