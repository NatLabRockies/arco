# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "arco",
# ]
#
# [tool.uv.sources]
# arco = { path = "../../bindings/python" }
# ///

from __future__ import annotations

import argparse
import json
from collections.abc import Sequence
from typing import Any, Callable

import arco


def _positive_int(value: str) -> int:
    parsed = int(value)
    if parsed <= 0:
        raise argparse.ArgumentTypeError("--n must be a positive integer")
    return parsed


def _resolve_row_values(
    *,
    n: int | None,
    row_values: Sequence[float] | None,
) -> tuple[list[float], int]:
    if row_values is None:
        if n is None:
            raise ValueError("n must be provided when row_values is omitted")
        if n <= 0:
            raise ValueError("n must be positive")
        return [float(value) for value in range(n)], n

    resolved_row_values = [float(value) for value in row_values]
    if not resolved_row_values:
        raise ValueError("row_values must not be empty")

    if n is None:
        resolved_n = len(resolved_row_values)
    else:
        if n <= 0:
            raise ValueError("n must be positive")
        if len(resolved_row_values) != n:
            raise ValueError("row_values length must match n")
        resolved_n = n

    return resolved_row_values, resolved_n


def build_model(
    *, n: int | None = None, row_values: Sequence[float] | None = None
) -> arco.Model:
    resolved_row_values, resolved_n = _resolve_row_values(n=n, row_values=row_values)

    model = arco.Model()
    row = arco.IndexSet("row", members=resolved_row_values)
    col = arco.IndexSet("col", members=list(range(resolved_n)))
    bounds = arco.Bounds(lower=-1.0e20, upper=1.0e20)

    x = model.add_variables(row, col, bounds=bounds, name="x")
    y = model.add_variables(row, col, bounds=bounds, name="y")
    x_view: Any = x
    y_view: Any = y

    model.add_constraints(x_view - y_view >= row, name="difference_floor")
    model.add_constraints(x_view + y_view >= 0.0, name="balance_floor")
    model.minimize((2 * x_view + y_view).sum())
    return model


def solve_model(*, model: arco.Model) -> arco.SolveResult:
    solver = arco.HiGHS(log_to_console=False)
    return model.solve(solver=solver)


def create_session(
    *, n: int | None = None, row_values: Sequence[float] | None = None
) -> tuple[arco.Model, Callable[[], arco.SolveResult]]:
    model = build_model(n=n, row_values=row_values)

    def solve() -> arco.SolveResult:
        return solve_model(model=model)

    return model, solve


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build or solve dense-lp with Arco Python bindings"
    )
    parser.add_argument(
        "--n",
        type=_positive_int,
        default=25,
        help="Square dimension for x and y",
    )
    parser.add_argument("--solve", action="store_true", help="Solve after building")
    parser.add_argument(
        "--json", action="store_true", help="Emit machine-readable output"
    )
    args = parser.parse_args()
    model, solve = create_session(n=args.n)
    solution: arco.SolveResult | None = None
    payload: dict[str, object] = {
        "example": "dense-lp",
        "n": args.n,
        "solved": False,
    }
    if args.solve:
        solution = solve()
        payload.update(
            {
                "solved": True,
                "status": str(solution.status),
                "is_optimal": solution.is_optimal(),
                "objective_value": solution.objective_value,
            }
        )
    if args.json:
        print(json.dumps(payload, indent=2))
    elif args.solve:
        print(
            f"dense-lp solved: n={args.n}, objective={payload['objective_value']}, "
            f"status={payload['status']}"
        )
    else:
        print(f"dense-lp built: n={args.n}")
    globals().update(
        {
            "model": model,
            "solve": solve,
            "solution": solution,
            "payload": payload,
        }
    )
    return 0

if __name__ == "__main__":
    main()
