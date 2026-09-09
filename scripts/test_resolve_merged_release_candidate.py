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
_FIXTURES = Path(__file__).parent / "fixtures" / "merged-release-candidate"
_MERGE_COMMIT = "0712d9dffa2647bf7a24dde88b7276b455c38520"
_CANDIDATE_COMMIT = "8375324f1d12ec20306ea919f73b9a1e746894ea"


def _fake_gh(tmp_path: Path) -> Path:
    executable = tmp_path / "gh"
    executable.write_text(
        """#!/usr/bin/env python3
import os
from pathlib import Path
import sys

endpoint = next(argument for argument in sys.argv[1:] if argument.startswith("repos/"))
endpoint = endpoint.partition("?")[0]
fixtures = Path(os.environ["GH_FIXTURES"])
mapping = {
    "repos/NatLabRockies/arco/pulls/422": os.environ["PR_FIXTURE"],
    f"repos/NatLabRockies/arco/commits/{os.environ['CANDIDATE_COMMIT']}/check-runs": os.environ["CHECKS_FIXTURE"],
    "repos/NatLabRockies/arco/actions/jobs/102176607730": "successful-job.json",
    "repos/NatLabRockies/arco/actions/runs/34258254258": "successful-run.json",
    "repos/NatLabRockies/arco/actions/workflows/351832674": "candidate-workflow.json",
}
path = mapping.get(endpoint)
if path is None:
    raise SystemExit(f"unexpected endpoint: {endpoint}")
content = (fixtures / path).read_text(encoding="utf-8")
if "--jq" in sys.argv:
    expression = sys.argv[sys.argv.index("--jq") + 1]
    if expression == ".path":
        import json

        print(json.loads(content)["path"])
    else:
        raise SystemExit(f"unexpected jq expression: {expression}")
else:
    print(content)
""",
        encoding="utf-8",
    )
    executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
    return executable


def _run_resolver(
    tmp_path: Path,
    *,
    pr_fixture: str = "release-pr.json",
    checks_fixture: str = "successful-check-runs.json",
) -> tuple[subprocess.CompletedProcess[str], dict[str, str]]:
    _fake_gh(tmp_path)
    output = tmp_path / "github-output"
    env = os.environ.copy()
    env.update(
        {
            "CANDIDATE_COMMIT": _CANDIDATE_COMMIT,
            "CHECKS_FIXTURE": checks_fixture,
            "GH_FIXTURES": str(_FIXTURES),
            "MERGE_COMMIT": _MERGE_COMMIT,
            "PATH": f"{tmp_path}{os.pathsep}{env['PATH']}",
            "PR_FIXTURE": pr_fixture,
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
    result, values = _run_resolver(tmp_path, pr_fixture="ordinary-pr.json")

    assert result.returncode == 0, result.stderr
    assert values == {"release": "false"}


def test_merged_release_pr_requires_a_successful_candidate_check(
    tmp_path: Path,
) -> None:
    result, _ = _run_resolver(tmp_path, checks_fixture="failed-check-runs.json")

    assert result.returncode != 0
    assert "successful candidate check" in result.stderr


def test_merged_release_pr_requires_one_successful_candidate_check(
    tmp_path: Path,
) -> None:
    result, _ = _run_resolver(tmp_path, checks_fixture="duplicate-check-runs.json")

    assert result.returncode != 0
    assert "successful candidate check" in result.stderr
