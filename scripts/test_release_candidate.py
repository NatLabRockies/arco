from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = Path(__file__).with_name("fixtures") / "release-candidate"


@pytest.fixture
def github(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    for name in ("pr.json", "run.json"):
        shutil.copyfile(
            FIXTURES / name, tmp_path / ("api-pr.json" if name == "pr.json" else name)
        )
    command = tmp_path / "gh"
    command.write_text(
        """#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  'api repos/example/arco/pulls/381') cat api-pr.json ;;
  'api repos/example/arco/commits/main --jq .sha') printf '%s\\n' "$LIVE_BASE" ;;
  'api repos/example/arco/compare/'*) printf '%s\\n' "$COMPARISON" ;;
  'api repos/example/arco/git/commits/'*) printf '%s\\n' "$TREE_SHA" ;;
  *'pulls?state=open&base=main&per_page=100'*) printf '381\\n' ;;
  *'pulls?state=closed&base=main&per_page=100'*) exit 0 ;;
  'api repos/example/arco/actions/runs/123') cat run-fixture.json ;;
  'api repos/example/arco/actions/workflows/42 --jq .path')
    printf '%s\\n' '.github/workflows/build-candidate.yml' ;;
  *) printf 'Unexpected GitHub request: %s\\n' "$*" >&2; exit 99 ;;
esac
""",
        encoding="utf-8",
    )
    command.chmod(0o755)
    env = {
        "PATH": f"{tmp_path}{os.pathsep}{os.environ['PATH']}",
        "BASE_BRANCH": "main",
        "BASE_SHA": "a" * 40,
        "HEAD_BRANCH": "release-please--branches--main",
        "HEAD_SHA": "b" * 40,
        "PR_NUMBER": "381",
        "REPOSITORY": "example/arco",
        "GH_REPO": "example/arco",
        "CANDIDATE_RUN": "123",
        "WORKFLOW_BRANCH": "main",
        "LIVE_BASE": "a" * 40,
        "TREE_SHA": "c" * 40,
        "COMPARISON": "ahead",
        "GITHUB_OUTPUT": str(tmp_path / "outputs"),
    }
    for key, value in env.items():
        monkeypatch.setenv(key, value)
    return tmp_path


def run_step(
    workflow: str, *, job: str, step: int, cwd: Path
) -> subprocess.CompletedProcess[str]:
    definition = yaml.safe_load((ROOT / ".github/workflows" / workflow).read_text())
    return subprocess.run(
        ["bash", "-c", definition["jobs"][job]["steps"][step]["run"]],
        cwd=cwd,
        text=True,
        capture_output=True,
        check=False,
    )


def test_candidate_uses_approved_pr_head_instead_of_merge_sha(
    github: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("GITHUB_SHA", "d" * 40)
    result = run_step("build-candidate.yml", job="plan", step=0, cwd=github)
    assert result.returncode == 0, result.stderr
    outputs = (github / "outputs").read_text().splitlines()
    assert f"commit={'b' * 40}" in outputs
    assert f"tree={'c' * 40}" in outputs


@pytest.mark.parametrize("side", ["head", "base"])
def test_approval_cannot_build_a_pr_updated_since_the_event(
    github: Path, side: str
) -> None:
    path = github / "api-pr.json"
    pr = json.loads(path.read_text())
    pr[side]["sha"] = "e" * 40
    path.write_text(json.dumps(pr))
    result = run_step("build-candidate.yml", job="plan", step=0, cwd=github)
    assert result.returncode == 1
    assert "changed after" in result.stderr
    assert not (github / "outputs").exists()


def test_approval_cannot_build_after_base_branch_advances(
    github: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("LIVE_BASE", "e" * 40)
    result = run_step("build-candidate.yml", job="plan", step=0, cwd=github)
    assert result.returncode == 1
    assert "base branch changed" in result.stderr


@pytest.mark.parametrize(
    ("event", "conclusion", "exit_code"),
    [
        ("pull_request", "success", 0),
        ("workflow_dispatch", "success", 1),
        ("pull_request", "failure", 1),
    ],
)
def test_promotion_accepts_only_successful_pr_candidate_runs(
    github: Path, event: str, conclusion: str, exit_code: int
) -> None:
    run = json.loads((github / "run.json").read_text())
    run["event"] = event
    run["conclusion"] = conclusion
    (github / "run-fixture.json").write_text(json.dumps(run))
    result = run_step("promote-release.yml", job="verify-candidate", step=0, cwd=github)
    assert result.returncode == exit_code, result.stderr


@pytest.mark.parametrize("mismatch", [None, "head_sha", "head_branch", "pr_number"])
def test_promotion_binds_candidate_to_original_run_and_release_pr(
    github: Path, mismatch: str | None
) -> None:
    candidate = github / "candidate"
    candidate.mkdir()
    shutil.copyfile(FIXTURES / "candidate.json", candidate / "candidate.json")
    native = {f"native-{index}.tar.gz": {} for index in range(15)}
    (candidate / "dist-manifest.json").write_text(
        json.dumps({"announcement_tag": "v0.12.0", "artifacts": native})
    )
    for name in native:
        (candidate / name).touch()
    for abi in ("cp310-cp310", "cp311-abi3"):
        for platform in ("manylinux_2_28_x86_64", "macosx_11_0_arm64", "win_amd64"):
            (candidate / f"arco-0.12.0-{abi}-{platform}.whl").touch()
    (candidate / "arco-0.12.0.tar.gz").touch()
    (candidate / "arco-kdl-vscode.vsix").touch()

    run = json.loads((github / "run.json").read_text())
    # GitHub refreshes these nested SHAs when the PR changes, even for old runs.
    run["pull_requests"][0].update(
        {"head": {"sha": "e" * 40}, "base": {"sha": "f" * 40}}
    )
    if mismatch == "pr_number":
        run["pull_requests"][0]["number"] = 382
    elif mismatch == "head_sha":
        run["head_sha"] = "a" * 40
    elif mismatch == "head_branch":
        run["head_branch"] = "another-release-pr"
    (github / "run.json").write_text(json.dumps(run))

    result = run_step("promote-release.yml", job="verify-candidate", step=2, cwd=github)
    assert result.returncode == (0 if mismatch is None else 1), result.stderr
    if mismatch is None:
        assert f"commit={'b' * 40}" in (github / "outputs").read_text().splitlines()
    else:
        assert not (github / "outputs").exists()
