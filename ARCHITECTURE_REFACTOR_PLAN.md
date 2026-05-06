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
7. **Interaction surfaces are shells.** CLI, Python, Julia, and future language
   bindings orchestrate user workflows through `arco-ops` rather than reaching
   into internals.
8. **No compatibility-driven architecture.** Breaking API changes are acceptable
   while this refactor lands. Prefer clean seams over legacy pass-throughs,
   compatibility crates, or duplicated APIs.

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
- `arco-json` — planned JSON authoring surface.
- `arco-yaml` — planned YAML authoring surface.

### Interaction surfaces

- `arco-cli` — command-line interaction surface.
- `arco-python` — Python interaction surface.
- `arco-julia` — planned Julia interaction surface.
- Other language bindings should follow the same thin interaction-surface seam.

## Dependency diagram

```text
Interaction surfaces
┌─────────────────────────────────────────────────────────────┐
│ CLI / Python / Julia / other language bindings              │
│ thin shells: user I/O, flags, language ergonomics only       │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ arco-ops                                                    │
│ application seam: model API, load, validate, compile,       │
│ inspect, export, solve                                      │
└───────┬───────────────┬──────────────┬──────────────┬───────┘
        │               │              │              │
        ▼               ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ authoring    │ │ canonical    │ │ validation   │ │ exchange     │
│ surfaces     │ │ model        │ │              │ │ over IR      │
│              │ │              │ │              │ │              │
│ arco-kdl     │ │ arco-model   │ │ arco-validate│ │ arco-exchange│
│ arco-json    │ │              │ │              │ │              │
│   planned    │ │              │ │              │ │              │
│ arco-yaml    │ │              │ │              │ │              │
│   planned    │ │              │ │              │ │              │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │                │
       └────────────────┴────────────────┘                ▼
                        │                         ┌──────────────┐
                        ▼                         │ arco-ir      │
┌─────────────────────────────────────────────┐   │ portable IR  │
│ arco-compile                                │   └──────────────┘
│ lowering / compilation                      │
└───────────────┬──────────────────────┬──────┘
                │                      │
                ▼                      ▼
        ┌──────────────┐       ┌──────────────┐
        │ arco-targets │       │ arco-ir      │
        │ solver IR    │       │ portable IR  │
        └──────┬───────┘       └──────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│ arco-solver                                 │
│ registry / selection / preflight            │
└───────────────┬─────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────┐
│ solver adapters                             │
│ arco-highs / arco-ipopt / arco-xpress       │
│ arco-scip                                   │
└───────────────┬─────────────────────────────┘
                │
                ▼
        ┌──────────────┐
        │ arco-runtime │
        │ execution    │
        └──────────────┘

Shared solver contracts:
  arco-solver and solver adapters depend on arco-contracts.

Final rule:
  interaction surfaces depend on arco-ops, not on model, compiler, solver,
  exchange, runtime, or adapter crates directly.
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

This is the intended shape relative to the current workspace. The migration does
not preserve legacy APIs by default; when old APIs conflict with the target
architecture, remove or replace them.

- `arco-core` is transitional only. Move canonical model ownership into
  `arco-model`, then retire `arco-core` instead of keeping a compatibility
  prelude crate.
- `arco-expr` must stop leaking to interaction surfaces and solver adapters.
  Either absorb expression-domain ownership into `arco-model` / `arco-algebra` or
  keep it as an internal canonical-model dependency only.
- `arco-export` should collapse into `arco-exchange` or be deleted. Exchange
  logic consumes `arco-ir`; it should not preserve a second public exchange API
  only for compatibility.
- `bindings/python` remains the Python package path, but its Rust crate should
  depend directly on `arco-ops` only. Python API changes are acceptable.
- `arco-cli` should depend directly on `arco-ops` only, aside from CLI-only
  libraries such as argument parsing and diagnostics.
- `arco-kdl` remains an authoring surface. It must stop producing solver-facing
  targets directly.
- Solver adapters remain sibling crates. They must consume `arco-targets`,
  `arco-contracts`, and runtime services, not canonical model internals.

## Refactor phases

### Phase 0: align the architecture rules

Update `.sentrux/rules.toml` so the checker enforces this document, not the
transitional repository shape.

Required final direct-dependency rules:

- interaction surfaces -> `arco-ops` only
- authoring surfaces -> `arco-model` and authoring-local dependencies only
- validation -> `arco-model`
- compilation -> `arco-model`, `arco-targets`, and `arco-ir`
- exchange -> `arco-ir`
- solver platform -> `arco-contracts` and `arco-targets`
- solver adapters -> `arco-contracts`, `arco-targets`, and `arco-runtime`

Keep violations for:

- `arco-kdl -> arco-targets`
- solver adapters -> `arco-core`, `arco-model`, or `arco-expr`
- interaction surfaces -> model, compiler, exchange, solver, runtime, or adapter
  crates
- exchange -> canonical model crates

Acceptance: `sentrux check .` reports only real migration debt and passes once
all phases are complete.

### Phase 1: make `arco-model` the canonical owner

Move canonical domain ownership out of `arco-core` and into `arco-model`:

- `Model`
- variables, constraints, objectives, bounds, and senses
- model errors and diagnostics-facing domain errors
- stable semantic handles and IDs
- snapshots, sparse export views, and model inspection data
- canonical expression ownership, or explicit integration with `arco-algebra`

Then remove `arco-core` from the workspace. Do not keep a compatibility crate
unless a short-lived branch-local transition is required to keep intermediate
commits buildable.

Acceptance:

- no workspace crate depends on `arco-core`
- `arco-core` is removed from `Cargo.toml`
- canonical model tests live under `arco-model`

### Phase 2: settle expression ownership

Decide and implement one expression boundary:

- If expressions are canonical domain concepts, move them into `arco-model`.
- If expressions are reusable algebra mechanics, keep them below `arco-model` as
  `arco-algebra` internals.

In either case, prevent expression IDs and internals from crossing into CLI,
Python, exchange, solver platform, or solver adapters.

Acceptance:

- interaction surfaces do not import `arco-expr`
- solver adapters do not import `arco-expr`
- public model/ops APIs expose domain handles or DTOs, not expression internals

### Phase 3: create the compilation seam

Create `arco-compile` as the only semantic bridge out of the canonical model.
Move lowering and artifact construction currently owned by `arco-kdl` into this
crate.

`arco-compile` owns:

- canonical model -> `arco-targets`
- canonical model -> `arco-ir`
- normalization and lowering diagnostics
- compile-time traceability metadata

Acceptance:

- `arco-kdl` no longer depends on `arco-targets`
- KDL tests that assert lowered artifacts move to `arco-compile`
- `arco-kdl` tests focus on parsing, source diagnostics, and canonical model
  construction

### Phase 4: make solver adapters target-only

Refactor `arco-highs`, `arco-ipopt`, `arco-xpress`, and `arco-scip` to consume
compiled targets and shared contracts only.

Adapters own:

- target -> native solver representation
- family-specific capability and option enforcement
- native invocation details
- raw result -> `arco-contracts` result translation

Adapters do not own:

- canonical model flattening
- authoring-surface parsing
- solver selection policy

Acceptance:

- adapters have no dependency on `arco-core`, `arco-model`, or `arco-expr`
- adapters have no dependency on one another
- adapter tests build targets directly or use test helpers from `arco-targets`

### Phase 5: clean solver platform orchestration

Narrow `arco-solver` to registry, selection, capability preflight, and invocation
through shared contracts.

`arco-solver` receives compiled targets. It does not parse files, lower models,
or depend on concrete adapter crates.

Acceptance:

- no `arco-solver -> arco-kdl`
- no `arco-solver -> arco-model` unless required by target metadata and approved
  as part of the final seam
- no `arco-solver -> arco-highs/arco-ipopt/arco-xpress/arco-scip`

### Phase 6: collapse exchange into `arco-exchange`

Make `arco-exchange` the only public import/export crate. It consumes `arco-ir`.
Remove `arco-export` unless it has a target-state responsibility that is not
covered by `arco-exchange`.

Acceptance:

- `arco-export` is removed or private to the exchange implementation
- exchange code does not depend on canonical model crates or solver adapters
- import/export documentation names `arco-exchange`

### Phase 7: rebuild `arco-ops` as the only application seam

`arco-ops` exposes the user-facing Rust workflow API used by all interaction
surfaces:

- model construction API
- load from authoring surfaces
- validation
- compilation
- inspection
- exchange/import/export
- solve selection and execution
- stable DTOs and handles for language bindings

Breaking changes are acceptable. Design the API around target architecture, not
old Python or CLI internals.

Acceptance:

- common CLI and Python workflows are expressible through `arco-ops`
- `arco-ops` owns app-level errors and result DTOs
- no interaction surface needs direct model, compiler, solver, exchange,
  runtime, or adapter access

### Phase 8: rewrite interaction surfaces

Rewrite CLI and Python bindings to call `arco-ops` only. Julia and future
language bindings must follow the same rule from their first implementation.

CLI owns:

- command-line parsing
- terminal formatting
- process exit behavior

Python/Julia/other bindings own:

- language-native object wrappers
- language-native error conversion
- language-native packaging

Acceptance:

- `bindings/python` has one direct internal dependency: `arco-ops`
- `arco-cli` has one direct internal dependency: `arco-ops`
- user-facing docs and examples use the new APIs

### Phase 9: delete legacy structure

Remove old compatibility paths rather than preserving them:

- `arco-core`
- `arco-export`, if replaced by `arco-exchange`
- direct solve APIs on adapters that accept canonical models
- old KDL compile artifact reexports
- duplicated result/status/error types
- stale tests and examples for removed APIs

Acceptance:

- no retired crate remains in the workspace
- no compatibility module exists only to preserve old import paths
- `sentrux check .` passes

### Phase 10: final documentation and verification

Update architecture and user documentation to describe the actual final state.

Required checks before completion:

```sh
cargo fmt
cargo clippy --all --benches --tests --examples --all-features -- -D warnings
cargo test --workspace
sentrux check .
```

Document any intentionally missing planned surfaces, such as `arco-json`,
`arco-yaml`, or `arco-julia`, as planned rather than present.

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
