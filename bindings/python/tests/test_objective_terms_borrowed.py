from __future__ import annotations

import pytest

import arco


def test_appended_expression_remains_reusable() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat)
    y = model.add_variable(bounds=arco.NonNegativeFloat)
    z = model.add_variable(bounds=arco.NonNegativeFloat)
    appended = x + 2.0 * y

    model.minimize(z)
    model.add_objective_terms(appended)
    model.add_objective_terms(appended)

    snapshot = model.inspect()
    assert snapshot.objective is not None
    assert snapshot.objective.terms == [(0, 2.0), (1, 4.0), (2, 1.0)]


def test_invalid_appended_expression_does_not_mutate_objective() -> None:
    model = arco.Model()
    x = model.add_variable(bounds=arco.NonNegativeFloat)
    y = model.add_variable(bounds=arco.NonNegativeFloat)
    model.minimize(x)

    with pytest.raises(arco.ExprCoefficientError, match="finite"):
        model.add_objective_terms(float("nan") * y)

    snapshot = model.inspect()
    assert snapshot.objective is not None
    assert snapshot.objective.terms == [(0, 1.0)]
