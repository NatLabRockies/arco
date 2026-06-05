from __future__ import annotations

import io
from pathlib import Path
import sys
import tarfile

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.sync_python_licenses import sync_python_licenses  # noqa: E402
from scripts.ci.python_distribution_matrix import (  # noqa: E402
    MatrixFilter,
    build_github_matrix,
    load_distribution_matrix,
)
from scripts.ci.validate_python_distributions import (  # noqa: E402
    validate_distribution_manifest,
)


def test_distribution_matrix_filters_single_failed_combo() -> None:
    matrix = load_distribution_matrix(
        path=Path(".github/python-distribution-matrix.json")
    )

    github_matrix = build_github_matrix(
        matrix=matrix,
        matrix_filter=MatrixFilter(platform="windows", python="abi3"),
    )

    assert github_matrix == {
        "include": [
            {
                "platform_label": "windows",
                "platform_os": "windows-latest",
                "wheel_python": "python",
                "python_label": "abi3",
                "python_version": "3.11",
                "wheel_features": "pyo3/extension-module,pyo3/abi3-py311,xpress",
            }
        ]
    }


def test_distribution_matrix_emits_all_expected_release_combos() -> None:
    matrix = load_distribution_matrix(
        path=Path(".github/python-distribution-matrix.json")
    )

    github_matrix = build_github_matrix(
        matrix=matrix,
        matrix_filter=MatrixFilter(platform="all", python="all"),
    )

    combo_ids = {
        f"{entry['platform_label']}:{entry['python_label']}"
        for entry in github_matrix["include"]
    }
    assert combo_ids == {
        "linux:cp310",
        "linux:abi3",
        "macos-x64:cp310",
        "macos-x64:abi3",
        "macos-arm64:cp310",
        "macos-arm64:abi3",
        "windows:cp310",
        "windows:abi3",
    }


def test_distribution_manifest_requires_all_wheels_and_sdist(tmp_path: Path) -> None:
    dist_dir = tmp_path / "dist"
    dist_dir.mkdir()
    _write_complete_dist(dist_dir=dist_dir, version="0.8.0")

    result = validate_distribution_manifest(
        dist_dir=dist_dir,
        matrix_path=Path(".github/python-distribution-matrix.json"),
        release_tag="v0.8.0",
    )

    assert result.wheel_count == 8
    assert result.sdist_count == 1


def test_distribution_manifest_rejects_incomplete_release(tmp_path: Path) -> None:
    dist_dir = tmp_path / "dist"
    dist_dir.mkdir()
    (dist_dir / "arco-0.8.0-cp310-cp310-win_amd64.whl").write_text(
        "placeholder", encoding="utf-8"
    )

    with pytest.raises(ValueError, match="Missing wheel for linux/cp310"):
        validate_distribution_manifest(
            dist_dir=dist_dir,
            matrix_path=Path(".github/python-distribution-matrix.json"),
            release_tag="v0.8.0",
        )


def test_distribution_manifest_rejects_sdist_missing_license_file(
    tmp_path: Path,
) -> None:
    dist_dir = tmp_path / "dist"
    dist_dir.mkdir()
    _write_complete_dist(dist_dir=dist_dir, version="0.8.0", complete_sdist=False)

    with pytest.raises(ValueError, match="BSD-3-Clause.txt"):
        validate_distribution_manifest(
            dist_dir=dist_dir,
            matrix_path=Path(".github/python-distribution-matrix.json"),
            release_tag="v0.8.0",
        )


def test_distribution_manifest_rejects_version_mismatch(tmp_path: Path) -> None:
    dist_dir = tmp_path / "dist"
    dist_dir.mkdir()
    _write_complete_dist(dist_dir=dist_dir, version="0.8.0")

    with pytest.raises(ValueError, match="does not match release tag version"):
        validate_distribution_manifest(
            dist_dir=dist_dir,
            matrix_path=Path(".github/python-distribution-matrix.json"),
            release_tag="v0.8.1",
        )


def test_distribution_manifest_rejects_wrong_linux_architecture(tmp_path: Path) -> None:
    dist_dir = tmp_path / "dist"
    dist_dir.mkdir()
    _write_complete_dist(dist_dir=dist_dir, version="0.8.0")
    (dist_dir / "arco-0.8.0-cp310-cp310-manylinux_2_38_x86_64.whl").unlink()
    (dist_dir / "arco-0.8.0-cp310-cp310-manylinux_2_38_aarch64.whl").write_text(
        "placeholder", encoding="utf-8"
    )

    with pytest.raises(ValueError, match="Missing wheel for linux/cp310"):
        validate_distribution_manifest(
            dist_dir=dist_dir,
            matrix_path=Path(".github/python-distribution-matrix.json"),
            release_tag="v0.8.0",
        )


def test_sync_python_licenses_accepts_explicit_repo_root(tmp_path: Path) -> None:
    repo_root = tmp_path / "source"
    (repo_root / "licenses").mkdir(parents=True)
    (repo_root / "bindings" / "python").mkdir(parents=True)
    (repo_root / "licenses" / "BSD-3-Clause.txt").write_text("bsd", encoding="utf-8")
    (repo_root / "licenses" / "HiGHS-MIT.txt").write_text("highs", encoding="utf-8")

    copied = sync_python_licenses(repo_root=repo_root)

    copied_relative = {path.relative_to(repo_root).as_posix() for path in copied}
    assert copied_relative == {
        "bindings/python/BSD-3-Clause.txt",
        "bindings/python/licenses/BSD-3-Clause.txt",
        "bindings/python/licenses/HiGHS-MIT.txt",
    }


def _write_complete_dist(
    *, dist_dir: Path, version: str, complete_sdist: bool = True
) -> None:
    for filename in (
        f"arco-{version}-cp310-cp310-manylinux_2_38_x86_64.whl",
        f"arco-{version}-cp311-abi3-manylinux_2_38_x86_64.whl",
        f"arco-{version}-cp310-cp310-macosx_13_0_x86_64.whl",
        f"arco-{version}-cp311-abi3-macosx_13_0_x86_64.whl",
        f"arco-{version}-cp310-cp310-macosx_14_0_arm64.whl",
        f"arco-{version}-cp311-abi3-macosx_14_0_arm64.whl",
        f"arco-{version}-cp310-cp310-win_amd64.whl",
        f"arco-{version}-cp311-abi3-win_amd64.whl",
    ):
        (dist_dir / filename).write_text("placeholder", encoding="utf-8")

    sdist_files = [
        f"arco-{version}/licenses/BSD-3-Clause.txt",
        f"arco-{version}/licenses/HiGHS-MIT.txt",
    ]
    if complete_sdist:
        sdist_files.append(f"arco-{version}/BSD-3-Clause.txt")
    _write_sdist(dist_dir / f"arco-{version}.tar.gz", files=tuple(sdist_files))


def _write_sdist(path: Path, *, files: tuple[str, ...]) -> None:
    with tarfile.open(path, mode="w:gz") as archive:
        for filename in files:
            payload = b"license"
            info = tarfile.TarInfo(filename)
            info.size = len(payload)
            archive.addfile(info, io.BytesIO(payload))
