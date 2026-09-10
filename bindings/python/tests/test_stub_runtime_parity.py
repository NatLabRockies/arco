"""Contracts binding `arco.pyi` to the objects the package actually exports.

`arco/arco.pyi` describes the compiled extension module `arco.arco`. When it
declares a class or function that the extension does not export, a type checker
resolving `arco.<name>` sees both the stub declaration and the pure-Python
definition in `arco/__init__.py`. The inferred type becomes a union of two
unrelated classes, and every operator declared on only one side is rejected.
"""

from __future__ import annotations

import ast
from pathlib import Path

import arco
from arco import arco as extension

STUB_PATH = Path(arco.__file__).resolve().parent / "arco.pyi"

# Type aliases and TypedDicts are checker-only declarations with no runtime
# attribute on the extension module.
CHECKER_ONLY_STUB_NAMES = frozenset(
    {
        "AxisSelection",
        "BlockFnT",
        "BoundValue",
        "CooExport",
        "CrsExport",
        "CscExport",
        "IndexMember",
        "JsonPrimitive",
        "JsonValue",
        "LinearOperand",
        "NumericOperand",
        "SolverInfo",
        "SolverRuntimeInfo",
    }
)


def _stub_top_level_names() -> set[str]:
    tree = ast.parse(STUB_PATH.read_text())
    names: set[str] = set()
    for node in tree.body:
        if isinstance(node, (ast.ClassDef, ast.FunctionDef)):
            names.add(node.name)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            names.add(node.target.id)
    return {name for name in names if not name.startswith("_")}


def _stub_class_names() -> set[str]:
    tree = ast.parse(STUB_PATH.read_text())
    return {
        node.name
        for node in tree.body
        if isinstance(node, ast.ClassDef) and not node.name.startswith("_")
    }


def _pure_python_class_names() -> set[str]:
    tree = ast.parse(Path(arco.__file__).resolve().read_text())
    return {
        node.name
        for node in tree.body
        if isinstance(node, ast.ClassDef) and not node.name.startswith("_")
    }


def test_stub_declares_only_names_the_extension_exports() -> None:
    declared = _stub_top_level_names() - CHECKER_ONLY_STUB_NAMES
    missing = sorted(name for name in declared if not hasattr(extension, name))
    assert missing == [], (
        "arco.pyi documents the compiled extension `arco.arco`; these names are "
        f"declared there but only exist in arco/__init__.py: {missing}"
    )


def test_stub_does_not_shadow_pure_python_classes() -> None:
    # A function defined in both places resolves to the pure-Python definition,
    # but a class does not: `arco.<Name>` becomes a union of two unrelated
    # classes, and any operator declared on only one side is rejected.
    shadowed = sorted(_stub_class_names() & _pure_python_class_names())
    assert shadowed == [], (
        "these classes are defined in arco/__init__.py, so declaring them in "
        f"arco.pyi makes `arco.<name>` resolve to a two-class union: {shadowed}"
    )


def test_package_exports_pure_python_labeled_param_api() -> None:
    for name in ("ParamArray", "param", "error_code", "diagnostic_codes", "block"):
        assert hasattr(arco, name), f"arco.{name} is part of the public API"
        assert name in arco.__all__, f"arco.{name} must be exported via __all__"
