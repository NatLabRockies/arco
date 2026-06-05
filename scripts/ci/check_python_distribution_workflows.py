from __future__ import annotations

from pathlib import Path
import sys
from typing import Sequence

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.ci.python_distribution_matrix import (  # noqa: E402
    MatrixFilter,
    build_github_matrix,
    load_distribution_matrix,
)


MATRIX_PATH = REPO_ROOT / ".github" / "python-distribution-matrix.json"
STAGE_WORKFLOW = ".github/workflows/python-distributions-stage.yaml"
PUBLISH_WORKFLOW = ".github/workflows/pypi-publish-staged.yaml"
CALLER_WORKFLOWS = (
    REPO_ROOT / ".github" / "workflows" / "pypi-manual-release.yaml",
    REPO_ROOT / ".github" / "workflows" / "release-please.yaml",
)
EXPECTED_COMBO_COUNT = 8


def check_python_distribution_workflows() -> None:
    matrix = load_distribution_matrix(path=MATRIX_PATH)
    emitted = build_github_matrix(
        matrix=matrix, matrix_filter=MatrixFilter(platform="all", python="all")
    )
    combo_count = len(emitted["include"])
    if combo_count != EXPECTED_COMBO_COUNT:
        raise ValueError(
            f"Expected {EXPECTED_COMBO_COUNT} Python distribution combos, got {combo_count}"
        )

    for workflow_path in CALLER_WORKFLOWS:
        text = workflow_path.read_text(encoding="utf-8")
        relative_path = workflow_path.relative_to(REPO_ROOT)
        if STAGE_WORKFLOW not in text:
            raise ValueError(f"{relative_path} must call {STAGE_WORKFLOW}")
        if PUBLISH_WORKFLOW not in text:
            raise ValueError(f"{relative_path} must call {PUBLISH_WORKFLOW}")
        if "wheel_features:" in text or "wheel_python:" in text:
            raise ValueError(
                f"{relative_path} contains an inline Python distribution matrix; "
                f"update {MATRIX_PATH.relative_to(REPO_ROOT)} instead"
            )

    for workflow in (STAGE_WORKFLOW, PUBLISH_WORKFLOW):
        workflow_path = REPO_ROOT / workflow
        if not workflow_path.is_file():
            raise FileNotFoundError(f"Required workflow is missing: {workflow}")


def main(*, argv: Sequence[str]) -> int:
    if argv:
        raise ValueError("This script does not accept positional arguments.")
    check_python_distribution_workflows()
    print("python-distribution-workflows-ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(argv=sys.argv[1:]))
