#!/usr/bin/env python3
"""Resolve, verify, and publish immutable Arco release candidates."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import NoReturn

import tomllib

type JsonValue = (
    None | bool | int | float | str | list[JsonValue] | dict[str, JsonValue]
)
type JsonObject = dict[str, JsonValue]

TARGETS = {
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
}


class ReleaseError(RuntimeError):
    """A release invariant was not satisfied."""


def fail(message: str) -> NoReturn:
    raise ReleaseError(message)


def require_env(name: str) -> str:
    value = os.environ.get(name, "")
    if not value:
        fail(f"{name} is required")
    return value


def run_gh(*args: str) -> str:
    completed = subprocess.run(
        ["gh", *args], check=True, capture_output=True, text=True, timeout=240
    )
    return completed.stdout


def gh_json(*args: str) -> JsonValue:
    output = run_gh(*args)
    try:
        return json.loads(output)
    except json.JSONDecodeError as error:
        fail(f"gh returned invalid JSON: {error}")


def write_output(name: str, value: str) -> None:
    output_path = Path(require_env("GITHUB_OUTPUT"))
    with output_path.open("a", encoding="utf-8") as output:
        output.write(f"{name}={value}\n")


def repository_version() -> str:
    manifest = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
    version = manifest.get("workspace", {}).get("package", {}).get("version")
    if not isinstance(version, str) or not version:
        fail("Cargo.toml workspace.package.version is missing")
    return version


def tag_commit(repository: str, tag: str) -> str:
    return run_gh("api", f"repos/{repository}/commits/{tag}", "--jq", ".sha").strip()


def resolve() -> None:
    repository = require_env("GITHUB_REPOSITORY")
    expected_sha = require_env("GITHUB_SHA")
    tag = f"v{repository_version()}"
    validate_tag(tag)
    releases = gh_json(
        "release",
        "list",
        "--repo",
        repository,
        "--limit",
        "100",
        "--json",
        "tagName,isDraft,isImmutable",
    )
    if not isinstance(releases, list):
        fail("gh release list returned a malformed response")
    matches: list[JsonObject] = []
    for release in releases:
        if not isinstance(release, dict):
            fail("gh release list returned a malformed release")
        if release.get("tagName") == tag:
            matches.append(release)
    reusable = [
        release
        for release in matches
        if release.get("isDraft") is True or release.get("isImmutable") is True
    ]
    if len(reusable) > 1:
        fail(f"multiple reusable releases found for {tag}")
    resolved = tag if reusable and tag_commit(repository, tag) == expected_sha else ""
    write_output("tag", resolved)


def preflight() -> None:
    repository = require_env("GITHUB_REPOSITORY")
    try:
        enabled = gh_json(
            "api", f"repos/{repository}/immutable-releases", "--jq", ".enabled"
        )
    except subprocess.CalledProcessError:
        fail(
            "could not read immutable release settings; RELEASE_PLEASE_TOKEN "
            "needs Administration read permission"
        )
    if enabled is not True:
        fail("GitHub Immutable Releases must be enabled before release writes")


def flatten_pages(value: JsonValue) -> list[JsonObject]:
    if not isinstance(value, list):
        fail("expected a paginated JSON array from gh")
    flattened: list[JsonObject] = []
    for item in value:
        if isinstance(item, list):
            for page in item:
                if not isinstance(page, dict):
                    fail("paginated gh response contains a malformed page")
                flattened.append(page)
        elif isinstance(item, dict):
            flattened.append(item)
        else:
            fail("paginated gh response contains a malformed item")
    return flattened


def candidate(tag: str) -> None:
    validate_tag(tag)
    repository = require_env("GITHUB_REPOSITORY")
    run_id = require_env("GITHUB_RUN_ID")
    response = gh_json(
        "api",
        "--paginate",
        "--slurp",
        f"repos/{repository}/actions/runs/{run_id}/artifacts",
    )
    pages = flatten_pages(response)
    artifacts: list[JsonObject] = []
    for page in pages:
        page_artifacts = page.get("artifacts", [])
        if isinstance(page_artifacts, list):
            for item in page_artifacts:
                if not isinstance(item, dict):
                    fail("artifact response contains a malformed artifact")
                artifacts.append(item)
        else:
            fail("artifact response has a malformed artifacts field")
    matches = [item for item in artifacts if item.get("name") == "release-candidate"]
    if any(item.get("expired") is True for item in matches):
        fail("the release-candidate artifact has expired")
    if len(matches) > 1:
        fail("multiple release-candidate artifacts exist for this run")
    if not matches:
        release = release_view(repository, tag)
        assets = release_assets(repository, release.get("databaseId"))
        if release.get("isImmutable") is True or assets:
            fail(
                "the original release-candidate artifact is missing after release "
                "publication started; restore the original candidate instead of rebuilding"
            )
    write_output("exists", "true" if matches else "false")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


def validate_tag(tag: str) -> str:
    if re.fullmatch(r"v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)", tag) is None:
        fail("release tag must be a stable semantic version such as v1.2.3")
    return tag.removeprefix("v")


def load_manifest(directory: Path, tag: str) -> tuple[JsonObject, set[str]]:
    manifest_path = directory / "dist-manifest.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail("release inventory is missing dist-manifest.json")
    except json.JSONDecodeError as error:
        fail(f"dist-manifest.json is invalid: {error}")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, dict) or not artifacts:
        fail("dist-manifest.json artifacts must be a non-empty object")
    manifest_tag = manifest.get("announcement_tag")
    if manifest_tag != tag:
        fail(f"dist-manifest.json tag does not match {tag}")
    artifact_names = set(artifacts)
    if not all(
        isinstance(name, str) and name == Path(name).name for name in artifact_names
    ):
        fail("dist-manifest.json artifact keys must be filenames")
    for name, artifact in artifacts.items():
        if not isinstance(artifact, dict) or artifact.get("name") != name:
            fail(f"dist-manifest.json artifact metadata does not match key: {name}")
    triples = {
        triple
        for artifact in artifacts.values()
        if isinstance(artifact, dict) and artifact.get("kind") == "executable-zip"
        for triple in artifact.get("target_triples", [])
        if isinstance(triple, str)
    }
    missing_targets = TARGETS - triples
    if missing_targets:
        fail(
            f"dist manifest is missing CLI targets: {', '.join(sorted(missing_targets))}"
        )
    if not any(name.endswith("installer.sh") for name in artifact_names):
        fail("dist manifest is missing the shell installer")
    if not any(name.endswith("installer.ps1") for name in artifact_names):
        fail("dist manifest is missing the PowerShell installer")
    if not any(name.endswith(".sha256") for name in artifact_names):
        fail("dist manifest is missing checksum artifacts")
    return manifest, artifact_names


def python_inventory(directory: Path, version: str) -> set[str]:
    names = {
        path.name
        for path in directory.iterdir()
        if path.is_file() and path.name.startswith(f"arco-{version}")
    }
    return python_inventory_from_names(names, version)


def verify(tag: str, directory: Path) -> set[str]:
    version = validate_tag(tag)
    if not directory.is_dir():
        fail(f"release directory does not exist: {directory}")
    _, cli_names = load_manifest(directory, tag)
    expected = (
        cli_names
        | python_inventory(directory, version)
        | {
            "dist-manifest.json",
            "arco-kdl-vscode.vsix",
        }
    )
    actual = {path.name for path in directory.iterdir() if path.is_file()}
    missing = expected - actual
    extra = actual - expected
    if missing or extra:
        fail(
            "release inventory mismatch; "
            f"missing={sorted(missing)}, extra={sorted(extra)}"
        )
    return expected


def release_view(repository: str, tag: str) -> JsonObject:
    release = gh_json(
        "release",
        "view",
        tag,
        "--repo",
        repository,
        "--json",
        "databaseId,isDraft,isImmutable",
    )
    if not isinstance(release, dict):
        fail("gh release view returned an invalid release")
    if release.get("isDraft") is not True and release.get("isImmutable") is not True:
        fail("release must be a draft or an immutable published release")
    return release


def release_assets(repository: str, release_id: JsonValue) -> list[JsonObject]:
    response = gh_json(
        "api",
        "--paginate",
        "--slurp",
        f"repos/{repository}/releases/{release_id}/assets",
    )
    return flatten_pages(response)


def indexed_assets(assets: list[JsonObject]) -> dict[str, JsonObject]:
    indexed: dict[str, JsonObject] = {}
    for asset in assets:
        name = asset.get("name")
        if not isinstance(name, str) or name in indexed:
            fail(f"release has a duplicate or invalid asset name: {name!r}")
        indexed[name] = asset
    return indexed


def check_remote_inventory(
    *, directory: Path, expected: set[str], assets: list[JsonObject], complete: bool
) -> list[Path]:
    remote = indexed_assets(assets)
    extras = set(remote) - expected
    if extras:
        fail(f"release has unexpected assets: {sorted(extras)}")
    missing: list[Path] = []
    for name in sorted(expected):
        asset = remote.get(name)
        if asset is None:
            missing.append(directory / name)
            continue
        digest = asset.get("digest")
        if not isinstance(digest, str) or not digest.startswith("sha256:"):
            fail(f"release asset has no SHA-256 digest: {name}")
        if digest != sha256(directory / name):
            fail(f"release asset has a different digest: {name}")
    if complete and missing:
        fail(f"release is missing assets: {[path.name for path in missing]}")
    return missing


def publish(tag: str, directory: Path) -> None:
    preflight()
    expected = verify(tag, directory)
    repository = require_env("GITHUB_REPOSITORY")
    expected_sha = require_env("GITHUB_SHA")
    release = release_view(repository, tag)
    if tag_commit(repository, tag) != expected_sha:
        fail(f"{tag} does not point to GITHUB_SHA")
    release_id = release.get("databaseId")
    assets = release_assets(repository, release_id)
    missing = check_remote_inventory(
        directory=directory, expected=expected, assets=assets, complete=False
    )
    if missing:
        if release.get("isDraft") is not True:
            fail("an immutable release is missing required assets")
        run_gh(
            "release",
            "upload",
            tag,
            *(str(path) for path in missing),
            "--repo",
            repository,
        )
    check_remote_inventory(
        directory=directory,
        expected=expected,
        assets=release_assets(repository, release_id),
        complete=True,
    )
    if release.get("isDraft") is True:
        preflight()
        if tag_commit(repository, tag) != expected_sha:
            fail(f"{tag} changed before publication")
        run_gh("release", "edit", tag, "--repo", repository, "--draft=false")
    final = release_view(repository, tag)
    if final.get("isImmutable") is not True or final.get("isDraft") is True:
        fail("release did not become immutable after publication")


def python_asset_names(assets: list[JsonObject], tag: str) -> set[str]:
    version = tag.removeprefix("v")
    names: set[str] = set()
    for asset in assets:
        name = asset.get("name")
        if isinstance(name, str):
            names.add(name)
    candidates = {name for name in names if name.startswith(f"arco-{version}")}
    expected = python_inventory_from_names(candidates, version)
    return expected


def python_inventory_from_names(names: set[str], version: str) -> set[str]:
    exact = {
        f"arco-{version}-cp310-cp310-manylinux_2_28_x86_64.whl",
        f"arco-{version}-cp311-abi3-manylinux_2_28_x86_64.whl",
        f"arco-{version}-cp310-cp310-win_amd64.whl",
        f"arco-{version}-cp311-abi3-win_amd64.whl",
        f"arco-{version}.tar.gz",
    }
    for abi in ("cp310-cp310", "cp311-abi3"):
        matches = {
            name
            for name in names
            if name.startswith(f"arco-{version}-{abi}-macosx-")
            and name.endswith("_arm64.whl")
        }
        if len(matches) != 1:
            fail(f"release must contain one macOS arm64 {abi} wheel")
        exact |= matches
    if not exact <= names:
        fail(f"release is missing Python files: {sorted(exact - names)}")
    wheels = {name for name in names if name.endswith(".whl")}
    sdists = {name for name in names if name.endswith(".tar.gz")}
    if wheels != exact - {f"arco-{version}.tar.gz"} or sdists != {
        f"arco-{version}.tar.gz"
    }:
        fail("release contains an unexpected Python wheel or sdist")
    return exact


def download_python(tag: str, directory: Path) -> None:
    validate_tag(tag)
    repository = require_env("GITHUB_REPOSITORY")
    release = release_view(repository, tag)
    if release.get("isDraft") is True or release.get("isImmutable") is not True:
        fail("Python files may only be downloaded from an immutable published release")
    assets = release_assets(repository, release.get("databaseId"))
    expected = python_asset_names(assets, tag)
    remote = indexed_assets(assets)
    directory.mkdir(parents=True, exist_ok=True)
    args = ["release", "download", tag, "--repo", repository, "--dir", str(directory)]
    for name in sorted(expected):
        args.extend(("--pattern", name))
    run_gh(*args)
    actual = {path.name for path in directory.iterdir() if path.is_file()}
    if actual != expected:
        fail(f"downloaded Python inventory mismatch: {sorted(actual ^ expected)}")
    for name in expected:
        digest = remote[name].get("digest")
        if not isinstance(digest, str) or not digest.startswith("sha256:"):
            fail(f"release asset has no SHA-256 digest: {name}")
        if sha256(directory / name) != digest:
            fail(f"downloaded release asset has a different digest: {name}")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("resolve")
    subparsers.add_parser("preflight")
    candidate_parser = subparsers.add_parser("candidate")
    candidate_parser.add_argument("--tag", required=True)
    for command in ("verify", "publish", "download-python"):
        command_parser = subparsers.add_parser(command)
        command_parser.add_argument("--tag", required=True)
        command_parser.add_argument(
            "--directory",
            type=Path,
            default=Path("dist" if command == "download-python" else "release-files"),
        )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        if args.command == "resolve":
            resolve()
        elif args.command == "preflight":
            preflight()
        elif args.command == "candidate":
            candidate(args.tag)
        elif args.command == "verify":
            verify(args.tag, args.directory)
        elif args.command == "publish":
            publish(args.tag, args.directory)
        elif args.command == "download-python":
            download_python(args.tag, args.directory)
    except (
        ReleaseError,
        subprocess.CalledProcessError,
        subprocess.TimeoutExpired,
    ) as error:
        print(f"release pipeline: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
