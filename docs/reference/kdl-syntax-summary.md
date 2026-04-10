# Arco Low-Level KDL Syntax Specification (KDL 2.0 Profile)

Version: 0.1.0 (Last updated: 2026-04-09)

Versioning: This specification follows [Semantic Versioning](https://semver.org/).
Minor versions (0.x.0) MAY introduce breaking changes while the major version is
0. Once the specification reaches 1.0.0, breaking changes require a major version
bump and deprecated features will be documented with a removal target version.

This document defines the low-level Arco DSL profile authored in KDL 2.0.

Scope of this specification:

- [`set`](#4-set-declaration-top-level) declarations (explicit domains)
- [`data`](#5-data-declaration) declarations (CSV-backed namespaces)
- [`param`](#54-param-inside-data) declarations (projection, indexing,
  filtering, aggregation)
- [`control`](#63-control) declarations (decision-variable families)
- [`expression`](#64-expression) declarations (named reusable formulas)
- [`constraint`](#65-constraint) declarations (low-level algebra rows)
- [`scenario`](#7-scenario-declaration) declarations (execution entrypoints)

High-level domain declarations (`technology`, `operation`, `rule`, `asset`) are
not part of this low-level specification.

---

## Table of Contents

1. [Conformance](#1-conformance)
2. [Terminology](#2-terminology)
3. [Top-level declarations](#3-top-level-declarations)
4. [`set` declaration (top-level)](#4-set-declaration-top-level)
5. [`data` declaration](#5-data-declaration)
   - [5.1 `map`](#51-map) | [5.2 `set`](#52-set-inside-data) | [5.3 `index`](#53-index-inside-data) | [5.4 `param`](#54-param-inside-data) | [5.5 Inline selectors](#55-inline-selectors)
6. [`model` declaration](#6-model-declaration)
   - [6.1 `set`](#61-set-inside-model) | [6.2 `param`](#62-param-inside-model) | [6.3 `control`](#63-control) | [6.4 `expression`](#64-expression) | [6.5 `constraint`](#65-constraint) | [6.6 Objective](#66-objective)
7. [`scenario` declaration](#7-scenario-declaration)
   - [7.1 `horizon`](#71-horizon) | [7.2 `use`](#72-use) | [7.3 `data`](#73-data-inside-scenario) | [7.4 `report`](#74-report-inside-scenario) | [7.5 Data scoping](#75-data-scoping)
8. [KDL 2.0 type annotations](#8-kdl-20-type-annotations-optional)
9. [Filter predicate semantics](#9-filter-predicate-semantics)
10. [Validation requirements](#10-validation-requirements)
11. [Grammar (low-level profile)](#11-grammar-low-level-profile)
12. [Algebra expression summary](#12-algebra-expression-summary)
- [Appendix A. Ergonomic syntax profile](#appendix-a-ergonomic-syntax-profile)

---

## 1. Conformance

Arco KDL files MUST conform to [KDL 2.0](https://kdl.dev/spec/):

- UTF-8 encoding
- KDL node/value type annotations are allowed
- File extension: `.kdl`

Arco adds semantic validation on top of KDL parsing. Errors are classified into
two categories:

- Parse errors: malformed KDL that violates KDL 2.0 syntax rules.
- Validation errors: well-formed KDL that violates Arco semantic rules defined
  in this specification (see [§10](#10-validation-requirements)).

KDL comments (`//` line comments and `/-` slashdash comments) are fully
supported. Slashdash (`/-`) comments out an entire node, property, or argument,
which is useful for toggling declarations during development.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

Naming convention:

Most declarations take their name as the first positional argument:

```kdl
param capacity_mw index_by=gen
```

Optionally, the name MAY be given as an explicit `name=` property instead. Both
forms are equivalent:

```kdl
// positional (preferred)
param capacity_mw from=cap_mw index_by=gen
// explicit name property (also valid)
param name=capacity_mw from=cap_mw index_by=gen
```

This applies to all named declarations: `set`, `data`, `model`, `scenario`,
`control`, `expression`, `constraint`, `minimize`, and `maximize`. (`use_data`
is not a named declaration — it takes data block references as arguments; see
[Appendix A.2](#a2-use_data-model-imports).) The positional form is RECOMMENDED
for brevity.

Alias uniqueness:

- Aliases (declared via `alias=<short>` on `set` declarations) MUST be unique
  across all set declarations (top-level, data-level, and model-level).
- An alias MUST NOT collide with any declared set name. For example, if a set is
  named `time`, no other set may use `alias=time`.
- If a conflict is detected, validation MUST fail.

---

## 2. Terminology

This specification uses the following terms consistently:

| Term                         | Meaning                                                                                                                         |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `algebra block`              | A `{ ... }` child block containing bare math. After KDL structural parsing, the normalizer converts it to a canonical `formula "..."` quoted string property before algebra interpretation. |
| `expression`                 | A named, reusable formula declared with `expression <name> { ... }` inside a [`model`](#6-model-declaration).                   |
| `constraint`                 | A named algebraic relation (equality or inequality) declared with `constraint <name> { ... }`.                                  |
| `objective`                  | The single optimization target, declared with `minimize <name> { ... }` or `maximize <name> { ... }`.                           |
| `expression` (in constraint) | The algebra body node inside a generated constraint. Written as `expression { ... }`.                                           |
| `if`                         | A row-filter predicate inside a generated constraint. Written as `if { ... }`. Multiple `if` blocks combine with AND semantics. |
| `control`                    | A decision-variable family.                                                                                                     |
| `slack`                      | A child on a constraint that auto-generates a slack variable and penalty term in the objective.                                 |
| `param`                      | A data-backed or model-declared parameter (known constant at solve time).                                                       |
| `set`                        | A named domain of indices.                                                                                                      |
| `over`                       | A row-generation clause in a generated constraint. Written as `over <var> in=<set>`.                                            |
| `bounds`                     | Ergonomic declarative bound override for a control family (see [Appendix A.4](#a4-declarative-bounds-and-fix)).                  |
| `fix`                        | Ergonomic declarative variable fix (equal lower and upper bounds; see [Appendix A.4](#a4-declarative-bounds-and-fix)).           |
| `use_data`                   | Ergonomic model import of sets/params from `data` blocks (see [Appendix A.2](#a2-use_data-model-imports)).                      |
| `map`                        | Binds a logical name to a CSV header inside a [`data`](#5-data-declaration) block.                                              |
| `index` (data-block)         | Default indexing declaration for all `param` nodes in a [`data`](#53-index-inside-data) block.                                  |
| `index` (param/control)      | Per-declaration index child specifying which set(s) a `param` or `control` is indexed over.                                     |
| `horizon`                    | Scenario child that defines the active time set (steps and resolution). See [§7.1](#71-horizon).                                |
| `report`                     | Scenario child requesting post-solve output (expression values or constraint duals). See [§7.4](#74-report-inside-scenario).    |
| `reduce` / `reducer`         | Aggregation function applied when indexing is non-unique (`sum`, `avg`, `min`, `max`, `first`, `last`). Two equivalent forms: `reduce=sum` (property) and `reduce sum` (child node). |

`expression` as declaration vs inside constraint: `expression` serves two roles
depending on context. As a model child, it declares a named reusable formula.
Inside a generated [`constraint`](#65-constraint), `expression { ... }` contains
the constraint's algebra body. Both use the same keyword — context determines
the meaning.

`index` as data-block default vs param/control child: Inside a `data` block,
`index` (§5.3) declares the default indexing columns for all `param` declarations
in that block. Inside a `param` or `control` declaration, `index` children
(§5.4, §6.3) specify per-declaration indexing that overrides the block default.
Both use the same keyword — the parent node determines the meaning.

`if` in algebra vs `if` in constraints: Inside algebra expressions, `if` is a
filter clause on reductions (`sum(x for a in set if cond)`). Inside generated
constraints, `if { ... }` is a row-filter block that controls which rows are
generated. Both use the same predicate syntax but serve different purposes.

```kdl
model dispatch {
  // "expression" — named reusable formula (model-level declaration)
  expression TotalFuelCost {
    sum(fuel_cost[g] * dispatch[g,t] for g in generators for t in time)
  }

  // "expression" inside constraint — the constraint's algebra body
  constraint capacity_limit {
    over g in=generators
    over t in=time
    if { active[g] }
    expression {
      dispatch[g,t] <= capacity[g]
    }
  }
}
```

---

## 3. Top-level declarations

A low-level document MAY contain these top-level declarations:

- [`set`](#4-set-declaration-top-level) (explicit domain)
- [`data`](#5-data-declaration) (CSV-backed namespace)
- [`model`](#6-model-declaration)
- [`scenario`](#7-scenario-declaration)

```kdl
// explicit sets with inline members
set bus { 1; 2; 3; 4; 5 }

// CSV-backed data with subsets via set { in ... }
data generators from="data/generators.csv" {
  set gen
  set solar { in gen; filter { type == solar } }
  param pmax index_by=gen
}

model dispatch_model { ... }

scenario day_ahead {
  horizon steps=24 resolution="PT1H"
  use dispatch_model
  data demand from="data/demand.csv"
}
```

`scenario` is the execution entrypoint.

Declaration order: top-level declarations MAY appear in any order. Forward
references are allowed (a `scenario` may reference a `model` declared after it).
All names are resolved after the full document is parsed.

---

## 4. `set` declaration (top-level)

A top-level `set` declares a named domain with explicit members listed inline.
This is useful for sets that are not backed by a CSV file — for example, index
ranges, scenario labels, or piecewise segments.

Explicit member list:

```kdl
set <name> { <member1>; <member2>; ... }
```

Members are KDL arguments (strings or numbers) separated by semicolons.

```kdl
set bus { 1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; 12; 13; 14; 15; 16; 17; 18; 19; 20; 21; 22; 23; 24 }
set gen { g1; g2; g3; g4; g5; g6; g7; g8; g9; g10; g11; g12 }
set k { 1; 2; 3 }
```

Alias:

```kdl
set time alias=t { 1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; 12; 13; 14; 15; 16; 17; 18; 19; 20; 21; 22; 23; 24 }
```

Top-level sets are globally visible, just like sets declared inside `data`
blocks. They can be used in any model, constraint, or algebra expression.

Top-level sets and data-level sets share a single namespace. A top-level `set`
MUST NOT have the same name as a set declared inside a `data` block.

---

## 5. `data` declaration

`data` declares one CSV-backed namespace. Sets and parameters declared inside
the block are globally visible — any `model` or algebra expression in the
document can reference them by name. A single `data` block can supply sets and
parameters to multiple models.

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

### 5.1 `map`

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

### 5.2 `set` (inside `data`)

`set` extracts unique values from a dataset column and exposes them as a named
domain. The column used is the one matching `<name>` (after `map` resolution).
Sets declared inside a `data` block are globally visible and can be referenced
by any model, constraint, or algebra expression in the document.

```kdl
set <name>
set <name> alias=<short>
set <name> {
  in <parent_set>
}
set <name> {
  in <parent_set>
  filter { <predicate> }
}
```

Semantics:

- `set class` extracts unique values from the `class` column.
- `alias` provides a short iteration variable name for use in algebra
  expressions. Example: `set asset_id alias=a` allows `dispatch[a,t]` instead of
  `dispatch[asset_id,time]`.
- `in <parent>` declares that this set is contained within `<parent>`. Each
  child value maps to exactly one parent value, forming a hierarchy edge.
- `filter { ... }` narrows set members using an algebra predicate block. The
  expression is evaluated per row against the dataset columns. This uses the
  same bare-math block syntax as `expression`, `if`, and `lower`/`upper`.
  Supported operators: `==`, `>`, `>=`, `<`, `<=`, `!=`.

```kdl
set thermal_gen {
  in gen
  filter { type == thermal }
}
set large_gen {
  in gen
  filter { capacity_mw >= 200 }
}
set mid_gen {
  in gen
  filter { capacity_mw >= 100 and capacity_mw <= 500 }
}
```

Set resolution:

A set is always resolved from exactly one CSV file — the `data` block that
declares it. The members of the set are the unique values found in the
corresponding column of that CSV. There is no implicit union or merge across
files.

If data is spread across multiple CSV files and you need a combined domain, you
have two options:

1. Consolidate the data into a single CSV so one `data` block produces the full
   set.
2. Declare separate sets per file and iterate over each one independently.

```kdl
// generators.csv contains: gen-01, gen-02, gen-03
data thermal from="data/thermal.csv" {
  set thermal_gen
  param heat_rate index_by=thermal_gen
}

// renewables.csv contains: gen-04
data renewable from="data/renewables.csv" {
  set renewable_gen
  param capacity_factor index_by=renewable_gen
}
```

In this example, `thermal_gen` resolves to `{gen-01, gen-02, gen-03}` and
`renewable_gen` resolves to `{gen-04}`. They are separate sets. A constraint
that needs to iterate over all generators must reference both:

```kdl
constraint total_output {
  sum(dispatch_thermal[g,t] for g in thermal_gen for t in time)
  + sum(dispatch_renewable[g,t] for g in renewable_gen for t in time)
  = demand[t]
}
```

To iterate over all generators with a single set, consolidate into one CSV with
a shared column (e.g., `generator_id`) and use `in` with `filter` to distinguish
subgroups:

```kdl
// all_generators.csv: generator_id, type, ...
// gen-01, thermal, ...
// gen-02, thermal, ...
// gen-03, thermal, ...
// gen-04, renewable, ...
data generators from="data/all_generators.csv" {
  map gen from=generator_id
  set gen
  set thermal_gen { in gen; filter { type == thermal } }
  set renewable_gen { in gen; filter { type == renewable } }
  param heat_rate index_by=gen
  param capacity_factor index_by=gen
}
```

Now `gen` resolves to `{gen-01, gen-02, gen-03, gen-04}`, while `thermal_gen`
and `renewable_gen` are subsets that can be iterated independently or together.

### 5.3 `index` (inside `data`)

`index` defines default indexing for `param` declarations in that `data` block.
Multiple columns can be listed as arguments to a single `index` declaration to
form a multi-column index.

```kdl
index <set_a>
index <set_a> <set_b>
index <set_a> <set_b> <set_c>
```

Semantics:

- `index` is optional.
- If omitted, default index is numeric row order.
- Every index symbol MUST be a declared set.
- At most one `index` declaration is allowed per `data` block. To index by
  multiple columns, list them all as arguments to a single `index` node.
- The declared `index` sets the default indexing for all `param` declarations in
  the same `data` block that do not specify their own `index_by` or `index`
  children (see [§5.4](#54-param-inside-data)).

```kdl
// valid: multi-column index on a single declaration
data plants from="data/plants.csv" {
  set plant_id
  set unit_id { in plant_id }
  index plant_id unit_id
  param capacity_mw
}

// INVALID: two separate index declarations
data plants from="data/plants.csv" {
  set plant_id
  set unit_id
  index plant_id
  index unit_id  // validation error: duplicate index declaration
}
```

### 5.4 `param` (inside `data`)

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

```kdl
// INVALID: index_by and index children on the same param
param cost index_by=asset { index time }  // validation error
```

Aggregation:

```kdl
// property form (single-dimension)
param <name> index_by=<set> reduce=<reducer>

// child node form (multi-dimension)
param <name> { index <set_a>; index <set_b>; reduce sum }
```

Both forms are equivalent. The child node form (`reduce sum`, no `=`) is used
inside index blocks; the property form (`reduce=sum`) is used on the param node
directly.

Example of each form:

```kdl
// property form
param total_cap index_by=region reduce=sum

// child node form
param avg_cost { index region; index fuel_type; reduce avg }
```

Supported reducers:

- `sum`, `avg`, `min`, `max`, `first`, `last`

Filtering:

```kdl
param <name> from=<field> { filter { <predicate> } }
```

The `filter` block uses the same bare-math algebra syntax as `set` filters and
constraint `if` blocks:

```kdl
param cc_capacity from=capacity_mw { filter { prime_mover == CC } }
param large_units from=capacity_mw { filter { capacity_mw >= 200 } }
```

Order of operations when `filter` and `reduce` are combined: filtering is
applied first, then the reducer operates on the filtered rows. For example:

```kdl
// first filter to thermal rows, then sum their capacity
param total_thermal_cap from=capacity_mw index_by=region reduce=sum {
  filter { type == thermal }
}
```

This produces, per region, the sum of `capacity_mw` across rows where
`type == thermal`. The filter narrows the row set before aggregation.

Units metadata:

```kdl
param capacity_mw units=MW
param fuel_cost units="$/MMBtu"
```

The `units` property accepts freeform string values. There is no predefined
vocabulary of valid unit tokens — any KDL string or identifier is accepted.
Units serve as documentation metadata and are preserved in solver output and
diagnostics. Implementations MAY use units for dimensional consistency checks
but are not required to validate unit semantics.

Scalar parameters:

A `param` with no `index_by`, no `index` children, and no block-level `index`
declaration is a scalar parameter. The CSV MUST contain exactly one data row
with the value column. For how scalar parameters are bound at scenario time,
see [§7.3](#73-data-inside-scenario).

```kdl
data settings from="data/settings.csv" {
  param discount_rate
  param voll units="$/MWh"
}
```

Semantics:

- If `from` is omitted, source field defaults to `<name>`.
- If neither `index_by` nor `index` children are present, default is the block's
  `index` declaration if present (see [§5.3](#53-index-inside-data)), else
  numeric row index.
- If indexing is non-unique, `reduce` MUST be provided.

### 5.5 Inline selectors

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
named filtered domains, use `set { in ... }` inside the relevant `data` block.

---

## 6. `model` declaration

`model` declares low-level optimization structure.

```kdl
model <name> { ... }
```

Allowed children:

- [`use_data`](#a2-use_data-model-imports) — **ergonomic profile only** (import
  sets/params from `data` blocks; defined in
  [Appendix A.2](#a2-use_data-model-imports), not part of the canonical low-level
  grammar)
- [`set`](#61-set-inside-model)
- [`param`](#62-param-inside-model)
- [`control`](#63-control)
- [`expression`](#64-expression)
- [`constraint`](#65-constraint)
- [`minimize` or `maximize`](#66-objective) (exactly one)

### 6.1 `set` (inside `model`)

Model-domain sets. These are abstract domains resolved at scenario time.

```kdl
set <name>
set <name> alias=<short>
set <name> from=horizon
```

Notes:

- `from=horizon` binds a set to scenario horizon steps. The built-in `time` set
  is created automatically from `from=horizon`. When a model declares
  `set time from=horizon`, the alias `t` is auto-assigned unless the declaration
  explicitly provides a different alias (e.g., `set time alias=step from=horizon`
  assigns `step` instead of `t`). The auto-assigned `t` alias follows the same
  uniqueness rules as explicit aliases (see §1) — if another set already uses
  `alias=t`, validation MUST fail. The only supported `from=` source for model
  sets is `horizon`. Other `from=` values MUST fail validation.
- `alias` provides a short iteration variable name. Example:
  `set asset_id alias=a`.
- Model sets are abstract. They acquire concrete members from scenario data
  bindings and `data` block sets at solve time. Hierarchy and filtering are
  defined in `data` blocks, not in models.
- Models do not need to re-declare sets that are already defined in a top-level
  `data` block. Data-level sets are globally visible and can be used directly in
  model algebra (constraints, expressions, objectives) without redeclaration.
  The same applies to data-level parameters.
- Name conflict rule: If a `model` declares a `set` with the same name as a
  `set` already declared in a `data` block, the model-level declaration MUST
  fail validation. Model sets and data sets share a single global namespace —
  a model cannot shadow or override a data-level set. To use a data-level set
  in a model, reference it directly without redeclaration.

```kdl
// data declares gen, capacity_mw, fuel_cost, and the thermal_gen subset
data generators from="data/generators.csv" {
  map gen from=generator_id
  set gen alias=g
  set thermal_gen { in gen; filter { type == thermal } }
  param capacity_mw index_by=gen
  param fuel_cost index_by=gen
}

// model uses gen, thermal_gen, and capacity_mw directly — no redeclaration
model dispatch {
  set time from=horizon
  control output { index gen; index time; lower=0 }

  constraint cap_limit {
    over g in=gen
    over t in=time
    expression {
      output[g,t] <= capacity_mw[g]
    }
  }

  // use a data-level subset in algebra
  expression ThermalOutput {
    sum(output[g,t] for g in thermal_gen for t in time)
  }

  minimize cost {
    sum(fuel_cost[g] * output[g,t] for g in gen for t in time)
  }
}
```

The only set a model needs to declare itself is `time` (via `from=horizon`) or
any abstract set that does not come from a `data` block.

Built-in set conventions:

| Set    | Alias | Source         | Description                      |
| ------ | ----- | -------------- | -------------------------------- |
| `time` | `t`   | `from=horizon` | Time steps from scenario horizon |

The `time` set with alias `t` is the default temporal domain. When a model
declares `set time from=horizon`, the alias `t` is auto-assigned. To use a
different alias, declare it explicitly: `set time alias=step from=horizon`.

### 6.2 `param` (inside `model`)

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
values via `data` declarations ([§7](#7-scenario-declaration)). A model
parameter name MUST match either a scenario `data` binding name or a top-level
`data` block `param` name for the scenario to resolve it.

### 6.3 `control`

Decision-variable families. A `control` declaration defines a family of decision
variables indexed over one or more sets.

The preferred form uses a child block with `index`, bounds, and `kind`:

```kdl
control <name> {
  index <set_a>
  index <set_b>
  lower=0
  upper=100
  kind=continuous
}
```

All children are optional except at least one `index`.

Compact single-dimension form:

```kdl
control <name> index_by=<set>
control <name> index_by=<set> kind=binary lower=0 upper=1
```

`index_by` and `index` children are mutually exclusive. Using both on the same
`control` MUST fail validation.

Index domain binding:

The `in=` property on `index` children binds the index variable to a named
domain set. This is useful when the iteration domain differs from the index
name:

```kdl
control <name> {
  index <set_a> in=<domain_a>
  index <set_b> in=<domain_b>
}
```

Properties:

- `index` children or `index_by`: indexing sets (at least one required)
- `lower`: lower bound (optional). Accepts a literal value or an algebra block.
- `upper`: upper bound (optional). Accepts a literal value or an algebra block.
- `kind`: variable type (optional). Allowed values:
  - `continuous` (default)
  - `integer`
  - `binary`

Bounds:

Bounds can be literal values or parameter-dependent algebra expressions:

```kdl
// literal bounds
control dispatch {
  index gen
  index time
  lower=0
}

// formula bounds using algebra blocks
control flow {
  index lines
  lower { -capacity[l] }
  upper { capacity[l] }
}

// mixed: literal lower, formula upper
control output {
  index gen
  index time
  lower=0
  upper { capacity[g] }
}
```

The algebra inside `lower { ... }` and `upper { ... }` uses the same bare-math
block syntax as `expression` and `constraint` bodies — no quoting needed.

Specifying both a literal value and a formula block for the same direction MUST
fail validation:

```kdl
// INVALID: two lower bounds on the same control
control flow {
  index lines
  lower=0
  lower { -capacity[l] }  // validation error
}
```

Bound algebra variable scoping:

The algebra inside `lower { ... }` and `upper { ... }` blocks MAY reference the
control's own index variables. The variable names used in the algebra MUST match
the index names declared on the same `control`. References to undeclared
variables MUST fail validation.

```kdl
control flow {
  index l in=lines
  lower { -capacity[l] }   // valid: `l` matches the index name
  upper { capacity[l] }    // valid
}

// INVALID: `x` is not a declared index on this control
control flow {
  index l in=lines
  upper { capacity[x] }    // validation error: unknown variable `x`
}
```

### 6.4 `expression`

Named reusable algebra formulas.

```kdl
expression <name> {
  sum(fuel_cost[a,t] * dispatch[a,t] for a in assets for t in time)
}
```

The algebra body is written directly inside `{ ... }` as bare math — no quoting
is needed. The normalizer automatically converts this to the canonical KDL form
(`formula "..."`) before parsing. This bare-math block syntax is available on
all algebra-bearing nodes: `expression`, `constraint`, `minimize`, `maximize`,
`lower`, and `upper`.

Expressions MAY reference other named expressions by identifier. Circular
references MUST fail validation.

### 6.5 `constraint`

Two supported forms.

Simple algebra body:

```kdl
constraint <name> {
  dispatch[a,t] <= capacity_mw[a]
}
```

In the simple form, iteration variables are inferred from indexed references in
the body. The compiler resolves each variable to its corresponding declared set
by matching against `control` index signatures. For example, if `dispatch` is
declared as `control dispatch { index asset_id; index time }`, then `a` resolves
to `asset_id` (first index position) and `t` resolves to `time` (second index
position). The simple form implicitly generates one constraint row per
combination of resolved index sets — it is equivalent to a generated form with
`over` clauses for each inferred variable. If a variable appears in multiple
controls with conflicting index signatures, validation MUST fail with an
ambiguity error.

Generated row form:

```kdl
constraint <name> {
  over a in=asset_id
  over t in=time
  if { active[a] }
  expression {
    dispatch[a,t] <= capacity_mw[a]
  }
}
```

- `over` creates explicit row generation domains.
- `if { ... }` filters generated rows (optional). The body is an algebra
  predicate that MUST evaluate to a boolean or truthy numeric result.
- `expression` contains the constraint algebra body.

The generated form is preferred when iteration domains need to be explicit or
when row filtering is required.

Row filters with `if`:

The `if` block filters which rows are generated. The predicate MUST reference at
least one of the iteration variables declared by the `over` clauses — a
condition that does not depend on any loop variable is a static condition and
SHOULD be handled outside the constraint:

```kdl
// valid: condition references loop variable `t`
if { t > 1 }

// valid: condition references loop variable `g`
if { active[g] }

// INVALID: condition does not reference any over variable
if { 1 > 0 }  // validation error
```

The `if` block supports arbitrary algebra predicates, including numeric
comparisons and temporal conditions:

```kdl
constraint ramp_up {
  over g in=generators
  over t in=time
  if { t > 1 }
  expression {
    dispatch[g,t] - dispatch[g,t-1] <= ramp_up_rate[g]
  }
}
```

Common `if` patterns:

- `if { t > 1 }` — skip the first time step (required when using `t-1`)
- `if { t < 24 }` — skip the last time step (required when using `t+1`; `24` here assumes a 24-step horizon — use the actual `steps` value)
- `if { t == 1 }` — apply only at the first time step
- `if { active[a] }` — filter by a boolean parameter

Nested `if` conditions:

Multiple `if` blocks MAY appear in the same constraint. They are combined with
AND semantics — all conditions must be true for the row to be generated:

```kdl
constraint conditional_ramp {
  over g in=generators
  over t in=time
  if { t > 1 }
  if { active[g] }
  expression {
    dispatch[g,t] - dispatch[g,t-1] <= ramp_up_rate[g]
  }
}
```

Temporal offsets and boundary guards:

Algebra expressions support temporal offset indexing (`t-1`, `t+1`) on ordered
sets. When a constraint references a previous or next time step, an `if` guard
MUST be present to exclude boundary steps where the offset would be
out-of-range. Failing to guard temporal offsets is a validation error.

```kdl
// INVALID: t-1 without a guard on the first time step
constraint unguarded_ramp {
  over g in=generators
  over t in=time
  expression {
    dispatch[g,t] - dispatch[g,t-1] <= ramp_rate[g]  // validation error
  }
}
```

Range constraints (chained inequalities):

Constraint bodies MAY use chained inequalities to express range bounds:

```kdl
constraint angle_bounds {
  over b in=buses
  over t in=time
  expression {
    -3.14159 <= theta[b,t] <= 3.14159
  }
}
```

Range constraints expand to two linear rows internally. The outer operators MUST
be `<=` or `>=` (both operators must be non-strict). Strict inequality operators
(`<`, `>`) are not allowed in range constraints — see [§10 rule 39](#10-validation-requirements).
The general form is:

```
<lower_expr> <op1> <middle_expr> <op2> <upper_expr>
```

Slack variables:

A `slack` child on a constraint automatically creates a slack variable that
relaxes the constraint. The slack variable is added to the appropriate side of
the inequality and a penalty term is added to the objective.

```kdl
constraint balance {
  over t in=time
  slack penalty=1000
  expression {
    sum(dispatch[g,t] for g in gen) = demand[t]
  }
}
```

This is equivalent to manually declaring a slack control, adding it to the
constraint body, and adding a penalty to the objective:

```kdl
// what the compiler generates from the slack declaration above:
control balance_slack { index time; lower=0 }

constraint balance {
  over t in=time
  expression {
    sum(dispatch[g,t] for g in gen) + balance_slack[t] = demand[t]
  }
}

// penalty term added to objective:
// + 1000 * sum(balance_slack[t] for t in time)
```

`slack` properties:

- `penalty`: cost coefficient in the objective (required). MUST be a positive
  numeric value.
- `name`: override the auto-generated slack variable name (optional). Defaults
  to `<constraint_name>_slack`.
- `lower`: lower bound on the slack variable (optional, default `0`).
- `upper`: upper bound on the slack variable (optional, default unbounded).

For equality constraints (`=`), the compiler generates two non-negative slack
variables (one for each direction) unless the user specifies bounds. The
`balance` example above (which uses `=`) expands to the following (shown as
pseudo-code, not literal KDL syntax):

```
// balance_slack_pos[t] and balance_slack_neg[t] are auto-generated controls
sum(dispatch[g,t] ...) + balance_slack_pos[t] - balance_slack_neg[t] = demand[t]
objective += 1000 * sum(balance_slack_pos[t] + balance_slack_neg[t] for t in time)
```

For inequality constraints (`<=` or `>=`), a single non-negative slack variable
is generated on the constrained side.

Multiple slacks on the same constraint are not allowed. A constraint MUST have
at most one `slack` child.

Name collision avoidance: The auto-generated slack variable names
(`<constraint>_slack`, `<constraint>_slack_pos`, `<constraint>_slack_neg`) MUST
NOT collide with any user-declared `control` name. If a collision is detected,
validation MUST fail. To avoid collisions, either rename the user-declared
control or use the `name=` property on `slack` to override the generated name.

### 6.6 Objective

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

Objective bodies MAY reference named [`expression`](#64-expression) declarations
by identifier.

A model with zero objectives or more than one objective MUST fail validation.

```kdl
// INVALID: model with two objectives
model bad {
  set time from=horizon
  minimize cost { sum(c[t] for t in time) }
  maximize profit { sum(p[t] for t in time) }  // validation error
}
```

---

## 7. `scenario` declaration

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

Every `scenario` MUST contain exactly one `use` declaration. A `horizon`
declaration is REQUIRED only when the referenced model declares a time set
(`set time from=horizon`). If the model has no temporal dimension, `horizon` MAY
be omitted.

```kdl
// valid: non-temporal model, no horizon needed
scenario distance_check {
  use distance_model
  data distances from="data/distances.csv"
}

// valid: temporal model requires horizon
scenario day_ahead {
  horizon steps=24 resolution="PT1H"
  use dispatch_model
}
```

### 7.1 `horizon`

Conditionally required. Defines the active time set. This produces the built-in `time` set
(alias `t`) with members `1, 2, ..., steps` (inclusive). Any model set declared with `from=horizon`
resolves to this set.

`horizon` is REQUIRED when the referenced model declares `set time from=horizon`
or any set bound to the horizon. If the model has no temporal sets, `horizon`
MAY be omitted. A `horizon` on a non-temporal model is allowed but has no
effect.

```kdl
horizon steps=24 resolution="PT1H"
```

Note: `resolution` values are ISO 8601 durations (e.g., `PT1H`, `PT15M`) and
MUST be quoted in KDL since bare identifiers starting with `P` followed by
digits are not valid KDL identifiers.

### 7.2 `use`

Required. References the model to solve.

```kdl
use dispatch_model
```

### 7.3 `data` (inside `scenario`)

Binds CSV data sources to model parameters. Each `data` declaration makes a
named parameter available to the model at solve time.

```kdl
data demand from="data/demand.csv"
data capacity from="data/capacity.csv"
data fuel_cost from="data/fuel_cost.csv"
```

The `<name>` of each binding MUST match a `param` declared in the referenced
model. The CSV structure determines how the parameter is indexed according to
the following rules:

Column-to-index matching:

1. The model `param` declaration specifies which sets the parameter is indexed
   over (via `index_by` or `index` children).
2. Each index set MUST correspond to a column in the bound CSV file. Column
   matching uses the set name (after any `map` resolution in the source `data`
   block). If the set has an `alias`, the alias is NOT used for column matching —
   only the canonical set name.
3. The value column is matched by the `param` name (or its `from` override).
4. Extra columns in the CSV that do not match any index set or the param name
   are ignored.
5. Missing required columns (index sets or value column) MUST fail validation.

Example: A model declares `param demand { index region; index time }`. The
scenario binds `data demand from="data/demand.csv"`. The CSV must contain
columns `region`, `time`, and `demand` (or the column specified by `from`).
Each row provides one value of `demand` for a `(region, time)` pair.

For scalar parameters (no index sets), the CSV MUST contain exactly one data
row with the value column. Multiple rows for a scalar parameter MUST fail
validation unless `reduce` is specified.

For data scoping and override rules between top-level and scenario-level
`data`, see [§7.5](#75-data-scoping).

### 7.4 `report` (inside `scenario`)

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

Dual report output structure:

- For generated constraints (those with `over` clauses), the dual report
  produces one shadow price per generated row. The output is indexed by the same
  sets declared in the constraint's `over` clauses.
- For simple (non-generated) constraints, the dual report produces a single
  scalar value.
- The output format (CSV columns, JSON keys, etc.) is implementation-defined but
  MUST include the index values and the corresponding dual value for each row.
  The RECOMMENDED default format is CSV with one column per `over` index set
  followed by a `dual` value column.

### 7.5 Data scoping

`data` can appear at two levels:

- Top-level `data` with children (`map`, `set`, `param`) declares a shared
  namespace. Sets and parameters declared inside are globally visible — any
  model in the document can use them directly in algebra without redeclaration.
- Scenario-level `data` without children is a simple CSV-to-model-parameter
  binding scoped to that scenario only.

The parser distinguishes these by context: top-level `data` has a `{ ... }`
block, scenario-level `data` does not.

If a scenario-level `data` binding resolves the same **param name** as a
top-level `data` block, the scenario-level binding takes precedence for that
parameter within that scenario. The override is by param name, not by data block
name:

```kdl
// top-level: declares a param named "demand" inside block "demand_data"
data demand_data from="data/demand_base.csv" {
  set region
  param demand index_by=region
}

scenario stress_test {
  horizon steps=24 resolution="PT1H"
  use dispatch_model
  // overrides the "demand" param (originally from demand_data) for this scenario
  data demand from="data/demand_stress.csv"
}
```

In this example, the `stress_test` scenario resolves the `demand` param from
`data/demand_stress.csv` instead of from the top-level `demand_data` block. The
match is on the param name (`demand`), not on the data block name
(`demand_data`).

Name collisions across `data` blocks:

Because sets and parameters are globally visible, name uniqueness MUST be
enforced across all `data` blocks:

- Two `data` blocks MUST NOT declare `set` declarations with the same name.
  Duplicate set names across different data sources MUST fail validation (see
  [§10](#10-validation-requirements), rule 6).
- Two `data` blocks MUST NOT declare `param` declarations with the same name
  (rule 7). If two CSV files contain columns with the same logical name, use
  `map` to give them distinct names, or consolidate into one `data` block.

Global namespace design note: All set and param names share a single flat
namespace by design. This simplifies algebra expression resolution — every
identifier resolves unambiguously without requiring qualified names. For
projects that compose models from multiple teams or libraries, use naming
conventions (e.g., prefixes like `gen_capacity`, `line_capacity`) to avoid
collisions. A formal namespacing or module mechanism is not currently provided.
This is a known limitation of the current specification and is tracked for
future consideration (see `docs/reference/rfds/` for related design
discussions).

```kdl
// INVALID: param "capacity" declared in two data blocks
data generators from="data/generators.csv" {
  set gen_id
  param capacity index_by=gen_id
}
data lines from="data/lines.csv" {
  set line_id
  param capacity index_by=line_id  // validation error: duplicate param name
}
```

To resolve, rename one of the parameters:

```kdl
data generators from="data/generators.csv" {
  set gen_id
  param gen_capacity from=capacity index_by=gen_id
}
data lines from="data/lines.csv" {
  set line_id
  param line_capacity from=capacity index_by=line_id
}
```

```kdl
// sets and params here are globally visible to all models
data units from="data/units.csv" {
  set plant_id
  set unit_id alias=u { in plant_id }
  param capacity_mw index_by=unit_id
}

// both models can use plant_id, unit_id, and capacity_mw directly
model dispatch_model {
  set time from=horizon
  control dispatch { index unit_id; index time; lower=0 }
  constraint cap_limit {
    dispatch[u,t] <= capacity_mw[u]
  }
  minimize cost {
    sum(dispatch[u,t] for u in unit_id for t in time)
  }
}

// no set or param declarations needed — plant_id and capacity_mw
// are globally visible from the data block above (see §6.1)
model planning_model {
  control build kind=binary { index plant_id }
  constraint budget {
    sum(capacity_mw[p] * build[p] for p in plant_id) <= 1000
  }
  maximize capacity {
    sum(capacity_mw[p] * build[p] for p in plant_id)
  }
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

## 8. KDL 2.0 type annotations (optional)

Arco supports KDL 2.0 type annotations for users who want stronger metadata and
literal intent.

Node annotation:

```kdl
(f64)param capacity_mw { index plant_id; index unit_id }
```

Typed value literals in filters:

```kdl
param large_units from=capacity_mw { filter { capacity_mw >= (f64)200 } }
param cc_capacity from=capacity_mw { filter { prime_mover == (prime_mover)CC } }
```

Typed metadata values:

```kdl
param fuel_cost units=(unit)"$/MMBtu"
```

Type annotations are optional unless project policy requires them.

---

## 9. Filter predicate semantics

The `filter { ... }` block is used on [`set`](#52-set-inside-data) and
[`param`](#54-param-inside-data) declarations to narrow members or rows. The
block contains a bare-math predicate expression using the same syntax as
constraint `if` blocks.

Supported operators in filter predicates:

| Operator | Description                       |
| -------- | --------------------------------- |
| `==`     | equality (numeric or categorical) |
| `!=`     | not equal                         |
| `>`      | strict greater-than               |
| `>=`     | greater-than-or-equal             |
| `<`      | strict less-than                  |
| `<=`     | less-than-or-equal                |
| `and`    | logical conjunction               |
| `or`     | logical disjunction               |

There is no `not` unary operator. Boolean negation MUST be expressed through
inverse comparison operators (e.g., use `!=` instead of `not ==`, use `<`
instead of `not >=`).

Rules:

- `>`, `>=`, `<`, `<=` require numeric column values. Using them on non-numeric
  data MUST fail validation.
- `==` and `!=` support both numeric and categorical values.
- `and` / `or` combine multiple conditions in a single filter block.
- The predicate references column names from the parent `data` block (after
  `map` resolution).

```kdl
data generators from="data/generators.csv" {
  set gen
  set thermal { in gen; filter { type == thermal } }
  set large { in gen; filter { capacity >= 200 } }
  set large_thermal { in gen; filter { type == thermal and capacity >= 200 } }
  param capacity index_by=gen
}
```

---

## 10. Validation requirements

Implementations MUST validate at least:

Name uniqueness:

1. Duplicate `data` block names.
2. Duplicate `model` names.
3. Duplicate `scenario` names.
4. Duplicate `map` targets within one `data` block.
5. Duplicate `set` names within one `data` block.
6. Set name collisions across `data` blocks (same name, different data source).
7. Param name collisions across `data` blocks (same name, different data
   source).

Column and field resolution:

8. `map` without `from` MUST resolve to an existing CSV column.
9. Unknown source columns in `map from=...` or `param from=...`.
10. Unknown symbols in `index` / `index_by` / `index` children.

Set hierarchy:

11. `in` parent MUST exist.
12. `in` cycles MUST be detected.
13. Child-to-parent hierarchy contradictions (one child maps to multiple
    parents).

Indexing:

14. `index_by` and `index` children on the same declaration are mutually
    exclusive.
15. At most one `index` declaration per `data` block.
16. Non-unique indexing without `reduce`.

Filtering:

17. `filter` predicate references unknown column names.
18. Numeric comparison operator on non-numeric column data.
19. Implementations SHOULD detect contradictory filter predicates for simple
    single-variable range conditions (e.g., `capacity >= 30 and capacity <= 20`).
    Complex multi-variable contradictions MAY be left undetected.

Type and metadata:

20. Invalid `units` metadata token (must be a valid KDL string or identifier).
21. Type annotation conflicts (example `(f64)param ...` on text column).

Model structure:

22. `model` MUST contain exactly one objective.
23. Circular `expression` references.
24. `control kind=<value>` MUST be one of `continuous`, `integer`, `binary`.
25. Constraint generation references (`over in=...`) MUST resolve to known sets.

Scenario resolution:

26. `scenario` MUST contain `use`. `horizon` is REQUIRED when the model declares
    a temporal set (`from=horizon`); otherwise it is optional.
27. `scenario use <model_name>` MUST resolve to an existing `model`.
28. Scenario `data` binding names MUST match model `param` declarations.
29. Scalar `report` targets MUST resolve to a declared `expression` or
    objective.
30. Dual `report` targets MUST resolve to a declared `constraint`.

Subset resolution:

31. `in` parent set MUST be declared in the same `data` block or a top-level
    `set` declaration.
32. Filtered subset members MUST be a subset of the parent set members. If a
    filter produces an empty set, implementations SHOULD emit a warning
    diagnostic.

Temporal safety:

33. Constraints using temporal offsets (`t-1`, `t+1`) without a boundary `if`
    guard MUST produce a diagnostic.

Data integrity:

34. Empty CSV files (no data rows) MUST produce a diagnostic.

Operator context:

35. `==` in a constraint body (where `=` is required) MUST fail validation.
36. `=` in an `if` predicate or reduction `if` filter (where `==` is required)
    MUST fail validation.

Nonlinear and solver compatibility:

37. Constraint or objective bodies containing nonlinear built-in functions
    (`sqrt`, `pow` with non-integer exponent, `exp`, `ln`) SHOULD produce a
    diagnostic indicating the problem class is NLP/MINLP.

Slack variable naming:

38. Auto-generated slack variable names (`<constraint>_slack`,
    `<constraint>_slack_pos`, `<constraint>_slack_neg`) MUST NOT collide with
    user-declared `control` names.

Strict inequalities:

39. Strict inequality operators (`<`, `>`) in constraint bodies SHOULD produce a
    diagnostic warning, since LP/MIP solvers only support non-strict inequalities
    (`<=`, `>=`, `=`). Prefer `<=` or `>=` in all constraint algebra.

Bound algebra scoping:

40. Variable references inside `control` bound algebra blocks (`lower { ... }`,
    `upper { ... }`) MUST resolve to index names declared on the same `control`.

Alias uniqueness:

41. Set aliases MUST be unique across all set declarations. An alias MUST NOT
    collide with any declared set name.

Not-equal operators in constraint bodies:

42. The not-equal operator (`!=`) in constraint bodies MUST fail validation.
    This operator has no representation in LP/MIP solvers. It is valid only
    in predicate contexts (`if` blocks, `filter` blocks, reduction `if` clauses).

Model/data set name conflicts:

43. A `model` set declaration MUST NOT use the same name as a `set` already
    declared in a `data` block or at the top level. Model sets and data sets
    share a single global namespace (see [§6.1](#61-set-inside-model)).

### 10.1 Error reporting strategy

Implementations SHOULD collect and report all validation errors rather than
failing on the first error encountered. This enables users to fix multiple
issues in a single edit cycle. Specifically:

- Parse errors (malformed KDL) MAY abort early since subsequent parsing is
  unreliable.
- Validation errors (well-formed KDL violating Arco semantic rules) SHOULD be
  collected across the entire document and reported together.
- Each diagnostic MUST include the source location (file, line, column) and a
  human-readable message identifying the violated rule.
- Implementations SHOULD categorize diagnostics by severity: `error` for MUST
  violations (which prevent model execution) and `warning` for SHOULD violations
  (which allow execution but indicate likely mistakes).

---

## 11. Grammar (low-level profile)

The grammar below is a compact EBNF-style reference for the canonical low-level
profile. It describes Arco declarations layered on top of valid KDL 2.0 syntax.

Appendix A defines ergonomic authoring syntax that desugars into this grammar.

```ebnf
document          := { toplevel_set_decl | data_decl | model_decl
                     | scenario_decl }

toplevel_set_decl := "set" name [ "alias" "=" name ]
                     "{" { value } "}"

data_decl         := "data" name from_prop data_block
data_block        := "{" { map_decl | data_set_decl | index_decl
                     | data_param_decl } "}"

map_decl          := "map" name [ from_prop ]

data_set_decl     := "set" name [ "alias" "=" name ]
                     [ "{" [ in_child ] [ filter_block ] "}" ]
in_child      := "in" name

index_decl        := "index" name { name }

data_param_decl   := [ type_annot ] "param" name
                     [ from_prop ]
                     ( [ "index_by" "=" name ] | [ index_children ] )
                     [ "reduce" "=" reducer ]
                     [ "{" filter_block "}" ]
                     [ "units" "=" value ]

index_children    := "{" index_child ";" { index_child ";" } "}"
index_child       := "index" name | "reduce" reducer

model_decl        := "model" name model_block
model_block       := "{" { model_set_decl
                         | model_param_decl
                         | control_decl
                         | expression_decl
                         | constraint_decl
                         | objective_decl } "}"
                     (* use_data_decl is ergonomic syntax defined in
                        Appendix A.2, not part of the canonical grammar. *)

model_set_decl    := "set" name [ "alias" "=" name ] [ "from" "=" "horizon" ]

model_param_decl  := [ type_annot ] "param" name
                     ( [ "index_by" "=" name ] | [ index_children ] )

control_decl      := [ type_annot ] "control" name
                     ( compact_control | block_control )
                     [ "kind" "=" kind ]

compact_control   := "index_by" "=" name
                     [ "lower" "=" value ] [ "upper" "=" value ]

block_control     := "{" ctrl_index_child ";" { ctrl_index_child ";" }
                     [ lower_decl ] [ upper_decl ]
                     [ "kind" "=" kind ] "}"
ctrl_index_child  := "index" name [ "in" "=" name ]

                     (* For each direction (lower/upper), exactly one form
                        is allowed — property OR block, not both.
                        algebra_expr is defined below in the algebra
                        expression sub-grammar. *)
lower_decl        := "lower" "=" value | "lower" "{" algebra_expr "}"
upper_decl        := "upper" "=" value | "upper" "{" algebra_expr "}"

expression_decl   := "expression" name "{" algebra_expr "}"

constraint_decl   := "constraint" name ( simple_body | generated_body )
simple_body       := "{" constraint_expr "}"
generated_body    := "{" { over_decl } { if_decl } [ slack_decl ]
                     expression_body "}"
over_decl         := "over" name "in" "=" name
if_decl           := "if" "{" algebra_expr "}"
slack_decl        := "slack" "penalty" "=" value
                     [ "name" "=" name ]
                     [ "lower" "=" value ] [ "upper" "=" value ]
expression_body   := "expression" "{" constraint_expr "}"

                     (* constraint_expr uses the shared comp_op production,
                        which is intentionally permissive. Validation rules
                        restrict operators by context — see §10, rules 35,
                        39, and 42 for constraint-specific restrictions. *)
constraint_expr   := algebra_expr
                   | algebra_expr comp_op algebra_expr
                   | algebra_expr comp_op algebra_expr comp_op algebra_expr

objective_decl    := ( "minimize" | "maximize" ) name "{" algebra_expr "}"

scenario_decl     := "scenario" name scenario_block
scenario_block    := "{" { scenario_child } "}"
                     (* Children may appear in any order. Exactly one use_decl
                        is required. horizon_decl is required when the model
                        declares a temporal set. *)
scenario_child    := horizon_decl | use_decl | scenario_data_decl | report_decl

horizon_decl      := "horizon" "steps" "=" integer "resolution" "=" string
use_decl          := "use" name
scenario_data_decl:= "data" name from_prop
report_decl       := "report" ( name | "dual" name )

inline_selector   := "[" { name "=" value } "]"
                     (* Multiple key=value pairs are space-separated:
                        data_name[class=solar area=north] *)

from_prop         := "from" "=" ( path | field_name )
filter_block      := "filter" "{" algebra_expr "}"
reducer           := "sum" | "avg" | "min" | "max" | "first" | "last"
kind              := "continuous" | "integer" | "binary"
comp_op           := "<=" | ">=" | "<" | ">" | "=" | "==" | "!="
                     (* Note: this production is intentionally permissive.
                        Validation rules restrict operators by context:
                        - constraint bodies: "==" MUST fail (use "="); see §10 rule 35
                        - predicate contexts (if, filter): "=" MUST fail (use "=="); see §10 rule 36
                        - constraint bodies: "!=" MUST fail; see §10 rule 42
                        - constraint bodies: "<" and ">" SHOULD warn; see §10 rule 39 *)

name              := kdl_string
field_name        := kdl_string
path              := kdl_string
value             := kdl_value
string            := kdl_string
integer           := kdl_integer
type_annot        := "(" kdl_string ")"
numeric_literal   := kdl_integer | kdl_decimal  (* e.g. 42, 3.14 *)
string_literal    := kdl_string                 (* e.g. "hello" *)
bool_literal      := "true" | "false"

algebra_expr      := or_expr
or_expr           := and_expr { "or" and_expr }
and_expr          := comparison { "and" comparison }
comparison        := additive [ comp_op additive [ comp_op additive ] ]
additive          := multiplicative { ( "+" | "-" ) multiplicative }
multiplicative    := unary { ( "*" | "/" ) unary }
unary             := [ "-" ] postfix
postfix           := atom [ "[" index_list "]" ]
index_list        := index_entry { "," index_entry }
index_entry       := name [ ( "+" | "-" ) integer ]
                   | name "=" value
atom              := numeric_literal | string_literal | bool_literal
                   | name
                   | "(" algebra_expr ")"
                   | reduction
                   | function_call
reduction         := reducer "(" algebra_expr
                     { "for" binding "in" name }
                     { "if" algebra_expr } ")"
binding           := name | "(" name { "," name } ")"
function_call     := builtin_fn "(" algebra_expr { "," algebra_expr } ")"
builtin_fn        := "sqrt" | "pow" | "exp" | "ln" | "abs"
```

Notes:

- `name`, `field_name`, and `path` follow KDL string rules (identifier or
  quoted).
- `kdl_value` MAY be annotated (example `(f64)200`, `(unit)"$/MWh"`).
- Single-dimension indexing uses the `index_by=<set>` property form.
  Multi-dimension indexing uses child nodes: `{ index <set_a>; index <set_b> }`.
  Using both on the same declaration is a validation error.
- `reduce` has two equivalent forms: as a property on `param`
  (`reduce=<reducer>`) and as a child node inside an index block
  (`reduce <reducer>`, no `=`). Both produce the same semantics.
- `model_block` MUST contain exactly one `objective_decl`.
- `scenario_block` MUST contain one `use_decl`. `horizon_decl` is required only
  when the referenced model uses a temporal set (`from=horizon`).
- `inline_selector` is Arco-specific syntax valid only inside algebra expression
  strings. It is distinguished from variable indexing by the presence of `=`
  inside brackets. For named filtered domains, use `set subset_of` inside
  `data`.

---

## 12. Algebra expression summary

Algebra expressions appear inside [`constraint`](#65-constraint),
[`expression`](#64-expression), [`minimize` / `maximize`](#66-objective) bodies.
They are parsed as opaque strings by the KDL layer and interpreted by the Arco
algebra parser.

Logical operator scope: The logical operators `and` and `or` are valid only
inside predicate contexts: `if` blocks (constraint row filters), `filter` blocks
(set/param filters), and reduction `if` clauses. They MUST NOT appear in
constraint, expression, or objective bodies outside of these predicate contexts.
Using `and`/`or` in a non-predicate context MUST fail validation.

### 12.1 Literals

| Form            | Description      |
| --------------- | ---------------- |
| `42`, `3.14`    | numeric literals |
| `"hello"`       | string literals  |
| `true`, `false` | boolean literals |

### 12.2 Arithmetic operators

| Operator | Description            | Precedence |
| -------- | ---------------------- | ---------- |
| `+`      | addition               | low        |
| `-`      | subtraction / negation | low        |
| `*`      | multiplication         | high       |
| `/`      | division               | high       |

Standard arithmetic precedence applies: `*` and `/` bind tighter than `+` and
`-`. Parentheses MAY be used to override precedence.

### 12.3 Comparison operators

| Operator | Description               |
| -------- | ------------------------- |
| `<=`     | less than or equal        |
| `>=`     | greater than or equal     |
| `<`      | strict less than          |
| `>`      | strict greater than       |
| `=`      | equality (in constraints) |
| `==`     | equality (in predicates)  |
| `!=`     | not equal (predicates only)                |

`!=` is valid only in predicate contexts (`if` blocks, `filter` blocks,
reduction `if` clauses). Using it in constraint bodies MUST fail validation
(see [§10](#10-validation-requirements), rule 42).

`=` and `==` serve distinct roles and MUST NOT be interchanged:

- In constraint bodies, `=` denotes an equality constraint (a linear relation the
  solver enforces). Using `==` in a constraint body MUST fail validation.
- In `if` predicates and reduction filters, `==` is used for boolean equality
  tests. Using `=` in a predicate context MUST fail validation.

This distinction avoids ambiguity: `dispatch[a,t] = capacity[a]` creates a
solver constraint, while `type == solar` tests a boolean condition.

Strict inequality warning: Strict inequality operators (`<`, `>`) are
syntactically valid in constraint bodies but cannot be represented exactly by
LP/MIP solvers, which only support non-strict inequalities (`<=`, `>=`).
Implementations SHOULD emit a warning when strict inequalities appear in
constraint bodies. Prefer `<=` or `>=` in all constraint algebra.

### 12.4 Indexing

| Form       | Description                |
| ---------- | -------------------------- |
| `x[a]`     | single-dimension index     |
| `x[a,t]`   | multi-dimension index      |
| `x[a,t-1]` | temporal offset (backward) |
| `x[a,t+1]` | temporal offset (forward)  |

Temporal offsets (`t-1`, `t+1`) are valid on ordered sets (typically the `time`
set). Constraints using temporal offsets MUST include an `if` guard to exclude
boundary steps where the offset would be out-of-range (see
[§6.5](#65-constraint)).

### 12.5 Reductions

| Form                                   | Description            |
| -------------------------------------- | ---------------------- |
| `sum(expr for v in set)`               | summation over one set |
| `sum(expr for v in set for w in set2)` | nested summation       |
| `sum(expr for v in set if cond)`       | filtered summation     |
| `sum(expr for v in set if c1 if c2)`   | multiple filters (AND; use `==` not `=` — see [§10 rule 36](#10-validation-requirements)) |
| `sum(expr for (i, j) in arc_set)`      | tuple binding          |

Reductions iterate over sets declared in [`data`](#5-data-declaration) blocks or
[`model`](#6-model-declaration) blocks. Data-level sets (including
hierarchy-derived subsets) can be used directly inside algebra for aggregation.

Tuple bindings:

When a domain contains composite keys (e.g., arcs defined by origin-destination
pairs), tuple destructuring binds multiple variables simultaneously:

```
sum(flow[i,j] for (i, j) in branches)
```

Declaring a tuple-keyed set: The corresponding `data` block must declare the
composite set using multiple `index` children to define the key columns:

```kdl
data branches from="data/branches.csv" {
  // CSV has columns: from_bus, to_bus, capacity, ...
  set from_bus
  set to_bus
  index from_bus to_bus
  param capacity
}
```

The tuple binding `for (i, j) in branches` iterates over the unique `(from_bus,
to_bus)` pairs found in the CSV. Each binding variable maps positionally to the
index columns in declaration order.

Multiple filters:

Multiple `if` clauses are combined with AND semantics:

```
sum(dispatch[g,t] for g in generators for t in time if active[g] if t > 1)
```

Domain selectors in reductions:

```
// sum over a data-declared set
sum(capacity_mw[g] for g in solar_assets)

// sum over a set declared via in
sum(dispatch[g,t] for g in generator_data[class=solar] for t in time)

// sum over a named data-level subset (set { in ... })
sum(capacity_mw[g] for g in solar_gen)

// nested aggregation mixing data sets and model sets
sum(cost[a] * dispatch[a,t] for a in asset_id for t in time)
```

`param` reduce vs. algebra `sum`:

Set-level `param` aggregations (`reduce=sum`, `reduce=avg`, etc.) are resolved
at data-loading time and produce scalar parameters indexed by the target set.
Algebra-level `sum(...)` reductions are resolved at constraint generation time
and produce linear expressions. Both are available and serve different purposes:

- `param` with `reduce`: precomputed aggregate, available as a parameter.
- `sum(...)` in algebra: dynamic linear expression, generates solver terms.

### 12.6 Built-in functions

| Function    | Description       | Example          |
| ----------- | ----------------- | ---------------- |
| `sqrt(x)`   | square root       | `sqrt(variance)` |
| `pow(x, y)` | power             | `pow(base, 2)`   |
| `exp(x)`    | exponential       | `exp(rate)`      |
| `ln(x)`     | natural logarithm | `ln(price)`      |
| `abs(x)`    | absolute value    | `abs(flow[l,t])` |

Built-in functions accept one or more algebra expressions as arguments.

Linearity warning: `sqrt`, `pow` (with non-integer exponent), `exp`, and `ln`
produce nonlinear expressions. If these functions appear in a constraint or
objective body, the resulting problem is no longer a linear program (LP) or
mixed-integer program (MIP) and requires a solver that supports nonlinear
optimization (NLP/MINLP). Implementations SHOULD emit a diagnostic when
nonlinear built-in functions are used, indicating the problem class has changed.
`abs(x)` can be linearized automatically by the compiler using auxiliary
variables and constraints; the other functions cannot.

### 12.7 Inline selectors (inside algebra only)

| Form                         | Description               |
| ---------------------------- | ------------------------- |
| `data_name[field=value ...]` | anonymous filtered subset |

Inline selectors use `key=value` pairs inside brackets and are distinguished
from variable indexing by the presence of `=` signs. See
[§5.5](#55-inline-selectors) for details.

### 12.8 Expression references

Named [`expression`](#64-expression) declarations MAY be referenced by
identifier inside other expressions, constraints, and objectives. Circular
references MUST fail validation.

### 12.9 Constraint body forms

Constraint algebra supports two body forms:

Comparison form — a single relational operator:

```
dispatch[a,t] <= capacity_mw[a]
```

Range form — chained inequalities:

```
-3.14159 <= theta[b,t] <= 3.14159
```

The range form expands to two linear rows internally. See [§6.5](#65-constraint)
for details.

---

## Appendix A. Ergonomic syntax profile

This section defines a supported ergonomic authoring profile. Implementations
that claim support for this profile MUST accept the forms in this section and
lower them to canonical §1–12 forms before model execution.

All forms in this section are valid KDL 2.0 nodes, properties, and child blocks.

### A.1 Capability summary

| Capability         | Form                                  | Canonical expansion target                       |
| ------------------ | ------------------------------------- | ------------------------------------------------ |
| Data imports       | `use_data` in `model`                 | model-local `set` and `param` declarations       |
| Row predicates     | `if { ... }` in generated constraints | generated constraints with row filter            |
| Declarative bounds | `bounds <var> { ... }`                | control bounds metadata                          |
| Declarative fix    | `fix <var> { ... }`                   | equal lower and upper bounds (or equivalent row) |
| Edge domains       | network-topology naming conventions   | reusable filtered domains for graph structures   |

Note: The `set { in <parent>; filter { ... } }` syntax itself is canonical
(§5.2). The "Edge domains" capability here refers to recommended patterns for
using that syntax to model network topology (e.g., active branches, connected
buses), not additional syntax.

### A.2 `use_data` model imports

Syntax:

```kdl
model <name> {
  use_data <data_name> <data_name> ...
}
```

Semantics:

- Each referenced `<data_name>` MUST resolve to a top-level `data` block.
- Imported symbols are restricted to eligible `set` and `param` declarations.
- Explicit model-local declarations with the same name override imported
  declarations.
- Duplicate imports that remain ambiguous after overrides MUST fail validation.

### A.3 Row filters in generated constraints

The `if { ... }` block is the standard row-filter syntax. Multiple `if` blocks
combine with AND semantics (see [§6.5](#65-constraint)).

```kdl
constraint <name> {
  over a in=assets
  over t in=time
  if { active[a] }
  if { t > 1 }
  expression {
    dispatch[a,t] <= capacity[a]
  }
}
```

### A.4 Declarative bounds and fix

Syntax:

```kdl
bounds <control_name> {
  over i in=<set>
  lower { <algebra_expr> }
  upper { <algebra_expr> }
}

fix <control_name> {
  over i in=<set>
  value <kdl_value>
}
```

Semantics:

- `bounds` lowers to lower/upper metadata for the targeted control family.
- `fix` lowers to equal lower and upper bounds for the targeted index points, or
  an equivalent equality constraint.
- Conflicting bound assignments for the same variable index MUST fail
  validation.

### A.5 Edge-domain patterns

This section documents recommended patterns for modeling network topology (graph
structures such as transmission lines, pipelines, or transport arcs) using the
canonical `set { in ...; filter { ... } }` syntax defined in §5.2. No
additional syntax is introduced — this is a usage guide, not new grammar.

```kdl
data branch_data from="data/branches.csv" {
  set edge
  set active_edge { in edge; filter { conex == 1 } }
  param conex index_by=edge
}
```

Recommendations:

- Edge-domain sets SHOULD produce reusable domain members for constraint
  generation (e.g., iterating over active branches in a power flow constraint).
- Use `filter` to distinguish connected vs. disconnected edges, directional
  subsets, or capacity tiers.

### A.6 Ergonomic grammar

The following EBNF productions define the ergonomic forms introduced in this
appendix. These desugar into the canonical grammar defined in §11.

```ebnf
use_data_decl     := "use_data" name { name }

bounds_decl       := "bounds" name "{"
                     { over_decl }
                     [ lower_decl ] [ upper_decl ]
                     "}"

fix_decl          := "fix" name "{"
                     { over_decl }
                     "value" value
                     "}"
```

Notes:

- `use_data_decl` is valid only inside `model_block`. When the ergonomic profile
  is active, `model_block` accepts `use_data_decl` in addition to the canonical
  children defined in §11.
- `bounds_decl` and `fix_decl` are valid only inside `model_block`.

### A.7 Lowering and diagnostics requirements

- Ergonomic forms MUST preserve semantics of canonical expansion.
- Diagnostics SHOULD reference the original ergonomic source span.
- `arco print-model` and solver export operate on canonical lowered output.
- If ergonomic and explicit canonical declarations conflict in the same scope,
  explicit canonical declarations take precedence.

---

This document is the canonical reference for Arco KDL syntax. §1–12 define the
canonical low-level profile. [Appendix A](#appendix-a-ergonomic-syntax-profile)
defines the supported ergonomic authoring profile that lowers into it.
