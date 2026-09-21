"""Type-inference contracts for the labeled array modeling API.

The ReEDS-style formulations in `examples/reeds-benchmark/formulation.py` are the
reference for what the annotations must support. Every expression here must type
check with no suppression comment, so downstream projects can build models under
a strict type checker.

Each case runs `ty` over a generated snippet and asserts both the absence of
diagnostics and the inferred type, so a stub that silences errors by widening to
`object` or `Any` still fails.
"""

from __future__ import annotations

import os
from pathlib import Path
import re
import shutil
import subprocess
import sysconfig
import textwrap

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
PYTHON_BINDINGS = REPO_ROOT / "bindings" / "python"
SITE_PACKAGES = Path(sysconfig.get_paths()["purelib"])

PREAMBLE = """
from typing import reveal_type

import numpy as np

import arco

I = arco.IndexSet(name="i", members=["gas", "wind"])
R = arco.IndexSet(name="r", members=["north", "south"])
H = arco.IndexSet(name="h", members=[0, 1, 2])
T = arco.IndexSet(name="t", members=[2030, 2035])
R_from = R.alias("from")
R_to = R.alias("to")

model = arco.Model()
cap = model.add_variables(I, R, T, bounds=arco.NonNegativeFloat, name="CAP")
gen = model.add_variables(I, R, H, T, bounds=arco.NonNegativeFloat, name="GEN")
flow = model.add_variables(R_from, R_to, H, T, bounds=arco.NonNegativeFloat, name="FLOW")
cf = arco.param(np.ones((2, 2, 3)), I, R, H)
emit_rate = arco.param(np.ones(2), I)
hours_weight = arco.param(np.ones(3), H)
emit_cap = arco.param(np.ones(2), T)
peak_load = arco.param(np.ones((2, 2)), R, T)
minloadfrac = arco.param(np.ones(2), I)
valcap = arco.param(np.ones((2, 2, 2), dtype=bool), I, R, T)

# A reduction's rank depends on how many axes it collapses, so narrow the result
# once with `isinstance` before using members that only exist on the array form.
imports_by_region = flow @ R_from
assert isinstance(imports_by_region, arco.ExprArray)

emissions_by_year = (emit_rate * hours_weight * gen) @ (I, R, H)
assert isinstance(emissions_by_year, arco.ExprArray)

capacity_by_region = cap @ I
assert isinstance(capacity_by_region, arco.ExprArray)
"""

# Each case is (identifier, expression, expected inferred type).
#
# `ParamArray | float` is the honest result of elementwise parameter arithmetic:
# `arco.param(scalar)` builds a rank-0 array whose operations return plain
# floats. Such an array raises on every labeled operation, so formulations never
# hold one, and the union still supports the operators used here.
LABELED_ARRAY_CASES = [
    ("param_times_variables", "cf * cap", "ExprArray"),
    ("variables_times_param", "cap * cf", "ExprArray"),
    ("param_scaled_by_float", "2.0 * emit_rate", "ParamArray | float"),
    ("param_compared_to_int", "minloadfrac > 0", "ParamArray | float"),
    ("param_mask_intersection", "valcap & (minloadfrac > 0)", "ParamArray | float"),
    ("param_mask_negation", "~valcap", "ParamArray"),
    ("variables_reduced_over_axis", "gen @ I", "Expr | ExprArray"),
    ("variables_reduced_over_axes", "gen @ (I, R, H)", "Expr | ExprArray"),
    ("variables_shifted_over_axis", "gen >> I", "Expr | ExprArray"),
    ("expressions_reduced_over_axis", "(hours_weight * gen) @ H", "Expr | ExprArray"),
    ("variables_summed_over_axis", "gen.sum(over=I)", "Expr | ExprArray"),
    ("variables_summed_over_all_axes", "gen.sum()", "Expr"),
    ("variables_differenced", "gen.diff(over=H)", "ExprArray"),
    ("variables_accumulated", "cap.cumsum(over=T)", "ExprArray"),
    ("variables_rolled", "gen.roll(shift=-1, over=H)", "ExprArray"),
    (
        "narrowed_reduction_is_relabelable",
        "imports_by_region.relabel_axis(R_to, R)",
        "ExprArray",
    ),
    ("constraint_from_param_bound", "gen <= cf * cap", "ConstraintArray"),
    ("constraint_from_reduction", "emissions_by_year <= emit_cap", "ConstraintArray"),
    (
        "constraint_from_reduced_capacity",
        "capacity_by_region >= 1.15 * peak_load",
        "ConstraintArray",
    ),
    ("constraint_from_ramping", "gen.diff(over=H) >= 0.0", "ConstraintArray"),
    ("objective_scalar_sum", "(emit_rate * cap).sum()", "Expr"),
]


DIAGNOSTIC_PATTERN = re.compile(
    r"^(?P<path>[^:]+):(?P<line>\d+):(?P<column>\d+): "
    r"(?P<severity>\w+)\[(?P<rule>[\w-]+)\] (?P<message>.*)$"
)


def _ty_diagnostics(source: str, tmp_path: Path) -> list[re.Match[str]]:
    """Return ty's diagnostics for one snippet checked against this arco tree."""
    snippet = tmp_path / "snippet.py"
    snippet.write_text(source)
    uv = shutil.which("uv")
    if uv is None:
        pytest.skip("uv is required to run the type checker")
    if not (SITE_PACKAGES / "numpy").is_dir():
        pytest.skip("numpy must be installed in the test environment")

    completed = subprocess.run(
        [
            uv,
            "run",
            "--no-project",
            "--with",
            "ty",
            "ty",
            "check",
            "--python-version",
            "3.11",
            # arco resolves from this working tree, so the stub under test wins
            # over any installed copy; numpy resolves from the test environment.
            "--extra-search-path",
            str(PYTHON_BINDINGS),
            "--extra-search-path",
            str(SITE_PACKAGES),
            "--output-format",
            "concise",
            str(snippet),
        ],
        cwd=tmp_path,
        env={**os.environ, "NO_COLOR": "1"},
        capture_output=True,
        text=True,
        timeout=300,
    )
    if not completed.stdout.strip():
        raise AssertionError(f"ty produced no output: {completed.stderr}")

    diagnostics = [
        match
        for line in completed.stdout.splitlines()
        if (match := DIAGNOSTIC_PATTERN.match(line)) is not None
    ]
    if not diagnostics:
        raise AssertionError(f"could not parse ty output:\n{completed.stdout}")
    return diagnostics


@pytest.mark.parametrize(
    ("case_id", "expression", "expected_type"),
    LABELED_ARRAY_CASES,
    ids=[case[0] for case in LABELED_ARRAY_CASES],
)
def test_labeled_array_expression_types_without_suppressions(
    case_id: str,
    expression: str,
    expected_type: str,
    tmp_path: Path,
) -> None:
    source = f"{PREAMBLE}\nreveal_type({expression})\n"
    diagnostics = _ty_diagnostics(textwrap.dedent(source), tmp_path)

    errors = [
        diagnostic["message"]
        for diagnostic in diagnostics
        if diagnostic["severity"] == "error"
    ]
    assert errors == [], (
        f"{case_id}: `{expression}` requires a suppression comment: "
        + "; ".join(errors)
    )

    revealed = [
        diagnostic["message"]
        for diagnostic in diagnostics
        if diagnostic["rule"] == "revealed-type"
    ]
    assert len(revealed) == 1, f"{case_id}: expected one revealed type, got {revealed}"
    assert f"`{expected_type}`" in revealed[0], (
        f"{case_id}: `{expression}` was inferred as {revealed[0]}, "
        f"expected `{expected_type}`"
    )
