from __future__ import annotations

from dataclasses import dataclass
import doctest
from pathlib import Path
import re
from typing import Iterable

import pytest


def _find_repo_root(*, start: Path) -> Path:
    for candidate in (start, *start.parents):
        if (candidate / "Cargo.toml").is_file():
            return candidate
    raise AssertionError(f"Could not locate repository root from {start}")


REPO_ROOT = _find_repo_root(start=Path(__file__).resolve().parent)
DOCS_DIR = REPO_ROOT / "docs"
DOCTEST_FENCE_PATTERN = re.compile(r"^```python\s+doctest\b")
COMMENTED_DOCTEST_LINE_PATTERN = re.compile(r"^(\s*)#\s?(.*)$")
FENCE_END = "```"
EXCLUDED_DOC_DIRS: set[str] = set()


@dataclass(frozen=True, slots=True)
class DoctestBlock:
    file_path: Path
    block_index: int
    line_start: int
    source: str

    @property
    def case_id(self) -> str:
        relative_path = self.file_path.relative_to(REPO_ROOT)
        return f"{relative_path}::block-{self.block_index}"


def _iter_markdown_files() -> Iterable[Path]:
    markdown_files = DOCS_DIR.rglob("*.md")
    return sorted(
        file_path
        for file_path in markdown_files
        if EXCLUDED_DOC_DIRS.isdisjoint(file_path.relative_to(DOCS_DIR).parts)
    )


def _extract_doctest_blocks(*, file_path: Path) -> list[DoctestBlock]:
    blocks: list[DoctestBlock] = []
    active_lines: list[str] = []
    block_index = 0
    in_doctest_block = False
    block_start_line = 0

    for line_number, line in enumerate(
        file_path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        if not in_doctest_block:
            if DOCTEST_FENCE_PATTERN.match(line.strip()) is None:
                continue
            in_doctest_block = True
            block_start_line = line_number + 1
            active_lines = []
            block_index += 1
            continue

        if line.strip() == FENCE_END:
            blocks.append(
                DoctestBlock(
                    file_path=file_path,
                    block_index=block_index,
                    line_start=block_start_line,
                    source="\n".join(active_lines),
                )
            )
            in_doctest_block = False
            active_lines = []
            continue

        active_lines.append(line)

    if in_doctest_block:
        raise AssertionError(f"Unterminated doctest block in {file_path}")

    return blocks


def _collect_doctest_blocks() -> list[DoctestBlock]:
    blocks: list[DoctestBlock] = []
    for markdown_file in _iter_markdown_files():
        blocks.extend(_extract_doctest_blocks(file_path=markdown_file))
    if not blocks:
        raise AssertionError(f"No markdown doctest blocks found under {DOCS_DIR}")
    return blocks


def _normalize_doctest_source(*, source: str) -> str:
    normalized_lines: list[str] = []
    for line in source.splitlines():
        match = COMMENTED_DOCTEST_LINE_PATTERN.match(line)
        if match is None:
            normalized_lines.append(line)
            continue

        indent, content = match.groups()
        normalized_lines.append(f"{indent}{content}")

    return "\n".join(normalized_lines)


DOCTEST_BLOCKS = _collect_doctest_blocks()


@pytest.mark.parametrize("block", DOCTEST_BLOCKS, ids=lambda block: block.case_id)
def test_markdown_doctest_blocks(block: DoctestBlock) -> None:
    source = _normalize_doctest_source(source=block.source)
    parser = doctest.DocTestParser()
    document_test = parser.get_doctest(
        source,
        globs={"__name__": "__main__"},
        name=block.case_id,
        filename=str(block.file_path),
        lineno=block.line_start - 1,
    )

    runner = doctest.DocTestRunner(optionflags=doctest.ELLIPSIS)
    result = runner.run(document_test)

    assert result.failed == 0, (
        f"Doctest failed for {block.case_id} "
        f"(attempted={result.attempted}, failed={result.failed})"
    )


def test_normalize_doctest_source_keeps_standard_doctest() -> None:
    source = ">>> answer = 2 + 2\n>>> answer\n4"

    normalized = _normalize_doctest_source(source=source)

    assert normalized == source


def test_normalize_doctest_source_supports_comment_prefixed_blocks() -> None:
    source = "# >>> answer = 2 + 2\n# >>> answer\n# 4"

    normalized = _normalize_doctest_source(source=source)

    assert normalized == ">>> answer = 2 + 2\n>>> answer\n4"


def test_normalize_doctest_source_supports_commented_expectations() -> None:
    source = ">>> answer = 2 + 2\n>>> answer\n# 4"

    normalized = _normalize_doctest_source(source=source)

    assert normalized == ">>> answer = 2 + 2\n>>> answer\n4"
