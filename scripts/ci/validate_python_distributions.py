from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
import sys
import tarfile
from typing import Callable, Sequence

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.ci.python_distribution_matrix import load_distribution_matrix  # noqa: E402


@dataclass(frozen=True, slots=True)
class WheelTags:
    filename: str
    distribution: str
    version: str
    python_tag: str
    abi_tag: str
    platform_tag: str


@dataclass(frozen=True, slots=True)
class ManifestValidationResult:
    wheel_count: int
    sdist_count: int


_REQUIRED_SDIST_PATHS: tuple[tuple[str, ...], ...] = (
    ("BSD-3-Clause.txt",),
    ("licenses", "BSD-3-Clause.txt"),
    ("licenses", "HiGHS-MIT.txt"),
)


def validate_wheel_file(
    *, wheel_path: Path, python_label: str, platform_label: str
) -> None:
    tags = parse_wheel_tags(wheel_path=wheel_path)
    validate_python_tags(tags=tags, python_label=python_label)
    validate_platform_tag(tags=tags, platform_label=platform_label)


def validate_wheel_directory(
    *, dist_dir: Path, python_label: str, platform_label: str
) -> WheelTags:
    wheels = sorted(dist_dir.glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError(
            f"Expected exactly one wheel in {dist_dir}, found {len(wheels)}"
        )
    validate_wheel_file(
        wheel_path=wheels[0], python_label=python_label, platform_label=platform_label
    )
    return parse_wheel_tags(wheel_path=wheels[0])


def validate_sdist_directory(
    *, dist_dir: Path, expected_version: str | None = None
) -> Path:
    sdists = sorted(dist_dir.glob("*.tar.gz"))
    if len(sdists) != 1:
        raise ValueError(
            f"Expected exactly one sdist in {dist_dir}, found {len(sdists)}"
        )
    sdist_path = sdists[0]
    distribution, version = parse_sdist_name(sdist_path=sdist_path)
    validate_package_name(name=distribution, filename=sdist_path.name)
    if expected_version is not None and version != expected_version:
        raise ValueError(
            f"sdist {sdist_path.name} version {version!r} does not match "
            f"release tag version {expected_version!r}"
        )
    validate_sdist_license_files(sdist_path=sdist_path)
    return sdist_path


def validate_distribution_manifest(
    *, dist_dir: Path, matrix_path: Path, release_tag: str | None = None
) -> ManifestValidationResult:
    matrix = load_distribution_matrix(path=matrix_path)
    wheels = sorted(dist_dir.glob("*.whl"))
    if not wheels:
        raise ValueError(f"No wheels found in {dist_dir}")

    expected_version = version_from_release_tag(release_tag=release_tag)
    matched_wheels: set[Path] = set()
    for platform in matrix.platform:
        for python_build in matrix.python:
            matching = [
                wheel
                for wheel in wheels
                if wheel_matches(
                    wheel_path=wheel,
                    python_label=python_build.label,
                    platform_label=platform.label,
                )
            ]
            if not matching:
                raise ValueError(
                    f"Missing wheel for {platform.label}/{python_build.label}"
                )
            if len(matching) > 1:
                filenames = ", ".join(wheel.name for wheel in matching)
                raise ValueError(
                    f"Multiple wheels for {platform.label}/{python_build.label}: {filenames}"
                )
            matched_wheels.add(matching[0])

    expected_combo_count = len(matrix.platform) * len(matrix.python)
    if len(matched_wheels) != expected_combo_count:
        raise ValueError(
            f"Expected {expected_combo_count} distinct wheels, got {len(matched_wheels)}"
        )

    unexpected_wheels = sorted(set(wheels) - matched_wheels)
    if unexpected_wheels:
        filenames = ", ".join(wheel.name for wheel in unexpected_wheels)
        raise ValueError(f"Unexpected wheels in release manifest: {filenames}")

    for wheel in matched_wheels:
        tags = parse_wheel_tags(wheel_path=wheel)
        validate_package_name(name=tags.distribution, filename=tags.filename)
        if expected_version is not None and tags.version != expected_version:
            raise ValueError(
                f"Wheel {tags.filename} version {tags.version!r} does not match "
                f"release tag version {expected_version!r}"
            )

    validate_sdist_directory(dist_dir=dist_dir, expected_version=expected_version)
    return ManifestValidationResult(wheel_count=len(wheels), sdist_count=1)


def wheel_matches(*, wheel_path: Path, python_label: str, platform_label: str) -> bool:
    tags = parse_wheel_tags(wheel_path=wheel_path)
    return python_tags_match(
        tags=tags, python_label=python_label
    ) and platform_tag_matches(tags=tags, platform_label=platform_label)


def parse_wheel_tags(*, wheel_path: Path) -> WheelTags:
    if wheel_path.suffix != ".whl":
        raise ValueError(f"Expected a wheel file, got {wheel_path}")
    stem = wheel_path.name.removesuffix(".whl")
    parts = stem.rsplit("-", maxsplit=4)
    if len(parts) != 5:
        raise ValueError(f"Invalid wheel filename: {wheel_path.name}")
    return WheelTags(
        filename=wheel_path.name,
        distribution=parts[0],
        version=parts[1],
        python_tag=parts[2],
        abi_tag=parts[3],
        platform_tag=parts[4],
    )


def parse_sdist_name(*, sdist_path: Path) -> tuple[str, str]:
    name = sdist_path.name
    if not name.endswith(".tar.gz"):
        raise ValueError(f"Expected a .tar.gz sdist, got {name}")
    stem = name.removesuffix(".tar.gz")
    distribution, separator, version = stem.rpartition("-")
    if not separator or not distribution or not version:
        raise ValueError(f"Invalid sdist filename: {name}")
    return distribution, version


def validate_package_name(*, name: str, filename: str) -> None:
    if name != "arco":
        raise ValueError(
            f"Distribution {filename} has unexpected package name {name!r}"
        )


def version_from_release_tag(*, release_tag: str | None) -> str | None:
    if release_tag is None or release_tag == "":
        return None
    if not release_tag.startswith("v"):
        raise ValueError(f"Release tag must start with 'v': {release_tag}")
    version = release_tag.removeprefix("v")
    if not version:
        raise ValueError("Release tag did not include a version after 'v'")
    return version


def validate_python_tags(*, tags: WheelTags, python_label: str) -> None:
    if not python_tags_match(tags=tags, python_label=python_label):
        raise ValueError(
            f"Wheel {tags.filename} does not match Python lane {python_label}: "
            f"python={tags.python_tag} abi={tags.abi_tag}"
        )


def python_tags_match(*, tags: WheelTags, python_label: str) -> bool:
    if python_label == "cp310":
        return tags.python_tag == "cp310" and tags.abi_tag == "cp310"
    if python_label == "abi3":
        return tags.python_tag == "cp311" and tags.abi_tag == "abi3"
    raise ValueError(f"Unknown Python distribution label: {python_label}")


def validate_platform_tag(*, tags: WheelTags, platform_label: str) -> None:
    if not platform_tag_matches(tags=tags, platform_label=platform_label):
        raise ValueError(
            f"Wheel {tags.filename} platform tag {tags.platform_tag!r} does not "
            f"match platform lane {platform_label}"
        )


def platform_tag_matches(*, tags: WheelTags, platform_label: str) -> bool:
    predicates: dict[str, Callable[[str], bool]] = {
        "linux": lambda tag: "linux" in tag and tag.endswith("x86_64"),
        "macos-x64": lambda tag: tag.startswith("macosx_") and "x86_64" in tag,
        "macos-arm64": lambda tag: tag.startswith("macosx_") and "arm64" in tag,
        "windows": lambda tag: tag == "win_amd64",
    }
    predicate = predicates.get(platform_label)
    if predicate is None:
        raise ValueError(f"Unknown platform distribution label: {platform_label}")
    return predicate(tags.platform_tag)


def validate_sdist_license_files(*, sdist_path: Path) -> None:
    with tarfile.open(sdist_path, mode="r:gz") as archive:
        member_paths = [Path(member.name) for member in archive.getmembers()]
    missing = [
        "/".join(required)
        for required in _REQUIRED_SDIST_PATHS
        if not has_sdist_member(member_paths=member_paths, required=required)
    ]
    if missing:
        missing_paths = ", ".join(missing)
        raise ValueError(
            f"sdist {sdist_path.name} is missing required license file(s): {missing_paths}"
        )


def has_sdist_member(
    *, member_paths: Sequence[Path], required: tuple[str, ...]
) -> bool:
    for member_path in member_paths:
        parts = member_path.parts
        if len(parts) != len(required) + 1:
            continue
        if tuple(parts[1:]) == required:
            return True
    return False


def _parse_args(*, argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Arco Python distribution artifacts before staging or publishing."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    wheel = subparsers.add_parser(
        "wheel", help="Validate the wheel from one matrix job."
    )
    wheel.add_argument("--dist-dir", type=Path, default=Path("dist"))
    wheel.add_argument("--python-label", required=True)
    wheel.add_argument("--platform-label", required=True)

    sdist = subparsers.add_parser(
        "sdist", help="Validate the sdist from one matrix job."
    )
    sdist.add_argument("--dist-dir", type=Path, default=Path("dist"))
    sdist.add_argument("--release-tag")

    manifest = subparsers.add_parser(
        "manifest", help="Validate a complete staged release manifest."
    )
    manifest.add_argument("--dist-dir", type=Path, default=Path("dist"))
    manifest.add_argument(
        "--matrix-path",
        type=Path,
        default=Path(".github/python-distribution-matrix.json"),
    )
    manifest.add_argument("--release-tag")
    return parser.parse_args(argv)


def main(*, argv: Sequence[str]) -> int:
    args = _parse_args(argv=argv)
    if args.command == "wheel":
        tags = validate_wheel_directory(
            dist_dir=args.dist_dir,
            python_label=args.python_label,
            platform_label=args.platform_label,
        )
        print(f"wheel-ok {tags.filename}")
        return 0
    if args.command == "sdist":
        sdist_path = validate_sdist_directory(
            dist_dir=args.dist_dir,
            expected_version=version_from_release_tag(release_tag=args.release_tag),
        )
        print(f"sdist-ok {sdist_path.name}")
        return 0
    if args.command == "manifest":
        result = validate_distribution_manifest(
            dist_dir=args.dist_dir,
            matrix_path=args.matrix_path,
            release_tag=args.release_tag,
        )
        print(f"manifest-ok wheels={result.wheel_count} sdists={result.sdist_count}")
        return 0
    raise ValueError(f"Unknown validation command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
