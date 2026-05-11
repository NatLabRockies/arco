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

from __future__ import annotations

import argparse
import json
import time
from typing import NotRequired, TypedDict

import arco
import numpy as np

from data_generator import SIZES, ProblemData, make_problem


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
    disp_irt = [
        key
        for key in active_irt
        if not data.is_vre[ii[key[0]]] and not data.is_storage[ii[key[0]]]
    ]
    storage_irt = [key for key in active_irt if data.is_storage[ii[key[0]]]]
    route_year = [(route, t) for route in data.routes for t in T]

    n_a = len(active_irt)
    n_d = len(disp_irt)
    n_s = len(storage_irt)
    n_f = len(route_year)
    n_h = len(H)
    n_t = len(T)
    n_rt = len(R) * n_t

    active_i = np.array([ii[i] for i, _r, _t in active_irt], dtype=np.int32)
    active_r = np.array([ri[r] for _i, r, _t in active_irt], dtype=np.int32)
    active_t = np.array([ti[t] for _i, _r, t in active_irt], dtype=np.int32)

    rt_members = [(r, t) for r in R for t in T]
    rt_idx = {key: pos for pos, key in enumerate(rt_members)}
    active_pos = {key: pos for pos, key in enumerate(active_irt)}

    active_rt = np.zeros((n_rt, n_a), dtype=float)
    active_rt_rows = np.array(
        [rt_idx[(r, t)] for _i, r, t in active_irt], dtype=np.int32
    )
    active_rt[active_rt_rows, np.arange(n_a, dtype=np.int32)] = 1.0

    storage_rt = np.zeros((n_rt, n_s), dtype=float)
    if n_s:
        storage_rt_rows = np.array(
            [rt_idx[(r, t)] for _i, r, t in storage_irt], dtype=np.int32
        )
        storage_rt[storage_rt_rows, np.arange(n_s, dtype=np.int32)] = 1.0

    flow_rt = np.zeros((n_rt, n_f), dtype=float)
    flow_rows_to = np.array(
        [rt_idx[(r_to, t)] for (_r_from, r_to), t in route_year], dtype=np.int32
    )
    flow_rows_from = np.array(
        [rt_idx[(r_from, t)] for (r_from, _r_to), t in route_year], dtype=np.int32
    )
    flow_cols = np.arange(n_f, dtype=np.int32)
    np.add.at(flow_rt, (flow_rows_to, flow_cols), 1.0 - float(data.tranloss))
    np.add.at(flow_rt, (flow_rows_from, flow_cols), -1.0)

    active_group = active_i * len(R) + active_r
    vintage = (
        (active_group[:, None] == active_group[None, :])
        & (active_t[:, None] >= active_t[None, :])
    ).astype(float)

    disp_active = np.array([active_pos[key] for key in disp_irt], dtype=np.int32)
    storage_active = np.array([active_pos[key] for key in storage_irt], dtype=np.int32)
    storage_pick = np.zeros((n_s, n_a), dtype=float)
    if n_s:
        storage_pick[np.arange(n_s, dtype=np.int32), storage_active] = 1.0

    cap_init = np.array(
        [float(data.cap_init[i, r]) for i, r in zip(active_i, active_r, strict=False)],
        dtype=float,
    )
    cf = np.array(
        [
            [float(data.cf[i, r, hi[h]]) for h in H]
            for i, r in zip(active_i, active_r, strict=False)
        ],
        dtype=float,
    )
    minload = np.array([float(data.minloadfrac[i]) for i in active_i], dtype=float)
    min_cf = np.array([float(data.min_cf[i]) for i in active_i], dtype=float)
    emit_rate = np.array([float(data.emit_rate[i]) for i in active_i], dtype=float)
    hours_weight = np.asarray(data.hours_weight, dtype=float)
    total_hw = float(hours_weight.sum())

    rt_load = np.array(
        [float(data.load[ri[r], hi[h], ti[t]]) for r in R for t in T for h in H],
        dtype=float,
    ).reshape(n_rt, n_h)
    peak_rt = np.array(
        [float(data.load[ri[r], :, ti[t]].max()) for r in R for t in T], dtype=float
    )
    emit_cap = np.array([float(data.emit_cap[ti[t]]) for t in T], dtype=float)

    flow_upper = np.array(
        [[float(data.transcap[route])] * n_h for route, _t in route_year], dtype=float
    )

    t0 = time.perf_counter()
    model = arco.Model()

    A = arco.IndexSet("active_irt", members=active_irt)
    Hset = arco.IndexSet("hour", members=H)
    D = arco.IndexSet("dispatchable_irt", members=disp_irt)
    Hm = arco.IndexSet("ramp_hour", members=H[:-1])
    S = arco.IndexSet("storage_irt", members=storage_irt) if n_s else None
    F = arco.IndexSet("route_year", members=route_year)

    cap = model.add_variables(A, bounds=arco.NonNegativeFloat, name="CAP")
    inv = model.add_variables(A, bounds=arco.NonNegativeFloat, name="INV")
    gen = model.add_variables(A, Hset, bounds=arco.NonNegativeFloat, name="GEN")
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

    model.add_constraints(cap - (vintage @ inv) == cap_init)
    for h in range(n_h):
        model.add_constraints(gen[:, h] <= cf[:, h] * cap)
        model.add_constraints(gen[:, h] >= minload * cap)

    for h in range(n_h):
        lhs = (active_rt @ gen[:, h]) + (flow_rt @ flow[:, h])
        if charge is not None:
            lhs -= storage_rt @ charge[:, h]
        model.add_constraints(lhs == rt_load[:, h])

    model.add_constraints((active_rt @ cap) >= (1.0 + float(data.prm)) * peak_rt)

    annual_gen = np.array(
        [np.dot(hours_weight, gen[a, :]) for a in range(n_a)], dtype=object
    )
    emit_map = np.zeros((n_t, n_a), dtype=float)
    emit_map[active_t, np.arange(n_a, dtype=np.int32)] = emit_rate
    for y in range(n_t):
        model.add_constraint(np.dot(emit_map[y, :], annual_gen) <= emit_cap[y])

    if n_h > 1 and n_d:
        for d, a in enumerate(disp_active.tolist()):
            model.add_constraints(rampup[d, :] >= gen[a, 1:] - gen[a, :-1])

    for a in range(n_a):
        if min_cf[a] <= 0.0:
            continue
        model.add_constraint(
            np.dot(hours_weight, gen[a, :]) >= min_cf[a] * total_hw * cap[a]
        )

    if charge is not None and soc is not None:
        cap_storage = storage_pick @ cap
        gen_storage = storage_pick @ gen
        for h in range(n_h):
            model.add_constraints(soc[:, h] <= float(data.duration_h) * cap_storage)
            model.add_constraints(charge[:, h] <= cap_storage)
            model.add_constraints(
                soc[:, (h + 1) % n_h]
                == soc[:, h] + float(data.charge_eff) * charge[:, h] - gen_storage[:, h]
            )

    pv_inv = np.array(
        [
            float(data.pvf[t]) * float(data.cost_inv[i])
            for i, t in zip(active_i, active_t, strict=False)
        ],
        dtype=float,
    )
    pv_op = np.array(
        [
            float(data.pvf[t]) * float(data.cost_op[i])
            for i, t in zip(active_i, active_t, strict=False)
        ],
        dtype=float,
    )
    objective = np.dot(pv_inv, inv) + np.sum(
        (pv_op[:, None] * hours_weight[None, :]) * gen
    )
    if n_h > 1 and n_d:
        start = np.array(
            [
                float(data.pvf[ti[t]]) * float(data.startcost[ii[i]])
                for i, _r, t in disp_irt
            ],
            dtype=float,
        )
        objective += sum(start[d] * rampup[d, :].sum() for d in range(n_d))
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
