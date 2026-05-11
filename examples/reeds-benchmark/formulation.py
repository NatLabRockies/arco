# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "arco",
#   "numpy",
# ]
#
# [tool.uv.sources]
# arco = { path = "../../bindings/python" }
# ///

"""
Arco Python implementation of the ReEDS-representative LP test problem.

This version uses Arco array variables over compact composite domains instead of
creating one Python object per scalar variable.  The formulation keeps the same
algebra as the other benchmark entries while using bulk variable construction,
vector bounds, and vector constraints where the current Python API supports
that efficiently.
"""

from __future__ import annotations

import argparse
import json
import time
from typing import NotRequired, TypedDict

import arco
import numpy as np

from data_generator import SIZES, ProblemData, make_problem


def _sum_terms(terms):
    total = 0.0
    for term in terms:
        total += term
    return total


def solve(
    data: ProblemData, solver: str = "highs", build_only: bool = False
) -> tuple[float, float, float]:
    if solver != "highs":
        raise ValueError("solve_arco_python only supports solver='highs'")

    R, techs, H, T = data.regions, data.techs, data.hours, data.years
    ri, ii, hi, ti = data.r_idx, data.i_idx, data.h_idx, data.t_idx

    active_irt = [
        (i, r, t)
        for i in techs
        for r in R
        for t in T
        if data.valcap[ii[i], ri[r], ti[t]]
    ]
    active_row = {key: pos for pos, key in enumerate(active_irt)}
    disp_irt = [
        key
        for key in active_irt
        if not data.is_vre[ii[key[0]]] and not data.is_storage[ii[key[0]]]
    ]
    storage_irt = [key for key in active_irt if data.is_storage[ii[key[0]]]]
    storage_row = {key: pos for pos, key in enumerate(storage_irt)}

    route_year = [(route, t) for route in data.routes for t in T]
    flow_row = {key: pos for pos, key in enumerate(route_year)}

    active_by_rt = {
        (r, t): [active_row[(i, r, t)] for i in techs if (i, r, t) in active_row]
        for r in R
        for t in T
    }
    storage_by_rt = {
        (r, t): [storage_row[(i, r, t)] for i in techs if (i, r, t) in storage_row]
        for r in R
        for t in T
    }
    imports_from = {r: [] for r in R}
    exports_to = {r: [] for r in R}
    for r_from, r_to in data.routes:
        imports_from[r_to].append(r_from)
        exports_to[r_from].append(r_to)

    t0 = time.perf_counter()
    model = arco.Model()

    A = arco.IndexSet("active_irt", size=len(active_irt))
    Hset = arco.IndexSet("hour", size=len(H))
    D = arco.IndexSet("dispatchable_irt", size=len(disp_irt))
    Hm = arco.IndexSet("ramp_hour", size=max(len(H) - 1, 0))
    S = arco.IndexSet("storage_irt", size=len(storage_irt)) if storage_irt else None
    F = arco.IndexSet("route_year", size=len(route_year))

    cap = model.add_variables(A, bounds=arco.NonNegativeFloat, name="CAP")
    inv = model.add_variables(A, bounds=arco.NonNegativeFloat, name="INV")
    gen = model.add_variables(A, Hset, bounds=arco.NonNegativeFloat, name="GEN")

    flow_upper = np.array(
        [[data.transcap[route]] * len(H) for route, _ in route_year], dtype=float
    )
    flow = model.add_variables(
        F,
        Hset,
        bounds=arco.Bounds(np.zeros_like(flow_upper), flow_upper),
        name="FLOW",
    )

    rampup = model.add_variables(D, Hm, bounds=arco.NonNegativeFloat, name="RAMPUP")
    if S is not None:
        charge = model.add_variables(
            S, Hset, bounds=arco.NonNegativeFloat, name="CHARGE"
        )
        soc = model.add_variables(S, Hset, bounds=arco.NonNegativeFloat, name="SOC")
    else:
        charge = None
        soc = None

    for row, (i, r, t) in enumerate(active_irt):
        model.add_constraint(
            cap[row]
            - _sum_terms(
                inv[active_row[(i, r, tt)]]
                for tt in T
                if tt <= t and (i, r, tt) in active_row
            )
            == data.cap_init[ii[i], ri[r]],
        )

    for row, (i, r, _t) in enumerate(active_irt):
        i_pos = ii[i]
        r_pos = ri[r]
        for h_pos, h in enumerate(H):
            model.add_constraint(
                gen[row, h_pos] <= data.cf[i_pos, r_pos, hi[h]] * cap[row],
            )

    for row, (i, _r, _t) in enumerate(active_irt):
        minload = data.minloadfrac[ii[i]]
        if minload <= 0.0:
            continue
        for h_pos in range(len(H)):
            model.add_constraint(
                gen[row, h_pos] >= minload * cap[row],
            )

    for r in R:
        for t in T:
            active_rows = active_by_rt[(r, t)]
            storage_rows = storage_by_rt[(r, t)]
            for h_pos, h in enumerate(H):
                lhs = _sum_terms(gen[row, h_pos] for row in active_rows)
                if charge is not None:
                    lhs -= _sum_terms(charge[row, h_pos] for row in storage_rows)
                lhs += _sum_terms(
                    (1.0 - data.tranloss) * flow[flow_row[((rf, r), t)], h_pos]
                    for rf in imports_from[r]
                )
                lhs -= _sum_terms(
                    flow[flow_row[((r, rt), t)], h_pos] for rt in exports_to[r]
                )
                model.add_constraint(
                    lhs == data.load[ri[r], hi[h], ti[t]],
                )

    for r in R:
        for t in T:
            peak = float(data.load[ri[r], :, ti[t]].max())
            model.add_constraint(
                _sum_terms(cap[row] for row in active_by_rt[(r, t)])
                >= (1.0 + data.prm) * peak,
            )

    for t in T:
        model.add_constraint(
            _sum_terms(
                data.emit_rate[ii[i]]
                * data.hours_weight[hi[h]]
                * gen[active_row[(i, r, t)], h_pos]
                for i in techs
                for r in R
                for h_pos, h in enumerate(H)
                if (i, r, t) in active_row and data.emit_rate[ii[i]] > 0.0
            )
            <= data.emit_cap[ti[t]],
        )

    for drow, key in enumerate(disp_irt):
        arow = active_row[key]
        if len(H) > 1:
            model.add_constraints(
                rampup[drow, :] >= gen[arow, 1:] - gen[arow, :-1],
            )

    total_hw = float(data.hours_weight.sum())
    hours_weight = np.asarray(data.hours_weight, dtype=float)
    for row, (i, _r, _t) in enumerate(active_irt):
        min_cf = data.min_cf[ii[i]]
        if min_cf <= 0.0:
            continue
        model.add_constraint(
            np.dot(hours_weight, gen[row, :]) >= min_cf * cap[row] * total_hw,
        )

    if charge is not None and soc is not None:
        for srow, key in enumerate(storage_irt):
            arow = active_row[key]
            model.add_constraints(soc[srow, :] <= data.duration_h * cap[arow])
            model.add_constraints(charge[srow, :] <= cap[arow])
            for h_pos in range(len(H)):
                next_h = (h_pos + 1) % len(H)
                model.add_constraint(
                    soc[srow, next_h]
                    == soc[srow, h_pos]
                    + data.charge_eff * charge[srow, h_pos]
                    - gen[arow, h_pos],
                )

    objective = 0.0
    for row, (i, _r, t) in enumerate(active_irt):
        pv = data.pvf[ti[t]]
        objective += pv * data.cost_inv[ii[i]] * inv[row]
        objective += np.dot(pv * data.cost_op[ii[i]] * hours_weight, gen[row, :])
    for drow, (i, _r, t) in enumerate(disp_irt):
        if len(H) > 1:
            objective += (data.pvf[ti[t]] * data.startcost[ii[i]]) * rampup[
                drow, :
            ].sum()
    model.minimize(objective)

    build_s = time.perf_counter() - t0
    if build_only:
        return float("nan"), build_s, 0.0

    t1 = time.perf_counter()
    result = model.solve(
        solver=arco.HiGHS(
            log_to_console=False,
            parameters={"solver": "ipm", "arco.fingerprint": "false"},
        )
    )
    solve_s = time.perf_counter() - t1
    if not result.is_optimal():
        raise RuntimeError(f"HiGHS did not find an optimal solution: {result.status}")
    return float(result.objective_value), build_s, solve_s


class ReedsPayload(TypedDict):
    example: str
    size: str
    summary: str
    solved: bool
    objective_value: NotRequired[float]
    build_seconds: float
    solve_seconds: float


def _size_name(value: str) -> str:
    if value not in SIZES:
        choices = ", ".join(SIZES)
        raise argparse.ArgumentTypeError(f"size must be one of: {choices}")
    return value


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build or solve the ReEDS-representative Arco Python benchmark"
    )
    parser.add_argument("--size", type=_size_name, default="small", help="Problem size")
    parser.add_argument("--build-only", action="store_true", help="Skip solve")
    parser.add_argument(
        "--json", action="store_true", help="Emit machine-readable output"
    )
    args = parser.parse_args()

    problem = make_problem(args.size)
    obj, build_s, solve_s = solve(problem, build_only=args.build_only)
    payload: ReedsPayload = {
        "example": "reeds-benchmark",
        "size": args.size,
        "summary": problem.summary(),
        "solved": not args.build_only,
        "build_seconds": build_s,
        "solve_seconds": solve_s,
    }
    if not args.build_only:
        payload["objective_value"] = obj

    if args.json:
        print(json.dumps(payload, indent=2))
    elif args.build_only:
        print(
            f"reeds-benchmark built: size={args.size}, {problem.summary()}, build={build_s:.3f}s"
        )
    else:
        print(
            f"reeds-benchmark solved: size={args.size}, obj={obj:,.0f}, "
            f"build={build_s:.3f}s, solve={solve_s:.3f}s"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
