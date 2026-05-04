# Architecture Refactor Plan

This document describes the target-state crate architecture for Arco. It is a
planning document, not a description of the repository as it exists today.

The goal is to make Arco easier to extend without blurring responsibilities:

- authoring surfaces build the canonical model
- compilation is the only semantic bridge out of the canonical model
- portable exchange stays separate from direct solver execution
- solver families plug into one common seam
- interaction surfaces stay thin
- memory-sensitive implementation details stay localized

## Design rules

The target architecture follows these rules:

1. **Authoring surfaces are replaceable.** KDL, JSON, YAML, and future frontends
   build the canonical model but do not own compilation, exchange, runtime, or
   solver policy.
2. **The canonical model is the semantic center.** Domain meaning lives there,
   not in parsers, bindings, or solver adapters.
3. **Compilation is the only semantic bridge out.** Anything downstream of the
   canonical model consumes compiled targets or portable IR, not authoring
   internals.
4. **Exchange consumes portable IR only.** Import/export formats should not pull
   on canonical-model or solver-specific code.
5. **Solver adapters are siblings.** One solver family must never depend on
   another.
6. **The solver platform depends on interfaces, not concrete adapters.**
7. **Interaction surfaces are shells.** CLI and Python should orchestrate user
   workflows through a small app-facing seam rather than reaching into internals.

## Target crate map

The target crate inventory is organized around seams and responsibilities.

### Support tooling

- `arco-tools` — developer tooling, diagnostics, profiling helpers, and other
  support utilities. Not part of the main solve pipeline.

### Canonical model

- `arco-model` — canonical domain model and stable semantic types.
- `arco-algebra` — symbolic algebra and expression-level operators used by the
  canonical model.
- `arco-blocks` — reusable model-building blocks layered on top of the
  canonical model.
- `arco-validate` — canonical-model validation and invariant checking.

### Compilation

- `arco-compile` — lowering and compile orchestration from canonical model to
  downstream artifacts.
- `arco-targets` — the solver-facing compile output seam. Defines the lowered
  targets consumed by runtime and solver families.
- `arco-ir` — portable IR for interchange, inspection, and external exchange.

### Exchange

- `arco-exchange` — import/export logic for portable IR and exchange formats.

### Shared interfaces

- `arco-contracts` — shared solve and solver-platform contracts: selections,
  requests, results, capabilities, registration, lifecycle, and invocation
  seams. This is the main contract crate for the solver-facing side of the
  architecture.

### Runtime and orchestration

- `arco-runtime` — execution mechanics, resource handling, and runtime services
  used during solves.
- `arco-solver` — solver registry, solver preflight, selection, and solve
  orchestration.
- `arco-ops` — small operations facade used by CLI and language bindings for
  load, validate, compile, exchange, inspect, and solve flows.

### Solver adapters

- `arco-highs` — HiGHS solver-family adapter.
- `arco-ipopt` — IPOPT solver-family adapter.
- `arco-xpress` — Xpress solver-family adapter.
- `arco-scip` — SCIP solver-family adapter.

### Authoring surfaces

- `arco-kdl` — KDL authoring surface.
- `arco-json` — JSON authoring surface.
- `arco-yaml` — YAML authoring surface.

### Interaction surfaces

- `arco-cli` — command-line interaction surface.
- `arco-python` — Python interaction surface.

## Dependency diagram

```text
+---------------------------+
| Interaction surfaces      |
|---------------------------|
| arco-cli                  |
| arco-python               |
+-------------+-------------+
              |
              v
+---------------------------+
| arco-ops                  |
| operations facade seam    |
+------+------+------+------+----------------+
       |      |      |      |                |
       |      |      |      |                v
       |      |      |      |      +-----------------------+
       |      |      |      |      | arco-solver           |
       |      |      |      |      | registry / preflight |
       |      |      |      |      | selection / solve     |
       |      |      |      |      +-----+-----------+-----+
       |      |      |      |            |           |
       |      |      |      |            |           v
       |      |      |      |            |   +------------------+
       |      |      |      |            |   | arco-runtime     |
       |      |      |      |            |   +------------------+
       |      |      |      |            |
       |      |      |      |            +--> +------------------+
       |      |      |      |                | arco-contracts   |
       |      |      |      |                +------------------+
       |      |      |      |
       |      |      |      +---------------> +----------------------+
       |      |      |                         | arco-exchange        |
       |      |      |                         | consumes arco-ir     |
       |      |      |                         +----------+-----------+
       |      |      |                                    |
       |      |      |                                    v
       |      |      |                         +----------------------+
       |      |      |                         | arco-ir              |
       |      |      |                         +----------------------+
       |      |      |
       |      |      +-----------------------> +----------------------+
       |      |                                | arco-compile         |
       |      |                                +----+----+----+-------+
       |      |                                     |    |    |
       |      |                                     |    |    +-------> arco-ir
       |      |                                     |    |
       |      |                                     |    +------------> arco-algebra
       |      |                                     |
       |      |                                     +------------+----> arco-model
       |      |                                                  |
       |      |                                                  +----> arco-blocks
       |      |
       |      +-------------------------------> +----------------------+
       |                                        | arco-validate        |
       |                                        +----------+-----------+
       |                                                   |
       |                                                   v
       |                                        +----------------------+
       |                                        | arco-model           |
       |                                        +----------+-----------+
       |                                                   ^
       |                                                   |
       +-------> +------------------+   +------------------+   +------------------+
                 | arco-kdl         |   | arco-json        |   | arco-yaml        |
                 +------------------+   +------------------+   +------------------+
                           \                  |                  /
                            \                 |                 /
                             +----------------+----------------+
                                              |
                                              v
                                        +------------------+
                                        | arco-model       |
                                        +--------+---------+
                                                 |
                                                 v
                                        +------------------+
                                        | arco-algebra     |
                                        +------------------+

Solver adapter seam:

  +------------------+     +------------------+     +------------------+     +------------------+
  | arco-highs       |     | arco-ipopt       |     | arco-xpress      |     | arco-scip        |
  +--------+---------+     +--------+---------+     +--------+---------+     +--------+---------+
           |                        |                        |                        |
           +------------+-----------+-----------+------------+------------------------+
                        |                       |                       |
                        v                       v                       v
               +------------------+   +------------------+   +------------------+
               | arco-targets     |   | arco-contracts   |   | arco-runtime     |
               +------------------+   +------------------+   +------------------+
```

## Responsibilities by seam

### 1. Authoring surfaces

Authoring surfaces are responsible for:

- parsing and source-location reporting
- source-specific syntax checks
- building canonical-model structures
- preserving enough provenance for diagnostics

Authoring surfaces are not responsible for:

- solver-family decisions
- runtime execution
- portable exchange ownership
- direct lowering to one specific solver family

If we add a new input format, it belongs here.

### 2. Canonical model

The canonical model is responsible for:

- domain meaning
- canonical sets, parameters, controls, objectives, and related semantics
- expression composition through stable semantic types
- reusable model-building blocks
- validation of semantic invariants

The canonical model should be the place where domain rules are easiest to find.
If a rule matters across all surfaces and all solver families, it belongs here or
in `arco-validate`.

### 3. Compilation

Compilation is responsible for:

- translating canonical semantics into executable or exchangeable artifacts
- normalization and lowering
- building the **Solver IR boundary** consumed by solver families
- building portable IR for exchange paths

Compilation must not know about concrete solver adapters. If a new solver family
needs a special translation, that logic belongs behind the solver-family seam,
not inside `arco-compile`.

### 4. Exchange

Exchange is responsible for:

- portable serialization and deserialization
- import/export formats that sit on top of portable IR

If a feature exists mainly to move models or compiled artifacts between systems,
it belongs here.

### 5. Shared interfaces

These crates define seams rather than heavyweight implementation:

- `arco-contracts` holds stable contracts shared across solver selection,
  preflight, invocation, results, capabilities, and adapter lifecycle

If multiple runtime or adapter modules need the same concept, put the contract
here before duplicating it elsewhere.

### 6. Runtime and orchestration

This layer is responsible for:

- solve execution mechanics
- registry and discovery of solver families
- solver selection and solver preflight
- capability enforcement policy
- top-level user workflows exposed to interaction surfaces

`arco-ops` should be the main entry seam for CLI and Python. If interaction
code needs to coordinate parsing, validation, compilation, exchange,
inspection, and solving, the operations belong in `arco-ops`, not in the
surface itself.

### 7. Solver adapters

Solver adapters are responsible for:

- translating `arco-targets` into one solver family’s native representation
- enforcing family-specific option and capability rules
- invoking the solver through the transport defined by the selected profile
- translating raw results back into shared solver result types

If we onboard a new solver family, it gets its own adapter crate and implements
the contracts defined in `arco-contracts`.

### 8. Interaction surfaces

Interaction surfaces are responsible for:

- user I/O
- CLI flags, commands, and formatting
- Python binding ergonomics and object translation
- presenting diagnostics and results

Interaction surfaces should stay thin. They should not own solver policy,
canonical semantics, or direct adapter wiring.

## Where new code goes as Arco grows

Use this section as the first placement guide when adding features.

| If we are adding...                                           | Put it in...                                                   | Why                                                |
| ------------------------------------------------------------- | -------------------------------------------------------------- | -------------------------------------------------- |
| a new input syntax or file format                             | `arco-kdl`, `arco-json`, `arco-yaml`, or a new authoring crate | authoring surfaces own parsing and provenance      |
| a semantic validation rule shared by all surfaces             | `arco-validate`                                                | one place for canonical invariants                 |
| a new canonical concept used across compilers and solvers     | `arco-model` or `arco-algebra`                                 | semantic center stays centralized                  |
| reusable model composition logic                              | `arco-blocks`                                                  | blocks are part of model construction, not runtime |
| a new lowering artifact consumed by solvers                   | `arco-targets`                                                 | solver-facing compile seam                         |
| a portable format for interchange                             | `arco-ir` plus `arco-exchange`                                 | portable exchange must stay separate               |
| solver selection, registry, profile, or preflight logic       | `arco-solver`                                                  | solver platform owns orchestration                 |
| adapter capability metadata or lifecycle contract             | `arco-contracts`                                               | shared solver-family seam                          |
| one concrete solver family                                    | a new `arco-<family>` crate                                    | adapters stay sibling modules                      |
| a CLI or Python operation that combines existing capabilities | `arco-ops`                                                     | keep surfaces thin                                 |
| a new end-user shell                                          | a new interaction crate                                        | interaction belongs at the edge                    |
| diagnostics/profiling/dev-only helpers                        | `arco-tools`                                                   | support tooling is not core runtime                |

## Current to target migration map

This is the intended shape relative to the current workspace.

- `arco-core` is expected to narrow or split toward `arco-model` and
  `arco-validate` responsibilities.
- `arco-export` is expected to narrow into `arco-exchange` semantics.
- `arco-solver` is expected to become a true solver platform crate, with shared
  adapter seams extracted into `arco-contracts`.
- `bindings/python` should converge on the `arco-python` role, even if the path
  stays the same during migration.
- `arco-kdl`, `arco-highs`, `arco-ipopt`, `arco-xpress`, `arco-scip`,
  `arco-blocks`, `arco-algebra`, and `arco-tools` already align closely with the
  target architecture and should mostly tighten seams rather than change purpose.

## Refactor phases

### Phase 1: establish the missing seams

Create and adopt the highest-value seam crates first:

- `arco-targets`
- `arco-contracts`
- `arco-ops`
- `arco-validate`

These four seams reduce the most coupling with the least conceptual churn.

### Phase 2: move behavior behind those seams

- move compile outputs behind `arco-targets`
- move solver-family contracts behind `arco-contracts`
- move shared validation into `arco-validate`
- move CLI/Python operation logic into `arco-ops`

### Phase 3: narrow legacy crates

- shrink `arco-core` toward canonical-model ownership only
- shrink `arco-export` toward exchange-only ownership
- shrink `arco-solver` toward platform/orchestration ownership only

### Phase 4: add new surfaces and adapters through the new seams

Only after the seams are stable should we expand with:

- additional authoring surfaces
- additional language bindings
- additional solver families
- additional exchange formats

### Phase 5: rename and retire legacy structure

Once responsibilities are stable:

- rename crates where needed for clarity
- remove transitional pass-through modules
- update architecture rules to enforce the final target-state seams

## Decision checklist for future contributors

Before creating a new crate or adding behavior to an existing one, answer these
questions:

1. Is this semantic meaning, or just translation/presentation?
2. Does this belong to the canonical model, compilation, exchange, runtime, or
   an outer surface?
3. Can this depend on an existing seam instead of a concrete implementation?
4. If we add a second implementation later, is the seam already explicit?
5. Would putting this elsewhere make CLI/Python, parsers, or adapters smarter
   than they should be?

If the answer is unclear, prefer strengthening an existing seam over adding a
cross-layer dependency.
