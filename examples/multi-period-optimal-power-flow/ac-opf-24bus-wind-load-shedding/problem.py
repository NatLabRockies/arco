"""Multi-period AC-OPF for IEEE 24-bus with wind, curtailment, and load shedding.

Python translation of `input.kdl` (Soroudi Gcode6.7) using the arco Python
bindings and the IPOPT nonlinear backend.

Reference objective value: 449240.74
"""

from __future__ import annotations

import csv
import math
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
    qd_mvar = {int(r["bus"]): float(r["qd_mvar"]) for r in buses_rows}
    is_slack = {int(r["bus"]): int(r["is_slack"]) for r in buses_rows}
    wcap_mw = {int(r["bus"]): float(r["wcap_mw"]) for r in buses_rows}

    gens = [r["generator"] for r in gens_rows]
    pmax = {r["generator"]: float(r["pmax"]) for r in gens_rows}
    pmin = {r["generator"]: float(r["pmin"]) for r in gens_rows}
    b_gen = {r["generator"]: float(r["b"]) for r in gens_rows}
    qmax = {r["generator"]: float(r["qmax"]) for r in gens_rows}
    qmin = {r["generator"]: float(r["qmin"]) for r in gens_rows}
    ru = {r["generator"]: float(r["ru"]) for r in gens_rows}
    rd = {r["generator"]: float(r["rd"]) for r in gens_rows}

    # connected[(bus, generator)] = 0/1
    connected = {(int(r["bus"]), r["generator"]): int(r["connected"]) for r in gen_bus_rows}

    line_attrs = {}
    for r in lines_rows:
        l = r["line"]
        i = int(r["from_bus"])
        j = int(r["to_bus"])
        rr = float(r["r"])
        xx = float(r["x"])
        bb = float(r["b"])
        lim = float(r["limit_mw"])
        line_attrs[l] = {
            "i": i,
            "j": j,
            "r": rr,
            "x": xx,
            "b": bb,
            "limit": lim,
            "y": 1.0 / math.sqrt(rr * rr + xx * xx),
            "theta": math.atan(xx / rr),
        }

    # Directed connected pairs (i, j) -> line attributes (use first line, no parallels assumed)
    pair_line: dict[tuple[int, int], dict] = {}
    for la in line_attrs.values():
        pair_line.setdefault((la["i"], la["j"]), la)
        pair_line.setdefault((la["j"], la["i"]), la)

    # Time set and successor map.
    periods = [int(r["t"]) for r in temporal_rows]
    t_next = {int(r["t"]): int(r["taf"]) for r in temporal_rows if r["taf"] != ""}
    # Filter wrap-around (last row has taf==t).
    t_next = {tp: tn for tp, tn in t_next.items() if tn != tp}

    wind_cf = {int(r["t"]): float(r["w"]) for r in profiles_rows}
    demand_scale = {int(r["t"]): float(r["d"]) for r in profiles_rows}

    # ------------------------------------------------------------------
    # Build model.
    # ------------------------------------------------------------------
    m = arco.Model()

    pg: dict[tuple[str, int], arco.Variable] = {}
    qg: dict[tuple[str, int], arco.Variable] = {}
    for g in gens:
        for tt in periods:
            pg[g, tt] = m.add_variable(
                arco.Bounds(pmin[g] / SBASE, pmax[g] / SBASE), name=f"pg[{g},{tt}]"
            )
            qg[g, tt] = m.add_variable(
                arco.Bounds(qmin[g] / SBASE, qmax[g] / SBASE), name=f"qg[{g},{tt}]"
            )

    v: dict[tuple[int, int], arco.Variable] = {}
    va: dict[tuple[int, int], arco.Variable] = {}
    for i in buses:
        for tt in periods:
            v[i, tt] = m.add_variable(arco.Bounds(0.9, 1.1), name=f"v[{i},{tt}]")
            slack = is_slack[i]
            half_pi = 1.5707963268 * (1 - slack)
            va[i, tt] = m.add_variable(
                arco.Bounds(-half_pi, half_pi), name=f"va[{i},{tt}]"
            )

    pw: dict[tuple[int, int], arco.Variable] = {}
    pc: dict[tuple[int, int], arco.Variable] = {}
    lsh: dict[tuple[int, int], arco.Variable] = {}
    for i in buses:
        for tt in periods:
            cap = wind_cf[tt] * wcap_mw[i] / SBASE
            pw[i, tt] = m.add_variable(arco.Bounds(0.0, cap), name=f"pw[{i},{tt}]")
            pc[i, tt] = m.add_variable(arco.Bounds(0.0, cap), name=f"pc[{i},{tt}]")
            shed_cap = demand_scale[tt] * pd_mw[i] / SBASE
            lsh[i, tt] = m.add_variable(
                arco.Bounds(0.0, shed_cap), name=f"lsh[{i},{tt}]"
            )

    # Directed flow variables only for connected pairs.
    pij: dict[tuple[int, int, int], arco.Variable] = {}
    qij: dict[tuple[int, int, int], arco.Variable] = {}
    for (i, j), la in pair_line.items():
        lim = la["limit"] / SBASE
        for tt in periods:
            pij[i, j, tt] = m.add_variable(
                arco.Bounds(-lim, lim), name=f"pij[{i},{j},{tt}]"
            )
            qij[i, j, tt] = m.add_variable(
                arco.Bounds(-lim, lim), name=f"qij[{i},{j},{tt}]"
            )

    # ------------------------------------------------------------------
    # Constraints.
    # ------------------------------------------------------------------
    # AC branch flow definitions for each connected directed pair.
    for (i, j), la in pair_line.items():
        theta_ij = la["theta"]
        y_ij = la["y"]
        b_half = la["b"] / 2.0
        for tt in periods:
            vi = v[i, tt]
            vj = v[j, tt]
            ang = va[i, tt] - va[j, tt]
            # pij = (Vi^2 * cos(theta) - Vi*Vj*cos(ang + theta)) * y
            p_expr = (
                vi * vi * math.cos(theta_ij)
                - vi * vj * arco.cos(ang + theta_ij)
            ) * y_ij
            m.add_nonlinear_constraint(
                pij[i, j, tt] == p_expr, name=f"flow_p[{i},{j},{tt}]"
            )
            # qij = (Vi^2 * sin(theta) - Vi*Vj*sin(ang + theta)) * y - (b/2) * Vi^2
            q_expr = (
                vi * vi * math.sin(theta_ij)
                - vi * vj * arco.sin(ang + theta_ij)
            ) * y_ij - b_half * vi * vi
            m.add_nonlinear_constraint(
                qij[i, j, tt] == q_expr, name=f"flow_q[{i},{j},{tt}]"
            )

    # Power balance.
    for i in buses:
        neighbors = [j for j in buses if (i, j) in pair_line]
        for tt in periods:
            gen_p = arco.Expr()
            gen_q = arco.Expr()
            for g in gens:
                c = connected.get((i, g), 0)
                if c:
                    gen_p = gen_p + pg[g, tt]
                    gen_q = gen_q + qg[g, tt]
            flow_p = arco.Expr()
            flow_q = arco.Expr()
            for j in neighbors:
                flow_p = flow_p + pij[i, j, tt]
                flow_q = flow_q + qij[i, j, tt]
            m.add_constraint(
                gen_p + pw[i, tt] + lsh[i, tt] - demand_scale[tt] * pd_mw[i] / SBASE
                == flow_p,
                name=f"p_balance[{i},{tt}]",
            )
            m.add_constraint(
                gen_q - demand_scale[tt] * qd_mvar[i] / SBASE == flow_q,
                name=f"q_balance[{i},{tt}]",
            )

    # Ramping.
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

    # Curtailment definition.
    for i in buses:
        for tt in periods:
            cap = wind_cf[tt] * wcap_mw[i] / SBASE
            m.add_constraint(
                pc[i, tt] + pw[i, tt] == cap, name=f"curtail[{i},{tt}]"
            )

    # ------------------------------------------------------------------
    # Objective (linear).
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
    result = m.solve(solver=arco.Ipopt(log_to_console=True))
    print("status:", result.status)
    print(f"objective: {result.objective_value:.2f}")
    print("reference:  449240.74")


if __name__ == "__main__":
    main()
