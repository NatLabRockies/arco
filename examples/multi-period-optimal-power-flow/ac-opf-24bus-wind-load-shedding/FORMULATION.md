# Multi-period AC-OPF (24-bus, wind, load shedding) — Formulation

> **Multi-period AC Optimal Power Flow** on the IEEE 24-bus test system with
> wind generation, wind curtailment, and load shedding, over a 24-hour horizon.
> Non-linear program (NLP) solved with IPOPT.

## Original reference

The formulation is adapted from:

> Soroudi, Alireza. _Power System Optimization Modeling in GAMS_. Springer, 2017. Chapter 6, Gcode6.7.
> [doi:10.1007/978-3-319-62350-4](https://doi.org/10.1007/978-3-319-62350-4)

The Arco implementation lives at
[examples/multi-period-optimal-power-flow/ac-opf-24bus-wind-load-shedding](.).

---

## Sets

| Notation                            | Index  | Description                           |
| ----------------------------------- | ------ | ------------------------------------- |
| $\mathcal{I}$                       | $i, j$ | Network buses (24)                    |
| $\mathcal{S} \subseteq \mathcal{I}$ | $i$    | Slack bus(es) (bus 13)                |
| $\mathcal{G}$                       | $g$    | Thermal generators                    |
| $\mathcal{L}$                       | $\ell$ | Transmission lines (directed records) |
| $\mathcal{T}$                       | $t$    | Hours (24)                            |

In Arco the bus pair-connectivity indicator $\mathit{cx}(i,j)$ is replaced by a
line membership predicate built from the line endpoint maps `from_w[l,i]` and
`to_w[l,i]`:

```math
\mathit{cx}(i,j) \;=\; \sum_{\ell \in \mathcal{L}}
  \bigl(\mathit{from}_{\ell,i}\,\mathit{to}_{\ell,j}
      + \mathit{from}_{\ell,j}\,\mathit{to}_{\ell,i}\bigr) \;\in\;\{0,1\}
```

Constraints over directed pairs $(i,j)$ are guarded by `if { cx(i,j) == 1 }`.

---

## Parameters

### System scalars

| Notation          | Description               | Value   |
| ----------------- | ------------------------- | ------- |
| $S_{\text{base}}$ | Per-unit base power       | $100$   |
| $\text{VOLL}$     | Value of lost load        | $10000$ |
| $\text{VOLW}$     | Value of wind curtailment | $50$    |

### Buses ([data/buses.csv](data/buses.csv))

| Notation         | Description                                    | Units     |
| ---------------- | ---------------------------------------------- | --------- |
| $P^{d}_{i}$      | Active demand at bus $i$ (`pd_mw`)             | MW        |
| $Q^{d}_{i}$      | Reactive demand at bus $i$ (`qd_mvar`)         | MVAr      |
| $\text{slack}_i$ | Slack indicator (`is_slack`)                   | $\{0,1\}$ |
| $W^{cap}_{i}$    | Installed wind capacity at bus $i$ (`wcap_mw`) | MW        |

### Generators ([data/generators.csv](data/generators.csv))

| Notation                   | Description                             | Units  |
| -------------------------- | --------------------------------------- | ------ |
| $P^{\min}_g$, $P^{\max}_g$ | Active output bounds (`pmin`, `pmax`)   | MW     |
| $Q^{\min}_g$, $Q^{\max}_g$ | Reactive output bounds (`qmin`, `qmax`) | MVAr   |
| $b_g$                      | Linear marginal generation cost (`b`)   | \$/MWh |
| $V^{g}_g$                  | Generator setpoint voltage (`vg`)       | p.u.   |
| $RU_g$, $RD_g$             | Ramp-up / ramp-down limits (`ru`, `rd`) | MW/h   |

### Generator-bus connectivity ([data/gen_bus.csv](data/gen_bus.csv))

| Notation     | Description                        | Domain     |
| ------------ | ---------------------------------- | ---------- |
| $\chi_{i,g}$ | 1 if generator $g$ sits at bus $i$ | $\{0, 1\}$ |

### Lines and endpoints ([data/lines.csv](data/lines.csv), [data/line_endpoints.csv](data/line_endpoints.csv))

| Notation                                         | Description                       | Units      |
| ------------------------------------------------ | --------------------------------- | ---------- |
| $r_\ell$, $x_\ell$                               | Line resistance, reactance        | p.u.       |
| $b_\ell$                                         | Total line susceptance (`b_line`) | p.u.       |
| $L_\ell$                                         | Thermal limit (`limit_mw`)        | MW         |
| $\mathit{from}_{\ell,i}$, $\mathit{to}_{\ell,i}$ | Endpoint indicators               | $\{0, 1\}$ |

### Time profiles ([data/profiles.csv](data/profiles.csv), [data/temporal_sets.csv](data/temporal_sets.csv))

| Notation         | Description                             | Domain        |
| ---------------- | --------------------------------------- | ------------- |
| $w_t$            | Hourly wind capacity factor (`wind_cf`) | $[0, 1]$      |
| $d_t$            | Hourly demand scaler (`demand_scale`)   | $[0, 1]$      |
| $\text{next}(t)$ | Successor map (`t_next`)                | $\mathcal{T}$ |

### Derived line quantities

The GAMS model precomputes per-pair impedance magnitude $z_{ji}$ and impedance
angle $\theta_{ji}$. Arco materializes these inline inside the flow equations:

```math
z_{i,j} \;=\; \sum_{\ell}
  \bigl(\mathit{from}_{\ell,i}\,\mathit{to}_{\ell,j}
      + \mathit{from}_{\ell,j}\,\mathit{to}_{\ell,i}\bigr)\,
  \sqrt{x_\ell^2 + r_\ell^2}
\qquad
\theta_{i,j} \;=\; \sum_{\ell}
  \bigl(\mathit{from}_{\ell,i}\,\mathit{to}_{\ell,j}
      + \mathit{from}_{\ell,j}\,\mathit{to}_{\ell,i}\bigr)\,
  \operatorname{atan}\!\bigl(x_\ell / r_\ell\bigr)
```

For lines with $r_\ell = 0$ the GAMS code substitutes $\theta = \pi/2$. The
data files carry $r_\ell > 0$ for every line in the test system, so this
fallback is not exercised here.

---

## Decision variables

| Notation           | Symbol in KDL | Description                                  | Units |
| ------------------ | ------------- | -------------------------------------------- | ----- |
| $P^{g}_{g,t}$      | `pg[g,t]`     | Thermal active dispatch                      | p.u.  |
| $Q^{g}_{g,t}$      | `qg[g,t]`     | Thermal reactive dispatch                    | p.u.  |
| $V_{i,t}$          | `v[i,t]`      | Bus voltage magnitude                        | p.u.  |
| $\delta_{i,t}$     | `va[i,t]`     | Bus voltage angle                            | rad   |
| $P^{w}_{i,t}$      | `pw[i,t]`     | Dispatched wind generation                   | p.u.  |
| $P^{c}_{i,t}$      | `pc[i,t]`     | Wind curtailment                             | p.u.  |
| $\text{lsh}_{i,t}$ | `lsh[i,t]`    | Load shedding                                | p.u.  |
| $P^{ij}_{i,j,t}$   | `pij[i,j,t]`  | Active power flow on directed pair $(i,j)$   | p.u.  |
| $Q^{ij}_{i,j,t}$   | `qij[i,j,t]`  | Reactive power flow on directed pair $(i,j)$ | p.u.  |

### Variable bounds

```math
\begin{aligned}
P^{\min}_g / S_{\text{base}} \;\le\; P^{g}_{g,t} &\;\le\; P^{\max}_g / S_{\text{base}} \\
Q^{\min}_g / S_{\text{base}} \;\le\; Q^{g}_{g,t} &\;\le\; Q^{\max}_g / S_{\text{base}} \\
0.9 \;\le\; V_{i,t} &\;\le\; 1.1 \\
-\tfrac{\pi}{2}\,(1-\text{slack}_i) \;\le\; \delta_{i,t} &\;\le\; \tfrac{\pi}{2}\,(1-\text{slack}_i) \\
0 \;\le\; P^{w}_{i,t},\; P^{c}_{i,t} &\;\le\; w_t \, W^{cap}_i / S_{\text{base}} \\
0 \;\le\; \text{lsh}_{i,t} &\;\le\; d_t \, P^{d}_i / S_{\text{base}} \\
-L_{\text{cx}(i,j)} / S_{\text{base}} \;\le\; P^{ij}_{i,j,t},\; Q^{ij}_{i,j,t} &\;\le\; L_{\text{cx}(i,j)} / S_{\text{base}}
\end{aligned}
```

The slack-bus angle is anchored at $0$ via the bound expression (the upper and
lower bounds collapse when $\text{slack}_i = 1$). Bounds whose right-hand side
depends on parameters are also re-emitted as explicit constraints
(`pg_lower_bound`, `pg_upper_bound`, `va_lower_bound`, …) so the solver always
sees them as constraints even when the symbolic bound metadata cannot be
forwarded directly.

---

## Constraints

### Eq1 — Active branch flow

For every connected directed pair $(i,j)$ with $\mathit{cx}(i,j) = 1$:

```math
P^{ij}_{i,j,t}
\;=\;
\frac{V_{i,t}^{2}\,\cos(\theta_{j,i})
   \;-\; V_{i,t}\,V_{j,t}\,\cos(\delta_{i,t} - \delta_{j,t} + \theta_{j,i})}
     {z_{j,i}}
```

### Eq2 — Reactive branch flow

```math
Q^{ij}_{i,j,t}
\;=\;
\frac{V_{i,t}^{2}\,\sin(\theta_{j,i})
   \;-\; V_{i,t}\,V_{j,t}\,\sin(\delta_{i,t} - \delta_{j,t} + \theta_{j,i})}
     {z_{j,i}}
\;-\;\frac{b_{j,i}}{2}\,V_{i,t}^{2}
```

In Arco both equations are emitted by `active_flow_definition` and
`reactive_flow_definition`, expanding $\theta_{j,i}$, $z_{j,i}$, and $b_{j,i}$
as the inline sums shown above.

### Eq3 — Active nodal power balance

```math
\sum_{g} \chi_{i,g}\, P^{g}_{g,t}
\;+\; P^{w}_{i,t}
\;+\; \text{lsh}_{i,t}
\;-\; d_t\,\frac{P^{d}_i}{S_{\text{base}}}
\;=\;\sum_{j \,:\, \mathit{cx}(j,i) = 1} P^{ij}_{i,j,t}
```

GAMS gates the wind, generator, and shedding terms with `$Wcap(i)`,
`$GenD(i,'Pmax')`, and `$BD(i,'pd')`. In Arco the corresponding variables are
declared over all buses with bounds that collapse to $0$ wherever the data
parameter is absent, so the symbolic indicator is unnecessary.

### Eq4 — Reactive nodal power balance

```math
\sum_{g} \chi_{i,g}\, Q^{g}_{g,t}
\;-\; d_t\,\frac{Q^{d}_i}{S_{\text{base}}}
\;=\;\sum_{j \,:\, \mathit{cx}(j,i) = 1} Q^{ij}_{i,j,t}
```

### Eq6 / Eq7 — Ramp limits

Using the successor map $\text{next}(t)$ in lieu of GAMS lead operators
`(t-1)` / `(t+1)`:

```math
\begin{aligned}
P^{g}_{g,\text{next}(t)} - P^{g}_{g,t} &\;\le\; RU_g / S_{\text{base}} \\
P^{g}_{g,t} - P^{g}_{g,\text{next}(t)} &\;\le\; RD_g / S_{\text{base}}
\end{aligned}
```

### Eq8 — Wind curtailment definition

```math
P^{c}_{i,t} \;=\; w_t\,\frac{W^{cap}_i}{S_{\text{base}}} \;-\; P^{w}_{i,t}
```

For buses with $W^{cap}_i = 0$ both $P^{w}$ and $P^{c}$ are pinned to $0$ by
their bounds and the equation reduces to $0 = 0$.

### Slack reference

The slack bus angle is fixed implicitly by the variable bound
$\delta_{i,t} \in \{0\}$ when $\text{slack}_i = 1$.

---

## Objective (Eq5)

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
`SheddingPenalty`, `CurtailmentPenalty`, `TotalCost`) plus the duals of the
active and reactive nodal balances, which give nodal $\text{LMP}_P$ and
$\text{LMP}_Q$ once divided by $S_{\text{base}}$.

---

## How Arco handles the model

- **Backend**: NLP via IPOPT. Build with `--features ipopt` and select the
  solver with `arco solver set ipopt` before running the scenario.
- **Sets**: GAMS scalar sets become Arco data-block sets; `cx(i,j)` is a
  derived predicate over the endpoint maps, not a stored parameter.
- **Bounds vs. constraints**: every parameter-dependent bound is mirrored as an
  explicit inequality so the lowering preserves the GAMS feasible region even
  when the bound metadata cannot be carried through to the solver layer.
- **Slack handling**: enforced through the symmetric voltage-angle bound
  $\pm\pi/2\,(1-\text{slack}_i)$, eliminating the need for a dedicated
  equality.
- **Ramping**: written against the successor map `t_next` from
  `data/temporal_sets.csv`, which makes the lead/lag relations explicit instead
  of relying on GAMS positional ordering.
- **Reporting**: the `MultiPeriod24BusACOPFCase` scenario reports the
  objective, each cost component, the full set of decision variables, and the
  duals of `power_balance` / `reactive_power_balance` for LMP recovery.

Run the example with:

```bash
cargo run -p arco-cli --features ipopt -- run \
  examples/multi-period-optimal-power-flow/ac-opf-24bus-wind-load-shedding/input.kdl
```
