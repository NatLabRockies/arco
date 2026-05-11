"""Multi-period DC-OPF for IEEE 24-bus with wind and load shedding.

Python translation of `input.kdl` (Soroudi Gcode6.6) using the arco Python
bindings. Pure LP — solved with the default HiGHS backend.

Reference objective value: 432292.56
"""

from __future__ import annotations

import csv
from pathlib import Path

import arco

DATA = Path(__file__).parent / "data"
SBASE = 100.0
VOLL = 10000.0
VOLW = 50.0


def _read_csv(name: str) -> list[dict[str, str]]:
    with (DATA / name).open() as fh:
        return list(csv.DictReader(fh))


def main() -> None:
    # ------------------------------------------------------------------
    # Read data.
    # ------------------------------------------------------------------
    buses_rows = _read_csv("buses.csv")
    gens_rows = _read_csv("generators.csv")
    gen_bus_rows = _read_csv("gen_bus.csv")
    lines_rows = _read_csv("lines.csv")
    profiles_rows = _read_csv("profiles.csv")
    temporal_rows = _read_csv("temporal_sets.csv")

    buses = [int(r["bus"]) for r in buses_rows]
    pd_mw = {int(r["bus"]): float(r["pd_mw"]) for r in buses_rows}
    is_slack = {int(r["bus"]): int(r["is_slack"]) for r in buses_rows}
    wcap_mw = {int(r["bus"]): float(r["wcap_mw"]) for r in buses_rows}

    gens = [r["generator"] for r in gens_rows]
    pmax = {r["generator"]: float(r["pmax"]) for r in gens_rows}
    pmin = {r["generator"]: float(r["pmin"]) for r in gens_rows}
    b_gen = {r["generator"]: float(r["b"]) for r in gens_rows}
    ru = {r["generator"]: float(r["ru"]) for r in gens_rows}
    rd = {r["generator"]: float(r["rd"]) for r in gens_rows}

    connected = {
        (int(r["bus"]), r["generator"]): int(r["connected"]) for r in gen_bus_rows
    }

    # Each row of lines.csv defines one line with its from/to buses, bij, limit.
    lines = []
    for r in lines_rows:
        lines.append(
            {
                "id": r["line"],
                "i": int(r["from_bus"]),
                "j": int(r["to_bus"]),
                "bij": float(r["bij"]),
                "limit": float(r["limit_mw"]),
            }
        )

    periods = [int(r["t"]) for r in temporal_rows]
    t_next = {
        int(r["t"]): int(r["taf"]) for r in temporal_rows if r["taf"] != ""
    }
    # Drop wrap-around entry (last row's taf == its own t).
    t_next = {tp: tn for tp, tn in t_next.items() if tn != tp}

    wind_cf = {int(r["t"]): float(r["w"]) for r in profiles_rows}
    demand_scale = {int(r["t"]): float(r["d"]) for r in profiles_rows}

    # ------------------------------------------------------------------
    # Build model.
    # ------------------------------------------------------------------
    m = arco.Model()

    pg: dict[tuple[str, int], arco.Variable] = {}
    for g in gens:
        for tt in periods:
            pg[g, tt] = m.add_variable(
                arco.Bounds(pmin[g] / SBASE, pmax[g] / SBASE), name=f"pg[{g},{tt}]"
            )

    # Voltage angle. Slack bus pinned to 0 via fixed bounds.
    delta: dict[tuple[int, int], arco.Variable] = {}
    half_pi = 1.5707963268
    for i in buses:
        for tt in periods:
            if is_slack[i]:
                bnd = arco.Bounds(0.0, 0.0)
            else:
                bnd = arco.Bounds(-half_pi, half_pi)
            delta[i, tt] = m.add_variable(bnd, name=f"delta[{i},{tt}]")

    pw: dict[tuple[int, int], arco.Variable] = {}
    pc: dict[tuple[int, int], arco.Variable] = {}
    lsh: dict[tuple[int, int], arco.Variable] = {}
    for i in buses:
        for tt in periods:
            wcap = wcap_mw[i] * wind_cf[tt] / SBASE
            pw[i, tt] = m.add_variable(arco.Bounds(0.0, wcap), name=f"pw[{i},{tt}]")
            pc[i, tt] = m.add_variable(arco.Bounds(0.0, wcap), name=f"pc[{i},{tt}]")
            shed_cap = demand_scale[tt] * pd_mw[i] / SBASE
            lsh[i, tt] = m.add_variable(
                arco.Bounds(0.0, shed_cap), name=f"lsh[{i},{tt}]"
            )

    # Per-line flow.
    flow: dict[tuple[str, int], arco.Variable] = {}
    for line in lines:
        lim = line["limit"] / SBASE
        for tt in periods:
            flow[line["id"], tt] = m.add_variable(
                arco.Bounds(-lim, lim), name=f"flow[{line['id']},{tt}]"
            )

    # ------------------------------------------------------------------
    # Constraints.
    # ------------------------------------------------------------------
    # const1: DC flow definition  flow_l = bij_l * (delta_from - delta_to).
    for line in lines:
        i, j, bij = line["i"], line["j"], line["bij"]
        for tt in periods:
            m.add_constraint(
                flow[line["id"], tt] == bij * (delta[i, tt] - delta[j, tt]),
                name=f"flow_def[{line['id']},{tt}]",
            )

    # Pre-compute per-bus signed incidence: lines leaving (+1) and entering (-1).
    out_lines: dict[int, list[tuple[str, int]]] = {i: [] for i in buses}
    for line in lines:
        out_lines[line["i"]].append((line["id"], +1))
        out_lines[line["j"]].append((line["id"], -1))

    # const2: nodal power balance.
    for i in buses:
        for tt in periods:
            gen_p = arco.Expr()
            for g in gens:
                if connected.get((i, g), 0):
                    gen_p = gen_p + pg[g, tt]
            net_flow = arco.Expr()
            for lid, sign in out_lines[i]:
                net_flow = net_flow + sign * flow[lid, tt]
            m.add_constraint(
                gen_p + pw[i, tt] + lsh[i, tt]
                - demand_scale[tt] * pd_mw[i] / SBASE
                == net_flow,
                name=f"power_balance[{i},{tt}]",
            )

    # const4 / const5: ramping.
    for g in gens:
        for tp, tn in t_next.items():
            m.add_constraint(
                pg[g, tn] - pg[g, tp] <= ru[g] / SBASE,
                name=f"ramp_up[{g},{tp}]",
            )
            m.add_constraint(
                pg[g, tp] - pg[g, tn] <= rd[g] / SBASE,
                name=f"ramp_dn[{g},{tp}]",
            )

    # const6: curtailment definition  pc + pw = wind_cf * wcap / sbase.
    for i in buses:
        for tt in periods:
            cap = wind_cf[tt] * wcap_mw[i] / SBASE
            m.add_constraint(
                pc[i, tt] + pw[i, tt] == cap,
                name=f"curtail[{i},{tt}]",
            )

    # ------------------------------------------------------------------
    # Objective.
    # ------------------------------------------------------------------
    cost = arco.Expr()
    for g in gens:
        for tt in periods:
            cost = cost + pg[g, tt] * (b_gen[g] * SBASE)
    for i in buses:
        for tt in periods:
            cost = cost + lsh[i, tt] * (VOLL * SBASE)
            cost = cost + pc[i, tt] * (VOLW * SBASE)

    m.minimize(cost)

    # ------------------------------------------------------------------
    # Solve.
    # ------------------------------------------------------------------
    result = m.solve()
    print("status:", result.status)
    print(f"objective: {result.objective_value:.2f}")
    print("reference:  432292.56")


if __name__ == "__main__":
    main()
