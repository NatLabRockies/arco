from __future__ import annotations

from time import perf_counter
from typing import Any

import numpy as np
from arco import Bounds, HiGHS, IndexSet, Model as ArcoModel
from linopy import Model as LinopyModel
from pyomo.environ import ConcreteModel, Constraint, Objective, Set, Var
from pyomo.opt import SolverFactory

from arco_benchmarks.model import num_variables


def _build_pyomo(n: int) -> ConcreteModel:
    model = ConcreteModel()
    model.i = Set(initialize=range(n))
    model.j = Set(initialize=range(n))
    model.x = Var(model.i, model.j, bounds=(None, None))
    model.y = Var(model.i, model.j, bounds=(None, None))

    def bound1(m: ConcreteModel, i: int, j: int) -> Any:
        return m.x[(i, j)] - m.y[(i, j)] >= i

    def bound2(m: ConcreteModel, i: int, j: int) -> Any:
        return m.x[(i, j)] + m.y[(i, j)] >= 0

    def objective(m: ConcreteModel) -> Any:
        return sum(2 * m.x[(i, j)] + m.y[(i, j)] for i in m.i for j in m.j)

    model.con1 = Constraint(model.i, model.j, rule=bound1)
    model.con2 = Constraint(model.i, model.j, rule=bound2)
    model.obj = Objective(rule=objective)
    return model


def _solve_pyomo(model: ConcreteModel, solver: str) -> None:
    solver_name = "appsi_highs" if solver == "highs" else solver
    opt = SolverFactory(solver_name)
    result = opt.solve(model)
    if result.solver.status is None:
        raise RuntimeError("Pyomo solver did not return a status")


def _build_linopy(n: int) -> LinopyModel:
    model = LinopyModel()
    idx = np.arange(n)
    x = model.add_variables(coords=[idx, idx], name="x")
    y = model.add_variables(coords=[idx, idx], name="y")
    model.add_constraints(x - y >= idx)
    model.add_constraints(x + y >= 0)
    model.add_objective((2 * x).sum() + y.sum())
    return model


def _solve_linopy(model: LinopyModel, solver: str) -> None:
    model.solve(solver_name=solver)


def _build_arco(n: int) -> ArcoModel:
    model = ArcoModel()
    i_set = IndexSet("i", members=list(range(n)))
    j_set = IndexSet("j", members=list(range(n)))
    bounds = Bounds(lower=-1.0e20, upper=1.0e20)
    x = model.add_variables(i_set, j_set, bounds=bounds, name="x")
    y = model.add_variables(i_set, j_set, bounds=bounds, name="y")
    x_view: Any = x
    y_view: Any = y

    model.add_constraints(x_view - y_view >= i_set)
    model.add_constraints(x_view + y_view >= 0.0)
    model.minimize((2 * x_view + y_view).sum())
    return model


def _solve_arco(model: ArcoModel, solver: str) -> None:
    if solver != "highs":
        raise ValueError("Arco runner currently supports only solver='highs'")
    model.solve(solver=HiGHS())


def _build_pyoptinterface(n: int) -> Any:
    import pyoptinterface as poi
    from pyoptinterface import highs as poi_highs

    model = poi_highs.Model()
    idx = range(n)
    x = model.add_variables(idx, idx, name="x")
    y = model.add_variables(idx, idx, name="y")

    for i in idx:
        for j in idx:
            model.add_linear_constraint(x[i, j] - y[i, j], poi.Geq, float(i))
            model.add_linear_constraint(x[i, j] + y[i, j], poi.Geq, 0.0)

    objective = poi.quicksum(
        (x[i, j] for i in idx for j in idx), lambda var: 2 * var
    ) + poi.quicksum(y.values())
    model.set_objective(objective, poi.ObjectiveSense.Minimize)
    return model


def _solve_pyoptinterface(model: Any, solver: str) -> None:
    if solver != "highs":
        raise ValueError("PyOptInterface runner currently supports only solver='highs'")
    model.optimize()


def _build_pulp(n: int) -> Any:
    import pulp

    model = pulp.LpProblem("arco_benchmark", pulp.LpMinimize)
    x = {
        (i, j): pulp.LpVariable(f"x_{i}_{j}", lowBound=None, upBound=None)
        for i in range(n)
        for j in range(n)
    }
    y = {
        (i, j): pulp.LpVariable(f"y_{i}_{j}", lowBound=None, upBound=None)
        for i in range(n)
        for j in range(n)
    }

    for i in range(n):
        for j in range(n):
            model += x[(i, j)] - y[(i, j)] >= float(i)
            model += x[(i, j)] + y[(i, j)] >= 0.0

    model += pulp.lpSum(2 * x[(i, j)] + y[(i, j)] for i in range(n) for j in range(n))
    return model


def _solve_pulp(model: Any, solver: str) -> None:
    import pulp

    if solver != "highs":
        raise ValueError("PuLP runner currently supports only solver='highs'")
    model.solve(pulp.HiGHS(msg=False))


def run_point(*, tool: str, phase: str, n: int, solver: str) -> dict[str, Any]:
    if n <= 0:
        raise ValueError("n must be positive")
    if phase not in {"build", "solve"}:
        raise ValueError("phase must be 'build' or 'solve'")

    start = perf_counter()

    if tool == "pyomo":
        model = _build_pyomo(n)
        if phase == "solve":
            _solve_pyomo(model, solver)
    elif tool == "linopy":
        model = _build_linopy(n)
        if phase == "solve":
            _solve_linopy(model, solver)
    elif tool == "arco":
        model = _build_arco(n)
        if phase == "solve":
            _solve_arco(model, solver)
    elif tool == "pyoptinterface":
        model = _build_pyoptinterface(n)
        if phase == "solve":
            _solve_pyoptinterface(model, solver)
    elif tool == "pulp":
        model = _build_pulp(n)
        if phase == "solve":
            _solve_pulp(model, solver)
    else:
        raise ValueError(f"Unsupported tool: {tool}")

    elapsed_seconds = perf_counter() - start
    return {
        "tool": tool,
        "phase": phase,
        "n": n,
        "solver": solver,
        "num_variables": num_variables(n),
        "elapsed_seconds": elapsed_seconds,
    }
