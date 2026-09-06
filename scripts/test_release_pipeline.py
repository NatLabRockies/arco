from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "release_pipeline.py"
MANIFEST = ROOT / "scripts" / "fixtures" / "release_pipeline" / "dist-manifest.json"
FAKE_GH = ROOT / "scripts" / "fixtures" / "release_pipeline" / "fake_gh.py"
VERSION = "1.2.3"
TAG = f"v{VERSION}"


def digest(path: Path) -> str:
    return f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"


@pytest.fixture
def fake_gh(tmp_path: Path) -> tuple[Path, Path]:
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    log = tmp_path / "gh.log"
    gh = bin_dir / "gh"
    shutil.copy2(FAKE_GH, gh)
    gh.chmod(0o755)
    return bin_dir, log


def gh_key(*args: str) -> str:
    return json.dumps(list(args), separators=(",", ":"))


def run_cli(
    tmp_path: Path,
    fake_gh: tuple[Path, Path],
    *args: str,
    responses: dict[str, object],
    extra_env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    response_path = tmp_path / "responses.json"
    response_path.write_text(json.dumps(responses))
    output = tmp_path / "github-output"
    env = os.environ | {
        "PATH": f"{fake_gh[0]}:{os.environ['PATH']}",
        "FAKE_GH_LOG": str(fake_gh[1]),
        "FAKE_GH_RESPONSES": str(response_path),
        "GITHUB_OUTPUT": str(output),
        "GITHUB_REPOSITORY": "owner/arco",
        "GITHUB_SHA": "abc123",
        "GITHUB_RUN_ID": "42",
    }
    if extra_env:
        env.update(extra_env)
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        cwd=tmp_path,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )


def release_directory(tmp_path: Path) -> Path:
    directory = tmp_path / "release-files"
    directory.mkdir()
    shutil.copy2(MANIFEST, directory / "dist-manifest.json")
    manifest = json.loads(MANIFEST.read_text())
    for name in manifest["artifacts"]:
        (directory / name).write_bytes(name.encode())
    names = {
        f"arco-{VERSION}-cp310-cp310-manylinux_2_28_x86_64.whl",
        f"arco-{VERSION}-cp311-abi3-manylinux_2_28_x86_64.whl",
        f"arco-{VERSION}-cp310-cp310-macosx-11_0_arm64.whl",
        f"arco-{VERSION}-cp311-abi3-macosx-11_0_arm64.whl",
        f"arco-{VERSION}-cp310-cp310-win_amd64.whl",
        f"arco-{VERSION}-cp311-abi3-win_amd64.whl",
        f"arco-{VERSION}.tar.gz",
        "arco-kdl-vscode.vsix",
    }
    for name in names:
        (directory / name).write_bytes(name.encode())
    return directory


def test_resolve_reuses_only_matching_release_commit(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    (tmp_path / "Cargo.toml").write_text('[workspace.package]\nversion = "1.2.3"\n')
    result = run_cli(
        tmp_path,
        fake_gh,
        "resolve",
        responses={
            gh_key(
                "release",
                "list",
                "--repo",
                "owner/arco",
                "--limit",
                "100",
                "--json",
                "tagName,isDraft,isImmutable",
            ): [{"tagName": TAG, "isDraft": True, "isImmutable": False}],
            gh_key("api", "repos/owner/arco/commits/v1.2.3", "--jq", ".sha"): "abc123",
        },
    )
    assert result.returncode == 0, result.stderr
    assert (tmp_path / "github-output").read_text() == f"tag={TAG}\n"


def test_resolve_emits_empty_tag_for_different_commit(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    (tmp_path / "Cargo.toml").write_text('[workspace.package]\nversion = "1.2.3"\n')
    result = run_cli(
        tmp_path,
        fake_gh,
        "resolve",
        responses={
            gh_key(
                "release",
                "list",
                "--repo",
                "owner/arco",
                "--limit",
                "100",
                "--json",
                "tagName,isDraft,isImmutable",
            ): [{"tagName": TAG, "isDraft": False, "isImmutable": True}],
            gh_key("api", "repos/owner/arco/commits/v1.2.3", "--jq", ".sha"): "other",
        },
    )
    assert result.returncode == 0
    assert (tmp_path / "github-output").read_text() == "tag=\n"


def test_preflight_fails_before_any_write_when_immutability_is_disabled(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    result = run_cli(
        tmp_path,
        fake_gh,
        "preflight",
        responses={
            gh_key(
                "api", "repos/owner/arco/immutable-releases", "--jq", ".enabled"
            ): False
        },
    )
    assert result.returncode == 1
    calls = [json.loads(line) for line in fake_gh[1].read_text().splitlines()]
    assert calls == [["api", "repos/owner/arco/immutable-releases", "--jq", ".enabled"]]


def test_preflight_explains_required_administration_permission(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    result = run_cli(tmp_path, fake_gh, "preflight", responses={})
    assert result.returncode == 1
    assert "Administration read permission" in result.stderr


@pytest.mark.parametrize(
    ("artifact", "expected", "success"),
    [
        ({"name": "release-candidate", "expired": False}, "true", True),
        (None, "false", True),
        ({"name": "release-candidate", "expired": True}, "", False),
    ],
)
def test_candidate_reports_current_run_artifact(
    tmp_path: Path,
    fake_gh: tuple[Path, Path],
    artifact: dict[str, object] | None,
    expected: str,
    success: bool,
) -> None:
    artifacts = [] if artifact is None else [artifact]
    responses: dict[str, object] = {
        gh_key(
            "api",
            "--paginate",
            "--slurp",
            "repos/owner/arco/actions/runs/42/artifacts",
        ): [{"artifacts": artifacts}]
    }
    if artifact is None:
        responses.update(
            {
                gh_key(
                    "release",
                    "view",
                    TAG,
                    "--repo",
                    "owner/arco",
                    "--json",
                    "databaseId,isDraft,isImmutable",
                ): {"databaseId": 7, "isDraft": True, "isImmutable": False},
                gh_key(
                    "api", "--paginate", "--slurp", "repos/owner/arco/releases/7/assets"
                ): [[]],
            }
        )
    result = run_cli(
        tmp_path,
        fake_gh,
        "candidate",
        "--tag",
        TAG,
        responses=responses,
    )
    assert (result.returncode == 0) is success
    if success:
        assert (tmp_path / "github-output").read_text() == f"exists={expected}\n"


def test_candidate_refuses_rebuild_after_release_assets_exist(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    result = run_cli(
        tmp_path,
        fake_gh,
        "candidate",
        "--tag",
        TAG,
        responses={
            gh_key(
                "api",
                "--paginate",
                "--slurp",
                "repos/owner/arco/actions/runs/42/artifacts",
            ): [{"artifacts": []}],
            gh_key(
                "release",
                "view",
                TAG,
                "--repo",
                "owner/arco",
                "--json",
                "databaseId,isDraft,isImmutable",
            ): {"databaseId": 7, "isDraft": True, "isImmutable": False},
            gh_key(
                "api", "--paginate", "--slurp", "repos/owner/arco/releases/7/assets"
            ): [[{"name": "partial.zip", "digest": "sha256:abc"}]],
        },
    )
    assert result.returncode == 1
    assert "original release-candidate artifact is missing" in result.stderr


def test_verify_accepts_only_the_manifest_inventory(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    directory = release_directory(tmp_path)
    result = run_cli(
        tmp_path,
        fake_gh,
        "verify",
        "--tag",
        TAG,
        "--directory",
        str(directory),
        responses={},
    )
    assert result.returncode == 0, result.stderr
    (directory / "extra.bin").write_bytes(b"extra")
    result = run_cli(
        tmp_path,
        fake_gh,
        "verify",
        "--tag",
        TAG,
        "--directory",
        str(directory),
        responses={},
    )
    assert result.returncode == 1
    assert "extra.bin" in result.stderr


def test_verify_rejects_a_manifest_for_another_tag(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    directory = release_directory(tmp_path)
    manifest_path = directory / "dist-manifest.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["announcement_tag"] = "v9.9.9"
    manifest_path.write_text(json.dumps(manifest))
    result = run_cli(
        tmp_path,
        fake_gh,
        "verify",
        "--tag",
        TAG,
        "--directory",
        str(directory),
        responses={},
    )
    assert result.returncode == 1
    assert "tag does not match" in result.stderr


def test_verify_rejects_a_missing_cli_archive(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    directory = release_directory(tmp_path)
    (directory / "arco-cli-aarch64-apple-darwin.tar.gz").unlink()
    result = run_cli(
        tmp_path,
        fake_gh,
        "verify",
        "--tag",
        TAG,
        "--directory",
        str(directory),
        responses={},
    )
    assert result.returncode == 1
    assert "missing=" in result.stderr


@pytest.mark.parametrize(
    ("case", "message"),
    [
        ("digest", "different digest"),
        ("extra", "unexpected assets"),
        ("duplicate", "duplicate"),
    ],
)
def test_publish_rejects_invalid_remote_assets_before_upload(
    tmp_path: Path, fake_gh: tuple[Path, Path], case: str, message: str
) -> None:
    directory = release_directory(tmp_path)
    manifest_path = directory / "dist-manifest.json"
    remote_assets = [{"name": "dist-manifest.json", "digest": "sha256:wrong"}]
    if case == "extra":
        remote_assets = [{"name": "unexpected.bin", "digest": "sha256:abc"}]
    elif case == "duplicate":
        remote_assets = [
            {"name": "dist-manifest.json", "digest": digest(manifest_path)},
            {"name": "dist-manifest.json", "digest": digest(manifest_path)},
        ]
    result = run_cli(
        tmp_path,
        fake_gh,
        "publish",
        "--tag",
        TAG,
        "--directory",
        str(directory),
        responses={
            gh_key(
                "api", "repos/owner/arco/immutable-releases", "--jq", ".enabled"
            ): True,
            gh_key(
                "release",
                "view",
                TAG,
                "--repo",
                "owner/arco",
                "--json",
                "databaseId,isDraft,isImmutable",
            ): {"databaseId": 7, "isDraft": True, "isImmutable": False},
            gh_key("api", "repos/owner/arco/commits/v1.2.3", "--jq", ".sha"): "abc123",
            gh_key(
                "api", "--paginate", "--slurp", "repos/owner/arco/releases/7/assets"
            ): [remote_assets],
        },
    )
    assert result.returncode == 1
    assert message in result.stderr
    assert digest(manifest_path) != "sha256:wrong"
    assert "upload" not in fake_gh[1].read_text()


def test_publish_disabled_immutability_performs_no_release_mutation(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    directory = release_directory(tmp_path)
    result = run_cli(
        tmp_path,
        fake_gh,
        "publish",
        "--tag",
        TAG,
        "--directory",
        str(directory),
        responses={
            gh_key(
                "api", "repos/owner/arco/immutable-releases", "--jq", ".enabled"
            ): False
        },
    )
    assert result.returncode == 1
    calls = [json.loads(line) for line in fake_gh[1].read_text().splitlines()]
    assert calls == [["api", "repos/owner/arco/immutable-releases", "--jq", ".enabled"]]


def test_publish_uploads_missing_draft_assets_then_requires_immutability(
    tmp_path: Path, fake_gh: tuple[Path, Path]
) -> None:
    directory = release_directory(tmp_path)
    names = sorted(path.name for path in directory.iterdir())
    assets = [{"name": name, "digest": digest(directory / name)} for name in names]
    view_key = gh_key(
        "release",
        "view",
        TAG,
        "--repo",
        "owner/arco",
        "--json",
        "databaseId,isDraft,isImmutable",
    )
    assets_key = gh_key(
        "api", "--paginate", "--slurp", "repos/owner/arco/releases/7/assets"
    )
    upload_args = ["release", "upload", TAG]
    upload_args.extend(str(directory / name) for name in names)
    upload_args.extend(("--repo", "owner/arco"))
    result = run_cli(
        tmp_path,
        fake_gh,
        "publish",
        "--tag",
        TAG,
        "--directory",
        str(directory),
        responses={
            gh_key(
                "api", "repos/owner/arco/immutable-releases", "--jq", ".enabled"
            ): True,
            view_key: [
                {
                    "_response": {
                        "databaseId": 7,
                        "isDraft": True,
                        "isImmutable": False,
                    }
                },
                {
                    "_response": {
                        "databaseId": 7,
                        "isDraft": False,
                        "isImmutable": True,
                    }
                },
            ],
            gh_key("api", "repos/owner/arco/commits/v1.2.3", "--jq", ".sha"): "abc123",
            assets_key: [
                {"_response": [[]]},
                {"_response": [assets]},
            ],
            gh_key(*upload_args): "",
            gh_key("release", "edit", TAG, "--repo", "owner/arco", "--draft=false"): "",
        },
    )
    assert result.returncode == 0, result.stderr
    calls = [json.loads(line) for line in fake_gh[1].read_text().splitlines()]
    assert upload_args in calls


@pytest.mark.parametrize("corrupt_digest", [False, True])
def test_download_python_uses_only_immutable_assets_and_checks_digests(
    tmp_path: Path, fake_gh: tuple[Path, Path], corrupt_digest: bool
) -> None:
    release_files = release_directory(tmp_path)
    python_names = {
        path.name
        for path in release_files.iterdir()
        if path.name.startswith(f"arco-{VERSION}")
    }
    assets = [
        {"name": name, "digest": digest(release_files / name)}
        for name in sorted(python_names)
    ]
    if corrupt_digest:
        assets[0]["digest"] = "sha256:wrong"
    destination = tmp_path / "dist"
    download_args = [
        "release",
        "download",
        TAG,
        "--repo",
        "owner/arco",
        "--dir",
        str(destination),
    ]
    for name in sorted(python_names):
        download_args.extend(("--pattern", name))
    result = run_cli(
        tmp_path,
        fake_gh,
        "download-python",
        "--tag",
        TAG,
        "--directory",
        str(destination),
        responses={
            gh_key(
                "release",
                "view",
                TAG,
                "--repo",
                "owner/arco",
                "--json",
                "databaseId,isDraft,isImmutable",
            ): {"databaseId": 7, "isDraft": False, "isImmutable": True},
            gh_key(
                "api", "--paginate", "--slurp", "repos/owner/arco/releases/7/assets"
            ): [assets],
            gh_key(*download_args): "",
        },
        extra_env={"FAKE_GH_DOWNLOADS": str(release_files)},
    )
    if corrupt_digest:
        assert result.returncode == 1
        assert "different digest" in result.stderr
    else:
        assert result.returncode == 0, result.stderr
        assert {path.name for path in destination.iterdir()} == python_names
