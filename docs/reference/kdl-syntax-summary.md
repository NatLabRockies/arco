# KDL Syntax Summary

Arco programs are written in KDL files. The language has two layers that share a
common algebra grammar and can coexist in the same file.

### Low-Level (Generic Optimization)

Self-contained models with explicit sets, parameters, variables, constraints,
and an objective. No domain assumptions.

### High-Level (Power-System Domain)

Technologies, operations, rules, assets, and scenarios that encode power-system
structure and get normalized into the same canonical form before solving. A
scenario can either use direct technology/operation/rule wiring or bind a
low-level model with `use`.

---

A `scenario` is the sole entry point for execution: it wires technologies,
operations, rules, data, assets, and an objective together with a time horizon.

## Naming Convention

Most declarations read their name from either position or property:

```kdl
technology "Battery"
technology name="Battery"
```

Both forms are equivalent everywhere a name is expected.

---

## Low-Level Layer (Generic Optimization)

The low-level layer is for writing optimization models directly, with no
power-system semantics. Everything the solver needs lives inside one `model`
block.

### `model`

A self-contained optimization model.

```kdl
model "GeneratorAllocation" {
  set "generators"
  set "time" from="horizon"

  param "capacity_mw" {
    g
  }
  param "demand" {
    t
  }

  control "dispatch" lower=0 {
    a
    t
  }

  constraint "capacity_limit" {
    dispatch[a,t] <= 10
  }

  minimize "TotalCost" {
    sum(dispatch[a,t] for a in assets for t in time)
  }
}
```

Children recognized inside `model`:

| Child                        | Required     | Description                                                                                                             |
| ---------------------------- | ------------ | ----------------------------------------------------------------------------------------------------------------------- |
| `set <name>`                 | no           | Declares an index set. Optional `from="..."` property for set source.                                                   |
| `param <name>`               | no           | Declares a parameter. Child nodes declare indexing (e.g. `param "demand" { t in time }`).                               |
| `control <name>`             | no           | Declares a decision variable family. Child nodes declare indexing. Optional `lower=<literal>` property for lower bound. |
| `constraint ...`             | no           | Inline constraint (same syntax as high-level, see below).                                                               |
| `minimize <name> { <math> }` | one required | Objective to minimize. Block body contains algebra expression.                                                          |
| `maximize <name> { <math> }` | one required | Objective to maximize. Block body contains algebra expression.                                                          |

Exactly one of `minimize` or `maximize` is required per model.

### Activating a model from a scenario

A scenario references a low-level model with `use`:

```kdl
scenario "AllocationDay" {
  horizon steps=3 resolution="PT1H"
  use "GeneratorAllocation"
}
```

---

## High-Level Layer (Power-System Domain)

The high-level layer provides domain-specific declarations that model
power-system structure. The compiler normalizes these into the same canonical
model form before solving. A `scenario` wires technologies, operations, and
rules directly (no intermediate abstraction).

### `technology`

Defines a reusable class of physical equipment and its variable families.

```kdl
technology "Battery" as="battery_units" {
  invest "power_charge" lower=0
  invest "power_discharge" lower=0
  invest "energy_cap" lower=0
  control "charge" lower=0
  control "discharge" lower=0
  control "charge_indicator" kind="binary"
  state "soc"
}
```

| Child                           | Description                                                                               |
| ------------------------------- | ----------------------------------------------------------------------------------------- |
| `invest <name>`                 | Investment (long-term) variable family.                                                   |
| `control <name>`                | Decision (operational) variable family, e.g. `charge[a,t]`.                               |
| `state <name>`                  | State variable family that carries across time steps, e.g. `soc[a,t]`.                    |
| **Optional on invest/control:** |                                                                                           |
| `lower=<value>`                 | Lower bound on the variable (literal number or algebraic expression).                     |
| `upper=<value>`                 | Upper bound on the variable (literal number or algebraic expression).                     |
| `kind="binary"`                 | Declare as binary (0/1). Default is continuous.                                           |
| **Optional on technology:**     |                                                                                           |
| `as="<set_name>"`               | Name the asset set for this technology (e.g. `battery_units`). Makes references explicit. |

### `operation`

Defines per-asset operational constraints tied to a technology.

```kdl
operation "PriceTakerBattery" {
  constraint "soc_balance" {
    soc[a,t] = soc[a,t-1]
      + charge_efficiency[a] * charge[a,t]
      - discharge[a,t] / discharge_efficiency[a]
  }
  constraint "charge_limit" {
    charge[a,t] <= power_mw[a]
  }
  constraint "soc_bounds" {
    0 <= soc[a,t] <= energy_mwh[a]
  }
}
```

Children recognized: `constraint` only.

### `rule`

Defines cross-asset or system-level constraints (e.g. energy balance).

Simple form (constraints use the same syntax as operations):

```kdl
rule "EnergyBalance" {
  constraint "balance" {
    sum(generation[a,t] for a in assets) + unserved_energy[t] = demand[t]
  }
}
```

Explicit generation form (using `over`/`when`/`expr` children):

```kdl
rule "NodeBalance" {
  constraint "balance" {
    over "n" in="nodes"
    over "t" in="time"
    when "active_node[n]"
    expr {
      sum(dispatch[g,t] for g in generators if generator_node[g] == n) = demand[n,t]
    }
  }
}
```

Children recognized: `constraint` only. Each constraint may optionally use the
explicit generation form described in the constraint syntax section below.

The `over`/`when`/`expr` form parses correctly but lowering to solver form is
not yet implemented. Use simple form for all production cases.

### `set`

Index set declaration at the top level or inside models. Two forms are
supported.

**Base set (no external source):**

```kdl
set "generators"
set "nodes"
```

**Sourced set (membership from external source):**

```kdl
set "time" from="horizon"
set "budget_periods" from="data/budget_periods.csv"
```

| Property     | Usage                                                                           |
| ------------ | ------------------------------------------------------------------------------- |
| `from="..."` | Source for set membership. Special values: `"horizon"` (scenario time horizon). |
|              | Path to CSV file loads membership from a column named `name` in the CSV.        |

Derived sets with filter conditions are not yet implemented.

### `expression`

A named reusable algebraic formula.

```kdl
expression "FuelCost" {
  sum(fuel_cost[a,t] * generation[a,t] for a in assets for t in time)
}

expression "TotalCost" {
  FuelCost + StartupCost + PenaltyCost
}
```

The block body is normalized internally to a `formula` child string. Expressions
can reference other named expressions by identifier.

### `minimize` / `maximize`

Top-level objective declarations. Define optimization objectives that can be
referenced and selected in scenarios.

```kdl
minimize "SystemCost" {
  FuelCost + StartupCost + PenaltyCost
}

maximize "ArbitrageProfit" {
  ArbitrageRevenue
}
```

The block body is an algebra expression that can reference named `expression`
declarations by identifier. A scenario selects which objective to optimize via
`minimize "Name"` or `maximize "Name"`.

These declarations can also appear inside `model` blocks with the same syntax.

### `asset`

Instantiates one concrete asset with scalar parameters.

```kdl
asset "Battery1" {
  technology "Battery"
  operation "PriceTakerBattery"
  power_mw 100
  energy_mwh 400
  charge_efficiency 0.92
  discharge_efficiency 0.92
  initial_soc_mwh 200
  terminal_soc_mwh 200
}

asset "TradeNode" {
  technology "Trade"
  operation "TradeOp"
  max_import 500
  max_export 500
}
```

| Child               | Required | Description                                                                                         |
| ------------------- | -------- | --------------------------------------------------------------------------------------------------- |
| `technology <name>` | yes      | Which technology this asset belongs to.                                                             |
| `operation <name>`  | no       | Which operation governs this asset.                                                                 |
| Any other node name | no       | Treated as a parameter. Value is the first literal argument (string, integer, decimal, or boolean). |

### `instances`

Bulk-instantiates many assets from a CSV file. Supports explicit column mapping
via `map` or auto-mapping (columns automatically map to parameters by name).

**Explicit mapping (recommended):**

```kdl
instances "ThermalUnits" from="data/thermal_units.csv" {
  technology "Thermal"
  operation "StandardUC"
  map "name" from="asset_name"
  map "p_min" from="p_min_mw"
  map "p_max" from="p_max_mw"
  map "ramp_up" from="ramp_up"
  map "ramp_down" from="ramp_down"
  map "startup_cost" from="startup_cost"
}
```

**Auto-mapping (columns must match parameter names):**

```kdl
instances "PVPlants" from="data/pv_plants.csv" {
  technology "PV"
  operation "PVDispatch"
}
```

| Child                         | Description                                                                           |
| ----------------------------- | ------------------------------------------------------------------------------------- |
| `technology <name>`           | Technology for all instantiated assets. Required.                                     |
| `operation <name>`            | Operation for all instantiated assets. Optional.                                      |
| `map <param> from="<column>"` | Maps a CSV column (`from`) to a DSL parameter (positional arg). Optional, repeatable. |

Required property on the declaration node: `from="<path>"` (path to CSV file).

When `map` directives are absent, CSV columns are auto-mapped to parameters by
name (column `p_max` becomes parameter `p_max`).

### `scenario`

Sole entry point for execution. Wires a time horizon, data sources, and
(optionally) technologies, operations, rules, and assets into an executable
case. Can either use direct technology/operation/rule wiring or bind a low-level
model with `use`.

**High-level layer (direct wiring):**

```kdl
scenario "BaseCase" {
  horizon steps=24 resolution="PT1H"
  technology "Thermal"
  operation "StandardUC"
  rule "EnergyBalance"
  set "budget_periods" from="data/budget_periods.csv"
  data "demand" from="data/demand.csv"
  data "fuel_cost" from="data/fuel_cost.csv"
  data "lol_penalty" from="data/lol_penalty.csv"
  data "initial_commitment" from="data/initial_commitment.csv"
  instances "ThermalUnits"
  minimize "SystemCost"
  report "FuelCost"
  report "StartupCost"
  report "PenaltyCost"
}
```

**Low-level layer (model binding):**

```kdl
scenario "AllocationDay" {
  horizon steps=3 resolution="PT1H"
  use "GeneratorAllocation"
  data "demand" from="data/demand.csv"
  minimize "TotalCost"
}
```

| Child                                     | Required | Description                                          |
| ----------------------------------------- | -------- | ---------------------------------------------------- |
| `horizon steps=<int> resolution=<string>` | yes      | Time structure.                                      |
| `technology <name>`                       | no       | Includes a technology (direct wiring). Repeatable.   |
| `operation <name>`                        | no       | Includes an operation (direct wiring). Repeatable.   |
| `rule <name>`                             | no       | Includes a rule (direct wiring). Repeatable.         |
| `set <name> from=<source>`                | no       | Declares or sources an index set. Repeatable.        |
| `data <name> from=<path>`                 | no       | Binds a named data source to a CSV file. Repeatable. |
| `asset <name>`                            | no       | Includes an asset. Repeatable.                       |
| `instances <name>`                        | no       | Includes an instances block. Repeatable.             |
| `use <name>`                              | no       | Binds a low-level model (low-level layer only).      |
| `minimize <name>` / `maximize <name>`     | no       | Selects which objective to optimize.                 |
| `report <name>`                           | no       | Output expression to include in results. Repeatable. |

Scenarios using direct wiring list technologies, operations, and rules directly.
Scenarios using the low-level layer bind a model with `use`.

### `workflow`

Staged execution DAG for composing multiple optimization problems into a
multi-step pipeline (e.g. unit commitment followed by economic dispatch).
Defined in [RFD-0013](./rfd-0013-problem-composition-and-staged-workflows.md).
Not yet implemented.

---

## Constraint Syntax

The constraint syntax is shared across `operation`, `rule`, and `model` blocks.

### Simple form

Named with block body:

```kdl
constraint "charge_limit" {
  charge[a,t] <= power_mw[a]
}
```

Named with `name=` property and generation filter:

```kdl
constraint name="dispatch_limit" if="apply_dispatch_limit[a] == 1" {
  dispatch[a,t] <= existing_capacity[a]
}
```

Unnamed (auto-named `constraint_1`, `constraint_2`, ...):

```kdl
constraint {
  0 <= soc[a,t] <= energy_mwh[a]
}
```

### Explicit generation form

For constraints that need explicit index domains and filtering (e.g. multi-node
network balances), use `over`, `when`, and `expr` children. This form is fully
parsed but lowering to solver form is not yet implemented.

```kdl
constraint "balance" {
  over "n" in="nodes"
  over "t" in="time"
  when "active_node[n]"
  expr {
    sum(dispatch[g,t] for g in generators if generator_node[g] == n) = demand[n,t]
  }
}
```

| Child                    | Required                         | Description                                                                                        |
| ------------------------ | -------------------------------- | -------------------------------------------------------------------------------------------------- |
| `over <var> in=<domain>` | no                               | Binds an index variable to a set. Repeatable.                                                      |
| `when <filter>`          | no                               | Boolean filter expression. Constraint is only generated for combinations where the filter is true. |
| `expr { <math> }`        | yes (when using generation form) | The constraint math expression.                                                                    |

When the body starts with `over`, `when`, or `expr`, the normalizer skips
math-block rewriting and the parser reads proper KDL children instead.

**Status:** Syntax is fully supported. Lowering to solver form is not yet
implemented. For production use, continue using the simple form.

### Normalized form

After surface normalization, simple constraints become:

```kdl
constraint "charge_limit" expression="charge[a,t] <= power_mw[a]"
```

---

## Algebra Grammar

Algebra appears in constraint bodies, expression formulas, `minimize`/`maximize`
bodies, `expr` blocks, `when` filters, and `if="..."` generation filters.

### Atoms

| Form              | Example                          |
| ----------------- | -------------------------------- |
| Number            | `0`, `100`, `0.92`               |
| String            | `"north"`                        |
| Boolean           | `true`, `false`                  |
| Identifier        | `demand`, `FuelCost`             |
| Indexed reference | `dispatch[a,t]`, `soc[a,t-1]`    |
| Parenthesized     | `(discharge[a,t] - charge[a,t])` |

Identifier rules: first character is ASCII letter or `_`, remaining characters
are ASCII letter, digit, or `_`.

### Arithmetic Operators

| Operator | Symbol      | Precedence |
| -------- | ----------- | ---------- |
| Negate   | `-` (unary) | highest    |
| Multiply | `*`         | 3          |
| Divide   | `/`         | 3          |
| Add      | `+`         | 2          |
| Subtract | `-`         | 2          |

### Comparison Operators

| Operator         | Symbol | Typical use         |
| ---------------- | ------ | ------------------- |
| Assignment equal | `=`    | Constraints         |
| Double equal     | `==`   | Filters             |
| Not equal        | `!=`   | Filters             |
| Less than        | `<`    | Parsed, not lowered |
| Less or equal    | `<=`   | Constraints         |
| Greater than     | `>`    | Parsed, not lowered |
| Greater or equal | `>=`   | Constraints         |

### Constraint Body Forms

Two-sided comparison:

```
dispatch[a,t] <= capacity_mw[a]
soc[a,t] = soc[a,t-1] - dispatch[a,t]
```

Three-part range (lowers to two constraints):

```
0 <= soc[a,t] <= energy_mwh[a]
-thermal_limit_mw[l] <= flow[l,t] <= thermal_limit_mw[l]
```

### Reductions

The only reduction operator currently implemented is `sum(...)`.

```
sum(expr for <binding> [for <binding>]... [if <filter>]...)
```

Binding patterns:

- Simple: `for a in assets`
- Tuple: `for (i,j) in arcs`

Multiple `for` bindings:

```
sum(fuel_cost[a,t] * generation[a,t] for a in assets for t in time)
```

With filters:

```
sum(dispatch[g,t] for g in generators if generator_node[g] == n)
sum(flow[l,t] for l in incoming_lines if line_to[l] == n)
```

Rules:

- At least one `for ... in ...` binding is required.
- Multiple `for` bindings are allowed.
- Optional `if` filters follow all bindings.

### Index Arithmetic

Index expressions support arithmetic for temporal lag:

```
soc[a,t-1]
commit[a,t-1]
generation[a,t-1]
```

The pattern `[..., t-1]` triggers chronology boundary validation.

### Expression Composition

Objective bodies and expression formulas can reference named `expression`
declarations by identifier and compose them:

```kdl
expression "FuelCost" { ... }
expression "StartupCost" { ... }
expression "PenaltyCost" { ... }

minimize "SystemCost" {
  FuelCost + StartupCost + PenaltyCost
}
```

---

## Surface-Syntax Normalization

The parser applies a normalization pass before KDL parsing. These block-body
math forms:

```kdl
constraint "name" { <math> }
expression "name" { <math> }
minimize "name" { <math> }
maximize "name" { <math> }
expr { <math> }
```

are rewritten to:

```kdl
constraint "name" expression="<math>"
expression "name" { formula "<math>" }
minimize "name" expression="<math>"
maximize "name" expression="<math>"
expr expression="<math>"
```

Multi-line math bodies are joined into a single line with spaces. Keep `{` on
the same line as the declaration header.

The normalizer skips constraint rewriting when the body starts with `over`,
`when`, or `expr` keywords, so that the explicit generation form is parsed as
proper KDL children.

---

## Not Yet Implemented

### Algebra & Reductions

- `min(...)` and `max(...)` reductions
- Exponent operator (`^` or `**`)
- Boolean keyword operators in formulas (`and`, `or`, `not`)
- Lowering of `if` filters inside reductions (parses but returns an error at
  solve time)
- Lowering of tuple bindings inside reductions (parses but returns an error at
  solve time)

### Declarations & Bindings

- Derived set declarations with filter conditions
  (`set "name" when="condition"`)
- `over`/`when`/`expr` constraint generation lowering (parses but not yet
  lowered to solver form; see [Constraint Syntax](#constraint-syntax))
- Explicit parameter/control/state declarations with index signatures
  ([RFD-0011](./rfd-0011-inferred-declarations-and-explicit-control.md))
- `workflow` declaration parsing and execution
  ([RFD-0013](./rfd-0013-problem-composition-and-staged-workflows.md))

### Deprecated (Removed)

The following declarations were part of earlier versions and have been
eliminated by the syntax redesign:

- `template` (replaced by direct technology/operation/rule wiring in scenarios)
- `objective` (replaced by top-level `minimize`/`maximize`)
- `study` (removed; modes and study selection no longer supported)
- `mode` (removed from variable domain policy)
