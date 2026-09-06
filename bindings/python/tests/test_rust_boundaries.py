from pathlib import Path
import tomllib


ROOT = Path(__file__).resolve().parents[3]
PYTHON_CRATE = ROOT / "bindings" / "python"
PYTHON_CORE_CRATE = ROOT / "bindings" / "python-core"
PYTHON_TYPES_CRATE = ROOT / "bindings" / "python-types"


def _dependency_names(crate: str) -> set[str]:
    manifest = tomllib.loads((ROOT / "crates" / crate / "Cargo.toml").read_text())
    dependencies = manifest.get("dependencies", {})
    return set(dependencies)


def _manifest_dependency_names(path: Path) -> set[str]:
    manifest = tomllib.loads((path / "Cargo.toml").read_text())
    dependencies = manifest.get("dependencies", {})
    return set(dependencies)


def test_python_cargo_depends_on_shared_public_contracts_only_among_arco_crates() -> (
    None
):
    extension_deps = _manifest_dependency_names(PYTHON_CRATE)
    core_deps = _manifest_dependency_names(PYTHON_CORE_CRATE)
    types_deps = _manifest_dependency_names(PYTHON_TYPES_CRATE)

    assert {"arco-python-core", "arco-python-types", "arco-solver"} <= extension_deps
    assert {
        "arco-arrays",
        "arco-blocks",
        "arco-format",
        "arco-highs",
        "arco-model",
        "arco-solver",
    } <= core_deps
    assert {"arco-arrays", "arco-model", "arco-solver"} <= types_deps

    for forbidden in {
        "arco-arrays",
        "arco-blocks",
        "arco-format",
        "arco-highs",
        "arco-kdl",
        "arco-model",
        "arco-ops",
        "arco-validate",
    }:
        assert forbidden not in extension_deps


def test_python_package_builds_with_xpress_support_by_default() -> None:
    cargo_manifest = tomllib.loads((PYTHON_CRATE / "Cargo.toml").read_text())
    pyproject = tomllib.loads((PYTHON_CRATE / "pyproject.toml").read_text())

    assert "xpress" in cargo_manifest["features"]["default"]
    assert "xpress" in pyproject["tool"]["maturin"]["features"]


def test_release_wheel_recipe_allows_the_abi3_matrix_features() -> None:
    justfile = (ROOT / "justfile").read_text()

    assert (
        'PYTHON_WHEEL_FEATURES="${PYTHON_WHEEL_FEATURES:-pyo3/extension-module,xpress,scip-from-source}"'
        in justfile
    )

    abi3_features = 'wheel_features: "pyo3/extension-module,pyo3/abi3-py311,xpress,scip-from-source"'
    workflow = ROOT / ".github/workflows/build-packages.yml"
    assert abi3_features in workflow.read_text()


def test_cli_cargo_depends_on_arco_ops_only_among_arco_modeling_crates() -> None:
    cargo_toml = (ROOT / "crates" / "arco-cli" / "Cargo.toml").read_text()

    assert "arco-ops" in cargo_toml
    for forbidden in (
        "arco-arrays",
        "arco-blocks",
        "arco-format",
        "arco-highs",
        "arco-kdl",
        "arco-model",
        "arco-solver",
        "arco-validate",
    ):
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
        "arco_ops::solver::",
        "arco_ops::model::",
        "arco_ops::expr::",
    )

    for source in (PYTHON_CRATE / "src").rglob("*.rs"):
        content = source.read_text()
        for token in forbidden_tokens:
            assert token not in content, f"found {token!r} in {source}"


def test_model_solve_rejects_unsupported_primal_start_path() -> None:
    core = (PYTHON_CRATE / "src" / "core.rs").read_text()
    model_solve = (PYTHON_CRATE / "src" / "model_solve.rs").read_text()

    assert "primal_start is not supported on this solve path" in core
    assert "let _ = primal_start" not in core
    assert "primal_start is not supported on the model-view solve path" in model_solve
    assert "SolverError::InvalidSettings" in model_solve
    assert "_warm_start_hint_count" not in model_solve


def test_solver_backend_crates_do_not_depend_on_user_surfaces_or_runtime_facade() -> (
    None
):
    allowed_arco_deps = {"arco-model", "arco-solver"}
    forbidden_arco_deps = {
        "arco-blocks",
        "arco-cli",
        "arco-kdl",
        "arco-ops",
        "arco-python",
        "arco-validate",
    }

    for crate in ("arco-highs", "arco-scip", "arco-xpress", "arco-ipopt"):
        cargo_toml = (ROOT / "crates" / crate / "Cargo.toml").read_text()
        for dep in allowed_arco_deps:
            assert dep in cargo_toml
        for dep in forbidden_arco_deps:
            assert dep not in cargo_toml, f"{crate} must not depend on {dep}"


def test_core_building_block_crates_do_not_depend_on_user_surfaces() -> None:
    forbidden_by_crate = {
        "arco-model": {
            "arco-arrays",
            "arco-blocks",
            "arco-builtin-solvers",
            "arco-cli",
            "arco-format",
            "arco-highs",
            "arco-ipopt",
            "arco-kdl",
            "arco-ops",
            "arco-scip",
            "arco-solver",
            "arco-validate",
            "arco-xpress",
        },
        "arco-arrays": {
            "arco-blocks",
            "arco-builtin-solvers",
            "arco-cli",
            "arco-highs",
            "arco-ipopt",
            "arco-kdl",
            "arco-ops",
            "arco-scip",
            "arco-solver",
            "arco-validate",
            "arco-xpress",
        },
        "arco-blocks": {
            "arco-builtin-solvers",
            "arco-cli",
            "arco-highs",
            "arco-ipopt",
            "arco-kdl",
            "arco-scip",
            "arco-xpress",
        },
    }

    for crate, forbidden_deps in forbidden_by_crate.items():
        dependencies = _dependency_names(crate)
        unexpected = dependencies & forbidden_deps
        assert not unexpected, f"{crate} must not depend on {sorted(unexpected)}"
