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
RAW_EXPERT_CALL_PATTERNS: tuple[str, ...] = (
    "Model.from_csc(",
    "model.export_csc(",
    "model.export_crs(",
    "model.export_coo(",
    "export_csc(",
    "export_crs(",
    "export_coo(",
    "model.set_coefficient(",
    "model.set_objective(",
    "model.set_variable_name(",
    "model.set_constraint_name(",
    "set_coefficient(",
    "set_objective(",
    "set_variable_name(",
    "set_constraint_name(",
    "solution.get_primal(",
    "solution.get_constraint_dual(",
    "solution.get_variable_dual(",
    "result.get_primal(",
    "result.get_constraint_dual(",
    "result.get_variable_dual(",
    "result.primal(",
    "result.constraint_dual(",
    "result.variable_dual(",
    "get_primal(index=",
    "get_constraint_dual(index=",
    "get_variable_dual(index=",
    "solution.primal_values",
    "solution.variable_duals",
    "solution.constraint_duals",
    ".primal_values",
    ".variable_duals",
    ".constraint_duals",
    "num_primal_values(",
    "num_variable_duals(",
    "num_constraint_duals(",
)
LEGACY_BEGINNER_CALL_PATTERNS: tuple[str, ...] = (
    "arco.Set(",
    "model.control(",
    "model.variable(",
    "model.constraint(",
)


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


def test_inspect_model_guide_keeps_raw_sparse_exports_in_expert_path() -> None:
    inspect_guide = (DOCS_DIR / "how-to" / "inspect-model.md").read_text(
        encoding="utf-8"
    )

    assert "[Use Expert APIs](./use-expert-apis.md)" in inspect_guide
    assert ">>> model.export_csc()" not in inspect_guide
    assert ">>> model.export_crs()" not in inspect_guide
    assert ">>> model.export_coo()" not in inspect_guide


def test_beginner_docs_doctests_avoid_raw_expert_calls() -> None:
    allowed_paths = {
        DOCS_DIR / "how-to" / "use-expert-apis.md",
        DOCS_DIR / "how-to" / "add-solver-backend.md",
    }
    checked_prefixes = (
        DOCS_DIR / "how-to",
        DOCS_DIR / "tutorials",
    )
    checked_files = {
        DOCS_DIR / "README.md",
    }

    for markdown_file in _iter_markdown_files():
        if not any(
            markdown_file.is_relative_to(prefix) for prefix in checked_prefixes
        ) and markdown_file not in checked_files:
            continue
        if markdown_file in allowed_paths:
            continue

        file_text = markdown_file.read_text(encoding="utf-8")
        for pattern in RAW_EXPERT_CALL_PATTERNS:
            assert pattern not in file_text, (
                f"{markdown_file.relative_to(REPO_ROOT)} includes expert raw API "
                f"usage `{pattern}` outside expert-only docs"
            )


def test_beginner_docs_use_canonical_ladder_vocabulary() -> None:
    allowed_paths = {
        DOCS_DIR / "how-to" / "use-expert-apis.md",
        DOCS_DIR / "how-to" / "add-solver-backend.md",
    }
    checked_prefixes = (
        DOCS_DIR / "how-to",
        DOCS_DIR / "tutorials",
    )
    checked_files = {
        DOCS_DIR / "README.md",
    }

    for markdown_file in _iter_markdown_files():
        if not any(
            markdown_file.is_relative_to(prefix) for prefix in checked_prefixes
        ) and markdown_file not in checked_files:
            continue
        if markdown_file in allowed_paths:
            continue

        file_text = markdown_file.read_text(encoding="utf-8")
        for pattern in LEGACY_BEGINNER_CALL_PATTERNS:
            assert pattern not in file_text, (
                f"{markdown_file.relative_to(REPO_ROOT)} includes legacy shorthand "
                f"`{pattern}` outside expert-only docs"
            )

        # Keep beginner docs on explicit Bounds/BoundType instead of
        # lower/upper shortcut kwargs for variable constructors.
        invalid_bounds_shorthand = (
            re.search(r"add_variable\s*\([^)]*\\blower\s*=", file_text) is not None
            or re.search(r"add_variable\s*\([^)]*\\bupper\s*=", file_text) is not None
            or re.search(r"add_variables\s*\([^)]*\\blower\s*=", file_text) is not None
            or re.search(r"add_variables\s*\([^)]*\\bupper\s*=", file_text)
            is not None
        )
        assert not invalid_bounds_shorthand, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes lower/upper "
            "shortcut kwargs for variable constructors outside expert-only docs"
        )

        positional_named_lookup = (
            re.search(r'get_variable\s*\(\s*["\']', file_text) is not None
            or re.search(r'get_constraint\s*\(\s*["\']', file_text) is not None
            or re.search(
                r"get_variable\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))", file_text
            )
            is not None
            or re.search(
                r"get_constraint\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))", file_text
            )
            is not None
        )
        assert not positional_named_lookup, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional named "
            "lookups; use keyword-only get_variable(name=...) and "
            "get_constraint(name=...)"
        )

        positional_solve_or_inspect = (
            re.search(r"solve\s*\(\s*['\"]", file_text) is not None
            or re.search(r"solve\s*\(\s*[-+]?\d", file_text) is not None
            or re.search(r"solve\s*\(\s*(True|False|None)\b", file_text) is not None
            or re.search(r"solve\s*\(\s*[A-Z][A-Za-z0-9_]*\s*\(", file_text)
            is not None
            or re.search(r"solve\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))", file_text)
            is not None
            or re.search(r"inspect\s*\(\s*['\"]", file_text) is not None
            or re.search(r"inspect\s*\(\s*[-+]?\d", file_text) is not None
            or re.search(r"inspect\s*\(\s*(True|False|None)\b", file_text) is not None
            or re.search(r"inspect\s*\(\s*[A-Z][A-Za-z0-9_]*\s*\(", file_text)
            is not None
            or re.search(
                r"inspect\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))", file_text
            )
            is not None
        )
        assert not positional_solve_or_inspect, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional solve/inspect "
            "arguments; use keyword-only solve(...) and inspect(...) configuration"
        )

        positional_add_constraints_config = (
            re.search(
                r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*['\"]",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[-+]?\d",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*(True|False|None)\b",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*['\"][^'\"]*['\"]\s*,\s*[-+]?\d+(?:\.\d+)?\s*,\s*None\s*,\s*['\"]",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
                file_text,
            )
            is not None
        )
        assert not positional_add_constraints_config, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional add_constraints "
            "configuration; use keyword-only sense=... and rhs=..."
        )

        positional_add_constraint_config = (
            re.search(
                r"add_constraint\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,\s*[^,\n][^,\n]*\)",
                file_text,
            )
            is not None
            or
            re.search(
                r"add_constraint\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*arco\.Bounds\s*\(",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraint\s*\([^)]*,\s*['\"]",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraint\s*\([^)]*,\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
                file_text,
            )
            is not None
            or re.search(
                r"add_constraint\s*\([^)]*,\s*arco\.Bounds\s*\([^)]*\)\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
                file_text,
            )
            is not None
        )
        assert not positional_add_constraint_config, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional add_constraint "
            "configuration; use keyword-only bounds=... and name=..."
        )

        positional_param_axes = (
            re.search(r"param\s*\(\s*[^,\n][^,\n]*,\s*\(", file_text) is not None
            or re.search(
                r"param\s*\(\s*[^,\n][^,\n]*,\s*[A-Za-z_][A-Za-z0-9_]*\s*\)",
                file_text,
            )
            is not None
            or re.search(
                r"param\s*\(\s*[^,\n][^,\n]*,\s*[A-Za-z_][A-Za-z0-9_]*\s*,",
                file_text,
            )
            is not None
        )
        assert not positional_param_axes, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional param axes "
            "usage; use keyword-only axes=..."
        )

        positional_param_name_first = (
            re.search(r'param\s*\(\s*["\']', file_text) is not None
        )
        assert not positional_param_name_first, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional param "
            "name-first usage; keep values positional and use name=..."
        )

        positional_model_constructor_config = (
            re.search(r"Model\s*\(\s*arco\.[A-Za-z_][A-Za-z0-9_\.]*\s*\)", file_text)
            is not None
            or (
                re.search(r"Model\s*\(\s*(solver|simplify_level)\s*=", file_text)
                is None
            )
            and re.search(r"Model\s*\(\s*[^)\s]", file_text) is not None
        )
        assert not positional_model_constructor_config, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional Model() "
            "configuration; use keyword-only simplify_level=.../solver=..."
        )

        positional_variable_constructor = (
            re.search(
                r"add_variable\s*\(\s*arco\.[A-Za-z_][A-Za-z0-9_\.]*",
                file_text,
            )
            is not None
            or re.search(
                r"add_variables\s*\(\s*\(",
                file_text,
            )
            is not None
            or re.search(
                r"add_variables\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,", file_text
            )
            is not None
        )
        assert not positional_variable_constructor, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional "
            "add_variable/add_variables usage; use keyword-only constructor "
            "configuration (bounds=..., axes=..., etc.)"
        )

        positional_bounds_constructor = (
            re.search(
                r"arco\.Bounds\s*\(\s*[A-Za-z0-9_\"'\.\-]+\s*,",
                file_text,
            )
            is not None
        )
        assert not positional_bounds_constructor, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional Bounds "
            "constructor usage; use keyword-only Bounds(lower=..., upper=...)"
        )

        positional_objective_name_first = (
            re.search(r'minimize\s*\(\s*["\']', file_text) is not None
            or re.search(r'maximize\s*\(\s*["\']', file_text) is not None
            or re.search(
                r"minimize\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,", file_text
            )
            is not None
            or re.search(
                r"maximize\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,", file_text
            )
            is not None
        )
        assert not positional_objective_name_first, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional objective "
            "name-first usage; keep expression positional and use name=..."
        )

        positional_objective_name_second = (
            re.search(r'minimize\s*\([^)]*,\s*["\']', file_text) is not None
            or re.search(r'maximize\s*\([^)]*,\s*["\']', file_text) is not None
            or re.search(
                r"minimize\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
                file_text,
            )
            is not None
            or re.search(
                r"maximize\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
                file_text,
            )
            is not None
        )
        assert not positional_objective_name_second, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional objective "
            "name argument usage; keep objective naming keyword-only via name=..."
        )

        positional_index_set_name = (
            re.search(r'IndexSet\s*\(\s*["\']', file_text) is not None
            or re.search(r"IndexSet\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,", file_text)
            is not None
        )
        assert not positional_index_set_name, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional IndexSet "
            "name usage; keep IndexSet naming keyword-only via name=..."
        )

        positional_index_set_config = (
            re.search(
                r"IndexSet\s*\(\s*name\s*=\s*[^,\)\n]+,\s*[^A-Za-z_\s]",
                file_text,
            )
            is not None
        )
        assert not positional_index_set_config, (
            f"{markdown_file.relative_to(REPO_ROOT)} includes positional IndexSet "
            "configuration; use keyword-only size=... and members=..."
        )


def test_docs_guard_detects_positional_model_constructor_configs() -> None:
    assert (
        re.search(r"Model\s*\(\s*(solver|simplify_level)\s*=", "m = Model(123)")
        is None
    )
    positional_model_constructor_config = (
        re.search(
            r"Model\s*\(\s*arco\.[A-Za-z_][A-Za-z0-9_\.]*\s*\)", "m = Model(123)"
        )
        is not None
        or (
            re.search(
                r"Model\s*\(\s*(solver|simplify_level)\s*=", "m = Model(123)"
            )
            is None
        )
        and re.search(r"Model\s*\(\s*[^)\s]", "m = Model(123)") is not None
    )
    assert positional_model_constructor_config


def test_docs_guard_detects_positional_model_constructor_variable_config() -> None:
    positional_model_constructor_config = (
        re.search(
            r"Model\s*\(\s*arco\.[A-Za-z_][A-Za-z0-9_\.]*\s*\)",
            "m = Model(config)",
        )
        is not None
        or (
            re.search(
                r"Model\s*\(\s*(solver|simplify_level)\s*=", "m = Model(config)"
            )
            is None
        )
        and re.search(r"Model\s*\(\s*[^)\s]", "m = Model(config)") is not None
        or (
            re.search(
                r"Model\s*\(\s*(solver|simplify_level)\s*=",
                "m = Model(arco.SimplifyLevel.NONE, solver_var)",
            )
            is None
            and re.search(
                r"Model\s*\(\s*[^)\s]",
                "m = Model(arco.SimplifyLevel.NONE, solver_var)",
            )
            is not None
        )
        or (
            re.search(
                r"Model\s*\(\s*(solver|simplify_level)\s*=",
                "m = Model(simplify_level_var, arco.HiGHS())",
            )
            is None
            and re.search(
                r"Model\s*\(\s*[^)\s]",
                "m = Model(simplify_level_var, arco.HiGHS())",
            )
            is not None
        )
    )
    assert positional_model_constructor_config


def test_docs_guard_detects_positional_index_set_name_usage() -> None:
    positional_index_set_name = (
        re.search(r'IndexSet\s*\(\s*["\']', 'asset = arco.IndexSet("asset", size=3)')
        is not None
        or re.search(
            r"IndexSet\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,",
            "asset = arco.IndexSet(asset_name, size=3)",
        )
        is not None
        or re.search(
            r"IndexSet\s*\(\s*['\"][^'\"]*['\"]\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "asset = arco.IndexSet('asset', size_var)",
        )
        is not None
        or re.search(
            r"IndexSet\s*\(\s*['\"][^'\"]*['\"]\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "asset = arco.IndexSet('asset', members_var)",
        )
        is not None
    )
    assert positional_index_set_name


def test_docs_guard_detects_positional_index_set_size_or_members_variable_usage() -> None:
    positional_index_set_size_or_members = (
        re.search(
            r"IndexSet\s*\(\s*['\"][^'\"]*['\"]\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "asset = arco.IndexSet('asset', size_var)",
        )
        is not None
        or re.search(
            r"IndexSet\s*\(\s*['\"][^'\"]*['\"]\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "asset = arco.IndexSet('asset', members_var)",
        )
        is not None
    )
    assert positional_index_set_size_or_members


def test_docs_guard_detects_positional_index_set_config_usage() -> None:
    positional_index_set_config = (
        re.search(
            r"IndexSet\s*\(\s*name\s*=\s*[^,\)\n]+,\s*[^A-Za-z_\s]",
            "idx = arco.IndexSet(name='asset', ['a', 'b'])",
        )
        is not None
    )
    assert positional_index_set_config


def test_docs_guard_detects_positional_bounds_constructor_usage() -> None:
    positional_bounds_constructor = (
        re.search(
            r"arco\.Bounds\s*\(\s*[A-Za-z0-9_\"'\.\-]+\s*,",
            "x = arco.Bounds(0.0, 10.0)",
        )
        is not None
    )
    assert positional_bounds_constructor


def test_docs_guard_detects_positional_solve_variable_usage() -> None:
    positional_solve_variable = (
        re.search(
            r"solve\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "result = model.solve(solver)",
        )
        is not None
        or re.search(
            r"solve\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "result = model.solve(solver, log_to_console)",
        )
        is not None
        or re.search(
            r"solve\s*\(\s*[A-Z][A-Za-z0-9_]*\s*\(",
            "result = model.solve(HiGHS(), log_to_console)",
        )
        is not None
        or re.search(
            r"solve\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "result = model.solve(solver, False)",
        )
        is not None
    )
    assert positional_solve_variable


def test_docs_guard_detects_positional_inspect_variable_usage() -> None:
    positional_inspect_variable = (
        re.search(
            r"inspect\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "snapshot = model.inspect(include_coeffs)",
        )
        is not None
        or re.search(
            r"inspect\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "snapshot = model.inspect(payload)",
        )
        is not None
    )
    assert positional_inspect_variable


def test_docs_guard_detects_positional_named_lookup_variable_usage() -> None:
    positional_named_lookup_variable = (
        re.search(
            r"get_variable\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "x = model.get_variable(variable_name)",
        )
        is not None
        or re.search(
            r"get_constraint\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "c = model.get_constraint(constraint_name)",
        )
        is not None
        or re.search(
            r"get_variable\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "x = model.get_variable(variable_name, extra_arg)",
        )
        is not None
        or re.search(
            r"get_constraint\s*\(\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "c = model.get_constraint(constraint_name, extra_arg)",
        )
        is not None
        or re.search(
            r'get_variable\s*\(\s*["\'][^"\']*["\']\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)',
            "x = model.get_variable('x', extra_var)",
        )
        is not None
        or re.search(
            r'get_constraint\s*\(\s*["\'][^"\']*["\']\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)',
            "c = model.get_constraint('minimum', extra_var)",
        )
        is not None
    )
    assert positional_named_lookup_variable


def test_docs_guard_detects_raw_result_vector_api_usage() -> None:
    raw_result_vector_api = (
        "result.get_primal(" in "value = result.get_primal(0)"
        or "result.get_constraint_dual(" in "dual = result.get_constraint_dual(0)"
        or "result.get_variable_dual(" in "dual = result.get_variable_dual(0)"
        or "result.primal(" in "value = result.primal(x)"
        or "result.constraint_dual(" in "dual = result.constraint_dual(c)"
        or "result.variable_dual(" in "dual = result.variable_dual(x)"
    )
    assert raw_result_vector_api


def test_docs_guard_detects_positional_param_axes_variable_usage() -> None:
    positional_param_axes_variable = (
        re.search(
            r"param\s*\(\s*[^,\n][^,\n]*,\s*[A-Za-z_][A-Za-z0-9_]*\s*\)",
            "cost = arco.param(values, axes)",
        )
        is not None
        or re.search(
            r"param\s*\(\s*[^,\n][^,\n]*,\s*[A-Za-z_][A-Za-z0-9_]*\s*,",
            "cost = arco.param(values, axes, name)",
        )
        is not None
    )
    assert positional_param_axes_variable


def test_docs_guard_detects_positional_objective_variable_name_usage() -> None:
    positional_objective_variable_name = (
        re.search(
            r"minimize\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,",
            "model.minimize(objective_name, expr)",
        )
        is not None
        or re.search(
            r"maximize\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,",
            "model.maximize(profit_name, expr)",
        )
        is not None
        or re.search(
            r"minimize\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "model.minimize(expr, objective_name)",
        )
        is not None
        or re.search(
            r"maximize\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "model.maximize(expr, profit_name)",
        )
        is not None
    )
    assert positional_objective_variable_name


def test_docs_guard_detects_positional_constraint_variable_usage() -> None:
    positional_constraint_variable = (
        re.search(
            r"add_constraint\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,\s*[^,\n][^,\n]*\)",
            "model.add_constraint(constraint_name, expr)",
        )
        is not None
        or
        re.search(
            r"add_constraint\s*\([^)]*,\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "model.add_constraint(expr, constraint_name)",
        )
        is not None
        or re.search(
            r"add_constraint\s*\([^)]*,\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "model.add_constraint(expr, bounds_obj, constraint_name)",
        )
        is not None
        or re.search(
            r"add_constraint\s*\([^)]*,\s*arco\.Bounds\s*\([^)]*\)\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "model.add_constraint(expr, arco.Bounds(lower=0.0, upper=1.0), constraint_name)",
        )
        is not None
        or re.search(
            r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "model.add_constraints(output, sense, rhs)",
        )
        is not None
        or re.search(
            r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*(?:,|\))",
            "model.add_constraints(output, sense, rhs, active)",
        )
        is not None
        or re.search(
            r"add_constraints\s*\(\s*[A-Za-z_][A-Za-z0-9_\.]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*,\s*[a-z_][A-Za-z0-9_]*\s*\)",
            "model.add_constraints(output, sense, rhs, active, name)",
        )
        is not None
    )
    assert positional_constraint_variable


def test_docs_guard_detects_positional_add_variables_axes_variable_usage() -> None:
    positional_add_variables_axes_variable = (
        re.search(
            r"add_variables\s*\(\s*[a-z_][A-Za-z0-9_]*\s*,",
            "x = model.add_variables(axes, bounds=arco.NonNegativeFloat)",
        )
        is not None
    )
    assert positional_add_variables_axes_variable
