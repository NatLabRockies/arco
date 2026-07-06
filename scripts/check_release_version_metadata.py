#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path


EXPECTED_RELEASE_PLEASE_EXTRA_FILES = {
    "Cargo.toml",
    "bindings/python/pyproject.toml",
    "bindings/python/uv.lock",
}


def read_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as file:
        return tomllib.load(file)


def fail(message: str) -> None:
    print(f"::error::{message}", file=sys.stderr)
    raise SystemExit(1)


def get_nested_string(data: dict[str, object], keys: tuple[str, ...]) -> str | None:
    value: object = data
    for key in keys:
        if not isinstance(value, dict):
            return None
        value = value.get(key)
    return value if isinstance(value, str) else None


def get_uv_arco_version(path: Path) -> str | None:
    uv_lock = read_toml(path)
    packages = uv_lock.get("package")
    if not isinstance(packages, list):
        return None

    for package in packages:
        if not isinstance(package, dict):
            continue
        if package.get("name") == "arco":
            version = package.get("version")
            return version if isinstance(version, str) else None
    return None


def get_manifest_version(path: Path) -> str | None:
    manifest = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict):
        return None
    version = manifest.get(".")
    return version if isinstance(version, str) else None


def get_release_please_extra_file_paths(path: Path) -> set[str]:
    config = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(config, dict):
        return set()

    packages = config.get("packages")
    if not isinstance(packages, dict):
        return set()

    root_package = packages.get(".")
    if not isinstance(root_package, dict):
        return set()

    extra_files = root_package.get("extra-files")
    if not isinstance(extra_files, list):
        return set()

    paths: set[str] = set()
    for entry in extra_files:
        if not isinstance(entry, dict):
            continue
        path_value = entry.get("path")
        if isinstance(path_value, str):
            paths.add(path_value)
    return paths


def validate_release_versions() -> str:
    versions = {
        "Cargo.toml workspace.package.version": get_nested_string(
            read_toml(Path("Cargo.toml")),
            ("workspace", "package", "version"),
        ),
        "bindings/python/pyproject.toml project.version": get_nested_string(
            read_toml(Path("bindings/python/pyproject.toml")),
            ("project", "version"),
        ),
        "bindings/python/uv.lock package[arco].version": get_uv_arco_version(
            Path("bindings/python/uv.lock")
        ),
        ".github/release-please-manifest.json .": get_manifest_version(
            Path(".github/release-please-manifest.json")
        ),
    }

    missing = [name for name, version in versions.items() if not version]
    if missing:
        fail("Missing release version metadata: " + ", ".join(missing))

    distinct_versions = sorted({version for version in versions.values() if version})
    if len(distinct_versions) != 1:
        details = ", ".join(f"{name}={version}" for name, version in versions.items())
        fail(f"Release version metadata mismatch: {details}")

    extra_file_paths = get_release_please_extra_file_paths(
        Path(".github/release-please-config.json")
    )
    missing_extra_files = sorted(EXPECTED_RELEASE_PLEASE_EXTRA_FILES - extra_file_paths)
    if missing_extra_files:
        fail(
            "release-please is not configured to update: "
            + ", ".join(missing_extra_files)
        )

    return distinct_versions[0]


def main() -> int:
    version = validate_release_versions()
    print(f"Release version metadata is consistent: {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
