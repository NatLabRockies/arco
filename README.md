<div align="center">

# Arco

**A memory-smart optimization DSL and solver for LP and MIP problems on
constrained hardware.**

[![CI](https://img.shields.io/github/actions/workflow/status/NatLabRockies/arco/ci.yaml?branch=main&label=CI)](https://github.com/NatLabRockies/arco/actions/workflows/ci.yaml)
[![Rust](https://img.shields.io/badge/rust-1.85-orange)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.9%2B-blue)](https://pypi.org/project/arco/)
[![License](https://img.shields.io/badge/license-BSD--3--Clause-green)](./licenses/BSD-3-Clause.txt)
[![Docs](https://img.shields.io/badge/docs-di%C3%A1taxis-blue)](./docs/)
[![PyPI](https://img.shields.io/pypi/v/arco)](https://pypi.org/project/arco/)

</div>

> [!WARNING] Arco is built primarily for internal use within our organization.
> You are welcome to try it, but we make no guarantees about API stability or
> robustness at this stage. For battle-tested alternatives, consider
> [Pyomo](https://www.pyomo.org/) (Python) or [JuMP](https://jump.dev/) (Julia).

Arco (**Assembled Resource-Constrained Optimization**) is an optimization
framework built around a KDL-based domain-specific language and a CLI
compiler/solver. You write optimization models in `.kdl` files, and the `arco`
CLI compiles, validates, inspects, and solves them. Language bindings (Python
today, more planned) provide programmatic access to the same engine.

Built for harder optimization problems on constrained resources, Arco is
intentional about every allocation, careful with stack and heap behavior, and
relentless about minimizing memory usage so more systems can run real workloads.

<p align="center">
  <a href="#quickstart">Quickstart</a> ·
  <a href="#installation">Installation</a> ·
  <a href="#kdl-language">KDL Language</a> ·
  <a href="#cli-reference">CLI Reference</a> ·
  <a href="#language-bindings">Language Bindings</a> ·
  <a href="#features">Features</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#benchmarking">Benchmarking</a> ·
  <a href="#contributing">Contributing</a> ·
  <a href="#license">License</a>
</p>

## Quickstart

Install the CLI:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NatLabRockies/arco/releases/latest/download/arco-installer.sh | sh
```

Write an optimization model. This one maximizes battery arbitrage revenue over
a 24-hour price curve:

```kdl
// input.kdl
technology Battery {
  control charge
  control discharge
  state soc
}

operation PriceTakerBattery {
  constraint soc_balance {
    soc[a,t] = soc[a,t-1]
      + charge_efficiency[a] * charge[a,t]
      - discharge[a,t] / discharge_efficiency[a]
  }
  constraint charge_limit { charge[a,t] <= power_mw[a] }
  constraint discharge_limit { discharge[a,t] <= power_mw[a] }
  constraint soc_bounds { 0 <= soc[a,t] <= energy_mwh[a] }
}

expression ArbitrageRevenue {
  sum(prices[t] * (discharge[a,t] - charge[a,t])
      for a in assets for t in time)
}

maximize ArbitrageProfit { ArbitrageRevenue }

asset Battery1 {
  technology Battery
  operation PriceTakerBattery
  power_mw 100
  energy_mwh 400
  charge_efficiency 0.92
  discharge_efficiency 0.92
  initial_soc_mwh 200
  terminal_soc_mwh 200
}

scenario BatteryArbitrageDay {
  horizon steps=24 resolution=PT1H
  technology Battery
  operation PriceTakerBattery
  data prices from="data/prices.csv"
  asset Battery1
  maximize ArbitrageProfit
  report ArbitrageRevenue
}
```

Supply a price curve in `data/prices.csv`:

```csv
t,prices
1,35
2,30
3,25
4,20
5,18
6,15
7,16
8,19
9,24
10,32
11,40
12,48
13,55
14,60
15,58
16,52
17,49
18,46
19,42
20,38
21,33
22,31
23,29
24,27
```

Solve it:

```bash
arco run input.kdl --compact
```

```json
{
  "solve_status": "optimal",
  "active_scenario": "BatteryArbitrageDay",
  "objective": {
    "name": "ArbitrageProfit",
    "sense": "maximize",
    "value": 13221.22
  },
  "reports": [
    { "name": "ArbitrageRevenue", "value": 13221.22 }
  ],
  "counts": { "parameters": 7, "variables": 3, "constraints": 4 },
  "timing": { "total_ms": 8.48 }
}
```

> [!NOTE] Arco embeds the HiGHS solver. No external solver installation or
> configuration required.

## Installation

### CLI

macOS and Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NatLabRockies/arco/releases/latest/download/arco-installer.sh | sh
```

Windows (PowerShell):

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/NatLabRockies/arco/releases/latest/download/arco-installer.ps1 | iex"
```

The installer places the `arco` binary in `~/.cargo/bin` and includes a
self-updater (`arco-update`).

### Python Binding

Python 3.9 or newer. Using `uv` (recommended) or `pip`:

```bash
uv add arco
```

```bash
pip install arco
```

### From Source

For development or to build everything locally:

```bash
git clone https://github.com/NatLabRockies/arco.git
cd arco

# Install just (command runner)
cargo install just

# Build CLI
just build

# Build and install Python extension in development mode
just py-dev

# Run tests
just test
uv run pytest
```

## KDL Language

Arco models are written in [KDL](https://kdl.dev) files. The language has two
layers that share a common algebra grammar and can coexist in the same file.

### High-Level Layer (Power-System Domain)

The high-level layer provides domain-specific declarations for modeling power
systems. You define technologies, operations, rules, assets, and scenarios. The
compiler normalizes these into a canonical form before solving.

**Technologies** declare the variable families for a class of equipment:

```kdl
technology Thermal {
  control generation
  state commit
  state start
  state shutdown
}
```

**Operations** define per-asset constraints tied to a technology:

```kdl
operation StandardUC {
  constraint commitment_transition {
    commit[a,t] - commit[a,t-1] = start[a,t] - shutdown[a,t]
  }
  constraint min_output { generation[a,t] >= p_min[a] * commit[a,t] }
  constraint max_output { generation[a,t] <= p_max[a] * commit[a,t] }
  constraint ramp_up_limit {
    generation[a,t] - generation[a,t-1] <= ramp_up[a]
  }
  constraint ramp_down_limit {
    generation[a,t-1] - generation[a,t] <= ramp_down[a]
  }
}
```

**Rules** define system-level constraints across assets:

```kdl
rule EnergyBalance {
  constraint balance {
    sum(generation[a,t] for a in assets) + unserved_energy[t] = demand[t]
  }
}
```

**Instances** bulk-create assets from CSV data:

```kdl
instances ThermalUnits from="data/thermal_units.csv" {
  technology Thermal
  operation StandardUC
  map name from=asset_name
  map p_min from=p_min_mw
  map p_max from=p_max_mw
  map ramp_up from=ramp_up
  map ramp_down from=ramp_down
  map startup_cost from=startup_cost
}
```

**Expressions** define named, reusable formulas. **Scenarios** wire everything
together for execution:

```kdl
expression FuelCost {
  sum(fuel_cost[a,t] * generation[a,t] for a in assets for t in time)
}
expression StartupCost {
  sum(startup_cost[a] * start[a,t] for a in assets for t in time)
}
expression PenaltyCost {
  sum(lol_penalty[t] * unserved_energy[t] for t in time)
}

minimize SystemCost { FuelCost + StartupCost + PenaltyCost }

scenario BaseCase {
  horizon steps=24 resolution=PT1H
  technology Thermal
  operation StandardUC
  rule EnergyBalance
  data demand from="data/demand.csv"
  data fuel_cost from="data/fuel_cost.csv"
  data lol_penalty from="data/lol_penalty.csv"
  data initial_commitment from="data/initial_commitment.csv"
  instances ThermalUnits
  minimize SystemCost
  report FuelCost
  report StartupCost
  report PenaltyCost
}
```

### Low-Level Layer (Generic Optimization)

For problems without power-system semantics, the low-level layer provides a
self-contained `model` block with explicit sets, variables, constraints, and an
objective. A `scenario` activates it with `use`:

```kdl
model GeneratorAllocation {
  control dispatch lower=0 {
    a
    t
  }

  constraint capacity_limit {
    dispatch[a,t] <= capacity[a]
  }

  minimize TotalCost {
    sum(cost[a] * dispatch[a,t] for a in assets for t in time)
  }
}

scenario AllocationDay {
  horizon steps=24 resolution=PT1H
  use GeneratorAllocation
  data capacity from="data/capacity.csv"
  data cost from="data/cost.csv"
}
```

> [!TIP] See the [KDL syntax reference](./docs/reference/kdl-syntax-summary.md)
> for the full grammar, algebra operators, constraint forms, and reduction
> syntax. Complete working examples live in the [`examples/`](./examples/)
> directory.

## CLI Reference

The `arco` CLI compiles and solves KDL optimization models.

```
arco <command> [options]
```

| Command                   | Description                                         |
| :------------------------ | :-------------------------------------------------- |
| `arco run <file>`         | Compile and solve a `.kdl` formulation              |
| `arco validate <file>`    | Validate a `.kdl` file without solving              |
| `arco inspect <file>`     | Inspect semantic model (sets, variables, parameters) |
| `arco print-model <file>` | Print the algebraic model sent to the solver        |
| `arco export <file>`      | Export as LP or MPS format                          |
| `arco debug <file>`       | Open an interactive IPython debug shell             |
| `arco solver show`        | Show the active solver backend                      |
| `arco solver set <name>`  | Set the solver backend (`highs` or `xpress`)        |

### Examples

Validate without solving:

```bash
$ arco validate input.kdl
Validated file://input.kdl in 4ms (arco 0.2.8)
```

Inspect the semantic model:

```bash
$ arco inspect input.kdl --section constraints
[constraint]
  name     : soc_balance
  template : soc[a,t] = soc[a,t-1] + charge_efficiency[a] * charge[a,t] - ...
  relation : equal
  ...

[constraint]
  name     : charge_limit
  template : charge[a,t] <= power_mw[a]
  relation : less_or_equal
  ...
```

Export to LP format for external solvers:

```bash
arco export input.kdl --format lp --output model.lp
```

Use `-v` for info-level tracing, `-vv` for debug-level. Pass `--compact` to
`arco run` to omit full variable value arrays from the JSON output. Use
`--filter-variable` or `--filter-asset` to narrow results.

## Language Bindings

Arco provides language bindings for programmatic access to the optimization
engine. Python is the first available binding, with more languages planned.

### Python

Install with `uv` (recommended) or `pip`:

```bash
uv add arco
```

Build and solve a production planning problem:

```python
import arco

model = arco.Model()

x = model.add_variable(
    bounds=arco.Bounds(lower=1.0, upper=float("inf")),
    name="product_x"
)
y = model.add_variable(
    bounds=arco.Bounds(lower=2.0, upper=float("inf")),
    name="product_y"
)

model.add_constraint(x + y >= 5.0, name="demand")
model.minimize(3.0 * x + 2.0 * y)

solution = model.solve()
assert solution.is_optimal()

print(f"Optimal: x={solution.value(x):.1f}, y={solution.value(y):.1f}")
print(f"Cost: {solution.objective_value:.1f}")
```

<details>
<summary><strong>Indexed Variables</strong></summary>

Work with structured, array-like variables for large-scale problems:

```python
import arco

model = arco.Model()

plants = model.add_index_set(["NYC", "LA", "CHI"])
products = model.add_index_set(range(5))

production = model.add_variables(
    index_sets=[plants, products],
    bounds=arco.Bounds(lower=0, upper=100),
    name="production"
)

total_by_plant = production.sum(axis=1)
model.add_constraint(total_by_plant >= 10)

solution = model.solve()
```

</details>

<details>
<summary><strong>Block Composition</strong></summary>

Compose multi-stage optimization workflows using blocks:

```python
from dataclasses import dataclass
import arco
from arco import block

@dataclass
class FacilityInput:
    capacity: float
    demand: float

@block
def facility_block(model, data: FacilityInput):
    x = model.add_variable(lb=0, ub=data.capacity, name="output")
    model.add_constraint(x >= data.demand)
    model.minimize(x)
    return {"output": x}

model = arco.Model()
block_handle = model.add_block(
    facility_block, FacilityInput(capacity=100, demand=50)
)
solution = model.solve()
```

</details>

> [!TIP] See the [tutorials](./docs/tutorials/) and
> [how-to guides](./docs/how-to/) for comprehensive Python examples.

## Features

| Feature                   | Status | Description                                                          |
| :------------------------ | :----: | :------------------------------------------------------------------- |
| **KDL Optimization DSL**  |   ✅   | Two-layer DSL for generic and power-system optimization models       |
| **CLI Compiler/Solver**   |   ✅   | Compile, validate, inspect, solve, and export from the command line  |
| **LP / MIP Solving**      |   ✅   | Linear and mixed-integer programming via embedded HiGHS              |
| **HiGHS Backend**         |   ✅   | Open-source solver embedded out of the box                           |
| **Xpress Backend**        |   ✅   | Commercial solver support for enterprise users                       |
| **Model Inspection**      |   ✅   | Semantic introspection of sets, variables, constraints, parameters   |
| **LP / MPS Export**       |   ✅   | Export algebraic models for external solvers                         |
| **CSV Data Binding**      |   ✅   | Wire model parameters to CSV data sources in scenarios               |
| **Block Orchestration**   |   ✅   | DAG-based composition for multi-stage problems                       |
| **Memory Diagnostics**    |   ✅   | Built-in tracking of memory usage and bottlenecks                    |
| **Warm Starting**         |   ✅   | Reuse solutions across sequential solves                             |
| **Python Binding**        |   ✅   | Programmatic model building with NumPy integration                   |
| **Editor Support**        |   ✅   | Tree-sitter grammar overlay for KDL + algebra syntax highlighting    |
| **Parallel Block Solve**  |   🚧   | Under testing for concurrent block execution                         |
| **Additional Bindings**   |   📋   | Planned language bindings beyond Python                              |
| **Distributed Execution** |   📋   | Planned for distributed optimization workflows                       |

**Legend:** ✅ Available | 🚧 Under Testing | 📋 Planned

## Architecture

Arco is organized as a Rust workspace. The KDL DSL and CLI are the primary
interface. Language bindings provide programmatic access to the same core.

```mermaid
graph TB
    subgraph DSL["KDL DSL + CLI"]
        A[".kdl Model Files"]
        B["arco CLI"]
        C[arco-kdl<br/>Parser & Compiler]
    end

    subgraph Bindings["Language Bindings"]
        D[Python<br/>arco-bindings-python]
        E["Future Bindings<br/>(planned)"]
    end

    subgraph Core["Rust Workspace"]
        F[arco-core<br/>Model Builder]
        G[arco-expr<br/>Expression Engine]
        H[arco-solver<br/>Solver Abstractions]
        I[arco-blocks<br/>Block Composition]
        J[arco-tools<br/>Memory Diagnostics]
    end

    subgraph Solvers["Solver Backends"]
        K[HiGHS<br/>Embedded]
        L[Xpress<br/>Optional]
    end

    A --> B
    B --> C
    C --> F
    D --> F
    E -.-> F
    F --> G
    F --> J
    H --> K
    H --> L
    F --> H
    I --> F
```

### Crate Overview

| Crate         | Purpose                                                |
| :------------ | :----------------------------------------------------- |
| `arco-cli`    | CLI compiler and solver for KDL optimization models    |
| `arco-kdl`    | KDL parser, semantic validation, and algebraic lowering |
| `arco-core`   | Model construction, variables, constraints, objectives |
| `arco-expr`   | Expression trees and constraint generation             |
| `arco-solver` | Solver-agnostic abstractions and solution handling     |
| `arco-highs`  | HiGHS solver integration (embedded)                    |
| `arco-blocks` | DAG-based block composition and orchestration          |
| `arco-tools`  | Memory instrumentation and diagnostics                 |
| `arco-bench`  | Benchmarking framework for regression testing          |

## Benchmarking

Use `arco-bench` to run performance benchmarks and catch regressions:

```bash
# Run default benchmark scenarios
just bench-run

# Run with custom parameters
just bench-run --scenario model-build,fac25 --cases 1000,10000 --repetitions 3

# Generate report
just bench-report artifacts/bench/results.jsonl

# Compare and gate on regressions
just bench-gate baseline.jsonl candidate.jsonl 5 5
```

## Contributing

Contributions are welcome. Please see [`CONTRIBUTING.md`](./CONTRIBUTING.md) for
the development workflow, testing expectations, and documentation requirements.

Quick start for contributors:

```bash
# Setup
just fmt      # Format code
just clippy   # Run linter
just check    # Type-check workspace

# Testing
just test     # Run Rust tests
just py-test  # Run Python doctests

# Full CI gate
just ci
```

Release and versioning behavior is defined in
[`RELEASE_POLICY.md`](./RELEASE_POLICY.md).

## License

Arco is licensed under the BSD 3-Clause License. See
[`licenses/BSD-3-Clause.txt`](./licenses/BSD-3-Clause.txt) for details.

The embedded HiGHS solver is licensed under the MIT License. See
[`licenses/HiGHS-MIT.txt`](./licenses/HiGHS-MIT.txt) for details.

---

<div align="center">

**[Documentation](./docs/)** ·
**[Examples](./examples/)** ·
**[Issues](https://github.com/NatLabRockies/arco/issues)** ·
**[Releases](https://github.com/NatLabRockies/arco/releases)**

</div>
