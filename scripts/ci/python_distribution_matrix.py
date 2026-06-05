from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
import sys
from typing import Any, Sequence, TypedDict


class GitHubMatrixEntry(TypedDict):
    platform_label: str
    platform_os: str
    wheel_python: str
    python_label: str
    python_version: str
    wheel_features: str


class GitHubMatrix(TypedDict):
    include: list[GitHubMatrixEntry]


@dataclass(frozen=True, slots=True)
class PythonBuild:
    label: str
    version: str
    wheel_features: str


@dataclass(frozen=True, slots=True)
class PlatformBuild:
    label: str
    os: str
    wheel_python: str


@dataclass(frozen=True, slots=True)
class DistributionMatrix:
    python: tuple[PythonBuild, ...]
    platform: tuple[PlatformBuild, ...]


@dataclass(frozen=True, slots=True)
class MatrixFilter:
    platform: str = "all"
    python: str = "all"


def load_distribution_matrix(*, path: Path) -> DistributionMatrix:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"Matrix file must contain a JSON object: {path}")

    python_builds = tuple(
        parse_python_build(item=item, path=path)
        for item in required_list(payload=payload, key="python", path=path)
    )
    platform_builds = tuple(
        parse_platform_build(item=item, path=path)
        for item in required_list(payload=payload, key="platform", path=path)
    )
    validate_unique_labels(kind="python", labels=[item.label for item in python_builds])
    validate_unique_labels(
        kind="platform", labels=[item.label for item in platform_builds]
    )
    return DistributionMatrix(python=python_builds, platform=platform_builds)


def build_github_matrix(
    *, matrix: DistributionMatrix, matrix_filter: MatrixFilter
) -> GitHubMatrix:
    selected_platforms = select_platforms(
        platforms=matrix.platform, label=matrix_filter.platform
    )
    selected_python = select_python(
        python_builds=matrix.python, label=matrix_filter.python
    )

    include: list[GitHubMatrixEntry] = []
    for platform in selected_platforms:
        for python_build in selected_python:
            include.append(
                {
                    "platform_label": platform.label,
                    "platform_os": platform.os,
                    "wheel_python": platform.wheel_python,
                    "python_label": python_build.label,
                    "python_version": python_build.version,
                    "wheel_features": python_build.wheel_features,
                }
            )
    if not include:
        raise ValueError(
            "Matrix filter selected no Python distribution build combinations."
        )
    return {"include": include}


def matrix_to_json(*, github_matrix: GitHubMatrix) -> str:
    return json.dumps(github_matrix, separators=(",", ":"), sort_keys=True)


def required_list(*, payload: dict[str, Any], key: str, path: Path) -> list[Any]:
    value = payload.get(key)
    if not isinstance(value, list) or not value:
        raise ValueError(f"Matrix file {path} must define a non-empty {key!r} list")
    return value


def parse_python_build(*, item: Any, path: Path) -> PythonBuild:
    if not isinstance(item, dict):
        raise ValueError(f"Python matrix entries must be objects in {path}")
    return PythonBuild(
        label=required_string(item=item, key="label", path=path),
        version=required_string(item=item, key="version", path=path),
        wheel_features=required_string(item=item, key="wheel_features", path=path),
    )


def parse_platform_build(*, item: Any, path: Path) -> PlatformBuild:
    if not isinstance(item, dict):
        raise ValueError(f"Platform matrix entries must be objects in {path}")
    return PlatformBuild(
        label=required_string(item=item, key="label", path=path),
        os=required_string(item=item, key="os", path=path),
        wheel_python=required_string(item=item, key="wheel_python", path=path),
    )


def required_string(*, item: dict[str, Any], key: str, path: Path) -> str:
    value = item.get(key)
    if not isinstance(value, str):
        raise ValueError(f"Matrix entry in {path} must define string field {key!r}")
    return value


def validate_unique_labels(*, kind: str, labels: Sequence[str]) -> None:
    duplicates = sorted({label for label in labels if labels.count(label) > 1})
    if duplicates:
        joined = ", ".join(duplicates)
        raise ValueError(
            f"Duplicate {kind} labels in Python distribution matrix: {joined}"
        )


def select_platforms(
    *, platforms: Sequence[PlatformBuild], label: str
) -> tuple[PlatformBuild, ...]:
    if label == "all":
        return tuple(platforms)
    selected = tuple(platform for platform in platforms if platform.label == label)
    if not selected:
        valid = ", ".join(["all", *(platform.label for platform in platforms)])
        raise ValueError(f"Unknown platform filter {label!r}; valid values: {valid}")
    return selected


def select_python(
    *, python_builds: Sequence[PythonBuild], label: str
) -> tuple[PythonBuild, ...]:
    if label == "all":
        return tuple(python_builds)
    selected = tuple(
        python_build for python_build in python_builds if python_build.label == label
    )
    if not selected:
        valid = ", ".join(
            ["all", *(python_build.label for python_build in python_builds)]
        )
        raise ValueError(f"Unknown Python filter {label!r}; valid values: {valid}")
    return selected


def _parse_args(*, argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Emit the GitHub Actions matrix for Python distribution builds."
    )
    parser.add_argument(
        "--matrix-path",
        type=Path,
        default=Path(".github/python-distribution-matrix.json"),
        help="Path to the canonical Python distribution matrix JSON file.",
    )
    parser.add_argument(
        "--platform",
        default="all",
        help="Platform label to emit, or 'all'.",
    )
    parser.add_argument(
        "--python",
        default="all",
        help="Python build label to emit, or 'all'.",
    )
    parser.add_argument(
        "--github-output",
        type=Path,
        help="Optional GITHUB_OUTPUT file to append matrix=<json> to.",
    )
    return parser.parse_args(argv)


def main(*, argv: Sequence[str]) -> int:
    args = _parse_args(argv=argv)
    matrix = load_distribution_matrix(path=args.matrix_path)
    github_matrix = build_github_matrix(
        matrix=matrix,
        matrix_filter=MatrixFilter(platform=args.platform, python=args.python),
    )
    matrix_json = matrix_to_json(github_matrix=github_matrix)
    if args.github_output is not None:
        with args.github_output.open("a", encoding="utf-8") as output_file:
            output_file.write(f"matrix={matrix_json}\n")
    print(matrix_json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
