# SDOM Formulation

> **Storage Deployment Optimization Model (SDOM)**: a mixed-integer linear
> program that co-optimizes investment in renewable generation, balancing units,
> and storage technologies while dispatching all resources to meet electrical
> demand at minimum annualized system cost subject to a clean-energy target.

## Original reference

The SDOM formulation is based on:

> O. J. Guerra, J. Eichman, and P. Denholm, "Optimal energy storage portfolio
> for high and ultrahigh carbon-free and renewable power systems," *Energy &
> Environmental Science*, vol. 14, no. 10, pp. 5132--5146, 2021.
> [doi:10.1039/D1EE01835C](https://doi.org/10.1039/D1EE01835C)

The reference implementation is available at
[github.com/NREL/SDOM](https://github.com/NREL/SDOM).

---

## Sets

| Notation                                | Index | Description                          |
| --------------------------------------- | ----- | ------------------------------------ |
| $\mathcal{P}$                           | $p$   | PV plants                            |
| $\mathcal{W}$                           | $w$   | Wind plants                          |
| $\mathcal{K}$                           | $k$   | Balancing units (dispatchable)       |
| $\mathcal{H}$                           | $h$   | Hours (time steps)                   |
| $\mathcal{B}$                           | $b$   | Hydro budget periods                 |
| $\mathcal{H}_b \subseteq \mathcal{H}$   | $h$   | Hours belonging to budget period $b$ |
| $\mathcal{J}$                           | $j$   | Storage technologies                 |
| $\mathcal{J}^{c} \subseteq \mathcal{J}$ | $j$   | Coupled storage technologies         |

---

## Parameters

### Derived quantities

The **capital recovery factor** annualizes a lump-sum investment over lifetime
$l$ at discount rate $r$:

```math
\text{CRF}(l) = \frac{r\,(1+r)^{l}}{(1+r)^{l} - 1}
```

### System

| Notation | Description                           | Units | Domain         |
| -------- | ------------------------------------- | ----- | -------------- |
| $d_h$    | Electrical demand at hour $h$         | MW    | $\mathbb{R}_+$ |
| $r$      | Discount rate                         | —     | $\mathbb{R}_+$ |
| $\tau$   | Minimum clean-energy generation share | —     | $[0, 1]$       |

### Fixed generation profiles

Exogenous time series scaled by activation parameters $\alpha$.

| Notation       | Description                        | Units | Domain         |
| -------------- | ---------------------------------- | ----- | -------------- |
| $\rho_h$       | Run-of-river hydro generation      | MW    | $\mathbb{R}_+$ |
| $\nu_h$        | Nuclear generation profile         | MW    | $\mathbb{R}_+$ |
| $\omega_h$     | Other renewable generation profile | MW    | $\mathbb{R}_+$ |
| $\alpha^{nuc}$ | Nuclear activation scalar          | —     | $\{0, 1\}$     |
| $\alpha^{hyd}$ | Hydro activation scalar            | —     | $\{0, 1\}$     |
| $\alpha^{oth}$ | Other renewables activation scalar | —     | $\{0, 1\}$     |

### Renewable investment

| Notation             | Description                                     | Units    | Domain         |
| -------------------- | ----------------------------------------------- | -------- | -------------- |
| $\sigma_{ph}$        | Solar capacity factor for plant $p$ at hour $h$ | —        | $[0, 1]$       |
| $\zeta_{wh}$         | Wind capacity factor for plant $w$ at hour $h$  | —        | $[0, 1]$       |
| $\bar{p}^{pv}_p$     | Max allowed PV capacity for plant $p$           | MW       | $\mathbb{R}_+$ |
| $\bar{p}^{wind}_w$   | Max allowed wind capacity for plant $w$         | MW       | $\mathbb{R}_+$ |
| $\kappa^{pv}_p$      | PV CAPEX                                        | \$/MW    | $\mathbb{R}_+$ |
| $\kappa^{wind}_w$    | Wind CAPEX                                      | \$/MW    | $\mathbb{R}_+$ |
| $\kappa^{tc,pv}_p$   | PV transmission capital cost                    | \$/MW    | $\mathbb{R}_+$ |
| $\kappa^{tc,wind}_w$ | Wind transmission capital cost                  | \$/MW    | $\mathbb{R}_+$ |
| $\phi^{pv}_p$        | PV fixed O\&M                                   | \$/MW-yr | $\mathbb{R}_+$ |
| $\phi^{wind}_w$      | Wind fixed O\&M                                 | \$/MW-yr | $\mathbb{R}_+$ |
| $l^{vre}$            | VRE lifetime (shared across all PV and wind)    | yr       | $\mathbb{Z}_+$ |

### Balancing units

| Notation                | Description                                        | Units    | Domain         |
| ----------------------- | -------------------------------------------------- | -------- | -------------- |
| $\underline{p}^{bal}_k$ | Min balancing capacity                             | MW       | $\mathbb{R}_+$ |
| $\bar{p}^{bal}_k$       | Max allowed balancing capacity                     | MW       | $\mathbb{R}_+$ |
| $\kappa^{bal}_k$        | Balancing unit CAPEX                               | \$/MW    | $\mathbb{R}_+$ |
| $\gamma_k$              | Marginal fuel cost (fuel price $\times$ heat rate) | \$/MWh   | $\mathbb{R}_+$ |
| $\phi^{bal}_k$          | Balancing fixed O\&M                               | \$/MW-yr | $\mathbb{R}_+$ |
| $\psi^{bal}_k$          | Balancing variable O\&M                            | \$/MWh   | $\mathbb{R}_+$ |
| $l^{bal}_k$             | Balancing unit lifetime                            | yr       | $\mathbb{Z}_+$ |

### Hydro

| Notation                | Description                                     | Units | Domain         |
| ----------------------- | ----------------------------------------------- | ----- | -------------- |
| $\underline{g}^{hyd}_h$ | Hourly hydro lower bound                        | MW    | $\mathbb{R}_+$ |
| $\overline{g}^{hyd}_h$  | Hourly hydro upper bound                        | MW    | $\mathbb{R}_+$ |
| $\epsilon_b$            | Energy budget for period $b$ (from time series) | MWh   | $\mathbb{R}_+$ |

### Trade

| Notation        | Description                     | Units  | Domain         |
| --------------- | ------------------------------- | ------ | -------------- |
| $\bar{\iota}_h$ | Max import capacity at hour $h$ | MW     | $\mathbb{R}_+$ |
| $\bar{\xi}_h$   | Max export capacity at hour $h$ | MW     | $\mathbb{R}_+$ |
| $c^{imp}_h$     | Import price at hour $h$        | \$/MWh | $\mathbb{R}_+$ |
| $c^{exp}_h$     | Export price at hour $h$        | \$/MWh | $\mathbb{R}_+$ |

### Storage

| Notation               | Description                                       | Units    | Domain         |
| ---------------------- | ------------------------------------------------- | -------- | -------------- |
| $\kappa^{P}_j$         | CAPEX power                                       | \$/MW    | $\mathbb{R}_+$ |
| $\kappa^{E}_j$         | CAPEX energy                                      | \$/MWh   | $\mathbb{R}_+$ |
| $\eta_j$               | Roundtrip efficiency (charge $\times$ discharge)  | —        | $(0, 1]$       |
| $\underline{\delta}_j$ | Min duration                                      | hr       | $\mathbb{R}_+$ |
| $\overline{\delta}_j$  | Max duration                                      | hr       | $\mathbb{R}_+$ |
| $\bar{p}^{stor}_j$     | Max installable power capacity                    | MW       | $\mathbb{R}_+$ |
| $\alpha_j$             | Cost ratio: fraction of power cost on charge side | —        | $[0, 1]$       |
| $\phi^{stor}_j$        | Fixed O\&M                                        | \$/MW-yr | $\mathbb{R}_+$ |
| $\psi^{stor}_j$        | Variable O\&M (applied to discharge only)         | \$/MWh   | $\mathbb{R}_+$ |
| $l^{stor}_j$           | Lifetime                                          | yr       | $\mathbb{Z}_+$ |
| $\kappa^{cyc}_j$       | Max lifetime cycles                               | —        | $\mathbb{R}_+$ |

> **Coupled vs decoupled storage.** A **coupled** technology (e.g. Li-Ion
> battery, pumped hydro) uses shared equipment for charge and discharge, so the
> power ratings are forced equal: $P^{ch}_j = P^{dis}_j$. A **decoupled**
> technology (e.g. CAES, hydrogen) has separate input and output equipment, so
> $P^{ch}_j$ and $P^{dis}_j$ can be independently sized.
>
> The cost ratio $\alpha_j$ distributes power-related costs (CAPEX, FOM) between
> the charge side (fraction $\alpha_j$) and the discharge side (fraction
> $1 - \alpha_j$). For coupled storage where $P^{ch}_j = P^{dis}_j$, the split
> is immaterial.

---

## Variables

### Renewable (investment and dispatch)

| Variable     | Description                            | Units | Domain         |
| ------------ | -------------------------------------- | ----- | -------------- |
| $F^{pv}_p$   | Fraction of max PV capacity to build   | —     | $[0, 1]$       |
| $F^{wind}_w$ | Fraction of max wind capacity to build | —     | $[0, 1]$       |
| $G^{pv}_h$   | Aggregate PV generation                | MW    | $\mathbb{R}_+$ |
| $G^{wind}_h$ | Aggregate wind generation              | MW    | $\mathbb{R}_+$ |
| $C^{pv}_h$   | PV curtailment                         | MW    | $\mathbb{R}_+$ |
| $C^{wind}_h$ | Wind curtailment                       | MW    | $\mathbb{R}_+$ |

### Balancing units (investment and dispatch)

| Variable       | Description                  | Units | Domain         |
| -------------- | ---------------------------- | ----- | -------------- |
| $P^{bal}_k$    | Installed balancing capacity | MW    | $\mathbb{R}_+$ |
| $G^{bal}_{kh}$ | Balancing unit generation    | MW    | $\mathbb{R}_+$ |

### Storage (investment and operations)

Both charge and discharge power capacity variables exist for **all** storage
technologies.

| Variable       | Description                        | Units | Domain         |
| -------------- | ---------------------------------- | ----- | -------------- |
| $P^{dis}_j$    | Installed discharge power capacity | MW    | $\mathbb{R}_+$ |
| $P^{ch}_j$     | Installed charge power capacity    | MW    | $\mathbb{R}_+$ |
| $E_j$          | Installed energy capacity          | MWh   | $\mathbb{R}_+$ |
| $D^{ch}_{jh}$  | Charge power                       | MW    | $\mathbb{R}_+$ |
| $D^{dis}_{jh}$ | Discharge power                    | MW    | $\mathbb{R}_+$ |
| $S_{jh}$       | State of charge                    | MWh   | $\mathbb{R}_+$ |
| $U_{jh}$       | Charge indicator ($1$ = charging)  | —     | $\{0, 1\}$     |

### Hydro

| Variable    | Description                   | Units | Domain         |
| ----------- | ----------------------------- | ----- | -------------- |
| $G^{hyd}_h$ | Dispatchable hydro generation | MW    | $\mathbb{R}_+$ |

### Trade

| Variable | Description                                     | Units | Domain         |
| -------- | ----------------------------------------------- | ----- | -------------- |
| $M_h$    | Imports                                         | MW    | $\mathbb{R}_+$ |
| $X_h$    | Exports                                         | MW    | $\mathbb{R}_+$ |
| $V_h$    | Net-load sign indicator ($1$ if net load $> 0$) | —     | $\{0, 1\}$     |

---

## Objective Function

```math
\min \; Z \;=\; Z^{pv} + Z^{wind} + Z^{bal} + Z^{stor} + Z^{trade}
```

### PV cost

Annualized CAPEX (including transmission) and FOM. VRE shares a single lifetime
$l^{vre}$:

```math
Z^{pv} = \sum_{p \in \mathcal{P}} \Bigl[
  \bigl(\text{CRF}(l^{vre}) \cdot (\kappa^{pv}_p + \kappa^{tc,pv}_p) + \phi^{pv}_p\bigr)
  \;\bar{p}^{pv}_p \; F^{pv}_p
\Bigr]
```

| Symbol             | Type      | Description                          |
| ------------------ | --------- | ------------------------------------ |
| $\mathcal{P}$      | Set       | PV plants                            |
| $p$                | Index     | PV plant                             |
| $l^{vre}$          | Parameter | VRE lifetime (yr)                    |
| $\kappa^{pv}_p$    | Parameter | PV CAPEX (\$/MW)                     |
| $\kappa^{tc,pv}_p$ | Parameter | PV transmission capital cost (\$/MW) |
| $\phi^{pv}_p$      | Parameter | PV fixed O\&M (\$/MW-yr)             |
| $\bar{p}^{pv}_p$   | Parameter | Max allowed PV capacity (MW)         |
| $F^{pv}_p$         | Variable  | Fraction of max PV capacity to build |

### Wind cost

```math
Z^{wind} = \sum_{w \in \mathcal{W}} \Bigl[
  \bigl(\text{CRF}(l^{vre}) \cdot (\kappa^{wind}_w + \kappa^{tc,wind}_w) + \phi^{wind}_w\bigr)
  \;\bar{p}^{wind}_w \; F^{wind}_w
\Bigr]
```

| Symbol               | Type      | Description                            |
| -------------------- | --------- | -------------------------------------- |
| $\mathcal{W}$        | Set       | Wind plants                            |
| $w$                  | Index     | Wind plant                             |
| $l^{vre}$            | Parameter | VRE lifetime (yr)                      |
| $\kappa^{wind}_w$    | Parameter | Wind CAPEX (\$/MW)                     |
| $\kappa^{tc,wind}_w$ | Parameter | Wind transmission capital cost (\$/MW) |
| $\phi^{wind}_w$      | Parameter | Wind fixed O\&M (\$/MW-yr)             |
| $\bar{p}^{wind}_w$   | Parameter | Max allowed wind capacity (MW)         |
| $F^{wind}_w$         | Variable  | Fraction of max wind capacity to build |

### Balancing unit cost

Per-unit CRF based on individual lifetimes. Marginal cost includes fuel and VOM:

```math
Z^{bal} = \sum_{k \in \mathcal{K}} \Bigl[
  \bigl(\text{CRF}(l^{bal}_k) \cdot \kappa^{bal}_k + \phi^{bal}_k\bigr)
  \; P^{bal}_k
  \;+\; \sum_{h \in \mathcal{H}} \bigl(\gamma_k + \psi^{bal}_k\bigr) \; G^{bal}_{kh}
\Bigr]
```

| Symbol           | Type      | Description                       |
| ---------------- | --------- | --------------------------------- |
| $\mathcal{K}$    | Set       | Balancing units (dispatchable)    |
| $k$              | Index     | Balancing unit                    |
| $\mathcal{H}$    | Set       | Hours (time steps)                |
| $h$              | Index     | Hour                              |
| $l^{bal}_k$      | Parameter | Balancing unit lifetime (yr)      |
| $\kappa^{bal}_k$ | Parameter | Balancing unit CAPEX (\$/MW)      |
| $\phi^{bal}_k$   | Parameter | Balancing fixed O\&M (\$/MW-yr)   |
| $\gamma_k$       | Parameter | Marginal fuel cost (\$/MWh)       |
| $\psi^{bal}_k$   | Parameter | Balancing variable O\&M (\$/MWh)  |
| $P^{bal}_k$      | Variable  | Installed balancing capacity (MW) |
| $G^{bal}_{kh}$   | Variable  | Balancing unit generation (MW)    |

### Storage cost

The cost ratio $\alpha_j$ distributes power CAPEX and FOM between the charge
side (fraction $\alpha_j$) and the discharge side (fraction $1 - \alpha_j$).
This applies uniformly to all storage technologies:

```math
Z^{stor} = \sum_{j \in \mathcal{J}} \Bigl[
  \text{CRF}(l^{stor}_j) \Bigl(
    \kappa^{P}_j \bigl(\alpha_j \; P^{ch}_j + (1 - \alpha_j) \; P^{dis}_j\bigr)
    \;+\; \kappa^{E}_j \; E_j
  \Bigr)
  \;+\; \phi^{stor}_j \bigl(\alpha_j \; P^{ch}_j + (1 - \alpha_j) \; P^{dis}_j\bigr)
  \;+\; \sum_{h \in \mathcal{H}} \psi^{stor}_j \; D^{dis}_{jh}
\Bigr]
```

> VOM is charged on **discharge only**.

| Symbol          | Type      | Description                                       |
| --------------- | --------- | ------------------------------------------------- |
| $\mathcal{J}$   | Set       | Storage technologies                              |
| $j$             | Index     | Storage technology                                |
| $\mathcal{H}$   | Set       | Hours (time steps)                                |
| $h$             | Index     | Hour                                              |
| $l^{stor}_j$    | Parameter | Storage lifetime (yr)                             |
| $\kappa^{P}_j$  | Parameter | CAPEX power (\$/MW)                               |
| $\kappa^{E}_j$  | Parameter | CAPEX energy (\$/MWh)                             |
| $\alpha_j$      | Parameter | Cost ratio: fraction of power cost on charge side |
| $\phi^{stor}_j$ | Parameter | Storage fixed O\&M (\$/MW-yr)                     |
| $\psi^{stor}_j$ | Parameter | Storage variable O\&M (\$/MWh)                    |
| $P^{ch}_j$      | Variable  | Installed charge power capacity (MW)              |
| $P^{dis}_j$     | Variable  | Installed discharge power capacity (MW)           |
| $E_j$           | Variable  | Installed energy capacity (MWh)                   |
| $D^{dis}_{jh}$  | Variable  | Discharge power (MW)                              |

### Trade cost

Import cost minus export revenue:

```math
Z^{trade} = \sum_{h \in \mathcal{H}} \bigl(
  c^{imp}_h \; M_h \;-\; c^{exp}_h \; X_h
\bigr)
```

| Symbol        | Type      | Description                       |
| ------------- | --------- | --------------------------------- |
| $\mathcal{H}$ | Set       | Hours (time steps)                |
| $h$           | Index     | Hour                              |
| $c^{imp}_h$   | Parameter | Import price at hour $h$ (\$/MWh) |
| $c^{exp}_h$   | Parameter | Export price at hour $h$ (\$/MWh) |
| $M_h$         | Variable  | Imports (MW)                      |
| $X_h$         | Variable  | Exports (MW)                      |

---

## Constraints

### System constraints

#### Energy supply balance

```math
\forall\, h \in \mathcal{H}: \quad
\underbrace{
  G^{pv}_h
  + G^{wind}_h
  + \sum_{k \in \mathcal{K}} G^{bal}_{kh}
  + G^{hyd}_h
  + \alpha^{nuc} \nu_h + \alpha^{oth} \omega_h
  + \sum_{j \in \mathcal{J}} D^{dis}_{jh}
  + M_h
}_{\text{supply}}
\;=\;
\underbrace{
  d_h
  + \sum_{j \in \mathcal{J}} D^{ch}_{jh}
  + X_h
}_{\text{demand + charging + exports}}
```

> Curtailment does not appear in the energy balance. It is handled
> per-technology by the VRE balance constraints below.

#### Clean-energy generation target

Total balancing-unit generation must not exceed $(1 - \tau)$ of adjusted demand
(demand plus net storage loading):

```math
\sum_{k \in \mathcal{K}} \sum_{h \in \mathcal{H}} G^{bal}_{kh}
\;\leq\;
(1 - \tau) \sum_{h \in \mathcal{H}} \Bigl(
  d_h + \sum_{j \in \mathcal{J}} D^{ch}_{jh} - \sum_{j \in \mathcal{J}} D^{dis}_{jh}
\Bigr)
```

**Symbols used in system constraints:**

| Symbol         | Type      | Description                             |
| -------------- | --------- | --------------------------------------- |
| $\mathcal{H}$  | Set       | Hours (time steps)                      |
| $\mathcal{K}$  | Set       | Balancing units (dispatchable)          |
| $\mathcal{J}$  | Set       | Storage technologies                    |
| $h$            | Index     | Hour                                    |
| $k$            | Index     | Balancing unit                          |
| $j$            | Index     | Storage technology                      |
| $d_h$          | Parameter | Electrical demand at hour $h$ (MW)      |
| $\alpha^{nuc}$ | Parameter | Nuclear activation scalar               |
| $\alpha^{oth}$ | Parameter | Other renewables activation scalar      |
| $\nu_h$        | Parameter | Nuclear generation profile (MW)         |
| $\omega_h$     | Parameter | Other renewable generation profile (MW) |
| $\tau$         | Parameter | Minimum clean-energy generation share   |
| $G^{pv}_h$     | Variable  | Aggregate PV generation (MW)            |
| $G^{wind}_h$   | Variable  | Aggregate wind generation (MW)          |
| $G^{bal}_{kh}$ | Variable  | Balancing unit generation (MW)          |
| $G^{hyd}_h$    | Variable  | Dispatchable hydro generation (MW)      |
| $D^{ch}_{jh}$  | Variable  | Charge power (MW)                       |
| $D^{dis}_{jh}$ | Variable  | Discharge power (MW)                    |
| $M_h$          | Variable  | Imports (MW)                            |
| $X_h$          | Variable  | Exports (MW)                            |

### VRE balance constraints

Generation plus curtailment equals total available resource. Per-technology
equality:

#### PV balance

```math
\forall\, h \in \mathcal{H}: \qquad
G^{pv}_h + C^{pv}_h \;=\; \sum_{p \in \mathcal{P}} \sigma_{ph} \;\bar{p}^{pv}_p \; F^{pv}_p
```

#### Wind balance

```math
\forall\, h \in \mathcal{H}: \qquad
G^{wind}_h + C^{wind}_h \;=\; \sum_{w \in \mathcal{W}} \zeta_{wh} \;\bar{p}^{wind}_w \; F^{wind}_w
```

**Symbols used in VRE balance constraints:**

| Symbol             | Type      | Description                                     |
| ------------------ | --------- | ----------------------------------------------- |
| $\mathcal{H}$      | Set       | Hours (time steps)                              |
| $\mathcal{P}$      | Set       | PV plants                                       |
| $\mathcal{W}$      | Set       | Wind plants                                     |
| $h$                | Index     | Hour                                            |
| $p$                | Index     | PV plant                                        |
| $w$                | Index     | Wind plant                                      |
| $\sigma_{ph}$      | Parameter | Solar capacity factor for plant $p$ at hour $h$ |
| $\bar{p}^{pv}_p$   | Parameter | Max allowed PV capacity (MW)                    |
| $\zeta_{wh}$       | Parameter | Wind capacity factor for plant $w$ at hour $h$  |
| $\bar{p}^{wind}_w$ | Parameter | Max allowed wind capacity (MW)                  |
| $G^{pv}_h$         | Variable  | Aggregate PV generation (MW)                    |
| $C^{pv}_h$         | Variable  | PV curtailment (MW)                             |
| $F^{pv}_p$         | Variable  | Fraction of max PV capacity to build            |
| $G^{wind}_h$       | Variable  | Aggregate wind generation (MW)                  |
| $C^{wind}_h$       | Variable  | Wind curtailment (MW)                           |
| $F^{wind}_w$       | Variable  | Fraction of max wind capacity to build          |

### Balancing unit constraints

#### Dispatch limit

```math
\forall\, k \in \mathcal{K} \;\; \forall\, h \in \mathcal{H}: \qquad
G^{bal}_{kh} \;\leq\; P^{bal}_k
```

#### Capacity bounds

```math
\forall\, k \in \mathcal{K}: \qquad
\underline{p}^{bal}_k \;\leq\; P^{bal}_k \;\leq\; \bar{p}^{bal}_k
```

**Symbols used in balancing unit constraints:**

| Symbol                  | Type      | Description                         |
| ----------------------- | --------- | ----------------------------------- |
| $\mathcal{K}$           | Set       | Balancing units (dispatchable)      |
| $\mathcal{H}$           | Set       | Hours (time steps)                  |
| $k$                     | Index     | Balancing unit                      |
| $h$                     | Index     | Hour                                |
| $\underline{p}^{bal}_k$ | Parameter | Min balancing capacity (MW)         |
| $\bar{p}^{bal}_k$       | Parameter | Max allowed balancing capacity (MW) |
| $P^{bal}_k$             | Variable  | Installed balancing capacity (MW)   |
| $G^{bal}_{kh}$          | Variable  | Balancing unit generation (MW)      |

### Storage constraints

#### Coupled power equality

For coupled technologies, charge and discharge power capacity must be equal:

```math
\forall\, j \in \mathcal{J}^{c}: \qquad
P^{ch}_j \;=\; P^{dis}_j
```

#### Power capacity bounds

```math
\forall\, j \in \mathcal{J}: \qquad
P^{ch}_j \;\leq\; \bar{p}^{stor}_j \;, \quad
P^{dis}_j \;\leq\; \bar{p}^{stor}_j
```

#### Charge and discharge hourly limits

```math
\forall\, j \in \mathcal{J} \;\; \forall\, h \in \mathcal{H}: \qquad
D^{ch}_{jh} \;\leq\; P^{ch}_j \;, \quad
D^{dis}_{jh} \;\leq\; P^{dis}_j
```

#### Charge or discharge only

A storage unit cannot simultaneously charge and discharge. Enforced with binary
indicator $U_{jh}$ using $\bar{p}^{stor}_j$ as a tight big-$\mathcal{M}$:

```math
\forall\, j \in \mathcal{J} \;\; \forall\, h \in \mathcal{H}: \qquad
D^{ch}_{jh} \;\leq\; \bar{p}^{stor}_j \;\cdot\; U_{jh}
```

```math
\forall\, j \in \mathcal{J} \;\; \forall\, h \in \mathcal{H}: \qquad
D^{dis}_{jh} \;\leq\; \bar{p}^{stor}_j \;\cdot\; (1 - U_{jh})
```

#### State-of-charge balance

```math
\forall\, j \in \mathcal{J} \;\; \forall\, h \in \mathcal{H} \setminus \{1\}: \qquad
S_{jh} \;=\; S_{j(h-1)}
  \;+\; \sqrt{\eta_j}\; D^{ch}_{jh}
  \;-\; \frac{1}{\sqrt{\eta_j}}\; D^{dis}_{jh}
```

Cyclic boundary condition (SOC wraps around):

```math
\forall\, j \in \mathcal{J}: \qquad
S_{j1} \;=\; S_{j|\mathcal{H}|}
  \;+\; \sqrt{\eta_j}\; D^{ch}_{j1}
  \;-\; \frac{1}{\sqrt{\eta_j}}\; D^{dis}_{j1}
```

#### State-of-charge limits

```math
\forall\, j \in \mathcal{J} \;\; \forall\, h \in \mathcal{H}: \qquad
0 \;\leq\; S_{jh} \;\leq\; E_j
```

#### Duration limits

The energy-to-power ratio must lie within the allowable duration window.
Duration is defined relative to discharge power corrected for discharge
efficiency:

```math
\forall\, j \in \mathcal{J}: \qquad
\frac{\underline{\delta}_j \; P^{dis}_j}{\sqrt{\eta_j}}
\;\leq\; E_j \;\leq\;
\frac{\overline{\delta}_j \; P^{dis}_j}{\sqrt{\eta_j}}
```

#### Cycle limits

Total discharge throughput is bounded by the annualized maximum number of full
cycles (lifetime cycles divided by lifetime):

```math
\forall\, j \in \mathcal{J}: \qquad
\sum_{h \in \mathcal{H}} D^{dis}_{jh} \;\leq\; \frac{\kappa^{cyc}_j}{l^{stor}_j} \;\cdot\; E_j
```

**Symbols used in storage constraints:**

| Symbol                 | Type      | Description                             |
| ---------------------- | --------- | --------------------------------------- |
| $\mathcal{J}$          | Set       | Storage technologies                    |
| $\mathcal{J}^{c}$      | Set       | Coupled storage technologies            |
| $\mathcal{H}$          | Set       | Hours (time steps)                      |
| $j$                    | Index     | Storage technology                      |
| $h$                    | Index     | Hour                                    |
| $\bar{p}^{stor}_j$     | Parameter | Max installable power capacity (MW)     |
| $\eta_j$               | Parameter | Roundtrip efficiency                    |
| $\underline{\delta}_j$ | Parameter | Min duration (hr)                       |
| $\overline{\delta}_j$  | Parameter | Max duration (hr)                       |
| $\kappa^{cyc}_j$       | Parameter | Max lifetime cycles                     |
| $l^{stor}_j$           | Parameter | Storage lifetime (yr)                   |
| $P^{ch}_j$             | Variable  | Installed charge power capacity (MW)    |
| $P^{dis}_j$            | Variable  | Installed discharge power capacity (MW) |
| $E_j$                  | Variable  | Installed energy capacity (MWh)         |
| $D^{ch}_{jh}$          | Variable  | Charge power (MW)                       |
| $D^{dis}_{jh}$         | Variable  | Discharge power (MW)                    |
| $S_{jh}$               | Variable  | State of charge (MWh)                   |
| $U_{jh}$               | Variable  | Charge indicator ($1$ = charging)       |

### Hydro budget constraints

#### Hourly bounds

```math
\forall\, h \in \mathcal{H}: \qquad
\alpha^{hyd} \; \underline{g}^{hyd}_h \;\leq\; G^{hyd}_h \;\leq\; \alpha^{hyd} \; \overline{g}^{hyd}_h
```

#### Energy budget (equality)

Total dispatchable hydro generation within each budget period must **equal** the
available energy budget:

```math
\forall\, b \in \mathcal{B}: \qquad
\sum_{h \in \mathcal{H}_b} G^{hyd}_h \;=\; \epsilon_b
```

**Symbols used in hydro budget constraints:**

| Symbol                  | Type      | Description                          |
| ----------------------- | --------- | ------------------------------------ |
| $\mathcal{H}$           | Set       | Hours (time steps)                   |
| $\mathcal{B}$           | Set       | Hydro budget periods                 |
| $\mathcal{H}_b$         | Set       | Hours belonging to budget period $b$ |
| $h$                     | Index     | Hour                                 |
| $b$                     | Index     | Budget period                        |
| $\alpha^{hyd}$          | Parameter | Hydro activation scalar              |
| $\underline{g}^{hyd}_h$ | Parameter | Hourly hydro lower bound (MW)        |
| $\overline{g}^{hyd}_h$  | Parameter | Hourly hydro upper bound (MW)        |
| $\epsilon_b$            | Parameter | Energy budget for period $b$ (MWh)   |
| $G^{hyd}_h$             | Variable  | Dispatchable hydro generation (MW)   |

### Import and export constraints

#### Capacity limits

```math
\forall\, h \in \mathcal{H}: \qquad
M_h \;\leq\; \bar{\iota}_h \;, \quad
X_h \;\leq\; \bar{\xi}_h
```

#### Net-load indicator

Define the **net load** as demand minus all VRE availability, fixed clean
generation, and dispatchable hydro:

```math
\Lambda_h \;=\; d_h
  - (G^{pv}_h + C^{pv}_h)
  - (G^{wind}_h + C^{wind}_h)
  - \alpha^{nuc} \nu_h
  - \alpha^{oth} \omega_h
  - G^{hyd}_h
```

A binary indicator $V_h$ encodes whether the system is a net importer
($\Lambda_h > 0$) or net exporter ($\Lambda_h \leq 0$). An $\varepsilon$ offset
prevents numerical degeneracy at $\Lambda_h = 0$:

```math
\forall\, h \in \mathcal{H}: \qquad
\Lambda_h \;\leq\; \mathcal{M} \;\cdot\; V_h
```

```math
\forall\, h \in \mathcal{H}: \qquad
-\Lambda_h + \varepsilon \;\leq\; \mathcal{M} \;\cdot\; (1 - V_h)
```

#### Import allowed only when net load is positive

```math
\forall\, h \in \mathcal{H}: \qquad
M_h \;\leq\; d_h \;\cdot\; V_h
```

#### Export allowed only when net load is negative

```math
\forall\, h \in \mathcal{H}: \qquad
X_h \;\leq\; \Bigl(\max_{h' \in \mathcal{H}} \bar{\xi}_{h'}\Bigr) \;\cdot\; (1 - V_h)
```

**Symbols used in import and export constraints:**

| Symbol          | Type      | Description                                     |
| --------------- | --------- | ----------------------------------------------- |
| $\mathcal{H}$   | Set       | Hours (time steps)                              |
| $h$             | Index     | Hour                                            |
| $d_h$           | Parameter | Electrical demand at hour $h$ (MW)              |
| $\bar{\iota}_h$ | Parameter | Max import capacity at hour $h$ (MW)            |
| $\bar{\xi}_h$   | Parameter | Max export capacity at hour $h$ (MW)            |
| $\alpha^{nuc}$  | Parameter | Nuclear activation scalar                       |
| $\alpha^{oth}$  | Parameter | Other renewables activation scalar              |
| $\nu_h$         | Parameter | Nuclear generation profile (MW)                 |
| $\omega_h$      | Parameter | Other renewable generation profile (MW)         |
| $\mathcal{M}$   | Parameter | Big-M constant                                  |
| $\varepsilon$   | Parameter | Small offset to prevent numerical degeneracy    |
| $\Lambda_h$     | Derived   | Net load at hour $h$ (MW)                       |
| $G^{pv}_h$      | Variable  | Aggregate PV generation (MW)                    |
| $C^{pv}_h$      | Variable  | PV curtailment (MW)                             |
| $G^{wind}_h$    | Variable  | Aggregate wind generation (MW)                  |
| $C^{wind}_h$    | Variable  | Wind curtailment (MW)                           |
| $G^{hyd}_h$     | Variable  | Dispatchable hydro generation (MW)              |
| $M_h$           | Variable  | Imports (MW)                                    |
| $X_h$           | Variable  | Exports (MW)                                    |
| $V_h$           | Variable  | Net-load sign indicator ($1$ if net load $> 0$) |
