from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
PYTHON_CRATE = ROOT / "bindings" / "python"


def test_python_cargo_depends_on_arco_ops_and_blocks_only_among_arco_crates() -> None:
    cargo_toml = (PYTHON_CRATE / "Cargo.toml").read_text()

    assert "arco-ops" in cargo_toml
    assert "arco-blocks" in cargo_toml
    for forbidden in ("arco-highs", "arco-solver", "arco-model"):
        assert forbidden not in cargo_toml


def test_python_sources_use_arco_ops_for_core_arco_apis() -> None:
    forbidden_tokens = (
        "extern crate arco_blocks as arco_ops",
        "extern crate arco_blocks as arco_solver",
        "extern crate arco_blocks as arco_highs",
        "arco_blocks::expr",
        "arco_blocks::model",
        "arco_blocks::solver",
        "arco_blocks::targets",
        "arco_blocks::highs",
        "arco_ops::highs",
        "arco_ops::scip",
        "arco_ops::targets",
    )

    for source in (PYTHON_CRATE / "src").rglob("*.rs"):
        content = source.read_text()
        for token in forbidden_tokens:
            assert token not in content, f"found {token!r} in {source}"
