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
        raise ValueError("solve only supports solver='highs'")

    regions, techs, hours, years = data.regions, data.techs, data.hours, data.years
    region_index = data.r_idx
    total_hours_weight = float(np.sum(data.hours_weight))

    route_active_matrix = np.zeros((len(regions), len(regions)), dtype=bool)
    transcap_matrix = np.zeros((len(regions), len(regions)), dtype=float)
    for r_from, r_to in data.routes:
        row = region_index[r_from]
        col = region_index[r_to]
        route_active_matrix[row, col] = True
        transcap_matrix[row, col] = float(data.transcap[(r_from, r_to)])

    t0 = time.perf_counter()
    model = arco.Model()

    I = arco.IndexSet("i", members=techs)
    R = arco.IndexSet("r", members=regions)
    H = arco.IndexSet("h", members=hours)
    T = arco.IndexSet("t", members=years)
    H_ramp = H[:-1]
    R_from = R.alias("from")
    R_to = R.alias("to")

    valcap = arco.param(data.valcap, I, R, T)
    is_vre = arco.param(data.is_vre, I)
    is_storage = arco.param(data.is_storage, I)
    is_dispatch = ~is_vre & ~is_storage
    storage_active = valcap & is_storage
    dispatch_active = valcap & is_dispatch

    cf = arco.param(data.cf, I, R, H)
    cap_init = arco.param(data.cap_init, I, R)
    load = arco.param(data.load, R, H, T)
    peak_load = arco.param(data.load.max(axis=1), R, T)
    minloadfrac = arco.param(data.minloadfrac, I)
    min_cf = arco.param(data.min_cf, I)
    emit_rate = arco.param(data.emit_rate, I)
    emit_cap = arco.param(data.emit_cap, T)
    hours_weight = arco.param(data.hours_weight, H)
    pvf = arco.param(data.pvf, T)
    cost_inv = arco.param(data.cost_inv, I)
    cost_op = arco.param(data.cost_op, I)
    startcost = arco.param(data.startcost, I)

    route_active = arco.param(route_active_matrix, R_from, R_to)
    transcap = arco.param(transcap_matrix, R_from, R_to)

    cap = model.add_variables(
        I,
        R,
        T,
        bounds=arco.NonNegativeFloat,
        active=valcap,
        name="CAP",
    )
    inv = model.add_variables(
        I,
        R,
        T,
        bounds=arco.NonNegativeFloat,
        active=valcap,
        name="INV",
    )
    gen = model.add_variables(
        I,
        R,
        H,
        T,
        bounds=arco.NonNegativeFloat,
        active=valcap,
        name="GEN",
    )
    flow = model.add_variables(
        R_from,
        R_to,
        H,
        T,
        bounds=arco.Bounds(0, transcap),
        active=route_active,
        name="FLOW",
    )
    rampup = model.add_variables(
        I,
        R,
        H_ramp,
        T,
        bounds=arco.NonNegativeFloat,
        active=dispatch_active,
        name="RAMPUP",
    )
    charge = model.add_variables(
        I,
        R,
        H,
        T,
        bounds=arco.NonNegativeFloat,
        active=storage_active,
        name="CHARGE",
    )
    soc = model.add_variables(
        I,
        R,
        H,
        T,
        bounds=arco.NonNegativeFloat,
        active=storage_active,
        name="SOC",
    )

    model.add_constraints(
        cap == cap_init + np.cumsum(inv, axis=T),
        name="eq_cap_accum",
    )
    model.add_constraints(
        gen <= cf * cap,
        name="eq_cap_limit",
    )
    model.add_constraints(
        gen >= minloadfrac * cap,
        active=valcap & (minloadfrac > 0) & ~is_storage,
        name="eq_mingen",
    )

    imports = ((1.0 - float(data.tranloss)) * flow) @ R_from
    exports = flow @ R_to
    net_flow = imports - exports
    model.add_constraints(
        (gen @ I) + net_flow - (charge @ I) == load,
        name="eq_supply_demand_balance",
    )
    model.add_constraints(
        (cap @ I) >= (1.0 + float(data.prm)) * peak_load,
        name="eq_reserve_margin",
    )
    model.add_constraints(
        (emit_rate * hours_weight * gen) @ (I, R, H) <= emit_cap,
        name="eq_emit_cap",
    )
    model.add_constraints(
        rampup >= np.diff(gen, axis=H),
        active=dispatch_active,
        name="eq_ramping",
    )
    model.add_constraints(
        (hours_weight * gen) @ H >= min_cf * total_hours_weight * cap,
        active=valcap & (min_cf > 0),
        name="eq_min_cf",
    )
    model.add_constraints(
        soc <= float(data.duration_h) * cap,
        active=storage_active,
        name="eq_soc_cap",
    )
    model.add_constraints(
        charge <= cap,
        active=storage_active,
        name="eq_charge_cap",
    )
    model.add_constraints(
        np.roll(soc, -1, axis=H)
        == soc + float(data.charge_eff) * charge - gen,
        active=storage_active,
        name="eq_soc",
    )

    objective = (pvf * cost_inv * inv).sum()
    objective += (pvf * cost_op * hours_weight * gen).sum()
    objective += (pvf * startcost * rampup).sum()
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
