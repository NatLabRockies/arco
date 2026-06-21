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
import sys
import time
from typing import NotRequired, TypedDict

try:
    import resource
except ModuleNotFoundError:  # pragma: no cover - resource is unavailable on Windows.
    resource = None

import arco
import numpy as np

from data_generator import SIZES, ProblemData, make_problem


class BuildStage(TypedDict):
    stage: str
    seconds: float
    rss_mb: float | None
    delta_rss_mb: float | None


LAST_BUILD_PROFILE: list[BuildStage] = []
LAST_MATRIX_PROFILE: dict[str, object] = {}
LAST_SOLVE_METADATA: dict[str, float] = {}
LAST_SOLVE_STATUS: str | None = None
HIGHS_PRIMAL_SOLUTION_FEASIBLE = 2.0


def _objective_value_for_reporting(
    objective_value: float, solve_metadata: dict[str, float]
) -> float:
    primal_status = solve_metadata.get("highs_primal_solution_status")
    if primal_status is not None and primal_status != HIGHS_PRIMAL_SOLUTION_FEASIBLE:
        return float("nan")
    return objective_value


def _format_objective_value(objective_value: float) -> str:
    if objective_value != objective_value:
        return "n/a"
    return f"{objective_value:,.0f}"


def _current_rss_mb() -> float | None:
    try:
        with open("/proc/self/status", encoding="utf-8") as status:
            for line in status:
                if line.startswith("VmRSS:"):
                    return float(line.split()[1]) / 1024.0
    except OSError:
        return None
    return None


def _ru_maxrss_to_mb(max_rss: float, platform: str = sys.platform) -> float:
    if platform == "darwin":
        return max_rss / (1024.0 * 1024.0)
    return max_rss / 1024.0


def _peak_rss_mb() -> float | None:
    if resource is None:
        return None
    max_rss = max(
        float(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss),
        float(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss),
    )
    if max_rss <= 0.0:
        return None
    return _ru_maxrss_to_mb(max_rss)


def _format_rss_mb(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.1f} MB"


def solve(
    data: ProblemData,
    solver: str = "highs",
    build_only: bool = False,
    profile_build: bool = False,
    profile_matrix: bool = False,
    time_limit: float | None = None,
    presolve: bool | None = None,
    threads: int | None = None,
    highs_solver: str = "ipm",
    highs_run_crossover: str | None = None,
    highs_load_path: str | None = None,
    require_optimal: bool = True,
) -> tuple[float, float, float]:
    if solver != "highs":
        raise ValueError("solve only supports solver='highs'")

    global LAST_MATRIX_PROFILE
    global LAST_SOLVE_METADATA
    global LAST_SOLVE_STATUS
    LAST_MATRIX_PROFILE = {}
    LAST_SOLVE_METADATA = {}
    LAST_SOLVE_STATUS = None

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
    build_profile: list[BuildStage] = []
    last_rss_mb = _current_rss_mb()

    def mark(stage: str) -> None:
        nonlocal last_rss_mb
        if not profile_build:
            return
        rss_mb = _current_rss_mb()
        delta_rss_mb = (
            None if rss_mb is None or last_rss_mb is None else rss_mb - last_rss_mb
        )
        build_profile.append(
            {
                "stage": stage,
                "seconds": time.perf_counter() - t0,
                "rss_mb": rss_mb,
                "delta_rss_mb": delta_rss_mb,
            }
        )
        last_rss_mb = rss_mb

    mark("model")

    I = arco.IndexSet(name="i", members=techs)  # noqa: E741
    R = arco.IndexSet(name="r", members=regions)
    H = arco.IndexSet(name="h", members=hours)
    T = arco.IndexSet(name="t", members=years)
    H_ramp = H[:-1]
    R_from = R.alias("from")
    R_to = R.alias("to")

    valcap = arco.param(data.valcap, axes=(I, R, T))
    is_vre = arco.param(data.is_vre, axes=(I,))
    is_storage = arco.param(data.is_storage, axes=(I,))
    is_dispatch = ~is_vre & ~is_storage
    storage_active = valcap & is_storage
    dispatch_active = valcap & is_dispatch

    cf = arco.param(data.cf, axes=(I, R, H))
    cap_init = arco.param(data.cap_init, axes=(I, R))
    load = arco.param(data.load, axes=(R, H, T))
    peak_load = arco.param(data.load.max(axis=1), axes=(R, T))
    minloadfrac = arco.param(data.minloadfrac, axes=(I,))
    min_cf = arco.param(data.min_cf, axes=(I,))
    emit_rate = arco.param(data.emit_rate, axes=(I,))
    emit_cap = arco.param(data.emit_cap, axes=(T,))
    hours_weight = arco.param(data.hours_weight, axes=(H,))
    pvf = arco.param(data.pvf, axes=(T,))
    cost_inv = arco.param(data.cost_inv, axes=(I,))
    cost_op = arco.param(data.cost_op, axes=(I,))
    startcost = arco.param(data.startcost, axes=(I,))

    route_active = arco.param(route_active_matrix, axes=(R_from, R_to))
    transcap = arco.param(transcap_matrix, axes=(R_from, R_to))
    mark("sets_and_params")

    cap = model.add_variables(
        axes=(I, R, T),
        bounds=arco.NonNegativeFloat,
        active=valcap,
        name="CAP",
    )
    inv = model.add_variables(
        axes=(I, R, T),
        bounds=arco.NonNegativeFloat,
        active=valcap,
        name="INV",
    )
    gen = model.add_variables(
        axes=(I, R, H, T),
        bounds=arco.NonNegativeFloat,
        active=valcap,
        name="GEN",
    )
    flow = model.add_variables(
        axes=(R_from, R_to, H, T),
        bounds=arco.Bounds(lower=0, upper=transcap),
        active=route_active,
        name="FLOW",
    )
    rampup = model.add_variables(
        axes=(I, R, H_ramp, T),
        bounds=arco.NonNegativeFloat,
        active=dispatch_active,
        name="RAMPUP",
    )
    charge = model.add_variables(
        axes=(I, R, H, T),
        bounds=arco.NonNegativeFloat,
        active=storage_active,
        name="CHARGE",
    )
    soc = model.add_variables(
        axes=(I, R, H, T),
        bounds=arco.NonNegativeFloat,
        active=storage_active,
        name="SOC",
    )
    mark("variables")

    model.add_constraints(
        cap == cap_init + np.cumsum(inv, axis=T),
        name="eq_cap_accum",
    )
    mark("eq_cap_accum")
    model.add_constraints(
        gen <= cf * cap,
        name="eq_cap_limit",
    )
    mark("eq_cap_limit")
    model.add_constraints(
        gen >= minloadfrac * cap,
        active=valcap & (minloadfrac > 0) & ~is_storage,
        name="eq_mingen",
    )
    mark("eq_mingen")

    gen_by_region = gen @ I
    charge_by_region = charge @ I
    tranloss_factor = 1.0 - float(data.tranloss)
    for r_idx, region in enumerate(regions):
        for h_idx, hour in enumerate(hours):
            for t_idx, year in enumerate(years):
                imports = tranloss_factor * flow[:, r_idx, h_idx, t_idx].sum()
                exports = flow[r_idx, :, h_idx, t_idx].sum()
                model.add_constraint(
                    gen_by_region[r_idx, h_idx, t_idx]
                    + imports
                    - exports
                    - charge_by_region[r_idx, h_idx, t_idx]
                    == load[r_idx, h_idx, t_idx],
                    name=f"eq_supply_demand_balance[{region},{hour},{year}]",
                )
    mark("eq_supply_demand_balance")
    model.add_constraints(
        (cap @ I) >= (1.0 + float(data.prm)) * peak_load,
        name="eq_reserve_margin",
    )
    mark("eq_reserve_margin")
    model.add_constraints(
        (emit_rate * hours_weight * gen) @ (I, R, H) <= emit_cap,
        name="eq_emit_cap",
    )
    mark("eq_emit_cap")
    model.add_constraints(
        rampup >= np.diff(gen, axis=H),
        active=dispatch_active,
        name="eq_ramping",
    )
    mark("eq_ramping")
    model.add_constraints(
        (hours_weight * gen) @ H >= min_cf * total_hours_weight * cap,
        active=valcap & (min_cf > 0),
        name="eq_min_cf",
    )
    mark("eq_min_cf")
    model.add_constraints(
        soc <= float(data.duration_h) * cap,
        active=storage_active,
        name="eq_soc_cap",
    )
    mark("eq_soc_cap")
    model.add_constraints(
        charge <= cap,
        active=storage_active,
        name="eq_charge_cap",
    )
    mark("eq_charge_cap")
    model.add_constraints(
        np.roll(soc, -1, axis=H) == soc + float(data.charge_eff) * charge - gen,
        active=storage_active,
        name="eq_soc",
    )
    mark("eq_soc")

    objective = (pvf * cost_inv * inv).sum()
    objective += (pvf * cost_op * hours_weight * gen).sum()
    objective += (pvf * startcost * rampup).sum()
    model.minimize(objective)
    LAST_MATRIX_PROFILE = model.matrix_profile() if profile_matrix else {}
    mark("objective")

    build_s = time.perf_counter() - t0
    global LAST_BUILD_PROFILE
    LAST_BUILD_PROFILE = build_profile
    if build_only:
        return float("nan"), build_s, 0.0

    t1 = time.perf_counter()
    highs_kwargs = {
        "log_to_console": False,
        "parameters": {
            "solver": highs_solver,
            "arco.fingerprint": "false",
        },
    }
    if time_limit is not None:
        highs_kwargs["time_limit"] = time_limit
    if presolve is not None:
        highs_kwargs["presolve"] = presolve
    if threads is not None:
        highs_kwargs["threads"] = threads
    if highs_run_crossover is not None:
        highs_kwargs["parameters"]["run_crossover"] = highs_run_crossover
    if highs_load_path not in (None, "wrapper"):
        highs_kwargs["parameters"]["arco.highs_load_path"] = highs_load_path
    result = model.solve(solver=arco.HiGHS(**highs_kwargs))
    solve_s = time.perf_counter() - t1
    LAST_SOLVE_METADATA = dict(getattr(result, "metadata", {}))
    LAST_SOLVE_STATUS = str(result.status)
    if require_optimal and not result.is_optimal():
        raise RuntimeError(f"HiGHS did not find an optimal solution: {result.status}")
    objective_value = _objective_value_for_reporting(
        float(result.objective_value), LAST_SOLVE_METADATA
    )
    return objective_value, build_s, solve_s


class ReedsPayload(TypedDict):
    example: str
    size: str
    summary: str
    solved: bool
    objective_value: NotRequired[float | None]
    build_seconds: float
    solve_seconds: float
    peak_rss_mb: float | None
    status: NotRequired[str]
    build_profile: NotRequired[list[BuildStage]]
    matrix_profile: NotRequired[dict[str, object]]
    solve_metadata: NotRequired[dict[str, float]]


def _size_name(value: str) -> str:
    if value not in SIZES:
        choices = ", ".join(SIZES)
        raise argparse.ArgumentTypeError(f"size must be one of: {choices}")
    return value


def _positive_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("value must be an integer") from exc
    if parsed < 1:
        raise argparse.ArgumentTypeError("value must be >= 1")
    return parsed


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build or solve the ReEDS-representative Arco Python benchmark"
    )
    parser.add_argument("--size", type=_size_name, default="small", help="Problem size")
    parser.add_argument("--build-only", action="store_true", help="Skip solve")
    parser.add_argument(
        "--profile-build",
        action="store_true",
        help="Include per-stage build timing and RSS in JSON output",
    )
    parser.add_argument(
        "--profile-matrix",
        action="store_true",
        help="Include sparse matrix column-density diagnostics in JSON output",
    )
    parser.add_argument(
        "--json", action="store_true", help="Emit machine-readable output"
    )
    parser.add_argument(
        "--time-limit",
        type=float,
        default=None,
        help="Optional HiGHS time limit in seconds",
    )
    parser.add_argument(
        "--presolve",
        choices=("on", "off"),
        default=None,
        help="Override HiGHS presolve for solver-memory probes",
    )
    parser.add_argument(
        "--threads",
        type=_positive_int,
        default=None,
        help="Override HiGHS thread count for solver-memory probes",
    )
    parser.add_argument(
        "--highs-solver",
        choices=("ipm", "simplex", "choose", "pdlp"),
        default="ipm",
        help="Override the HiGHS algorithm for solver-memory probes",
    )
    parser.add_argument(
        "--highs-run-crossover",
        choices=("on", "off", "choose"),
        default=None,
        help="Override HiGHS run_crossover for IPM memory probes",
    )
    parser.add_argument(
        "--highs-load-path",
        choices=("wrapper", "direct"),
        default=None,
        help="Accepted for compatibility; direct load path requires a later solver slice",
    )
    parser.add_argument(
        "--allow-nonoptimal",
        action="store_true",
        help="Return JSON/text output for time-limit or other non-optimal statuses",
    )
    args = parser.parse_args()

    problem = make_problem(args.size)
    obj, build_s, solve_s = solve(
        problem,
        build_only=args.build_only,
        profile_build=args.profile_build,
        profile_matrix=args.profile_matrix,
        time_limit=args.time_limit,
        presolve=None if args.presolve is None else args.presolve == "on",
        threads=args.threads,
        highs_solver=args.highs_solver,
        highs_run_crossover=args.highs_run_crossover,
        highs_load_path=args.highs_load_path,
        require_optimal=not args.allow_nonoptimal,
    )
    solved = not args.build_only and LAST_SOLVE_STATUS == "SolutionStatus.OPTIMAL"
    payload: ReedsPayload = {
        "example": "reeds-benchmark",
        "size": args.size,
        "summary": problem.summary(),
        "solved": solved,
        "build_seconds": build_s,
        "solve_seconds": solve_s,
        "peak_rss_mb": _peak_rss_mb(),
    }
    if not args.build_only:
        payload["objective_value"] = None if obj != obj else obj
        if LAST_SOLVE_STATUS is not None:
            payload["status"] = LAST_SOLVE_STATUS
        payload["solve_metadata"] = LAST_SOLVE_METADATA
    if args.profile_build:
        payload["build_profile"] = LAST_BUILD_PROFILE
    if args.profile_matrix:
        payload["matrix_profile"] = LAST_MATRIX_PROFILE

    if args.json:
        print(json.dumps(payload, indent=2))
    elif args.build_only:
        print(
            f"reeds-benchmark built: size={args.size}, {problem.summary()}, "
            f"build={build_s:.3f}s, peak={_format_rss_mb(payload['peak_rss_mb'])}"
        )
    elif not solved:
        print(
            f"reeds-benchmark finished: size={args.size}, status={LAST_SOLVE_STATUS}, "
            f"obj={_format_objective_value(obj)}, build={build_s:.3f}s, solve={solve_s:.3f}s, "
            f"peak={_format_rss_mb(payload['peak_rss_mb'])}"
        )
    else:
        print(
            f"reeds-benchmark solved: size={args.size}, obj={_format_objective_value(obj)}, "
            f"build={build_s:.3f}s, solve={solve_s:.3f}s, "
            f"peak={_format_rss_mb(payload['peak_rss_mb'])}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
