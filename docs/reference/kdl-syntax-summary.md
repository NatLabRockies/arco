# Arco Low-Level KDL Syntax Specification (KDL 2.0 Profile)

Version: 0.1.0

This document defines the low-level Arco DSL profile authored in KDL 2.0.

Scope of this specification:

- `data` declarations (CSV-backed namespaces)
- `set` declarations (dataset and model domains)
- `subset` declarations (named filtered views)
- `param` declarations (projection, indexing, filtering, aggregation)
- `control` declarations (decision-variable families)
- `expression` declarations (named reusable formulas)
- `constraint` declarations (low-level algebra rows)
- `scenario` declarations (execution entrypoints)

High-level domain declarations (`technology`, `operation`, `rule`, `asset`) are
not part of this low-level specification.

---

## 1. Conformance

Arco KDL files MUST conform to KDL 2.0:

- KDL 2.0 spec: https://kdl.dev/spec/
- UTF-8 encoding
- KDL node/value type annotations are allowed
- File extension: `.kdl`

Arco adds semantic validation on top of KDL parsing.

KDL comments (`//` line comments and `/-` slashdash comments) are fully
supported. Slashdash (`/-`) comments out an entire node, property, or argument,
which is useful for toggling declarations during development.

Normative keywords:

- MUST, MUST NOT
- SHOULD, SHOULD NOT
- MAY

---

## 2. Top-level declarations

A low-level document MAY contain these top-level declarations:

- `data`
- `subset`
- `model`
- `scenario`

```kdl
// top-level data is shared across all scenarios
data plants from="data/plants.csv" { ... }
data units from="data/units.csv" { ... }

subset solar_north from=generator_data class=solar area=north

model dispatch_model { ... }

scenario day_ahead {
  horizon steps=24 resolution="PT1H"
  use dispatch_model
  // scenario-level data is scoped to this scenario only
  data demand from="data/demand.csv"
}
```

`scenario` is the execution entrypoint.

Declaration order: top-level declarations MAY appear in any order. Forward
references are allowed (a `subset` may reference a `data` block declared after
it, a `scenario` may reference a `model` declared after it). All names are
resolved after the full document is parsed.

---

## 3. `data` declaration

`data` declares one CSV-backed namespace. Fields, sets, and parameters declared
inside the block become available to models and scenarios that reference them.

```kdl
data <name> from=<path> { ... }
```

Required properties:

- `from`: CSV path

Allowed children:

- `map`
- `set`
- `index`
- `param`

### 3.1 `map`

`map` binds logical names to CSV headers.

```kdl
map <logical_name>
map <logical_name> from=<source_header>
```

Semantics:

- If `from` is omitted, source header defaults to `<logical_name>`. The column
  MUST exist in the CSV.
- Mapping is optional. Unmapped columns remain available.
- Duplicate logical targets MUST fail validation.

### 3.2 `set` (inside `data`)

`set` extracts unique values from a dataset column and exposes them as a named
domain. The column used is the one matching `<name>` (after `map` resolution).

```kdl
set <name>
set <name> alias=<short>
set <name> subset_of=<parent_set>
set <name> subset_of=<parent_set> filter_by=<field> eq=<value>
set <name> subset_of=<parent_set> filter_by=<field> geq=<value> leq=<value>
```

Semantics:

- `set class` extracts unique values from the `class` column.
- `alias` provides a short iteration variable name for use in algebra
  expressions. Example: `set asset_id alias=a` allows `dispatch[a,t]` instead of
  `dispatch[asset_id,time]`.
- `subset_of` declares that each child value maps to exactly one parent value,
  forming a hierarchy edge.
- Comparator filters narrow members. `filter_by` MUST be present when
  comparators are used.
- `filter_by` without `subset_of` filters the root column directly.

Comparator properties:

- `eq`, `ge` (`>`), `geq` (`>=`), `le` (`<`), `leq` (`<=`)

### 3.3 `index` (inside `data`)

`index` defines default indexing for `param` declarations in that `data` block.

```kdl
index <set_a> <set_b> ...
```

Semantics:

- `index` is optional.
- If omitted, default index is numeric row order.
- Every index symbol MUST be a declared set.
- At most one `index` declaration is allowed per `data` block.

### 3.4 `param` (inside `data`)

`param` projects values from dataset fields.

Single-dimension indexing (property form):

```kdl
param <name>
param <name> from=<field>
param <name> index_by=<set>
```

Multi-dimension indexing (child node form):

```kdl
param <name> { index <set_a>; index <set_b> }
param <name> from=<field> { index <set_a>; index <set_b> }
```

`index_by` and `index` children are mutually exclusive. Using both on the same
`param` MUST fail validation.

Aggregation:

```kdl
param <name> index_by=<set> reduce=<reducer>
param <name> { index <set_a>; index <set_b>; reduce sum }
```

Supported reducers:

- `sum`, `avg`, `min`, `max`, `first`, `last`

Filtering:

```kdl
param <name> from=<field> filter_by=<field> eq=<value>
param <name> from=<field> geq=<value>
param <name> from=<field> geq=<value> leq=<value>
```

Units metadata:

```kdl
param capacity_mw units=MW
param fuel_cost units="$/MMBtu"
```

Semantics:

- If `from` is omitted, source field defaults to `<name>`.
- If neither `index_by` nor `index` children are present, default is `index` if
  declared, else numeric row index.
- If indexing is non-unique, `reduce` MUST be provided.
- If `filter_by` is omitted, filter field defaults to `from` field.

### 3.5 Inline selectors

Dataset rows MAY be filtered inline using bracket notation on the dataset name
inside algebra expressions. This produces an anonymous subset without requiring
a named declaration.

```
<data_name>[<field>=<value> ...]
```

Inline selectors use `key=value` pairs inside brackets. Variable indexing uses
positional comma-separated indices (`dispatch[a,t]`). The parser distinguishes
these by the presence of `=` signs inside the brackets.

```
sum(dispatch[g,t] for g in generator_data[class=solar area=north] for t in time)
```

Inline selectors are valid only inside algebra expression strings. For top-level
named subsets, use the `subset` declaration (section 4).

---

## 4. `subset` declaration (top-level)

`subset` creates a named filtered view from a dataset. Filter fields are
expressed as KDL properties on the node.

```kdl
subset <name> from=<data_name> <field>=<value> ...
```

Semantics:

- `from` references a declared `data` block.
- Remaining properties are field filters applied to the dataset.
- Filter fields MUST exist in the referenced dataset (after `map` resolution).
- The resulting subset is available as a named domain in algebra expressions.

```kdl
subset solar_north from=generator_data class=solar area=north
subset big_units from=units capacity_mw=250
subset cheap_solar from=generator_data class=solar vom=0.3
```

`subset` also supports comparator filters using the same properties as `set`:

```kdl
subset large_units from=units filter_by=capacity_mw geq=200
subset mid_range from=units filter_by=capacity_mw geq=100 leq=200
```

---

## 5. `model` declaration

`model` declares low-level optimization structure.

```kdl
model <name> { ... }
```

Allowed children:

- `set`
- `param`
- `control`
- `expression`
- `constraint`
- `minimize` or `maximize` (exactly one)

### 5.1 `set` (inside `model`)

Model-domain sets. These are abstract domains resolved at scenario time.

```kdl
set <name>
set <name> alias=<short>
set <name> from=horizon
```

Notes:

- `from=horizon` binds a set to scenario horizon steps. The built-in `time` set
  is created automatically from `from=horizon`. The conventional alias for the
  time set is `t`.
- `alias` provides a short iteration variable name. Example:
  `set asset_id alias=a`.
- Model sets are abstract. They acquire concrete members from scenario data
  bindings and `data` block sets at solve time. Hierarchy and filtering are
  defined in `data` blocks, not in models.

Built-in set conventions:

| Set    | Alias | Source         | Description                      |
| ------ | ----- | -------------- | -------------------------------- |
| `time` | `t`   | `from=horizon` | Time steps from scenario horizon |

The `time` set with alias `t` is the default temporal domain. When a model
declares `set time from=horizon`, the alias `t` is available automatically
unless overridden.

### 5.2 `param` (inside `model`)

Model parameters are declared with index intent.

Single-dimension:

```kdl
param <name> index_by=<set>
```

Multi-dimension:

```kdl
param <name> { index <set_a>; index <set_b> }
```

`index_by` and `index` children are mutually exclusive.

Model parameters are resolved at scenario time. The scenario binds concrete
values via `data` declarations (section 6). A model parameter name MUST match
either a scenario `data` binding name or a top-level `data` block `param` name
for the scenario to resolve it.

### 5.3 `control`

Decision-variable families.

Single-dimension:

```kdl
control <name> index_by=<set>
control <name> index_by=<set> kind=binary lower=0 upper=1
```

Multi-dimension:

```kdl
control <name> lower=0 { index <set_a>; index <set_b> }
```

`index_by` and `index` children are mutually exclusive.

Properties:

- `index_by` or `index` children: indexing sets
- `lower`: lower bound (optional)
- `upper`: upper bound (optional)
- `kind`: variable type (optional). Allowed values:
  - `continuous` (default)
  - `integer`
  - `binary`

### 5.4 `expression`

Named reusable algebra formulas.

```kdl
expression <name> {
  sum(fuel_cost[a,t] * dispatch[a,t] for a in assets for t in time)
}
```

Expressions MAY reference other named expressions by identifier. Circular
references MUST fail validation.

### 5.5 `constraint`

Two supported forms.

Simple algebra body:

```kdl
constraint <name> {
  dispatch[a,t] <= capacity_mw[a]
}
```

In the simple form, iteration variables are inferred from indexed references in
the body. The compiler resolves `a` and `t` to their corresponding declared sets
by matching variable family index signatures.

Generated row form:

```kdl
constraint <name> {
  over "a" in=asset_id
  over "t" in=time
  when "active[a]"
  expr {
    dispatch[a,t] <= capacity_mw[a]
  }
}
```

- `over` creates explicit row generation domains.
- `when` filters generated rows (optional). The value is an algebra predicate
  expression that MUST evaluate to a boolean or truthy numeric result.
- `expr` contains the algebra body.

The generated form is preferred when iteration domains need to be explicit or
when `when` filtering is required.

### 5.6 Objective

Exactly one objective is required per model.

```kdl
minimize total_cost {
  sum(variable_cost[a] * dispatch[a,t] for a in asset_id for t in time)
}
```

or

```kdl
maximize welfare {
  ...
}
```

Objective bodies MAY reference named `expression` declarations by identifier.

---

## 6. `scenario` declaration

`scenario` is the low-level execution entrypoint. It wires a model to concrete
data, defines the time horizon, and activates execution.

```kdl
scenario <name> {
  horizon steps=<int> resolution=<iso_duration>
  use <model_name>
  data <name> from=<path>
  report <expression_name>
  report dual <constraint_name>
}
```

### 6.1 `horizon`

Required. Defines the active time set. This produces the built-in `time` set
(alias `t`) with members `1..steps`. Any model set declared with
`from=horizon` resolves to this set.

```kdl
horizon steps=24 resolution="PT1H"
```

### 6.2 `use`

Required. References the model to solve.

```kdl
use dispatch_model
```

### 6.3 `data` (inside `scenario`)

Binds CSV data sources to model parameters. Each `data` declaration makes a
named parameter available to the model at solve time.

```kdl
data demand from="data/demand.csv"
data capacity from="data/capacity.csv"
data fuel_cost from="data/fuel_cost.csv"
```

The `<name>` of each binding MUST match a `param` declared in the referenced
model. The CSV structure determines how the parameter is indexed (by `t`, by
`asset_name`, or by both).

### 6.4 `report` (inside `scenario`)

`report` requests post-solve output values. Two forms are supported.

Scalar report evaluates a named expression at the primal solution:

```kdl
report FuelCost
report StartupCost
report PenaltyCost
```

Dual report extracts constraint shadow prices (dual values):

```kdl
report dual balance
report dual capacity_limit
```

Semantics:

- Scalar report targets MUST resolve to a declared `expression` or objective
  name.
- Dual report targets MUST resolve to a declared `constraint` name.
- Reports are evaluated after the solver returns a feasible solution. If the
  model is infeasible, report evaluation is skipped.

### 6.5 Data scoping

`data` can appear at two levels:

- Top-level `data` with children (`map`, `set`, `param`) declares a shared
  namespace available to all scenarios.
- Scenario-level `data` without children is a simple CSV-to-model-parameter
  binding scoped to that scenario only.

The parser distinguishes these by context: top-level `data` has a `{ ... }`
block, scenario-level `data` does not.

If a scenario-level `data` binding has the same name as a top-level `data`
block, the scenario-level binding takes precedence for parameter resolution
within that scenario.

```kdl
// shared across all scenarios
data units from="data/units.csv" {
  set plant_id
  set unit_id subset_of=plant_id
  param capacity_mw index_by=unit_id
}

scenario base_case {
  horizon steps=24 resolution="PT1H"
  use dispatch_model
  // only available in this scenario
  data demand from="data/demand_base.csv"
}

scenario high_demand {
  horizon steps=24 resolution="PT1H"
  use dispatch_model
  // different demand for this scenario
  data demand from="data/demand_high.csv"
}
```

---

## 7. KDL 2.0 type annotations (optional)

Arco supports KDL 2.0 type annotations for users who want stronger metadata and
literal intent.

Node annotation:

```kdl
(f64)param capacity_mw { index plant_id; index unit_id }
```

Typed value literals in filters:

```kdl
param large_units from=capacity_mw geq=(f64)200
param cc_capacity from=capacity_mw filter_by=prime_mover eq=(prime_mover)CC
```

Typed metadata values:

```kdl
param fuel_cost units=(unit)"$/MMBtu"
```

Type annotations are optional unless project policy requires them.

---

## 8. Comparator semantics

Comparators are available on low-level `set` and `param` filters.

- `eq`  -> `==`
- `ge`  -> `>`
- `geq` -> `>=`
- `le`  -> `<`
- `leq` -> `<=`

Rules:

- `ge/geq/le/leq` require numeric values.
- `eq` supports both numeric and categorical equality.

---

## 9. Validation requirements

Implementations MUST validate at least:

Name uniqueness:

1. Duplicate `data` block names.
2. Duplicate `model` names.
3. Duplicate `scenario` names.
4. Duplicate `map` targets within one `data` block.
5. Duplicate `set` names within one `data` block.
6. Set name collisions across `data` blocks (same name, different data source).

Column and field resolution:

7. `map` without `from` MUST resolve to an existing CSV column.
8. Unknown source columns in `map from=...` or `param from=...`.
9. Unknown symbols in `index` / `index_by` / `index` children.

Set hierarchy:

10. `subset_of` parent MUST exist.
11. `subset_of` cycles MUST be detected.
12. Child-to-parent hierarchy contradictions (one child maps to multiple parents).

Indexing:

13. `index_by` and `index` children on the same declaration are mutually
    exclusive.
14. At most one `index` declaration per `data` block.
15. Non-unique indexing without `reduce`.

Filtering:

16. Unknown `filter_by` fields.
17. Numeric comparator on non-numeric data.
18. Contradictory filter bounds (example `geq=30`, `leq=20`).

Type and metadata:

19. Invalid `units` metadata token.
20. Type annotation conflicts (example `(f64)param ...` on text column).

Model structure:

21. `model` MUST contain exactly one objective.
22. Circular `expression` references.
23. `control kind=<value>` MUST be one of `continuous`, `integer`, `binary`.
24. Constraint generation references (`over in=...`) MUST resolve to known sets.

Scenario resolution:

25. `scenario` MUST contain `horizon` and `use`.
26. `scenario use <model_name>` MUST resolve to an existing `model`.
27. Scenario `data` binding names MUST match model `param` declarations.
28. Scalar `report` targets MUST resolve to a declared `expression` or objective.
29. Dual `report` targets MUST resolve to a declared `constraint`.

Subset resolution:

28. `subset from=<data_name>` MUST resolve to an existing `data` block.
29. `subset` filter fields MUST exist in the referenced dataset.

Data integrity:

30. Empty CSV files (no data rows) MUST produce a diagnostic.

---

## 10. Grammar (low-level profile)

The grammar below is a compact EBNF-style reference for this low-level profile.
It describes Arco declarations layered on top of valid KDL 2.0 syntax.

```ebnf
document          := { data_decl | subset_decl | model_decl | scenario_decl }

data_decl         := "data" name from_prop data_block
data_block        := "{" { map_decl | set_decl | index_decl | data_param_decl } "}"

map_decl          := "map" name [ from_prop ]

set_decl          := "set" name
                     [ "alias" "=" name ]
                     [ "subset_of" "=" name ]
                     [ filter_clause ]

index_decl        := "index" name { name }

data_param_decl   := [ type_annot ] "param" name
                     [ from_prop ]
                     ( [ "index_by" "=" name ] | [ index_children ] )
                     [ "reduce" "=" reducer ]
                     [ filter_clause ]
                     [ "units" "=" value ]

index_children    := "{" index_child ";" { index_child ";" } "}"
index_child       := "index" name | "reduce" reducer

subset_decl       := "subset" name "from" "=" name { name "=" value }

model_decl        := "model" name model_block
model_block       := "{" { model_set_decl
                         | model_param_decl
                         | control_decl
                         | expression_decl
                         | constraint_decl
                         | objective_decl } "}"

model_set_decl    := "set" name [ "alias" "=" name ] [ "from" "=" "horizon" ]

model_param_decl  := [ type_annot ] "param" name
                     ( [ "index_by" "=" name ] | [ index_children ] )

control_decl      := [ type_annot ] "control" name
                     ( [ "index_by" "=" name ] | [ index_children ] )
                     [ "lower" "=" value ]
                     [ "upper" "=" value ]
                     [ "kind" "=" kind ]

expression_decl   := "expression" name "{" algebra_expr "}"

constraint_decl   := "constraint" name ( simple_body | generated_body )
simple_body       := "{" algebra_expr "}"
generated_body    := "{" { over_decl } [ when_decl ] expr_decl "}"
over_decl         := "over" string "in" "=" name
when_decl         := "when" string
expr_decl         := "expr" "{" algebra_expr "}"

objective_decl    := ( "minimize" | "maximize" ) name "{" algebra_expr "}"

scenario_decl     := "scenario" name scenario_block
scenario_block    := "{" horizon_decl use_decl { scenario_data_decl } { report_decl } "}"

horizon_decl      := "horizon" "steps" "=" integer "resolution" "=" string
use_decl          := "use" name
scenario_data_decl:= "data" name from_prop
report_decl       := "report" ( name | "dual" name )

inline_selector   := "[" { name "=" value } "]"

from_prop         := "from" "=" ( path | field_name )
filter_clause     := [ "filter_by" "=" field_name ] { comparator }
comparator        := ( "eq" | "ge" | "geq" | "le" | "leq" ) "=" value
reducer           := "sum" | "avg" | "min" | "max" | "first" | "last"
kind              := "continuous" | "integer" | "binary"

name              := kdl_string
field_name        := kdl_string
path              := kdl_string
value             := kdl_value
string            := kdl_string
integer           := kdl_integer
type_annot        := "(" kdl_string ")"

algebra_expr      := <see algebra summary below>
```

Notes:

- `name`, `field_name`, and `path` follow KDL string rules (identifier or
  quoted).
- `kdl_value` MAY be annotated (example `(f64)200`, `(unit)"$/MWh"`).
- Single-dimension indexing uses the `index_by=<set>` property form.
  Multi-dimension indexing uses child nodes:
  `{ index <set_a>; index <set_b> }`.
  Using both on the same declaration is a validation error.
- `model_block` MUST contain exactly one `objective_decl`.
- `scenario_block` MUST contain one `horizon_decl` and one `use_decl`.
- `inline_selector` is Arco-specific syntax valid only inside algebra expression
  strings. It is distinguished from variable indexing by the presence of `=`
  inside brackets. For top-level named subsets, use the `subset` declaration.

---

## 11. Algebra expression summary

Algebra expressions appear inside `constraint`, `expression`, `minimize`, and
`maximize` bodies. They are parsed as opaque strings by the KDL layer and
interpreted by the Arco algebra parser.

Supported operators:

| Operator | Description              |
| -------- | ------------------------ |
| `+`      | addition                 |
| `-`      | subtraction / negation   |
| `*`      | multiplication           |
| `/`      | division                 |
| `<=`     | less than or equal       |
| `>=`     | greater than or equal    |
| `=`      | equality (in constraints)|

Indexing:

| Form              | Description                        |
| ----------------- | ---------------------------------- |
| `x[a]`            | single-dimension index             |
| `x[a,t]`          | multi-dimension index              |
| `x[a,t-1]`        | temporal offset (ordered sets)     |

Reductions:

| Form                                        | Description             |
| ------------------------------------------- | ----------------------- |
| `sum(expr for v in set)`                    | summation over one set  |
| `sum(expr for v in set for w in set2)`      | nested summation        |
| `sum(expr for v in set if condition)`       | filtered summation      |

Reductions iterate over sets declared in `data` blocks or `model` blocks. This
means data-level sets (including hierarchy-derived subsets) can be used directly
inside algebra for aggregation:

```
// sum over a data-declared set
sum(capacity_mw[g] for g in solar_assets)

// sum over a subset declared via subset_of hierarchy
sum(dispatch[g,t] for g in generator_data[class=solar] for t in time)

// sum over a named top-level subset
sum(capacity_mw[g] for g in solar_north)

// nested aggregation mixing data sets and model sets
sum(cost[a] * dispatch[a,t] for a in asset_id for t in time)
```

Set-level `param` aggregations (`reduce=sum`, `reduce=avg`, etc.) are resolved
at data-loading time and produce scalar parameters indexed by the target set.
Algebra-level `sum(...)` reductions are resolved at constraint generation time
and produce linear expressions. Both are available and serve different purposes:

- `param` with `reduce`: precomputed aggregate, available as a parameter.
- `sum(...)` in algebra: dynamic linear expression, generates solver terms.

Inline selectors (inside algebra only):

| Form                                        | Description                    |
| ------------------------------------------- | ------------------------------ |
| `data_name[field=value ...]`                | anonymous filtered subset      |

Expression references:

Named `expression` declarations MAY be referenced by identifier inside other
expressions, constraints, and objectives.

---

This document is the canonical reference for the low-level Arco KDL syntax.
