# Architecture Refactor Plan

This document describes Arco's target-state architecture. It is a planning
contract, not a description of the current transitional repository.

Arco's north star is to provide the ultimate building blocks for optimization
modeling: **primitives, not prescribed workflows**. Users should be able to build
modeling libraries, data layers, solver integrations, and interaction surfaces on
top of Arco without core-developer intervention.

## Design rules

1. **`arco-model` is the primitive crate.** It owns finite optimization model
   primitives, indexed in-memory data primitives, expression primitives, and
   stable primitive document schemas.
2. **Primitives are concrete and finite.** `arco-model` stores concrete
   optimization instances and concrete indexed data. Templates, scenarios,
   parameter binding, and multi-objective workflows live above it.
3. **Primitives do not dictate workflow.** `arco-model` exposes mechanisms:
   variables, expressions, constraints, objective, sets, tuple sets, parameter
   tables, attribute tables, documents, and read-only views.
4. **Interaction surfaces go through `arco-ops`.** `arco-ops` is the stability
   adapter between primitive Rust APIs and CLI/Python/Julia/user-facing bindings.
5. **Authoring surfaces are replaceable.** KDL, JSON, YAML, and future frontends
   parse their own syntax, resolve language-specific semantics, and build
   primitives. They do not own solving, format/export, or runtime policy.
6. **The frozen model is solve-ready.** Direct solve/export paths consume
   `ModelView` or patched model views. There is no mandatory model → IR → target
   handoff that duplicates the problem before solver-native loading.
7. **Transformations are optional and demand-driven.** Row-major buffers,
   solver-specific tapes, LP/MPS/NL render trees, and other alternate layouts are
   built only when an adapter/exporter genuinely needs them.
8. **Solver families plug into `arco-solver` primitives.** Solver adapters are
   siblings and depend on model views, solver-facing contracts, and runtime
   services, not on authoring surfaces or duplicate IR/target crates.
9. **Memory behavior is architectural.** Hot numeric storage must be compact,
   cache-friendly, sidecar-free, and generic over `f32`/`f64` where useful.
10. **No compatibility-driven architecture.** During this refactor, prefer clean
    seams over legacy pass-throughs or public API duplication.

## Target crate map

### Primitive model crate

- `arco-model` — the primitive optimization modeling crate.

  It owns:
  - finite model primitives: variables, bounds, variable kinds, expressions,
    bounded constraints, a single objective, names/metadata sidecars, structural
    facts, fingerprints, patches, and read-only model views
  - expression primitives: detached `Expr<S>` values, LP/QP fast paths, local
    promotion to symbolic expressions, built-in nonlinear operators, and opaque
    namespaced custom operators with declared arity
  - indexed data primitives under `arco_model::indexed`: ordered sets, tuple
    sets, domains, index keys, numeric parameter tables, attribute tables,
    projection/filter primitives, and `IndexedData`
  - stable primitive documents: `ModelDocument`, `IndexedDataDocument`, and a
    combined `ArcoDocument`

  It does **not** own:
  - KDL/JSON/YAML syntax
  - reusable parametric templates or late-bound parameters
  - scenario orchestration
  - multi-objective workflows
  - solver selection, solving, results, logs, or artifacts
  - dataframe/CSV/Parquet/database ingestion
  - source-specific naming policies such as `x[north,solar]`

- `arco-expr` — retired in the target architecture. Expression and ID primitives
  move into `arco-model`.
- `arco-algebra` — retired in the target architecture. It is currently only a
  migration seam over `arco-expr` and should be absorbed into `arco-model`.

### Diagnostics

- `arco-diagnostics` — planned foundational diagnostics/provenance crate used by
  model, authoring, validation, and compilation layers.

  It owns shared diagnostic codes, severity, source IDs/spans, and coarse
  provenance types. It must not depend on authoring formats.

### Authoring surfaces

- `arco-kdl` — KDL authoring DSL. It keeps KDL parser/AST/semantic machinery for
  syntax, scoping, aliases, source spans, KDL-specific diagnostics, and authoring
  conveniences, then builds `arco-model` primitives.
- `arco-json` — planned JSON authoring surface.
- `arco-yaml` — planned YAML authoring surface.

Authoring surfaces may build `Model`, `IndexedData`, and primitive documents.
They must not invoke solvers, own format/export behavior, or require an
intermediate IR/target handoff.

### Validation

- `arco-validate` — user-facing validation/reporting over `arco-model` views.

`arco-model` validates structural invariants while building or finishing a
model. `arco-validate` provides friendly reports, policy checks, semantic
warnings, and capability-requirement extraction. It is a user-facing validation
layer over model views, not a required step in the direct solve path.

### Optional transformations and retired handoff crates

The target architecture has no mandatory `arco-model -> arco-compile -> arco-ir
-> arco-targets` handoff. The frozen `arco-model::Model` is the canonical
solve-ready representation, and consumers read it through `ModelView` or patched
model views.

- `arco-compile` — retired as a mandatory bridge in the target architecture.
  Reusable transformation or analysis helpers may be introduced later only when
  duplication across adapters/exporters proves they are needed.
- `arco-targets` — retired in the target architecture. Solver adapters consume
  model views plus `arco-solver` contracts directly, allocating target-specific
  buffers only when necessary.
- `arco-ir` — retired in the target architecture. The name is avoided because it
  obscures the distinction between primitive model storage, optional
  transformations, and solver-native representations.

### Format primitives and concrete formats

- `arco-format` — format-side primitives and contracts: export/import
  requests, common format errors, format capability declarations, numeric
  rendering policy, naming/escaping hooks, model-view traversal helpers, and
  format result/report DTOs. Current transitional crate: `arco-exchange`.
- `arco-export` — legacy export crate to collapse into concrete format crates or
  retire.
- planned format crates such as `arco-lp`, `arco-mps`, and `arco-nl` implement
  concrete downstream formats on top of `arco-format` and `arco-model` views.

`arco-format` is foundational because every concrete export/import format needs
one shared vocabulary. It must stay format-neutral unless concrete formats are
behind optional features that do not pull format dependencies into the primitive
contract.

Canonical model serialization belongs to `arco-model` documents, not to
`arco-format`. LP/MPS/NL are format/export views and may be lossy or
subset-specific.

### Solver primitives, runtime, and adapters

- `arco-solver` — solver-side primitives and contracts: solve requests, results,
  statuses, capabilities, option/profile/selection types, solver traits, generic
  registry/preflight types, and model-view compatibility requirements.
- `arco-runtime` — execution mechanics and runtime services used during solves.
- `arco-highs`, `arco-ipopt`, `arco-xpress`, `arco-scip` — concrete solver-family
  adapters that translate `ModelView` data into solver-native objects.

`arco-solver` is foundational because solver adapters, `arco-ops`, and result
views need one shared vocabulary. It must stay adapter-neutral: no concrete
solver family depends back into it through registrations or built-in wiring.

Solver results live outside `arco-model`. They should carry model fingerprints
and values keyed by stable model IDs. Joined ergonomic result views belong in
`arco-solver` or `arco-ops`, not in the primitive model.

### Stability adapter and composition layers

- `arco-ops` — stability adapter for interaction surfaces. It exposes stable
  wrapper/DTO types over primitive model, indexed data, document, validation,
  format/export, and solve concepts. It should not primarily re-export
  primitive crates.
- `arco-blocks` — high-level run-container composition layer over `arco-ops`.
  It models multiple optimization containers, typed ports, feedforward links,
  block DAG execution, block runs, diagnostics, and extracted outputs.

`arco-blocks` is not a primitive model kernel. It is a composition layer. Its
core target should be Rust/language-neutral and should not require PyO3; Python
schema/callback ergonomics belong in `arco-ops` adapters or Python bindings.

### Interaction surfaces

- `arco-cli` — command-line shell over `arco-ops`.
- `arco-python` / `bindings/python` — Python shell over `arco-ops`.
- `arco-julia` — planned Julia shell over `arco-ops`.

Interaction surfaces own language ergonomics, I/O, process behavior, and error
presentation. They should not depend directly on primitives, retired handoff
crates, solvers, format/export layers, runtime, or adapters.

## Dependency diagram

```text
Interaction surfaces
  arco-cli / arco-python / arco-julia
        │
        ▼
arco-ops
  stable primitive/document/validate/export/solve adapters
        │
        ├──────────────► arco-blocks? (composition exposed through ops adapters)
        │
        ├──────────────► arco-kdl / future authoring surfaces
        │                         │
        │                         ▼
        ├──────────────► arco-model
        │                 finite model + expressions + indexed data + documents
        │                         ▲
        │                         │
        ├──────────────► arco-validate
        │
        ├──────────────► arco-format ───────► LP / MPS / NL / other exports
        │                         ▲
        │                         │
        ├──────────────► arco-solver ───────► solver adapters ───► native solvers
        │                         ▲                  ▲
        │                         │                  │
        └──────────────► arco-runtime
```

Foundational diagnostics:

```text
arco-diagnostics -> used by arco-model, authoring, validation, formats, solvers
```

## `arco-model` primitive design

### Finite model kernel

The finite kernel represents concrete optimization instances:

- scalar variables with explicit `Continuous`, `Integer`, or `Binary` kind
- concrete `Bounds<S>` with `±infinity`, not `Option<S>`
- detached expressions `Expr<S>`
- bounded constraints: `lower <= expr <= upper`
- one active objective with sense and expression
- optional names, provenance, and user metadata sidecars
- model fingerprints and structural facts
- stable lossless serialization documents

It does not represent late-bound named parameters, templates, scenario sweeps, or
multi-objective workflows.

### Construction and immutability

Construction uses mutable builders. Durable artifacts are frozen:

```rust
ModelBuilder<S> -> Model<S>
IndexedDataBuilder<S> -> IndexedData<S>
```

`ModelBuilder::finish()` compacts and normalizes storage. `Model<S>` is frozen,
shareable, cacheable, and consumed through read-only `ModelView` APIs.

Efficient updates use value-only patches:

```rust
Model<S> + ModelPatch<S> -> PatchedModelView<S>
```

`ModelPatch` can update values such as bounds, coefficients, objective data, and
sidecar metadata. It cannot add/remove variables, constraints, expression nodes,
or custom operators. Downstream export/solve APIs should consume views instead
of materializing a full patched model.

### Numeric scalar strategy

`arco-model` is scalar-generic over supported floating types:

```rust
Model<S>
Model64 = Model<f64>
Model32 = Model<f32>
Model = Model64
```

The default user-facing type is `f64`. `f32` is first-class for memory-sensitive
or solver-specific paths. Downcasting from `f64` to `f32` must be explicit.

Public IDs remain compact `u32` wrappers:

```rust
VariableId(u32)
ConstraintId(u32)
ExpressionId(u32)
```

### Expression representation

Public expressions are detached values so third-party libraries can build and
return expressions without owning a model reference. Internally, the frozen model
may ingest expressions into compact storage.

`Expr<S>` should optimize common cases:

```text
constant -> variable -> affine -> quadratic -> symbolic
```

Promotion is local. A model with one symbolic nonlinear constraint must not force
all LP/QP data into symbolic storage.

Built-in nonlinear operators are compact enum variants. Custom operators are
opaque namespaced atoms with declared arity:

```rust
CustomOperatorId { namespace, name }
CustomOperatorDecl { id, arity, metadata }
```

`arco-model` stores custom operators but does not evaluate, differentiate,
simplify, lower, or solve them.

### Hot storage layout

Frozen hot storage should favor structure-of-arrays and contiguous buffers:

```rust
LinearMatrix<S> {
    col_offsets: Vec<u32>,
    row_indices: Vec<u32>,
    values: Vec<S>,
}
```

Avoid hot-path layouts such as `Vec<Vec<_>>` or padded `(u32, f64)` tuples when a
compact columnar layout is available. Sidecars for names, provenance, and
metadata must be lazy and separate from numeric storage.

Streaming/chunked construction should support append-only/order-declared column
input so huge models do not need a full temporary matrix.

### Structural facts, classification, and validation

`arco-model` may expose cheap structural facts:

- has integer variables
- has quadratic terms
- has symbolic expressions
- has custom operators
- max expression degree where cheap

Final LP/QP/NLP/MILP/MIQP/MINLP classification and solver compatibility checks
belong in validation, compilation, and solver capability layers.

### Serialization documents

Primitive serialization is a stable ecosystem contract. It is not solver/export
serialization.

`arco-model` owns stable DTOs:

- `ModelDocument`
- `IndexedDataDocument`
- combined `ArcoDocument`

Documents have one shared primitive `schema_version` and a `document_kind`.
Scalar precision is preserved by a document-level `scalar_type`, and scalar
values serialize as canonical strings to preserve infinities and roundtrip
intent.

Example shape:

```json
{
  "schema_version": 1,
  "document_kind": "model",
  "scalar_type": "f64"
}
```

The DTOs are stable. Internal storage is private and may change.

## Indexed data primitives

`arco_model::indexed` is part of the primitive crate, not a separate crate in the
target architecture. It provides in-memory data/index primitives used by KDL,
Python, Rust libraries, and third-party modeling layers.

It owns:

- ordered unique sets
- tuple sets
- domains / index keys
- numeric `ParameterTable<S>` with dense and sparse storage
- non-numeric `AttributeTable`
- shared value/string pool inside `IndexedData`
- lazy projection views and optional materialization
- low-level Rust predicate filters
- basic numeric duplicate reducers for table construction: sum, min, max, count,
  mean
- stable lossless `IndexedDataDocument`

It does not own:

- variable families
- constraint families
- indexed objectives
- templates
- scenarios
- joins/group-by/dataframe pipelines
- file/database ingestion
- row/cell-level provenance
- naming/rendering policies

Index values support string, integer, canonical decimal, and boolean values in
v1. Dates, timestamps, UUIDs, nulls, and floats are not primitive index values.
Dates/timestamps may be represented by strings or integers in outer layers.

`Set` and tuple rows enforce uniqueness by default. Numeric `ParameterTable`
construction may explicitly aggregate duplicate keys with a reducer. Missing
values are explicit; the default missing policy is error.

## `arco-ops` stability adapter

`arco-ops` exists to decouple primitive Rust APIs from interaction surfaces. It
should expose stable wrappers/DTOs rather than making raw primitive re-exports the
main public contract.

Target v1 focus:

- primitive model adapters
- indexed data adapters
- document load/save adapters
- validation adapters
- format/export adapters
- solve adapters
- stable errors/reports/results

Avoid opinionated workflow bundles in v1, such as `load_validate_solve`, scenario
sweeps, or multi-objective orchestration. Those can be added later or live in
workflow crates.

## `arco-blocks` composition layer

`arco-blocks` is Arco's high-level block/run-container layer, inspired by block
composition in algebraic modeling systems. It lets users compose multiple
optimization containers and feed outputs forward between them.

A block run may:

- receive typed inputs
- build or patch a model
- validate/format-export/solve through `arco-ops`
- extract outputs from data, model, or solution
- feed outputs to downstream blocks
- record diagnostics

`arco-blocks` should depend on `arco-ops`, not directly on primitive or solver
crates. KDL block graph authoring is deferred; KDL may eventually build block
graphs, but the first target is model/data authoring.

## Migration map

- Absorb `arco-expr` into `arco-model`.
- Absorb `arco-algebra` into `arco-model` and retire the crate.
- Move finite model ownership fully into `arco-model` and remove old compatibility
  structures that duplicate the primitive API.
- Add `arco_model::indexed` inside `arco-model` for in-memory indexed data
  primitives.
- Add stable primitive document DTOs in `arco-model`.
- Keep `arco-kdl` as a KDL parser/semantic layer that builds primitives.
- Keep `arco-validate` as a user-facing validation/reporting layer over model
  views.
- Retire `arco-compile`, `arco-ir`, and `arco-targets` as mandatory handoff
  crates; introduce shared transformation helpers later only if measured
  duplication justifies them.
- Absorb `arco-contracts` into `arco-solver` when practical.
- Collapse or retire `arco-export`; rename `arco-exchange` to `arco-format` for the target format primitive crate.
- Rebuild `arco-ops` as the stable adapter over primitives and solver workflows.
- Refactor `arco-blocks` into a language-neutral run-container composition layer
  over `arco-ops`; move Python/PyO3-specific ergonomics to adapters/bindings.
- Rewrite CLI/Python to depend on `arco-ops` only among Arco crates.

## Refactor phases

### Phase 0: encode the target architecture

Update `.sentrux/rules.toml` to enforce this target, not the current transition.
Expected migration-debt violations include direct interaction-surface access to
primitives, KDL-to-retired-handoff coupling, dependencies on retired handoff
crates, `arco-ops` raw re-exports, and remaining `arco-expr`/`arco-algebra`
dependencies.

### Phase 1: redesign `arco-model`

Implement the primitive finite model and indexed data design:

- `ModelBuilder<S> -> Model<S>`
- detached `Expr<S>` with LP/QP fast paths and local symbolic promotion
- frozen model views and value-only patches
- compact CSC/SoA storage
- `Model32` / `Model64` aliases
- `arco_model::indexed` primitives
- primitive documents and fingerprints

### Phase 2: absorb expression/algebra crates

Move expression IDs, expression builders, and operators into `arco-model`.
Remove dependencies on `arco-expr` and `arco-algebra`, then retire those crates.

### Phase 3: authoring builds primitives

Refactor `arco-kdl` so its parser/semantic layer builds `Model`, `IndexedData`,
or primitive documents. It must not produce retired target/IR representations or
call solve paths.

### Phase 4: retire mandatory handoff crates

Remove the mandatory `arco-compile`, `arco-ir`, and `arco-targets` bridge from
solve/export paths. Move any still-useful tests to model-view, format, solver,
or optional transformation-helper coverage.

### Phase 5: model-view solver and format consumers

Refactor solver primitives, solver adapters, and format/export crates to consume
`ModelView` / patched model views directly. They may allocate target-specific
buffers only when a solver/export format genuinely requires another layout.

### Phase 6: rebuild `arco-ops`

Expose stable wrappers and DTOs over primitive/document/validation/export/solve
capabilities. Remove primary raw re-exports of primitive/internal crates.

### Phase 7: rebuild `arco-blocks`

Make block composition depend on `arco-ops` and move Python-specific schema and
callback mechanics out of the core block layer.

### Phase 8: rewrite interaction surfaces

Rewrite CLI and Python bindings to use `arco-ops` only among Arco crates. They
own user I/O and language ergonomics, not architecture policy.

### Phase 9: delete legacy structure

Remove retired crates, compatibility modules, duplicated result/error/status
contracts, direct adapter solve APIs over models, and stale tests/examples.

### Phase 10: document and verify

Update user and contributor documentation to describe the actual final state.
Run the relevant checks for changed areas, then full workspace checks before
shipping the refactor.

## Decision checklist for future contributors

Before adding a crate, API, or dependency, answer:

1. Is this a primitive, an adapter, an optional transformation, a solver concern,
   or a workflow?
2. Does it belong in `arco-model`, `arco-ops`, `arco-blocks`, an authoring
   surface, format layer, solver primitives, or a solver adapter?
3. Does this force a workflow where a primitive mechanism would be enough?
4. Does this add strings, metadata, provenance, or dynamic dispatch to a hot
   numeric path?
5. Can this use a stable DTO/view instead of depending on an internal
   representation?
6. Would a third-party crate be able to build the same thing without core changes?
7. Does this preserve interaction-surface stability by routing through
   `arco-ops`?
