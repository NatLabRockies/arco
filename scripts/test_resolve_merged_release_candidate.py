from __future__ import annotations

import os
import stat
import subprocess
from pathlib import Path

import yaml

_SCRIPT = Path(__file__).with_name("resolve_merged_release_candidate.sh")
_WORKFLOW = (
    _SCRIPT.parent.parent / ".github" / "workflows" / "publish-merged-release.yml"
)
_FIXTURE = Path(__file__).parent / "fixtures" / "merged-release-candidate.json"
_MERGE_COMMIT = "0712d9dffa2647bf7a24dde88b7276b455c38520"
_CANDIDATE_COMMIT = "8375324f1d12ec20306ea919f73b9a1e746894ea"


def _fake_gh(tmp_path: Path) -> Path:
    executable = tmp_path / "gh"
    executable.write_text(
        """#!/usr/bin/env python3
import copy
import json
import os
from pathlib import Path
import sys

endpoint = next(argument for argument in sys.argv[1:] if argument.startswith("repos/"))
endpoint = endpoint.partition("?")[0]
fixture = json.loads(Path(os.environ["GH_FIXTURE"]).read_text(encoding="utf-8"))
mapping = {
    "repos/NatLabRockies/arco/pulls/422": "pull_request",
    f"repos/NatLabRockies/arco/commits/{os.environ['CANDIDATE_COMMIT']}/check-runs": "check_runs",
    "repos/NatLabRockies/arco/actions/jobs/102176607730": "job",
    "repos/NatLabRockies/arco/actions/runs/34258254258": "run",
    "repos/NatLabRockies/arco/actions/workflows/351832674": "workflow",
}
key = mapping.get(endpoint)
if key is None:
    raise SystemExit(f"unexpected endpoint: {endpoint}")
payload = copy.deepcopy(fixture[key])
if key == "pull_request" and os.environ["PR_MODE"] == "ordinary":
    payload["user"]["login"] = "pesap"
    payload["head"]["ref"] = "fix/ordinary-change"
    payload["labels"] = []
if key == "check_runs" and os.environ["CHECKS_MODE"] == "failed":
    payload["check_runs"][0]["conclusion"] = "failure"
if key == "check_runs" and os.environ["CHECKS_MODE"] == "duplicate":
    duplicate = copy.deepcopy(payload["check_runs"][0])
    duplicate["id"] += 1
    payload["check_runs"].append(duplicate)
    payload["total_count"] = 2
if "--jq" in sys.argv:
    expression = sys.argv[sys.argv.index("--jq") + 1]
    if expression != ".path":
        raise SystemExit(f"unexpected jq expression: {expression}")
    print(payload["path"])
else:
    print(json.dumps(payload))
""",
        encoding="utf-8",
    )
    executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
    return executable


def _run_resolver(
    tmp_path: Path,
    *,
    pr_mode: str = "release",
    checks_mode: str = "successful",
) -> tuple[subprocess.CompletedProcess[str], dict[str, str]]:
    _fake_gh(tmp_path)
    output = tmp_path / "github-output"
    env = os.environ.copy()
    env.update(
        {
            "CANDIDATE_COMMIT": _CANDIDATE_COMMIT,
            "CHECKS_MODE": checks_mode,
            "GH_FIXTURE": str(_FIXTURE),
            "PATH": f"{tmp_path}{os.pathsep}{env['PATH']}",
            "PR_MODE": pr_mode,
        }
    )
    result = subprocess.run(
        [
            "bash",
            str(_SCRIPT),
            "NatLabRockies/arco",
            "422",
            _MERGE_COMMIT,
            "main",
            str(output),
        ],
        check=False,
        capture_output=True,
        env=env,
        text=True,
    )
    values = {}
    if output.is_file():
        values = dict(line.split("=", 1) for line in output.read_text().splitlines())
    return result, values


def test_release_publication_runs_for_merged_pull_requests() -> None:
    workflow = yaml.load(_WORKFLOW.read_text(encoding="utf-8"), Loader=yaml.BaseLoader)

    assert workflow["on"] == {
        "pull_request": {
            "types": ["closed"],
            "branches": ["main", "release/*"],
        }
    }
    assert (
        "github.event.pull_request.merged == true" in workflow["jobs"]["discover"]["if"]
    )


def test_merged_release_pr_resolves_its_successful_candidate_run(
    tmp_path: Path,
) -> None:
    result, values = _run_resolver(tmp_path)

    assert result.returncode == 0, result.stderr
    assert values == {
        "release": "true",
        "candidate_run": "34258254258",
        "pr": "422",
        "candidate_commit": _CANDIDATE_COMMIT,
        "candidate_base": "6c05293d23f7e6d7c364b4ba12417fb0c956fc9f",
        "base_branch": "main",
        "merge_commit": _MERGE_COMMIT,
    }


def test_non_release_merge_is_ignored(tmp_path: Path) -> None:
    result, values = _run_resolver(tmp_path, pr_mode="ordinary")

    assert result.returncode == 0, result.stderr
    assert values == {"release": "false"}


def test_merged_release_pr_requires_a_successful_candidate_check(
    tmp_path: Path,
) -> None:
    result, _ = _run_resolver(tmp_path, checks_mode="failed")

    assert result.returncode != 0
    assert "successful candidate check" in result.stderr


def test_merged_release_pr_requires_one_successful_candidate_check(
    tmp_path: Path,
) -> None:
    result, _ = _run_resolver(tmp_path, checks_mode="duplicate")

    assert result.returncode != 0
    assert "successful candidate check" in result.stderr
