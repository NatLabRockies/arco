# Arco Low-Level KDL Syntax Specification (KDL 2.0 Profile)

Version: 0.1.0 (Last updated: 2026-04-10)

Versioning: This specification follows
[Semantic Versioning](https://semver.org/). Minor versions (0.x.0) MAY introduce
breaking changes while the major version is 0. Once the specification reaches
1.0.0, breaking changes require a major version bump and deprecated features
will be documented with a removal target version.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

This document defines the low-level Arco DSL profile authored in KDL 2.0.

Scope of this specification:

- [`set`](#4-set-declaration-top-level) declarations (explicit domains)
- [`data`](#5-data-declaration) declarations (CSV-backed namespaces)
- [`param`](#31-inline-scalar-parameters) declarations (inline scalar constants)
- [`param`](#54-param-inside-data) declarations (CSV-backed projection,
  indexing, filtering, aggregation)
- [`model`](#6-model-declaration) declarations (optimization structure)
- [`control`](#63-control) declarations (decision-variable families)
- [`expression`](#64-expression) declarations (named reusable formulas)
- [`constraint`](#65-constraint) declarations (low-level algebra rows)
- [`minimize` / `maximize`](#66-objective) declarations (objective function)
- [`scenario`](#7-scenario-declaration) declarations (execution entrypoints)

---

## 1. Conformance

Arco files are KDL-based with non-KDL subgrammars. The structural layer MUST conform to [KDL 2.0](https://kdl.dev/spec/) everywhere Arco does not define an algebra or predicate subgrammar:

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

Unknown nodes: Implementations MUST reject unknown top-level node types
(anything other than `set`, `data`, `param`, `model`, `scenario`). Inside
blocks, unknown child node types MUST also fail validation. This ensures forward
compatibility is explicit: new node types require a spec version bump.

### 1.1 KDL compatibility

Arco targets [KDL 2.0](https://kdl.dev) as its host syntax. The live
specification is available at <https://kdl.dev/spec/>.

Arco's structural layer — declarations, properties, arguments, and nesting — is
KDL 2.0. Certain nodes, however, contain **algebra blocks**: `{ ... }` bodies
with bare math expressions (e.g., `dispatch[a,t] <= capacity[a]`) that are not
KDL nodes. This makes Arco a **superset** of KDL 2.0 — a standard KDL parser
cannot parse algebra block content.

Algebra blocks occur in the following contexts (exhaustive list):

| Parent node  | Context                         | Example                                  |
| ------------ | ------------------------------- | ---------------------------------------- |
| `filter`     | set/param row filter body       | `filter { type == thermal }`             |
| `if`         | constraint row-filter predicate | `if { active[g] }`                       |
| `expression` | constraint algebra body         | `expression { dispatch[a,t] <= cap[a] }` |
| `expression` | named reusable formula body     | `expression TotalCost { sum(...) }`      |
| `constraint` | simple-form constraint body     | `constraint cap { x[a] <= cap[a] }`      |
| `minimize`   | objective body                  | `minimize cost { sum(...) }`             |
| `maximize`   | objective body                  | `maximize profit { sum(...) }`           |
| `lower`      | control bound formula           | `lower { -capacity[l] }`                 |
| `upper`      | control bound formula           | `upper { capacity[l] }`                  |

All other `{ ... }` blocks in Arco (children of `data`, `model`, `scenario`,
`control`, `set`, `param`, `bounds`, `index`, `slack`, `report`) contain
standard KDL nodes and are parsed with normal KDL 2.0 rules.

Editor support: The `tree-sitter-arco-kdl` grammar (see
`tools/tree-sitter-arco-kdl/`) extends `tree-sitter-kdl` to recognize algebra
blocks. Editors configured with this grammar parse Arco files without errors.
The grammar exposes algebra content as `arco_math_text` nodes for language
injection and syntax highlighting.

KDL features used by Arco (structural layer):

- Nodes and children: every Arco declaration (`set`, `data`, `model`,
  `scenario`, `control`, `expression`, `constraint`, `minimize`, `maximize`)
  maps to a KDL node. Nested structure uses KDL child blocks (`{ ... }`).
- Arguments and properties: positional arguments carry names and values; named
  properties carry options (`source=`, `index=`, `alias=`, etc.).
- Type annotations: KDL 2.0 type annotations (e.g., `(f64)200`) are supported
  but optional (see [§8](#8-kdl-20-type-annotations-optional)).
- Comments: line comments (`//`) and slashdash comments (`/-`) are fully
  supported in both KDL and algebra block contexts.
- Multi-line strings and raw strings: KDL 2.0 multi-line and raw string literals
  are accepted wherever a string value is expected.

KDL features not used by Arco:

- Multi-line nodes (line continuations with `\`): accepted by the KDL parser but
  have no special Arco semantics.
- Null values: KDL `null` has no defined meaning in Arco. Implementations MUST
  reject `null` wherever a concrete value is expected. This applies to all value
  positions: arguments, property values (`source=`, `index=`, `units=`), and
  algebra literals. See [§10](#10-validation-requirements), rule 61.

### 1.2 Naming convention

Most declarations take their name as the first positional argument:

```kdl
param capacity_mw index=gen
```

Optionally, the name MAY be given as an explicit `name=` property instead. Both
forms are equivalent:

```kdl
// positional (preferred)
param capacity_mw from=cap_mw index=gen
// explicit name property (also valid)
param name=capacity_mw from=cap_mw index=gen
```

This applies to all named declarations: `set`, `data`, `model`, `scenario`,
`control`, `expression`, `constraint`, `minimize`, and `maximize`. (`use_data`
is not a named declaration; it takes data block references as arguments. See
[Appendix A.2](#a2-use_data-model-imports).) The positional form is RECOMMENDED
for brevity.

### 1.3 Alias rules

Alias uniqueness:

- Aliases (declared via `alias=<short>` on `set` declarations) MUST be unique
  across all set declarations (top-level, data-level, and model-level).
- An alias MUST NOT collide with any declared set name. For example, if a set is
  named `time`, no other set MAY use `alias=time`.
- If a conflict is detected, validation MUST fail.

Alias references:

- Anywhere this specification expects a set reference (`index`, the
  `{ in <set> }` clause, and algebra iteration domains), implementations MUST
  accept either the canonical set name or its alias. In the form
  `index <var> { in <set> }`, alias resolution applies only to the `<set>`
  position, not the `<var>` position (which is an iteration variable name). In
  the shorthand form `index <name>` (without `{ in ... }`), the `<name>` serves
  as both variable name and set reference, so alias resolution applies.
- Alias references MUST be resolved to the canonical set name before semantic
  validation and lowering.
- When both a canonical set name and an alias could match, implementations MUST
  prefer exact canonical-name matches first, then alias matches. With required
  alias uniqueness, resolution is unambiguous.

---

## 2. Terminology

This specification uses the following terms consistently:

| Term                     | Meaning                                                                                                                                                                                                                                                                     |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `algebra block`          | A `{ ... }` child block whose content is parsed as an algebra expression, not as KDL nodes. Also called a "bare-math block". This is Arco's extension to KDL 2.0; see [§1.1](#11-kdl-compatibility) for the exhaustive list of contexts and parsing implications.           |
| `expression`             | Two roles depending on context: (1) as a model child, a named reusable formula (`expression <name> { ... }`); (2) inside a generated constraint, the algebra body node (`expression { ... }`). See disambiguation note below.                                               |
| `constraint`             | A named algebraic relation (equality or inequality) declared with `constraint <name> { ... }`.                                                                                                                                                                              |
| `objective`              | The single optimization target, declared with `minimize <name> { ... }` or `maximize <name> { ... }`.                                                                                                                                                                       |
| `if`                     | Two roles: (1) a row-filter predicate inside a generated constraint, written as `if { ... }`; (2) a conditional guard in a reduction, written as `if cond` after `for ... in ...`. Multiple `if` blocks/clauses combine with AND semantics. Body uses algebra-block syntax. |
| `control`                | A decision-variable family.                                                                                                                                                                                                                                                 |
| `slack`                  | A child on a constraint that auto-generates a slack variable and penalty term in the objective.                                                                                                                                                                             |
| `param`                  | A data-backed or model-declared parameter (known constant at solve time).                                                                                                                                                                                                   |
| `set`                    | A named domain of indices.                                                                                                                                                                                                                                                  |
| `index` (constraint)     | Row-generation index in a generated constraint. Written as `index <var> { in <set> }` or `index <set>` when variable name matches the set.                                                                                                                                  |
| `bounds`                 | Child block inside `control` that contains formula-based `lower` and `upper` bound expressions. See [§6.3](#63-control).                                                                                                                                                    |
| `use_data` _(ergonomic)_ | Model import of sets/params from `data` blocks. Ergonomic profile only (see [Appendix A.2](#a2-use_data-model-imports)).                                                                                                                                                    |
| `map`                    | Binds a logical name to a CSV header inside a [`data`](#5-data-declaration) block.                                                                                                                                                                                          |
| `index` (data-block)     | Default indexing declaration for all `param` nodes in a [`data`](#53-index-inside-data) block.                                                                                                                                                                              |
| `index` (param/control)  | Per-declaration index child specifying which set(s) a `param` or `control` is indexed over. Written as `index <set>` or `index <var> { in <set> }`.                                                                                                                         |
| `report`                 | Scenario child requesting post-solve output (expression values or constraint duals). See [§7.3](#73-report-inside-scenario).                                                                                                                                                |
| `use`                    | Required scenario child that references the model to solve. Written as `use <model_name>`. See [§7.1](#71-use).                                                                                                                                                             |
| `reduce`                 | Aggregation function applied when indexing is non-unique (`sum`, `avg`, `min`, `max`, `first`, `last`). Two equivalent forms: `reduce=sum` (property) and `reduce sum` (child node).                                                                                        |

`expression` as declaration vs inside constraint: `expression` serves two roles
depending on context. As a model child, it declares a named reusable formula.
Inside a generated [`constraint`](#65-constraint), `expression { ... }` contains
the constraint's algebra body. Both use the same keyword; context determines the
meaning.

`index` as data-block default vs param/control child: Inside a `data` block,
`index` (§5.3) declares the default indexing columns for all `param`
declarations in that block. Inside a `param` or `control` declaration, `index`
children ([§5.4](#54-param-inside-data), [§6.3](#63-control)) specify
per-declaration indexing that overrides the block default. Both use the same
keyword; the parent node determines the meaning.

`if` in algebra vs `if` in constraints: Inside algebra expressions, `if` is a
filter clause on reductions (`sum(x for a in set if cond)`). Inside generated
constraints, `if { ... }` is a row-filter block that controls which rows are
generated. Both use the same predicate syntax but serve different purposes.

```arco
model dispatch {
  // "expression" - named reusable formula (model-level declaration)
  expression TotalFuelCost {
    sum(fuel_cost[g] * dispatch[g,t] for g in generators for t in time)
  }

  // "expression" inside constraint - the constraint's algebra body
  constraint capacity_limit {
    index g { in generators }
    index t { in time }
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
- [`param`](#31-inline-scalar-parameters) (inline scalar constant)
- [`data`](#5-data-declaration) (CSV-backed namespace)
- [`model`](#6-model-declaration)
- [`scenario`](#7-scenario-declaration)

```arco
// global scalar constants
param voll 9000 units="$/MWh"
param big_m 1e6

// explicit sets with inline members
set bus { 1; 2; 3; 4; 5 }

// CSV-backed data with subsets via set { in ... }
data generators source="data/generators.csv" {
  set gen
  set solar { in gen; filter { type == solar } }
  param pmax index=gen
}

model dispatch_model {
  // ... (model body omitted for brevity)
}

scenario day_ahead {
  use dispatch_model
  data demand source="data/demand.csv"
}
```

`scenario` is the execution entrypoint. Scenario-level `data` declarations are
simple CSV-to-param bindings, not namespaced blocks; see
[§7.2](#72-data-inside-scenario).

Declaration order: top-level declarations MAY appear in any order. Forward
references are allowed (a `scenario` MAY reference a `model` declared after it).
All names are resolved after the full document is parsed.

### 3.1 Inline scalar parameters

A scalar `param` MAY be declared with an inline literal value as its first
positional argument, without requiring a CSV file:

```kdl
param voll 9000 units="$/MWh"
param discount_rate 0.05
param big_m 1e6
```

The value MUST be a numeric literal. In algebra, reference the param by name as
a constant (no index brackets): `voll`, `discount_rate`, `big_m`.

Inline scalars (literal constants) are valid at the top level (as global
constants) and inside `model` blocks (as model-local constants). They MUST NOT
appear inside `data` blocks — `data` blocks are strictly CSV-backed, so every
`param` inside a `data` block MUST read its value from the CSV file (either
indexed or scalar via a single-row CSV). Inline scalars MUST NOT have `index`,
`index` children, `from`, or `reduce` properties.

---

## 4. `set` declaration (top-level)

A top-level `set` declares a named domain with explicit members listed inline.
This is useful for sets that are not backed by a CSV file, for example index
ranges, scenario labels, or piecewise segments.

Explicit member list:

```
set <name> { <member1>; <member2>; ... }
```

Members are KDL arguments (strings or numbers) separated by semicolons. Members
MUST be unique within a single `set` declaration. Duplicate members MUST fail
validation (see [§10](#10-validation-requirements), rule 51).

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
the block are globally visible. Any `model` or algebra expression in the
document can reference them by name. A single `data` block can supply sets and
parameters to multiple models.

```
data <name> source=<path> { ... }
```

Required properties:

- `source`: CSV file path. Relative paths are resolved from the directory
  containing the `.kdl` file being parsed. Absolute paths are also accepted.

CSV parsing: Arco expects RFC 4180-compliant CSV files (comma-delimited,
optional double-quote escaping, CRLF or LF line endings). The first row MUST be
a header row containing column names. Column names in the header MUST be unique;
duplicate column names MUST fail validation (see
[§10](#10-validation-requirements), rule 73). CSV files MUST be UTF-8 encoded.
Implementations SHOULD accept files with or without a UTF-8 BOM. Empty cells in
a numeric column MUST fail validation. Empty cells in a string/categorical
column are treated as empty strings. Column matching is always by name, not by
position.

Allowed children:

- `map`
- `set`
- `index`
- `param`

### 5.1 `map`

`map` binds logical names to CSV headers.

```
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
Unlike `param`, `set` declarations inside a `data` block do not accept a `source=`
property; the set name itself determines the column. Sets declared inside a
`data` block are globally visible and can be referenced by any model,
constraint, or algebra expression in the document.

```
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
- `alias` provides a short set reference that MAY be used wherever a set
  reference is expected (`index=` property, `index` children,
  `index ... { in ... }`, and algebra iteration domains). Example:
  `set asset_id alias=a` allows `param capacity { index a }`,
  `index a_idx { in a }`, and `dispatch[a,t]`.
- `in <parent>` declares that this set is contained within `<parent>`. Each
  child value maps to exactly one parent value, forming a hierarchy edge. If a
  child value in the CSV maps to more than one distinct parent value, validation
  MUST fail with a hierarchy contradiction error (see
  [§10](#10-validation-requirements), rule 13). The parent set MUST be declared
  in the same `data` block or as a top-level `set` (see
  [§10](#10-validation-requirements), rule 32). Model-level sets MUST NOT be
  used as `in` parents.
- `filter { ... }` narrows set members using an algebra predicate block. The
  expression is evaluated per row against the dataset columns. This uses the
  same bare-math block syntax as `expression`, `if`, and `lower`/`upper`.
  Supported operators: `==`, `>`, `>=`, `<`, `<=`, `!=`.

```arco
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

// combined: subset of a parent set with a filter applied
// "solar generators in region A" — gen is the parent, filter narrows to solar+region
set solar_region_a {
  in gen
  filter { type == solar and region == A }
}
```

Set resolution:

A set is always resolved from exactly one CSV file: the `data` block that
declares it. The members of the set are the unique values found in the
corresponding column of that CSV. There is no implicit union or merge across
files. The set name MUST resolve to a CSV column in the parent `data` block
(after `map` resolution); see [§10](#10-validation-requirements), rule 66.

If data is spread across multiple CSV files and you need a combined domain:

1. Consolidate the data into a single CSV so one `data` block produces the full
   set, using `in` with `filter` to distinguish subgroups.
2. Declare separate sets per file and iterate over each one independently.

For a full worked example of both approaches, see the data scoping discussion in
[§7.4](#74-data-scoping).

### 5.3 `index` (inside `data`)

`index` defines default indexing for `param` declarations in that `data` block.
Multiple columns can be listed as arguments to a single `index` declaration to
form a multi-column index.

```
index <set_a>
index <set_a> <set_b>
index <set_a> <set_b> <set_c>
```

Semantics:

- `index` is optional.
- If omitted, default index is 1-based numeric row order (row 1, 2, 3, ...).
  Parameters indexed by row order are referenced in algebra using integer
  indices: `x[1]`, `x[2]`, etc. For example, given a CSV with three rows and no
  `index` declaration, `param cost` produces `cost[1]`, `cost[2]`, `cost[3]`.
- Every index symbol MUST resolve to a declared set name or set alias.
- At most one `index` declaration is allowed per `data` block. To index by
  multiple columns, list them all as arguments to a single `index` node.
- The declared `index` sets the default indexing for all `param` declarations in
  the same `data` block that do not specify their own `index` property or
  `index` children (see [§5.4](#54-param-inside-data)).
- If a model declares a `set` with the same name as a data-level `set`, the
  model-level set shadows the data-level set within that model's scope. To
  prevent silent dimension mismatches when parameters inherit block-level
  indexing, model-level sets MUST NOT shadow data-level or top-level set names
  (see [§10](#10-validation-requirements), rule 44).

```kdl
// valid: multi-column index on a single declaration
data plants source="data/plants.csv" {
  set plant_id
  set unit_id { in plant_id }
  index plant_id unit_id
  param capacity_mw
}

// INVALID: two separate index declarations
data plants source="data/plants.csv" {
  set plant_id
  set unit_id
  index plant_id
  index unit_id  // validation error: duplicate index declaration
}
```

### 5.4 `param` (inside `data`)

`param` projects values from CSV columns into named parameters.

By default, the param name is used to match the CSV column header. If the CSV
column has a different name, use `from=<column>` to specify which CSV column
supplies the values. The param's logical name (used in algebra) is always the
`<name>` given to the `param` declaration. `from=` only controls where the data
is read from.

Single-dimension indexing (property form):

```
param <name>
param <name> from=<csv_column>
param <name> index=<set>
param <name> { index <set> }
```

Multi-dimension indexing (child node form):

```
param <name> { index <set_a>; index <set_b> }
param <name> from=<csv_column> { index <set_a>; index <set_b> }
```

`<set>` references in the `index=` property and `index` children MAY use either
canonical set names or aliases. Implementations MUST normalize aliases to
canonical set names.

Example, given this CSV:

```csv
gen_id,capacity_mw,heat_rate
g1,500,10.5
g2,300,11.2
g3,150,9.8
```

```kdl
data generators source="data/generators.csv" {
  set gen_id
  // reads from the "capacity_mw" column (name matches)
  param capacity_mw index=gen_id
  // reads from the "heat_rate" column but exposes it as "hr" in algebra
  param hr from=heat_rate index=gen_id
}
```

In algebra, reference `capacity_mw[g]` and `hr[g]`. The `source=` property only
affects which CSV column is read, not the param's logical name.

The `index=` property and `index` children are mutually exclusive. Using both on
the same `param` MUST fail validation.

```kdl
// INVALID: index and index children on the same param
param cost index=asset { index time }  // validation error
```

Aggregation:

```
// property form (single-dimension)
param <name> index=<set> reduce=<reducer>

// child node form (multi-dimension)
param <name> { index <set_a>; index <set_b>; reduce sum }
```

Both forms are equivalent. The child node form (`reduce sum`, no `=`) is used
inside index blocks; the property form (`reduce=sum`) is used on the param node
directly.

Example of each form:

```kdl
// property form
param total_cap index=region reduce=sum

// child node form
param avg_cost { index region; index fuel_type; reduce avg }
```

Supported reducers:

- `sum`, `avg`, `min`, `max`, `first`, `last`

`first` and `last` select the first or last value encountered in CSV row order
(the order rows appear in the source file). `first` and `last` over an empty
group (after filtering) MUST produce a data-loading error. `avg` over an empty
group (after filtering) MUST produce a data-loading error (division by zero).
`min` and `max` over an empty group MUST produce a data-loading error. `sum`
over an empty group evaluates to `0`.

Note: The empty-group errors above apply to `reduce` aggregation at data-loading
time. For `min(...)` and `max(...)` in algebra expressions, domain emptiness may
depend on runtime data, so those errors occur at solve time (see
[§10](#10-validation-requirements), rule 67).

Filtering:

```
param <name> from=<field> { filter { <predicate> } }
```

The `filter` block uses the same bare-math algebra syntax as `set` filters and
constraint `if` blocks:

Given a CSV with columns `gen_id`, `capacity_mw`, and `prime_mover`:

```arco
param cc_capacity from=capacity_mw { filter { prime_mover == CC } }
param large_units from=capacity_mw { filter { capacity_mw >= 200 } }
```

Order of operations when `filter` and `reduce` are combined: filtering is
applied first, then the reducer operates on the filtered rows. For example:

```arco
// first filter to thermal rows, then sum their capacity
param total_thermal_cap from=capacity_mw index=region reduce=sum {
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
vocabulary of valid unit tokens; any KDL string or identifier is accepted. Units
serve as documentation metadata and are preserved in solver output and
diagnostics. Implementations MAY use units for dimensional consistency checks
but are not required to validate unit semantics.

Scalar parameters:

A `param` with no `index`, no `index` children, and no block-level `index`
declaration is a scalar parameter.

Inline scalar form:

Inline scalar parameters (literal constants without CSV backing) are defined in
[§3.1](#31-inline-scalar-parameters). They are valid at the top level and inside
`model` blocks, but MUST NOT appear inside `data` blocks.

CSV-backed scalar form:

When the scalar value comes from a CSV file, the CSV MUST contain exactly one
data row with the value column. For how scalar parameters are bound at scenario
time, see [§7.2](#72-data-inside-scenario).

```csv
discount_rate,value_of_lost_load
0.05,9000
```

```kdl
data settings source="data/settings.csv" {
  // column name matches param name, no from= needed
  param discount_rate
  // column name differs, use source= to read "value_of_lost_load" as "voll"
  param voll from=value_of_lost_load units="$/MWh"
}
```

In algebra, reference these as scalar constants: `discount_rate` and `voll` (no
index brackets needed since they are not indexed over any set).

Semantics:

- If `source` is omitted, source field defaults to `<name>`. Column header
  matching is case-sensitive. CSV column names MUST match exactly (after `map`
  resolution).
- If neither the `index=` property nor `index` children are present, default is
  the block's `index` declaration if present (see
  [§5.3](#53-index-inside-data)), else 1-based numeric row position (row 1, 2,
  3, ...). In this fallback case, no named set is created; the parameter is
  indexed by implicit row order and referenced in algebra by its positional
  integer index.
- If indexing is non-unique, `reduce` MUST be provided.

### 5.5 Inline selectors

Dataset rows MAY be filtered inline using bracket notation on a `data` block
name inside algebra expressions. This produces an anonymous subset without
requiring a named declaration. The name before the brackets MUST resolve to a
declared `data` block (not a `set` name); see
[§10](#10-validation-requirements), rule 46.

```
<data_name>[<field>=<value> ...]
```

Inline selectors use `key=value` pairs inside brackets. Variable indexing uses
positional comma-separated indices (`dispatch[a,t]`). The parser distinguishes
these by the presence of `=` signs inside the brackets. The `=` in inline
selectors is selector syntax, not an equality operator; it is exempt from the
operator context restrictions in [§10](#10-validation-requirements) rule 37.

```
// single-field filter
sum(capacity_mw[g] for g in generator_data[class=solar])

// multi-field filter (space-separated key=value pairs)
sum(dispatch[g,t] for g in generator_data[class=solar area=north] for t in time)

// inside a nested reduction
sum(cost[g] * dispatch[g,t]
    for g in generator_data[fuel=gas]
    for t in time)
```

Inline selectors are valid only inside algebra expression strings. Value
resolution in inline selectors follows the same rules as filter predicates
([§9](#9-filter-predicate-semantics)): bare identifiers on the RHS are treated
as categorical string values, not column references. For top-level named
filtered domains, use `set { in ... }` inside the relevant `data` block.

---

## 6. `model` declaration

`model` declares low-level optimization structure.

```
model <name> { ... }
```

Allowed children:

- [`use_data`](#a2-use_data-model-imports) - ergonomic profile only (import
  sets/params from `data` blocks; defined in
  [Appendix A.2](#a2-use_data-model-imports), not part of the canonical
  low-level grammar)
- [`set`](#61-set-inside-model)
- [`param`](#62-param-inside-model)
- [`control`](#63-control)
- [`expression`](#64-expression)
- [`constraint`](#65-constraint)
- [`minimize` or `maximize`](#66-objective) (exactly one)

### 6.1 `set` (inside `model`)

Model-domain sets. These are abstract domains resolved at scenario time.

```
set <name>
set <name> alias=<short>
```

> [!NOTE]
>
> - `alias` provides a short set reference usable in set-reference positions
>   (`index`, `index`, `index ... { in ... }`) and algebra iteration domains.
>   Example: `set asset_id alias=a`.
> - Model sets are abstract. They acquire concrete members from scenario data
>   bindings and `data` block sets at solve time. Hierarchy and filtering are
>   defined in `data` blocks, not in models.
> - Models do not need to re-declare sets that are already defined in a
>   top-level `data` block. Data-level sets are globally visible and can be used
>   directly in model algebra (constraints, expressions, objectives) without
>   redeclaration. The same applies to data-level parameters.

- Name conflict rule: If a `model` declares a `set` with the same name as a
  `set` already declared in a `data` block, the model-level declaration MUST
  fail validation. Model sets and data sets share a single global namespace; a
  model cannot shadow or override a data-level set. To use a data-level set in a
  model, reference it directly without redeclaration.

```arco
// data declares gen, capacity_mw, fuel_cost, and the thermal_gen subset
data generators source="data/generators.csv" {
  map gen from=generator_id
  set gen alias=g
  set thermal_gen { in gen; filter { type == thermal } }
  param capacity_mw index=gen
  param fuel_cost index=gen
}

// model uses gen, thermal_gen, and capacity_mw directly, no redeclaration
model dispatch {
  set time alias=t
  control output { lower 0; index gen; index time }

  constraint cap_limit {
    index g { in gen }
    index t { in time }
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

The only sets a model needs to declare are abstract sets that do not come from a
`data` block. For example, a model that needs a `time` domain declares
`set time alias=t` and the scenario provides the concrete members via a `data`
binding or a top-level `set` declaration.

### 6.2 `param` (inside `model`)

Model parameters are declared with index intent.

Single-dimension:

```
param <name> index=<set>
```

Multi-dimension:

```
param <name> { index <set_a>; index <set_b> }
```

`<set>` references in the `index=` property and `index` children MAY use either
canonical set names or aliases. Implementations MUST normalize aliases to
canonical set names.

The `index=` property and `index` children are mutually exclusive.

Model parameters are resolved at scenario time. The scenario binds concrete
values via `data` declarations ([§7](#7-scenario-declaration)). A model
parameter name MUST match either a scenario `data` binding name or a top-level
`data` block `param` name for the scenario to resolve it.

### 6.3 `control`

Decision-variable families. A `control` declaration defines a family of decision
variables indexed over one or more sets.

The preferred form uses a child block with `index` children. Literal bounds MAY
be written either as `lower=`/`upper=` properties on the `control` node or as
child nodes inside the block, and `kind` remains a property on the `control`
node:

```
control <name> kind=continuous {
  lower 0
  upper 100
  index <set_a>
  index <set_b>
}
```

Equivalent property form:

```
control <name> lower=0 upper=100 kind=continuous {
  index <set_a>
  index <set_b>
}
```

All children are optional except at least one `index`.

Compact single-dimension form:

```
control <name> index=<set>
control <name> index=<set> kind=binary lower=0 upper=1
```

The `index=` property and `index` children are mutually exclusive. Using both on
the same `control` MUST fail validation.

Index domain binding:

The `{ in <set> }` child block on `index` binds the index variable to a named
domain set (canonical name or alias). This is useful when the iteration domain
differs from the index name:

```
control <name> {
  index <set_a> { in <domain_a> }
  index <set_b> { in <domain_b> }
}
```

Properties and children:

- `index` children or `index`: indexing sets (at least one required)
- `lower`: lower bound (optional). Accepts a literal value as a property
  (`lower=0`) or child node (`lower 0`), or an algebra block inside
  `bounds { lower { ... } }`.
- `upper`: upper bound (optional). Accepts a literal value as a property
  (`upper=100`) or child node (`upper 100`), or an algebra block inside
  `bounds { upper { ... } }`.
- `value`: fixed value (optional). Sugar for `lower=X upper=X`. Mutually
  exclusive with `lower` and `upper` in any form.
- `kind`: variable type (optional). Allowed values:
  - `continuous` (default)
  - `integer`
  - `binary`

When `kind=binary`, implementations MUST validate that explicit `lower` and
`upper` bounds (if provided) are within the [0, 1] range. Bounds outside this
range MUST fail validation (see [§10](#10-validation-requirements), rule 62).

Bounds:

There are four ways to specify bounds on a `control`:

1. **Literal bounds as child nodes** — scalar values inside the `control`
   block:

```kdl
control dispatch {
  lower 0
  upper 500
  index gen
  index time
}
```

2. **Literal bounds as properties** — scalar values on the `control` node:

```kdl
control dispatch lower=0 upper=500 {
  index gen
  index time
}
```

3. **Formula bounds** — algebra expressions inside a `bounds` child block:

```arco
control flow {
  index l { in lines }
  bounds {
    lower { -capacity[l] }
    upper { capacity[l] }
  }
}
```

4. **Mixed** — literal property or child node for one direction, formula in
   `bounds` for the other:

```arco
control output {
  lower 0
  index g { in gen }
  index time
  bounds {
    upper { capacity[g] }
  }
}
```

5. **Fixed value** — `value=` sets both lower and upper to the same value:

```kdl
control dispatch value=100.0 {
  index gen
  index time
}
```

`value=` is syntactic sugar for `lower=X upper=X`. Specifying `value=` together
with `lower=`, `upper=`, or a `bounds` block MUST fail validation.

The `bounds` child block contains `lower { ... }` and/or `upper { ... }` nodes
whose bodies use the same bare-math algebra syntax as `expression` and
`constraint` bodies. No quoting is needed.

Specifying both a literal property and a formula block for the same direction
MUST fail validation:

```arco
// INVALID: two lower bounds on the same control
control flow {
  lower 0
  index lines
  bounds {
    lower { -capacity[l] }  // validation error: conflicts with lower 0
  }
}
```

Bound algebra variable scoping:

The algebra inside `bounds { lower { ... } }` and `bounds { upper { ... } }` MAY
reference the control's own index variables. The variable names used in the
algebra MUST match the index names declared on the same `control`. References to
undeclared variables MUST fail validation.

```arco
control flow {
  index l { in lines }
  bounds {
    lower { -capacity[l] }   // valid: `l` matches the index name
    upper { capacity[l] }    // valid
  }
}

// INVALID: `x` is not a declared index on this control
control flow {
  index l { in lines }
  bounds {
    upper { capacity[x] }    // validation error: unknown variable `x`
  }
}
```

### 6.4 `expression`

Named reusable algebra formulas.

```
expression <name> {
  sum(fuel_cost[a,t] * dispatch[a,t] for a in assets for t in time)
}
```

The algebra body is written directly inside `{ ... }` as bare math. No quoting
is needed. The normalizer automatically converts the bare-math content into a
canonical internal representation before parsing. This bare-math block syntax is
available on all algebra-bearing nodes: `expression`, `constraint`, `minimize`,
`maximize`, `lower`, `upper`, `if`, and `filter`.

Expressions MAY reference other named expressions by identifier. Circular
references MUST fail validation.

Free variables in expression bodies (index variables that appear in indexed
references but are not bound by a `for` clause in a reduction) are resolved at
the point of use. When an expression is referenced inside a constraint with
`index` clauses, the constraint's iteration variables are in scope for the
expression body.

### 6.5 `constraint`

Two supported forms.

Simple algebra body:

```
constraint <name> {
  dispatch[a,t] <= capacity_mw[a]
}
```

In the simple form, iteration variables are inferred from indexed references in
the body. The compiler resolves each variable to its corresponding declared set
by matching against `control` and `param` index signatures. For example, if
`dispatch` is declared as `control dispatch { index asset_id; index time }`,
then `a` resolves to `asset_id` (first index position) and `t` resolves to
`time` (second index position). The simple form implicitly generates one
constraint row per combination of resolved index sets. It is equivalent to a
generated form with `index` clauses for each inferred variable. If a variable
appears in multiple declarations (`control` or `param`) with conflicting index
signatures (i.e., the same positional variable resolves to different set names
across declarations), validation MUST fail with an ambiguity error. Similarly,
if index inference fails because a referenced declaration has no index signature
at all, validation MUST fail with a missing-index error.

Simple-form constraints do not support `if` guards or explicit `index` clauses.
`slack` children ARE supported on simple-form constraints (see below).
Constraints that require temporal offset guards (`t-1`, `t+1`) or row filtering
MUST use the generated form.

Implementer note: Simple-form inference relies on positional matching, which can
be fragile when `control` and `param` declarations have different index arities.
When in doubt, prefer the generated form with explicit `index` clauses for
clarity and safety.

Generated row form:

```
constraint <name> {
  index a { in asset_id }
  index t { in time }
  if { active[a] }
  expression {
    dispatch[a,t] <= capacity_mw[a]
  }
}
```

- `index` creates explicit row generation domains.
- `if { ... }` filters generated rows (optional). The body MUST be an explicit
  boolean predicate (using a comparison operator such as `==`, `!=`, `>`, `>=`,
  `<`, `<=`). Bare numeric references without a comparison operator (e.g.,
  `if { count[a] }`) MUST fail validation. This prevents silent bugs where a
  zero value is indistinguishable from a "false" condition. `NaN` MUST fail
  validation.
- `expression` contains the constraint algebra body.

The generated form is preferred when iteration domains need to be explicit or
when row filtering is required.

Row filters with `if`:

The `if` block filters which rows are generated. The predicate MUST reference at
least one of the iteration variables declared by the `index` clauses. A
condition that does not depend on any loop variable is a static condition and
MUST fail validation (see [§10](#10-validation-requirements), rule 45):

```arco
// valid: condition references loop variable `t`
if { t > 1 }

// valid: condition references loop variable `g`
if { active[g] }

// INVALID: condition does not reference any index variable
if { 1 > 0 }  // validation error
```

The `if` block supports arbitrary algebra predicates, including numeric
comparisons and temporal conditions:

```arco
constraint ramp_up {
  index g { in generators }
  index t { in time }
  if { t > 1 }
  expression {
    dispatch[g,t] - dispatch[g,t-1] <= ramp_up_rate[g]
  }
}
```

Common `if` patterns:

- `if { t > 1 }` - skip the first time step (required when using `t-1`)
- `if { t < num_steps }` - skip the last time step (required when using `t+1`;
  `num_steps` is a user-declared scalar param representing the step count)
- `if { t == 1 }` - apply only at the first time step
- `if { active[a] }` - filter by a boolean parameter

Nested `if` conditions:

Multiple `if` blocks MAY appear in the same constraint. They are combined with
AND semantics. All conditions MUST be true for the row to be generated:

```arco
constraint conditional_ramp {
  index g { in generators }
  index t { in time }
  if { t > 1 }
  if { active[g] }
  expression {
    dispatch[g,t] - dispatch[g,t-1] <= ramp_up_rate[g]
  }
}
```

Temporal offsets and boundary guards:

An **ordered set** is a set whose members have a well-defined sequence. Ordering
is determined by: (1) for numeric members, numeric sort order; (2) for top-level
`set` declarations with inline members, declaration order; (3) for data-level
sets, CSV row order (first occurrence of each unique value). A set whose
ordering cannot be determined by any of these three rules is **unordered**.
Temporal offsets (`t-1`, `t+1`) are valid only on ordered sets. Implementations
MUST reject temporal offsets applied to unordered sets (validation error). This
prevents silently producing undefined iteration sequences.

Algebra expressions support temporal offset indexing (`t-1`, `t+1`) on ordered
sets. When a constraint references a previous or next time step, an `if` guard
MUST be present to exclude boundary steps where the offset would be
out-of-range. Failing to guard temporal offsets is a validation error.

```arco
// INVALID: t-1 without a guard on the first time step
constraint unguarded_ramp {
  index g { in generators }
  index t { in time }
  expression {
    dispatch[g,t] - dispatch[g,t-1] <= ramp_rate[g]  // validation error
  }
}
```

Range constraints (chained inequalities):

Constraint bodies MAY use chained inequalities to express range bounds:

```arco
constraint angle_bounds {
  index b { in buses }
  index t { in time }
  expression {
    -3.14159 <= theta[b,t] <= 3.14159
  }
}
```

Range constraints expand to two linear rows internally. The outer operators MUST
be `<=` or `>=` (both operators MUST be non-strict). Strict inequality operators
(`<`, `>`) in range constraints MUST fail validation (see
[§10 rule 40](#10-validation-requirements)). The general form is:

```
<lower_expr> <op1> <middle_expr> <op2> <upper_expr>
```

Slack variables:

A `slack` child on a constraint automatically creates a slack variable that
relaxes the constraint. The slack variable is added to the appropriate side of
the inequality and a penalty term is added to the objective.

```arco
constraint balance {
  index t { in time }
  slack penalty=1000
  expression {
    sum(dispatch[g,t] for g in gen) = demand[t]
  }
}
```

This is equivalent to manually declaring a slack control, adding it to the
constraint body, and adding a penalty to the objective:

```arco
// what the compiler generates from the slack declaration above:
control balance_slack { lower 0; index time }

constraint balance {
  index t { in time }
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

Slack on range constraints: For range constraints (chained inequalities like
`a <= x <= b`), `slack` applies to **both** generated inequality rows. The
compiler generates one slack variable per row (two total), both penalized in the
objective. The slack variable names follow the pattern `<constraint>_slack_lo`
and `<constraint>_slack_hi`.

`slack` on simple-form constraints: `slack` declarations are valid on both
simple-form and generated-form constraints. On a simple-form constraint, the
slack variable is indexed over the same inferred iteration domains as the
constraint itself.

Name collision avoidance: All auto-generated slack variable names MUST NOT
collide with any user-declared `control` name. The full set of generated name
patterns is: `<constraint>_slack` (inequality), `<constraint>_slack_pos` and
`<constraint>_slack_neg` (equality), `<constraint>_slack_lo` and
`<constraint>_slack_hi` (range). If a collision is detected, validation MUST
fail (see [§10](#10-validation-requirements), rule 39). To avoid collisions,
either rename the user-declared control or use the `name=` property on `slack`
to override the generated name.

### 6.6 Objective

Exactly one objective is required per model.

```arco
minimize total_cost {
  sum(variable_cost[a] * dispatch[a,t] for a in asset_id for t in time)
}
```

or

```kdl
maximize welfare {
  // ... (algebra body omitted for brevity)
}
```

Objective bodies MAY reference named [`expression`](#64-expression) declarations
by identifier.

A model with zero objectives or more than one objective MUST fail validation.

```arco
// INVALID: model with two objectives
model bad {
  set time alias=t
  minimize cost { sum(c[t] for t in time) }
  maximize profit { sum(p[t] for t in time) }  // validation error
}
```

---

## 7. `scenario` declaration

`scenario` is the low-level execution entrypoint. It wires a model to concrete
data and activates execution.

```
scenario <name> {
  use <model_name>
  data <name> source=<path>
  report <expression_name>
  report dual <constraint_name>
}
```

Every `scenario` MUST contain exactly one `use` declaration. When multiple
`scenario` declarations exist in a document, the execution order is
implementation-defined. Implementations MAY execute scenarios in parallel or
sequentially. Authors MUST NOT rely on declaration order or any implicit
execution ordering across scenarios. Each scenario is independent and MUST NOT
share mutable state with other scenarios.

```kdl
scenario distance_check {
  use distance_model
  data distances source="data/distances.csv"
}

scenario day_ahead {
  use dispatch_model
  data demand source="data/demand.csv"
  data gen_data source="data/generators.csv"
}
```

### 7.1 `use`

Required. References the model to solve.

```kdl
use dispatch_model
```

### 7.2 `data` (inside `scenario`)

Binds CSV data sources to model parameters. Each `data` declaration makes a
named parameter available to the model at solve time. Scenario-level `data`
declarations MUST NOT have a child block (`{ ... }`). They are simple
name-to-CSV bindings, not namespaced declarations like top-level `data` blocks.

```kdl
data demand source="data/demand.csv"
data capacity source="data/capacity.csv"
data fuel_cost source="data/fuel_cost.csv"
```

The `<name>` of each binding MUST match either a `param` declared in the
referenced model or a `param` declared in a top-level `data` block. Top-level
`data` block params are already resolved from their own `source=` path and do not
need scenario-level bindings, but a scenario MAY override them by providing a
binding with the same param name (see [§7.4](#74-data-scoping) for override
rules). Scenario `data` bindings that match neither a model param nor a
top-level data param MUST fail validation (see
[§10](#10-validation-requirements), rule 29). The CSV structure determines how
the parameter is indexed according to the following rules:

Column-to-index matching:

1. The model `param` declaration specifies which sets the parameter is indexed
   over (via the `index=` property or `index` children).
2. Each index set MUST correspond to a column in the bound CSV file. If an alias
   is used in `index=` or `index` children, it MUST first resolve to its
   canonical set name. Column matching then uses that canonical set name (after
   any `map` resolution in the source `data` block).
3. The value column is matched by the `param` name (or its `from` override).
4. Extra columns in the CSV that do not match any index set or the param name
   are ignored.
5. Missing required columns (index sets or value column) MUST fail validation.

Example: A model declares `param demand { index region; index time }`. The
scenario binds `data demand source="data/demand.csv"`. The CSV MUST contain
columns `region`, `time`, and `demand` (or the column specified by `from`). Each
row provides one value of `demand` for a `(region, time)` pair.

For scalar parameters (no index sets), the CSV MUST contain exactly one data row
with the value column. Multiple rows for a scalar parameter MUST fail
validation.

Every model `param` (that is not an inline scalar) MUST be resolved at scenario
time by either a scenario-level `data` binding or a top-level `data` block
`param`. An unresolved model parameter MUST fail validation (see
[§10](#10-validation-requirements), rule 63).

For data scoping and override rules between top-level and scenario-level `data`,
see [§7.4](#74-data-scoping).

### 7.3 `report` (inside `scenario`)

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

Filtered report narrows indexed output to a subset of rows:

```kdl
report soc {
  filter { storage_tech == "Li-Ion" }
}
```

The `filter` child block uses the same bare-math predicate syntax as `set` and
`param` filter blocks (see [§9](#9-filter-predicate-semantics)). It restricts
the reported output to rows matching the predicate. The predicate MUST
reference index variables from the reported expression or control. A `filter`
block is valid on both scalar and dual report forms. The `filter` block is
optional; when omitted, all rows are reported.

Semantics:

- Scalar report targets MUST resolve to a declared `expression`, `control`, or
  objective name.
- Dual report targets MUST resolve to a declared `constraint` name.
- Reports are evaluated after the solver returns a feasible solution. If the
  model is infeasible or unbounded, report evaluation is skipped and
  implementations MUST report the solver status.

Runtime scope: This specification defines the structure and validation of Arco
documents. Solver selection, solver options, time limits, and output file
formats are implementation-defined. Implementations MUST report at minimum the
solver status (optimal, infeasible, unbounded, time limit) after execution.

Expression report output structure:

- If the reported expression is a fully aggregated scalar (e.g., a `sum(...)`
  over all index sets), the output is a single value.
- If the reported expression has free variables (index variables that appear in
  indexed references but are not aggregated away by a `sum`, `avg`, `min`, or
  `max` reduction), the output is indexed by those free variables, producing one
  value per combination. The output format follows the same conventions as dual
  reports (see below).

Dual report output structure:

- For generated constraints (those with `index` clauses), the dual report
  produces one shadow price per generated row. The output is indexed by the same
  sets declared in the constraint's `index` clauses.
- For simple (non-generated) constraints, the dual report produces a single
  scalar value.
- The RECOMMENDED output format is CSV. For scalar reports, the CSV MUST contain
  a column named `value`. For indexed reports (dual or expression), the CSV MUST
  contain one column per `index` set (using the canonical set name) followed by
  a value column: `dual` for dual reports, or the expression name for scalar
  reports. Implementations MAY support alternative formats (JSON, etc.) but the
  column naming convention above MUST be preserved in any tabular output.

### 7.4 Data scoping

`data` can appear at two levels:

- Top-level `data` with children (`map`, `set`, `param`) declares a shared
  namespace. Sets and parameters declared inside are globally visible. Any model
  in the document can use them directly in algebra without redeclaration.
- Scenario-level `data` without children is a simple CSV-to-model-parameter
  binding scoped to that scenario only.

The parser distinguishes these by context: top-level `data` has a `{ ... }`
block, scenario-level `data` does not.

Scenario-level `data` bindings override parameter values only. Set declarations
from top-level `data` blocks are not overridable at scenario level. Sets are
resolved once from their declaring `data` block and remain fixed across all
scenarios.

If a scenario-level `data` binding resolves the same param name as a top-level
`data` block, the scenario-level binding takes precedence for that parameter
within that scenario. The override is by param name, not by data block name.
Scenario `data` bindings that do not match any model param or top-level data
param MUST fail validation (see [§10](#10-validation-requirements), rule 29),
which prevents typos from silently producing unbound data. Implementations
SHOULD emit a diagnostic when a scenario-level binding overrides a top-level
param, so users are aware of the override:

```kdl
// top-level: declares a param named "demand" inside block "demand_data"
data demand_data source="data/demand_base.csv" {
  set region
  param demand index=region
}

scenario stress_test {
  use dispatch_model
  // overrides the "demand" param (originally from demand_data) for this scenario
  data demand source="data/demand_stress.csv"
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
  (see [§10](#10-validation-requirements), rule 7). If two CSV files contain
  columns with the same logical name, use `map` to give them distinct names, or
  consolidate into one `data` block.

#### Global namespace design note

All set and param names share a single flat namespace by design. This simplifies
algebra expression resolution since every identifier resolves unambiguously
without requiring qualified names. For projects that compose models from
multiple teams or libraries, use naming conventions (e.g., prefixes like
`gen_capacity`, `line_capacity`) to avoid collisions. A formal namespacing or
module mechanism is not currently provided. This is a known limitation of the
current specification and is tracked for future consideration (see
`docs/reference/rfds/` for related design discussions).

If two CSV files have a column with the same logical name, use `from=` to give
them distinct param names, or consolidate into one `data` block:

```kdl
data generators source="data/generators.csv" {
  set gen_id
  // reads from CSV column "capacity", exposes as "gen_capacity" in algebra
  param gen_capacity from=capacity index=gen_id
}
data lines source="data/lines.csv" {
  set line_id
  // reads from CSV column "capacity", exposes as "line_capacity" in algebra
  param line_capacity from=capacity index=line_id
}
```

#### Multi-model data sharing example

```arco
// sets and params here are globally visible to all models
data units source="data/units.csv" {
  set plant_id
  set unit_id alias=u { in plant_id }
  param capacity_mw index=unit_id
}

// both models can use plant_id, unit_id, and capacity_mw directly
model dispatch_model {
  set time alias=t
  param demand { index time }
  control dispatch { lower 0; index unit_id; index time }
  constraint cap_limit {
    dispatch[u,t] <= capacity_mw[u]
  }
  constraint balance {
    index t { in time }
    expression {
      sum(dispatch[u,t] for u in unit_id) = demand[t]
    }
  }
  minimize cost {
    sum(dispatch[u,t] for u in unit_id for t in time)
  }
}

// no set or param declarations needed, plant_id and capacity_mw
// are globally visible from the data block above (see §6.1 in spec)
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
  use dispatch_model
  // only available in this scenario
  data demand source="data/demand_base.csv"
}

scenario high_demand {
  use dispatch_model
  // different demand for this scenario
  data demand source="data/demand_high.csv"
}
```

## 8. KDL 2.0 type annotations (optional)

Arco supports KDL 2.0 type annotations for users who want stronger metadata and
literal intent.

Node annotation:

```kdl
(f64)param capacity_mw { index plant_id; index unit_id }
```

Typed value literals in filters:

```arco
param large_units from=capacity_mw { filter { capacity_mw >= (f64)200 } }
param cc_capacity from=capacity_mw { filter { prime_mover == (prime_mover)CC } }
```

Typed metadata values:

```kdl
param fuel_cost units=(unit)"$/MMBtu"
```

Type annotations are optional unless project policy requires them. See
[§10](#10-validation-requirements), rules 21–22 for validation requirements on
type annotations.

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

Value resolution in predicates:

- The left-hand side of a comparison MUST resolve to a column name from the
  parent `data` block (after `map` resolution).
- The right-hand side resolves as follows: a numeric literal (e.g., `200`) is a
  number; a quoted string (e.g., `"thermal"`) is a string value; a bare
  identifier (e.g., `thermal`) is treated as a categorical string value matched
  against column contents. Bare identifiers on the RHS are never interpreted as
  column references, even if they happen to match a column name. To compare two
  columns is not supported in this version of the specification.

Rules:

- `>`, `>=`, `<`, `<=` require numeric column values. Using them on non-numeric
  data MUST fail validation.
- `==` and `!=` support both numeric and categorical values.
- `and` / `or` combine multiple conditions in a single filter block.
- The predicate references column names from the parent `data` block (after
  `map` resolution) on the left-hand side of each comparison.

```arco
data generators source="data/generators.csv" {
  set gen
  set thermal { in gen; filter { type == thermal } }
  set large { in gen; filter { capacity >= 200 } }
  set large_thermal { in gen; filter { type == thermal and capacity >= 200 } }
  set flexible { in gen; filter { type == hydro or type == battery } }
  param capacity index=gen
}
```

---

## 10. Validation requirements

Quick-reference index:

| #   | Category                  | Rule summary                                                                      |
| --- | ------------------------- | --------------------------------------------------------------------------------- |
| 1   | Name uniqueness           | Duplicate `data` block names                                                      |
| 2   | Name uniqueness           | Duplicate `model` names                                                           |
| 3   | Name uniqueness           | Duplicate `scenario` names                                                        |
| 4   | Name uniqueness           | Duplicate `map` targets within one `data` block                                   |
| 5   | Name uniqueness           | Duplicate `set` names within one `data` block                                     |
| 6   | Name uniqueness           | Set name collisions across `data` blocks                                          |
| 7   | Name uniqueness           | Param name collisions across `data` blocks                                        |
| 8   | Column resolution         | `map` without `from` must match CSV column                                        |
| 9   | Column resolution         | Unknown source columns in `map from=` or `param from=`                            |
| 10  | Column resolution         | Unknown set references in `index=` property or `index` children                   |
| 11  | Set hierarchy             | `in` parent must resolve (see [rule 32](#10-validation-requirements) for scoping) |
| 12  | Set hierarchy             | `in` cycles detected                                                              |
| 13  | Set hierarchy             | Child-to-parent hierarchy contradictions                                          |
| 14  | Indexing                  | `index=` property and `index` children mutually exclusive                         |
| 15  | Indexing                  | At most one `index` per `data` block                                              |
| 16  | Indexing                  | Non-unique indexing without `reduce`                                              |
| 17  | Indexing                  | `reduce` on scalar parameter                                                      |
| 18  | Filtering                 | Filter predicate references unknown columns                                       |
| 19  | Filtering                 | Numeric comparison on non-numeric column                                          |
| 20  | Filtering                 | Contradictory filter predicates (SHOULD)                                          |
| 21  | Type/metadata             | `units` value must be valid KDL string or identifier                              |
| 22  | Type/metadata             | Type annotation conflicts                                                         |
| 23  | Model structure           | Model must have exactly one objective                                             |
| 24  | Model structure           | Circular expression references                                                    |
| 25  | Model structure           | `control kind=` must be continuous/integer/binary                                 |
| 26  | Model structure           | Constraint index refs must resolve to known sets                                  |
| 27  | Scenario resolution       | `scenario` must have `use`                                                        |
| 28  | Scenario resolution       | `use <model>` must resolve to existing model                                      |
| 29  | Scenario resolution       | Scenario `data` binding must match model `param`                                  |
| 30  | Scenario resolution       | Scalar `report` must resolve to expression/control/objective                      |
| 31  | Scenario resolution       | Dual `report` must resolve to constraint                                          |
| 32  | Subset resolution         | `in` parent must be in same data block or top-level                               |
| 33  | Subset resolution         | Filtered subset must be subset of parent; warn if empty                           |
| 34  | Temporal safety           | Temporal offsets without boundary `if` guard                                      |
| 35  | Data integrity            | Empty CSV files                                                                   |
| 36  | Operator context          | `==` in constraint body (use `=`)                                                 |
| 37  | Operator context          | `=` in predicate context (use `==`)                                               |
| 38  | Nonlinear/solver          | Nonlinear built-ins trigger NLP/MINLP diagnostic (SHOULD)                         |
| 39  | Slack naming              | Auto-generated slack names must not collide with controls                         |
| 40  | Strict inequalities       | `<`/`>` MUST fail in range constraints; SHOULD warn in non-range                  |
| 41  | Bound algebra scoping     | Bound algebra vars must match control's own index names                           |
| 42  | Alias uniqueness          | Set aliases unique; no alias-name collisions                                      |
| 43  | Operator context          | `!=` in constraint body                                                           |
| 44  | Model/data set conflicts  | Model set must not shadow data/top-level set name                                 |
| 45  | Row filter scoping        | `if` predicate must reference at least one index variable                         |
| 46  | Inline selector           | Inline selector data ref must resolve to `data` block                             |
| 47  | Inline selector           | Inline selector fields must resolve to data columns                               |
| 48  | Ergonomic profile         | `use_data` must resolve to top-level `data` block                                 |
| 49  | Reserved                  | Former `bounds`-related slot; see rules 60 and 68                                 |
| 50  | Reserved                  | Reserved for historical numbering stability                                       |
| 51  | Top-level set members     | Duplicate members in top-level `set`                                              |
| 52  | Literal type restrictions | Boolean/string literals outside predicate contexts                                |
| 53  | Expression/objective      | Comparison operators in expression/objective bodies                               |
| 54  | Constraint structure      | Constraint body must contain at least one comparison operator                     |
| 55  | Operator context          | `and`/`or` outside predicate contexts                                             |
| 56  | Inline scalar             | Inline scalar must be numeric; no `index` prop/children/`from`/`reduce`           |
| 57  | Scenario structure        | Scenario-level `data` must not have child block                                   |
| 58  | Control structure         | `control` must have at least one `index` property or `index` child                |
| 59  | Top-level set structure   | Top-level `set` must have non-empty member list                                   |
| 60  | Control bounds            | Literal and formula bounds on same direction conflict                             |
| 61  | Null values               | KDL `null` in any value position MUST fail                                        |
| 62  | Binary bounds             | `control kind=binary` with bounds outside [0,1] MUST fail                         |
| 63  | Param resolution          | Unresolved model `param` at scenario time MUST fail                               |
| 64  | Param namespace           | Top-level `param` name collision with data-level `param`/`set`                    |
| 65  | Tuple arity               | Tuple binding arity must match `data` block `index` column count                  |
| 66  | Set column resolution     | `set <name>` inside `data` must resolve to CSV column                             |
| 67  | Empty-domain aggregation  | `min()`/`max()` over empty domain MUST produce solve-time error                   |
| 68  | Control bounds            | `value=` with `lower`, `upper`, or bound blocks MUST fail                         |
| 69  | Slack naming (range)      | `_slack_lo`/`_slack_hi` names must not collide with controls                      |
| 70  | Scalar CSV rows           | Scalar param bound to CSV with multiple rows MUST fail                            |
| 71  | Slack penalty             | `slack` penalty MUST be a positive numeric value                                  |
| 72  | `if` predicate form       | `if` body must be explicit boolean predicate (comparison required)                |
| 73  | Duplicate CSV columns     | CSV header with duplicate column names MUST fail                                  |
| 74  | Temporal offset ordering  | Temporal offset on unordered set MUST fail                                        |

Implementations MUST validate at least:

Name uniqueness (rules 1–7): Duplicate names for `data` blocks, `model`s,
`scenario`s, `map` targets, and `set`s within and across `data` blocks MUST fail
validation. `param` name collisions across `data` blocks MUST also fail.
Duplicate `param` names within a single `data` block or `model` block are also
prohibited (subsumed by the global uniqueness requirement of rules 7 and 64).

Column and field resolution (rules 8–10): `map` without `from` MUST resolve to
an existing CSV column. Unknown source columns and unknown set references in the
`index=` property or `index` children MUST fail validation.

Set hierarchy (rules 11–13): `in` parent MUST resolve to a set in the same
`data` block or a top-level `set` (see [rule 32](#10-validation-requirements)
for full scoping). `in` cycles MUST be detected, and child-to-parent hierarchy
contradictions (a child value mapping to multiple distinct parent values) MUST
fail validation.

Indexing (rules 14–16): The `index=` property and `index` children are mutually
exclusive. At most one `index` declaration per `data` block. Non-unique indexing
without `reduce` MUST fail. 17. `reduce` on a scalar parameter (one with no
`index`, no `index` children, and no block-level `index` declaration) MUST fail
validation. Aggregation requires at least one indexing dimension.

Filtering:

18. `filter` predicate references unknown column names.
19. Numeric comparison operator on non-numeric column data.
20. Implementations MUST detect contradictory filter predicates when a single
    variable has range bounds that form an empty interval (e.g.,
    `capacity >= 30 and capacity <= 20`). Specifically: if all predicates in a
    conjunction reference the same variable and the resulting interval is empty,
    validation MUST fail. Contradictions involving multiple variables or
    disjunctions (`or`) MAY be left undetected.

Type and metadata:

21. The `units` property value MUST be a syntactically valid KDL string or
    identifier. Values that are not valid KDL tokens (e.g., a raw number node
    where a string is expected) MUST fail validation.
22. Type annotation conflicts (example `(f64)param ...` on text column).

Model structure:

23. `model` MUST contain exactly one objective.
24. Circular `expression` references.
25. `control kind=<value>` MUST be one of `continuous`, `integer`, `binary`.
26. Constraint generation references (`index` / `index { in ... }`) MUST resolve
    to known sets (canonical names or aliases).

Scenario resolution:

27. `scenario` MUST contain exactly one `use` declaration.
28. `scenario use <model_name>` MUST resolve to an existing `model`.
29. Scenario `data` binding names MUST match model `param` declarations.
30. Scalar `report` targets MUST resolve to a declared `expression`, `control`,
    or objective.
31. Dual `report` targets MUST resolve to a declared `constraint`.

Subset resolution:

32. `in` parent set MUST be declared in the same `data` block or a top-level
    `set` declaration. Model-level sets MUST NOT be used as `in` parents (data
    is resolved before models).
33. Filtered subset members MUST be a subset of the parent set members. If a
    filter produces an empty set, implementations SHOULD emit a warning
    diagnostic.

Temporal safety:

34. Constraints using temporal offsets (`t-1`, `t+1`) without a boundary `if`
    guard MUST fail validation.

Data integrity:

35. Empty CSV files (no data rows) MUST produce a diagnostic.

Operator context:

36. `==` in a constraint body (where `=` is required) MUST fail validation.
37. `=` in an `if` predicate or reduction `if` filter (where `==` is required)
    MUST fail validation.

Nonlinear and solver compatibility:

38. Constraint or objective bodies containing nonlinear built-in functions
    (`sqrt`, `pow` with non-integer exponent, `exp`, `ln`) SHOULD produce a
    diagnostic indicating the problem class is NLP/MINLP.

Slack variable naming:

39. Auto-generated slack variable names (`<constraint>_slack`,
    `<constraint>_slack_pos`, `<constraint>_slack_neg`) MUST NOT collide with
    user-declared `control` names.

Strict inequalities:

40. Strict inequality operators (`<`, `>`) in range constraints (chained
    inequalities) MUST fail validation. In non-range constraint bodies, strict
    inequalities SHOULD produce a diagnostic warning, since LP/MIP solvers only
    support non-strict inequalities (`<=`, `>=`, `=`). Prefer `<=` or `>=` in
    all constraint algebra.

Bound algebra scoping:

41. Variable references inside `control` bound algebra blocks (`lower { ... }`,
    `upper { ... }`) MUST resolve to index names declared on the same `control`.

Alias uniqueness:

42. Set aliases MUST be unique across all set declarations. An alias MUST NOT
    collide with any declared set name.

Not-equal operators in constraint bodies:

43. The not-equal operator (`!=`) in constraint bodies MUST fail validation.
    This operator has no representation in LP/MIP solvers. It is valid only in
    predicate contexts (`if` blocks, `filter` blocks, reduction `if` clauses).

Model/data set name conflicts:

44. A `model` set declaration MUST NOT use the same name as a `set` already
    declared in a `data` block or at the top level. Model sets and data sets
    share a single global namespace (see [§6.1](#61-set-inside-model)).

Row filter scoping:

45. `if` predicates in generated constraints MUST reference at least one
    iteration variable declared by the constraint's `index` clauses. A predicate
    that does not depend on any loop variable is a static condition and MUST
    fail validation.

Inline selector resolution:

46. Inline selector data references (`data_name[field=value ...]`) MUST resolve
    to a declared `data` block name.
47. Each `field` in an inline selector MUST resolve to a column in the
    referenced `data` block (after `map` resolution).

Ergonomic profile resolution:

48. `use_data` references in a `model` block MUST resolve to a top-level `data`
    block name (see
    [Appendix A.2](appendix-a-ergonomic-profile.md#a2-use_data-model-imports)).

Top-level set members ([rule 51](#10-validation-requirements)): Duplicate
members in a top-level `set` MUST fail validation.

Literal type restrictions:

52. Boolean literals (`true`, `false`) and string literals in constraint bodies,
    expression bodies, or objective bodies outside of predicate contexts (`if`,
    `filter`, reduction `if`) MUST fail validation.

Expression and objective body restrictions:

53. Expression and objective bodies MUST NOT contain comparison operators (`<=`,
    `>=`, `<`, `>`, `=`, `==`, `!=`). Comparison operators are valid only in
    constraint bodies (for relational constraints) and predicate contexts (`if`,
    `filter`, reduction `if`). Note: Reduction `if` clauses inside expression
    and objective bodies are predicate contexts and MAY contain comparison
    operators.

Constraint structure:

54. Constraint bodies MUST contain at least one comparison operator (`<=`, `>=`,
    `=`). A constraint body consisting solely of an arithmetic expression with
    no relational operator MUST fail validation.

Logical operator context:

55. Logical operators (`and`, `or`) in constraint, expression, or objective
    bodies outside of predicate contexts (`if`, `filter`, reduction `if`) MUST
    fail validation.

Structural rules (rules 56–60): Inline scalars MUST be numeric with no `index=`
property, `index` children, `from`, or `reduce`
([rule 56](#10-validation-requirements)). Scenario-level `data` MUST NOT have a
child block ([rule 57](#10-validation-requirements)). `control` MUST have at
least one `index=` property or `index` child
([rule 58](#10-validation-requirements)). Top-level `set` MUST have a non-empty
member list ([rule 59](#10-validation-requirements)). Literal and formula bounds
for the same direction on a `control` MUST NOT both be specified
([rule 60](#10-validation-requirements)).

Null values:

61. KDL `null` in any value position (arguments, property values, algebra
    literals) MUST fail validation (see [§1.1](#11-kdl-compatibility)).

Binary bounds:

62. `control kind=binary` with explicit `lower` or `upper` bounds outside the
    [0, 1] range MUST fail validation (see [§6.3](#63-control)).

Fixed-value bounds:

68. `value=` on a `control` MUST NOT appear together with `lower`, `upper`,
    `lower { ... }`, or `upper { ... }`. Any combination MUST fail validation
    (see [§6.3](#63-control)).

Param resolution:

63. Every model `param` (that is not an inline scalar) MUST be resolved at
    scenario time by either a scenario-level `data` binding or a top-level
    `data` block `param`. An unresolved model parameter MUST fail validation
    (see [§7.2](#72-data-inside-scenario)).

Param namespace:

64. A top-level `param` name MUST NOT collide with any data-level `param` name
    or any `set` name. Names share a single flat namespace.

Tuple arity:

65. The number of variables in a tuple binding MUST match the number of columns
    in the referenced `data` block's `index` declaration. A mismatch MUST fail
    validation (see [§12.5](#125-reductions)).

Set column resolution:

66. `set <name>` inside a `data` block MUST resolve to a CSV column (after `map`
    resolution). If the name does not match any column, validation MUST fail.

Empty-domain aggregation:

67. `min(...)` and `max(...)` over an empty domain MUST produce a solve-time
    error (not a pre-solve validation error, since domain emptiness may depend
    on runtime data filtering). Implementations MUST report a clear diagnostic
    identifying the empty domain.

Slack naming for range constraints:

69. Auto-generated slack names `<constraint>_slack_lo` and
    `<constraint>_slack_hi` (from range constraints) MUST NOT collide with any
    user-declared `control` name. This extends
    [rule 39](#10-validation-requirements) to cover range-constraint slack
    patterns.

Scalar CSV rows:

70. A scalar parameter (no index sets) bound to a CSV with more than one data
    row MUST fail validation. The CSV MUST contain exactly one data row with the
    value column (see [§7.2](#72-data-inside-scenario)).

Slack penalty:

71. The `slack` `penalty` property MUST be a positive numeric value (greater
    than zero). Non-numeric, zero, negative, or missing penalty values MUST fail
    validation (see [§6.5](#65-constraint)).

Predicate form:

72. `if` block bodies MUST contain an explicit comparison operator (`==`, `!=`,
    `>`, `>=`, `<`, `<=`). Bare numeric or parameter references without a
    comparison operator MUST fail validation.

Duplicate CSV columns:

73. CSV files with duplicate column names in the header row MUST fail
    validation. Each column name MUST be unique within a single CSV file.

Temporal offset ordering:

74. Temporal offsets (`t-1`, `t+1`) applied to a set that is not an ordered set
    (see [§6.5](#65-constraint), temporal offsets section) MUST fail validation.

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
document          := { toplevel_set_decl | toplevel_param_decl
                     | data_decl | model_decl | scenario_decl }

toplevel_set_decl := "set" name [ "alias" "=" name ]
                     "{" { value } "}"

toplevel_param_decl := [ type_annot ] "param" name numeric_literal
                       [ "units" "=" value ]
                       (* inline scalar constant, no CSV backing *)

data_decl         := "data" name from_prop data_block
data_block        := "{" { map_decl | data_set_decl | index_decl
                     | data_param_decl } "}"

map_decl          := "map" name [ from_prop ]

data_set_decl     := "set" name [ "alias" "=" name ]
                     [ "{" in_child [ filter_block ] "}" ]
                     (* A child block on a data set requires 'in'.
                        'filter' without 'in' is not valid; see §5.2. *)
in_child      := "in" name

index_decl        := "index" name { name }

data_param_decl   := [ type_annot ] "param" name
                     [ from_prop ]
                     [ "units" "=" value ]
                     [ index_by_param | param_block ]
                     (* Indexing is optional. When omitted, the block-level
                        index declaration applies (§5.3), else 1-based row
                        order. Inline scalars are NOT valid inside data blocks;
                        use toplevel_param_decl or model_param_decl instead.
                        index (property form) and param_block (child form)
                        are mutually exclusive. *)

index_by_param    := "index" "=" name [ "reduce" "=" reducer ]
                     [ "{" filter_block "}" ]
                     (* When index is used, the optional filter block
                        appears as a child block on the param node:
                        param x index=s reduce=sum { filter { ... } } *)

param_block       := "{" { param_block_child ";" } [ filter_block ] "}"
                     (* At least one param_block_child or a filter_block
                        MUST be present (an empty block is invalid).
                        When param_block is used, the filter block appears
                        inside the same block (a KDL node has at most one
                        children block):
                        param x { index s; reduce sum; filter { ... } }
                        param x { filter { capacity >= 200 } } *)
param_block_child := "index" name | "reduce" reducer

model_decl        := "model" name model_block
model_block       := "{" { model_set_decl
                         | model_param_decl
                         | control_decl
                         | expression_decl
                         | constraint_decl
                         | objective_decl } "}"
                     (* use_data_decl is ergonomic syntax defined in
                        Appendix A.2, not part of the canonical grammar. *)

model_set_decl    := "set" name [ "alias" "=" name ]

model_param_decl  := [ type_annot ] "param" name
                     ( numeric_literal [ "units" "=" value ]
                                                    (* inline scalar *)
                     | [ "units" "=" value ]
                       [ "index" "=" name
                       | model_param_block ]
                     )
                     (* Model params are abstract; filter and reduce are
                        not valid. Use model_param_block, not param_block. *)
model_param_block := "{" { "index" name } "}"
                     (* Only index children; no filter or reduce. *)

control_decl      := [ type_annot ] "control" name
                     ( compact_control | block_control )

compact_control   := "index" "=" name
                     ( control_bounds | "value" "=" value )
                     [ "kind" "=" kind ]

block_control     := ( control_bounds | "value" "=" value )
                     [ "kind" "=" kind ]
                     "{" ctrl_index_child ";" { ctrl_index_child ";" }
                     [ bounds_block ] "}"
                     (* Literal bounds may be written either as properties
                        (lower=, upper=) on the control node line or as child
                        nodes (lower 0, upper 100) inside the control block.
                        Formula bounds (lower { ... }, upper { ... }) are
                        child nodes inside a bounds { ... } child block. For
                        each direction, at most one form (property OR literal
                        child OR bounds child) is allowed. value= is sugar for
                        lower=X upper=X and MUST NOT appear with lower, upper,
                        or bounds block. *)

control_bounds    := [ "lower" "=" value ] [ "upper" "=" value ]

bounds_block      := "bounds" "{" [ lower_block ] [ upper_block ] "}"
ctrl_index_child  := "index" name [ "{" "in" name "}" ]

lower_block       := "lower" "{" algebra_expr "}"
upper_block       := "upper" "{" algebra_expr "}"

expression_decl   := "expression" name "{" algebra_expr "}"

constraint_decl   := "constraint" name ( simple_body | generated_body )
simple_body       := "{" constraint_expr [ slack_decl ] "}"
generated_body    := "{" { index_child } { if_decl } [ slack_decl ]
                     expression_body "}"
index_child       := "index" name [ "{" "in" name "}" ]
if_decl           := "if" "{" algebra_expr "}"
slack_decl        := "slack" slack_props
slack_props       := "penalty" "=" value
                     [ "name" "=" name ]
                     [ "lower" "=" value ] [ "upper" "=" value ]
                     (* penalty, name, lower, upper are KDL properties
                        on the slack node: slack penalty=1000 name=my_slack *)
expression_body   := "expression" "{" constraint_expr "}"

                     (* constraint_expr uses the shared comp_op production,
                        which is intentionally permissive. Validation rules
                        restrict operators by context. See §10, rules 36,
                        40, and 43 for constraint-specific restrictions. *)
constraint_expr   := algebra_expr
                   | algebra_expr comp_op algebra_expr
                   | algebra_expr comp_op algebra_expr comp_op algebra_expr

objective_decl    := ( "minimize" | "maximize" ) name "{" algebra_expr "}"

scenario_decl     := "scenario" name scenario_block
scenario_block    := "{" { scenario_child } "}"
                     (* Children may appear in any order. Exactly one use_decl
                        is required. *)
scenario_child    := use_decl | scenario_data_decl | report_decl
use_decl          := "use" name
scenario_data_decl:= "data" name from_prop
report_decl       := "report" ( name | "dual" name )
                     [ "{" filter_block "}" ]

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
                        - constraint bodies: "==" MUST fail (use "="); see §10 rule 36
                        - predicate contexts (if, filter): "=" MUST fail (use "=="); see §10 rule 37
                        - constraint bodies: "!=" MUST fail; see §10 rule 43
                        - range constraints: "<" and ">" MUST fail; see §10 rule 40
                        - non-range constraint bodies: "<" and ">" SHOULD warn; see §10 rule 40 *)

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
                     (* All entries in a single index_list MUST be of the
                        same form: either all positional (variable indexing)
                        or all key=value (inline selectors). Mixing
                        positional and selector entries in a single bracket
                        MUST fail validation. See §5.5. *)
index_entry       := name [ ( "+" | "-" ) integer ]
                   | name "=" value
atom              := numeric_literal | string_literal | bool_literal
                   | name
                   | "(" algebra_expr ")"
                   | reduction
                   | function_call
reduction         := reducer "(" algebra_expr
                     { "for" binding "in" iteration_domain }
                     { "if" algebra_expr } ")"
iteration_domain  := name [ inline_selector ]
binding           := name | "(" name { "," name } ")"
function_call     := builtin_fn "(" algebra_expr { "," algebra_expr } ")"
builtin_fn        := "sqrt" | "pow" | "exp" | "ln" | "abs"
```

> [!NOTE]
>
> - `name`, `field_name`, and `path` follow KDL string rules (identifier or
>   quoted).
> - In productions where a set is referenced (`index`, `index`,
>   `index ... { in ... }`), a `name` token MAY be either the canonical set name
>   or a set alias. Implementations MUST normalize aliases to canonical set
>   names before validation/lowering.
> - `kdl_value` MAY be annotated (example `(f64)200`, `(unit)"$/MWh"`).
> - Single-dimension indexing uses the `index=<set>` property form.
>   Multi-dimension indexing uses child nodes:
>   `{ index <set_a>; index <set_b> }`. Using both on the same declaration is a
>   validation error.

- `reduce` has two equivalent forms: as a property on `param`
  (`reduce=<reducer>`) and as a child node inside an index block
  (`reduce <reducer>`, no `=`). Both produce the same semantics.
- `model_block` MUST contain exactly one `objective_decl`.
- `scenario_block` MUST contain exactly one `use_decl`.
- `inline_selector` is Arco-specific syntax valid only inside algebra expression
  strings. It is distinguished from variable indexing by the presence of `=`
  inside brackets. For named filtered domains, use
  `set <name> { in <parent>; filter { ... } }` inside `data`.

---

## 12. Algebra expression summary

Algebra expressions appear inside [`constraint`](#65-constraint),
[`expression`](#64-expression), [`minimize` / `maximize`](#66-objective),
`lower`, `upper`, `if`, and `filter` bodies. They are parsed as opaque strings
by the KDL layer and interpreted by the Arco algebra parser.

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

Literal type restrictions: Boolean literals (`true`, `false`) and string
literals are valid only in predicate contexts: `if` blocks, `filter` blocks, and
reduction `if` clauses (e.g., `filter { type == "thermal" }`,
`if { active[g] == true }`). Using boolean or string literals in arithmetic
expressions, constraint bodies, or objective bodies outside of predicate
contexts MUST fail validation (see [§10](#10-validation-requirements), rule 52).
Numeric literals are valid in all algebra contexts.

### 12.2 Operators and precedence

Arithmetic operators:

| Operator | Description                       | Precedence |
| -------- | --------------------------------- | ---------- |
| `*`, `/` | multiplication / division         | highest    |
| `+`, `-` | addition / subtraction / negation | middle     |

Logical operators (predicate contexts only):

| Operator | Description         | Precedence |
| -------- | ------------------- | ---------- |
| `and`    | logical conjunction | low        |
| `or`     | logical disjunction | lowest     |

Standard arithmetic precedence applies: `*` and `/` bind tighter than `+` and
`-`. In predicate contexts, `and` binds tighter than `or`, and both bind looser
than comparison operators. Parentheses MAY be used to override precedence.
Logical operators (`and`, `or`) are valid only in predicate contexts (`if`,
`filter`, reduction `if`); see [§10](#10-validation-requirements), rule 55.

### 12.3 Comparison operators

| Operator | Description                 |
| -------- | --------------------------- |
| `<=`     | less than or equal          |
| `>=`     | greater than or equal       |
| `<`      | strict less than            |
| `>`      | strict greater than         |
| `=`      | equality (in constraints)   |
| `==`     | equality (in predicates)    |
| `!=`     | not equal (predicates only) |

`!=` is valid only in predicate contexts (`if` blocks, `filter` blocks,
reduction `if` clauses). Using it in constraint bodies MUST fail validation (see
[§10](#10-validation-requirements), rule 43).

`=` and `==` serve distinct roles and MUST NOT be interchanged:

- In constraint bodies, `=` denotes an equality constraint (a linear relation
  the solver enforces). Using `==` in a constraint body MUST fail validation.
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
set). Offsets are restricted to literal integers (e.g., `t-1`, `t+2`); variable
or parameter-dependent offsets (e.g., `t-lag[g]`) are not supported. Constraints
using temporal offsets MUST include an `if` guard to exclude boundary steps
where the offset would be out-of-range (see [§6.5](#65-constraint)).

### 12.5 Reductions

| Form                                   | Description                                                                              |
| -------------------------------------- | ---------------------------------------------------------------------------------------- |
| `sum(expr for v in set)`               | summation over one set                                                                   |
| `sum(expr for v in set for w in set2)` | nested summation                                                                         |
| `sum(expr for v in set if cond)`       | filtered summation                                                                       |
| `sum(expr for v in set if c1 if c2)`   | multiple filters (AND; use `==` not `=`, see [§10 rule 37](#10-validation-requirements)) |
| `sum(expr for (i, j) in arc_set)`      | tuple binding                                                                            |

Reductions iterate over sets declared in [`data`](#5-data-declaration) blocks or
[`model`](#6-model-declaration) blocks. Data-level sets (including
hierarchy-derived subsets) can be used directly inside algebra for aggregation.

Empty set iteration: `sum(...)` over an empty domain evaluates to `0`.
`min(...)` and `max(...)` over an empty domain MUST produce a solve-time error
(not a pre-solve validation error, since domain emptiness may depend on runtime
data filtering). Implementations MUST report a clear diagnostic identifying the
empty domain (see [§10](#10-validation-requirements), rule 67).

Tuple bindings:

In all reductions shown above, the iteration domain after `in` is a set name.
Tuple bindings are the one exception: they iterate over a `data` block name (not
a set name) to destructure composite keys. When a domain contains composite keys
(e.g., arcs defined by origin-destination pairs), tuple destructuring binds
multiple variables simultaneously:

```
sum(flow[i,j] for (i, j) in branch_data)
```

In tuple bindings, the iteration domain (`branch_data` above) is a `data` block
name — not a set name. This is the one context where a `data` block name MAY
appear as an iteration domain. The iteration domain MUST reference a `data`
block that has a multi-column `index` declaration (see
[§5.3](#53-index-inside-data)). Single-variable bindings (`for v in X`) require
a set name; `data` block names are not valid in that context.

Declaring a tuple-keyed domain: The corresponding `data` block MUST declare sets
for the component domains and use a multi-column `index` to define the composite
key:

```kdl
data branch_data source="data/branches.csv" {
  // CSV has columns: from_bus, to_bus, capacity, ...
  set from_bus
  set to_bus
  index from_bus to_bus
  param capacity
}
```

The tuple binding `for (i, j) in branch_data` iterates over the unique
`(from_bus, to_bus)` pairs found in the CSV. Each binding variable maps
positionally to the index columns in declaration order (`i` → `from_bus`, `j` →
`to_bus`). The number of variables in a tuple binding MUST match the number of
columns in the referenced `data` block's `index` declaration. A mismatch MUST
fail validation (see [§10](#10-validation-requirements), rule 65).

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
`abs(x)` is classified as piecewise-linear rather than nonlinear and does not
trigger the NLP/MINLP diagnostic. Implementations MAY linearize it using
auxiliary variables and constraints.

Numeric edge cases: Division by zero in algebra expressions MUST produce a
solve-time error. The values `NaN`, `Inf`, and `-Inf` are not valid numeric
literals in Arco; if a CSV contains such values or a computation produces them
at solve time, implementations MUST report a diagnostic error. All numeric
values in Arco are IEEE 754 double-precision floating-point unless a type
annotation specifies otherwise.

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

See [§6.5](#65-constraint) for constraint body forms (comparison and range).

---

This document is the canonical reference for Arco KDL syntax.
[§1](#1-conformance)–[§12](#12-algebra-expression-summary) define the canonical
low-level profile. [Appendix A](appendix-a-ergonomic-profile.md) defines the
supported ergonomic authoring profile that lowers into it.
