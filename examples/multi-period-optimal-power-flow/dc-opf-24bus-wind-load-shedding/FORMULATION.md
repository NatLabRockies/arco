# Multi-period DC-OPF (24-bus, wind, load shedding) — Formulation

> **Multi-period DC Optimal Power Flow** on the IEEE 24-bus test system with
> wind generation, wind curtailment, and load shedding, over a 24-hour horizon.
> Linear program (LP) solved with HiGHS.

## Original reference

The formulation is adapted from:

> Soroudi, Alireza. _Power System Optimization Modeling in GAMS_. Springer,
> 2017. Chapter 6, Gcode6.6.
> [doi:10.1007/978-3-319-62350-4](https://doi.org/10.1007/978-3-319-62350-4)

The Arco implementation lives at
[examples/multi-period-optimal-power-flow/dc-opf-24bus-wind-load-shedding](.).

---

## Sets

| Notation                              | Index    | Description                            |
| ------------------------------------- | -------- | -------------------------------------- |
| $\mathcal{I}$                         | $i, j$   | Network buses (24)                     |
| $\mathcal{S} \subseteq \mathcal{I}$   | $i$      | Slack bus(es) (bus 13)                 |
| $\mathcal{G}$                         | $g$      | Thermal generators (12 units)          |
| $\mathcal{L}$                         | $\ell$   | Transmission lines                     |
| $\mathcal{T}$                         | $t$      | Hours (24)                             |

The GAMS bus-pair connectivity matrix `conex(bus,node)` is replaced by a
membership predicate over the line endpoint maps:

```math
\mathit{conex}(i,j) \;=\;
  \sum_{\ell} \bigl(\mathit{from}_{\ell,i}\,\mathit{to}_{\ell,j}
                   + \mathit{from}_{\ell,j}\,\mathit{to}_{\ell,i}\bigr) \in \{0,1\}
```

Lines are stored as undirected records; the signed bus-line incidence is
recovered from `from_w[l,i] - to_w[l,i]`.

---

## Parameters

### System scalars

| Notation        | Description                            | Value  |
| --------------- | -------------------------------------- | ------ |
| $S_{\text{base}}$ | Per-unit base power                  | $100$  |
| $\text{VOLL}$   | Value of lost load                     | $10000$ |
| $\text{VOLW}$   | Value of wind curtailment              | $50$   |

### Buses ([data/buses.csv](data/buses.csv))

| Notation       | Description                                       | Units    |
| -------------- | ------------------------------------------------- | -------- |
| $P^{d}_{i}$    | Active demand at bus $i$ (`pd_mw`)                | MW       |
| $\text{slack}_i$ | Slack indicator (`is_slack`)                    | $\{0,1\}$ |
| $W^{cap}_{i}$  | Installed wind capacity at bus $i$ (`wcap_mw`)    | MW       |

### Generators ([data/generators.csv](data/generators.csv))

| Notation        | Description                            | Units   |
| --------------- | -------------------------------------- | ------- |
| $P^{\min}_g$, $P^{\max}_g$ | Active output bounds (`pmin`, `pmax`) | MW      |
| $b_g$           | Linear marginal generation cost (`b`)  | \$/MWh  |
| $RU_g$, $RD_g$  | Ramp-up / ramp-down limits (`ru`, `rd`) | MW/h    |

### Generator-bus connectivity ([data/gen_bus.csv](data/gen_bus.csv))

| Notation            | Description                          | Domain     |
| ------------------- | ------------------------------------ | ---------- |
| $\chi_{i,g}$        | 1 if generator $g$ sits at bus $i$   | $\{0, 1\}$ |

### Lines and endpoints ([data/lines.csv](data/lines.csv), [data/line_endpoints.csv](data/line_endpoints.csv))

| Notation                                          | Description                              | Units |
| ------------------------------------------------- | ---------------------------------------- | ----- |
| $x_\ell$                                          | Line reactance                           | p.u.  |
| $b^{ij}_\ell = 1/x_\ell$ (`bij`)                  | Susceptance                              | p.u.  |
| $L_\ell$                                          | Thermal limit (`limit_mw`)               | MW    |
| $\mathit{from}_{\ell,i}$, $\mathit{to}_{\ell,i}$  | Endpoint indicators                      | $\{0, 1\}$ |

The DC formulation only requires $x_\ell$, $b^{ij}_\ell$, and the line limits;
resistance and shunt susceptance from the AC table are dropped at the data
layer.

### Time profiles ([data/profiles.csv](data/profiles.csv), [data/temporal_sets.csv](data/temporal_sets.csv))

| Notation                  | Description                              | Domain         |
| ------------------------- | ---------------------------------------- | -------------- |
| $w_t$                     | Hourly wind capacity factor (`wind_cf`)  | $[0, 1]$       |
| $d_t$                     | Hourly demand scaler (`demand_scale`)    | $[0, 1]$       |
| $\text{next}(t)$          | Successor map (`t_next`)                 | $\mathcal{T}$  |

---

## Decision variables

| Notation         | Symbol in KDL  | Description                                    | Units  |
| ---------------- | -------------- | ---------------------------------------------- | ------ |
| $P^{g}_{g,t}$    | `pg[g,t]`      | Thermal active dispatch                        | p.u.   |
| $\delta_{i,t}$   | `delta[i,t]`   | Bus voltage angle                              | rad    |
| $f_{\ell,t}$     | `flow[l,t]`    | Active power flow on line $\ell$               | p.u.   |
| $P^{w}_{i,t}$    | `pw[i,t]`      | Dispatched wind generation                     | p.u.   |
| $P^{c}_{i,t}$    | `pc[i,t]`      | Wind curtailment                               | p.u.   |
| $\text{lsh}_{i,t}$ | `lsh[i,t]`   | Load shedding                                  | p.u.   |

Note: in the GAMS reference, flows are indexed over directed bus pairs
$P^{ij}_{i,j,t}$. Arco indexes them over the line set $\mathcal{L}$ instead and
recovers the signed nodal contribution from `(from_w[l,i] - to_w[l,i])`.

### Variable bounds

```math
\begin{aligned}
P^{\min}_g / S_{\text{base}} \;\le\; P^{g}_{g,t} &\;\le\; P^{\max}_g / S_{\text{base}} \\
-\tfrac{\pi}{2} \;\le\; \delta_{i,t} &\;\le\; \tfrac{\pi}{2}\quad,\quad \delta_{i,t} = 0 \text{ if } i \in \mathcal{S} \\
0 \;\le\; P^{w}_{i,t},\; P^{c}_{i,t} &\;\le\; w_t \, W^{cap}_i / S_{\text{base}} \\
0 \;\le\; \text{lsh}_{i,t} &\;\le\; d_t \, P^{d}_i / S_{\text{base}} \\
-L_\ell / S_{\text{base}} \;\le\; f_{\ell,t} &\;\le\; L_\ell / S_{\text{base}}
\end{aligned}
```

Bounds whose right-hand side depends on parameters are also re-emitted as
explicit inequalities (`pg_lower_bound`, `pg_upper_bound`, …) so the LP
lowering carries them through the solver interface.

---

## Constraints

### const1 — DC line flow definition

```math
f_{\ell,t} \;=\; b^{ij}_\ell \,\sum_{i \in \mathcal{I}}
   \bigl(\mathit{from}_{\ell,i} - \mathit{to}_{\ell,i}\bigr)\,\delta_{i,t}
\qquad \forall\, \ell, t
```

This is exactly the GAMS `Pij(bus,node,t) = bij·(δ(bus,t) - δ(node,t))`
restricted to the connected pair, expressed against the line set so that each
physical line contributes one equation rather than two directed copies.

### const2 — Nodal active power balance (LMP equation)

```math
\text{lsh}_{i,t} \;+\; P^{w}_{i,t}
\;+\; \sum_{g} \chi_{i,g}\, P^{g}_{g,t}
\;-\; d_t\,\frac{P^{d}_i}{S_{\text{base}}}
\;=\; \sum_{\ell} \bigl(\mathit{from}_{\ell,i} - \mathit{to}_{\ell,i}\bigr)\,
        f_{\ell,t}
```

The dual of `power_balance` is reported by the scenario and yields nodal
locational marginal prices $\lambda_{i,t}$ once divided by $S_{\text{base}}$.

### const4 / const5 — Ramp limits

```math
\begin{aligned}
P^{g}_{g,\text{next}(t)} - P^{g}_{g,t} &\;\le\; RU_g / S_{\text{base}} \\
P^{g}_{g,t} - P^{g}_{g,\text{next}(t)} &\;\le\; RD_g / S_{\text{base}}
\end{aligned}
```

Arco uses the explicit `t_next` successor map from
`data/temporal_sets.csv` instead of GAMS positional ordering.

### const6 — Wind curtailment definition

```math
P^{c}_{i,t} \;=\; w_t\,\frac{W^{cap}_i}{S_{\text{base}}} \;-\; P^{w}_{i,t}
```

For buses with $W^{cap}_i = 0$ both $P^{w}$ and $P^{c}$ are pinned to $0$ by
their bounds and the equation reduces to $0 = 0$.

### Slack reference

```math
\delta_{i,t} \;=\; 0 \qquad \forall\, i \in \mathcal{S},\; t \in \mathcal{T}
```

Implemented as the explicit constraint `slack_angle` guarded by
`if { is_slack[i] > 0 }`.

---

## Objective (const3)

```math
\min\;\; \text{OF}
\;=\;
\underbrace{\sum_{g,t} P^{g}_{g,t}\, b_g\, S_{\text{base}}}_{\text{GenerationCost}}
\;+\;
\underbrace{\sum_{i,t} \text{VOLL}\, \text{lsh}_{i,t}\, S_{\text{base}}}_{\text{SheddingPenalty}}
\;+\;
\underbrace{\sum_{i,t} \text{VOLW}\, P^{c}_{i,t}\, S_{\text{base}}}_{\text{CurtailmentPenalty}}
```

The Arco scenario reports each term separately (`GenerationCost`,
`SheddingPenalty`, `CurtailmentPenalty`, `TotalCost`) plus the dual of the
nodal active-power balance for LMP recovery.

---

## How Arco handles the model

- **Backend**: LP via the bundled HiGHS solver (default). No optional features
  required.
- **Line indexing**: physical lines, not directed bus pairs. Signed nodal
  contributions are recovered from the endpoint indicators
  $\mathit{from}_{\ell,i} - \mathit{to}_{\ell,i}$, which keeps the constraint
  matrix at $|\mathcal{L}|$ flow definitions instead of $|\mathcal{I}|^2$.
- **Bounds vs. constraints**: every parameter-dependent bound is mirrored as
  explicit inequalities so the LP lowering preserves the GAMS feasible region
  even when symbolic bounds cannot be forwarded directly.
- **Slack handling**: explicit equality constraint `slack_angle`.
- **Ramping**: written against the successor map `t_next` from
  `data/temporal_sets.csv`, replacing GAMS positional `(t-1)` / `(t+1)` lookups.
- **Reporting**: the `MultiPeriod24BusDCOPFCase` scenario reports the
  objective, each cost component, the full set of decision variables, and the
  dual of `power_balance` for LMPs and congestion analysis.

Run the example with:

```bash
cargo run -p arco-cli -- run \
  examples/multi-period-optimal-power-flow/dc-opf-24bus-wind-load-shedding/input.kdl
```
