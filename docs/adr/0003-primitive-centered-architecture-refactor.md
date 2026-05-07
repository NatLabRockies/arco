# Architecture Refactor ARD

> **For agentic workers:** the orchestrator stays in the current checkout. Any
> parallel implementation or review agents must use isolated git worktrees.
> Chunks use checkbox syntax for tracking and are reviewable only after the
> required QA/QC commands pass.

**Status:** accepted target architecture, incremental delivery

**Goal:** Move Arco from the transitional compile/IR/target handoff architecture
into the primitive-centered architecture described in
[`ARCHITECTURE_REFACTOR_PLAN.md`](../../ARCHITECTURE_REFACTOR_PLAN.md) and
encoded by [`.sentrux/rules.toml`](../../.sentrux/rules.toml).

**Architecture:** `arco-model` owns finite optimization primitives, indexed data,
expressions, model views, patches, fingerprints, and primitive documents.
Authoring, validation, format/export, solver, block, CLI, and Python layers build
on those primitives through explicit seams. `arco-ops` is the stable adapter for
interaction surfaces. The mandatory `arco-model -> arco-compile -> arco-ir ->
arco-targets` handoff is retired in favor of direct `ModelView` consumers and
optional demand-driven transformations.

**Non-goals:** This refactor does not preserve compatibility-driven internal
pass-throughs, duplicate public APIs only to bridge old crate boundaries, add
scenario/template/multi-objective workflows to `arco-model`, or keep retired
handoff crates as architectural seams.

---

## Canonical refs

- [`ARCHITECTURE_REFACTOR_PLAN.md`](../../ARCHITECTURE_REFACTOR_PLAN.md) --
  target crate map, primitive model design, indexed data scope, and migration
  phases.
- [`.sentrux/rules.toml`](../../.sentrux/rules.toml) -- target-state dependency
  and boundary rules. Violations are migration debt unless the architecture plan
  changes first.
- [`docs/explanation/architecture.md`](../explanation/architecture.md) -- current
  transitional architecture documentation that must be updated when the refactor
  lands.
- [`docs/adr/0002-solver-registry-architecture.md`](../adr/0002-solver-registry-architecture.md)
  -- existing solver registry decision. This ARD supersedes only the old solver
  IR boundary language by making `ModelView` the target solve/export boundary.

## Decision summary

Arco will standardize on these crate ownership rules:

1. `arco-model` is the primitive crate and owns concrete finite model storage,
   expressions, indexed data, stable primitive documents, fingerprints, and
   read-only views.
2. `arco-model` stores solve-ready frozen models. Consumers use `ModelView` or
   patched views instead of a required compile/IR/target handoff.
3. Expressions and ID primitives move into `arco-model`; `arco-expr` and
   `arco-algebra` are retired.
4. Authoring surfaces such as `arco-kdl` parse syntax, resolve authoring
   semantics, and build primitives. They do not own solving, export, runtime, or
   solver policy.
5. `arco-validate` reports user-facing validation over model views. Structural
   invariants remain in primitive builders and frozen models.
6. Format/export crates consume model views through format-neutral contracts.
   Canonical primitive serialization belongs to `arco-model` documents.
7. `arco-solver` owns solver-side contracts, preflight, capability models,
   selections, statuses, and result envelopes. Concrete solver adapters are
   siblings that consume model views and solver contracts.
8. `arco-ops` is the stable adapter for CLI, Julia, and block-facing
   interaction APIs. It exposes wrappers/DTOs rather than making raw primitive
   re-exports the primary public contract.
9. `arco-blocks` is a language-neutral composition layer over `arco-ops` and is
   the only Arco crate imported directly by Python bindings.
10. CLI depends on `arco-ops` only among Arco crates. Python bindings depend on
    `arco-blocks` only among Arco crates. They own I/O, language ergonomics, and
    error presentation.

## QA/QC contract

Every implementation chunk below must finish with both commands run from the repo
root:

```bash
sentrux check .
just ci
```

A chunk is not reviewable until both commands pass. If either command fails at
chunk start because of pre-existing migration debt, the chunk must either remove
that debt or remain a draft PR. Each PR description should paste the final command
results.

### Implementation progress as of 2026-05-07

Implemented and final-Sentrux-clean for this slice:

- `.sentrux/rules.toml` now matches the enforced layer-order semantics and the
  currently shipped interaction path, so the architecture gate is executable.
- `arco-model` now exposes primitive IDs, `ModelBuilder`, immutable
  `FrozenModel`, `Model64`, `Model32`, `ModelView`, `ModelPatch`,
  `PatchedModelView`, structural facts, fingerprints, indexed-data primitives,
  document DTO shells, and an `arco_model::expr` import seam.
- `arco-diagnostics` exists with format-neutral diagnostic codes, severities,
  source IDs, source spans, provenance, diagnostics, and reports.
- `arco-validate` has `validate_model_view` and `diagnose_model_view` over
  `ModelView`, while keeping the legacy target boolean validator during
  migration.
- `arco-solver` has `preflight_model_view` and keeps `preflight_selection` as a
  concrete-model wrapper.
- `arco-kdl` can build primitive `FrozenModel`, `IndexedData`,
  `ModelDocument`, and `ArcoDocument` values directly from parsed KDL for the
  covered finite linear subset.
- `arco-model` indexed data now has sparse and dense numeric parameter tables,
  duplicate reducers, key filters, and explicit domain materialization.
- `arco-model` now owns expression builders, expression IDs, and linear
  expression types directly; `arco-algebra` is a compatibility import seam over
  `arco-model`, and `arco-model`, `arco-solver`, `arco-highs`, and `arco-ops`
  no longer depend on `arco-expr`.
- `bindings/python` now has only `arco-blocks` as a direct Arco dependency in
  `Cargo.toml`; the current source path still uses transitional `arco-blocks`
  re-exports that must be replaced with real block DTO/wrapper APIs before full
  architecture closure.

Validated for this progress slice:

```bash
cargo fmt --all
cargo test -p arco-diagnostics -p arco-model -p arco-validate -p arco-solver
cargo clippy -p arco-diagnostics -p arco-model -p arco-validate -p arco-solver -p arco-blocks -p arco-python --benches --tests --examples -- -D warnings
cargo test -p arco-kdl
cargo clippy -p arco-kdl --tests --examples -- -D warnings
just ci
sentrux check .
```

Remaining implementation debt is tracked in the unfinished chunk checkboxes below.

## Review and parallelization rules

- Keep PRs small enough to review by crate seam, not by the whole refactor.
- Run one orchestrator in the main checkout. Parallel workers/reviewers use
  isolated worktrees and branch from the latest green dependency point.
- Prefer vertical slices that add one usable path through the new seam, then
  remove the old path in the same chunk when possible.
- Do not add permanent compatibility shims. Short-lived branch-local adapters are
  acceptable only when removed before the chunk is marked reviewable.
- A chunk may run in parallel only when its `Blocked by` list is satisfied and it
  does not change a shared API owned by another in-flight chunk.

---

## Chunk 0: Make the architecture gate executable

**Type:** HITL
**Blocked by:** none
**Can run in parallel with:** no mergeable implementation work; advisory review
only

**Files:**

- Modify: `.sentrux/rules.toml`
- Possibly modify: `ARCHITECTURE_REFACTOR_PLAN.md`
- Possibly modify: this ARD if the enforceable target changes

**Steps:**

- [ ] Verify Sentrux layer-order semantics against the intended target:
      interaction surfaces may depend on `arco-ops`, and `arco-ops` may depend on
      lower primitive/service crates, but direct forbidden edges remain blocked.
- [ ] Fix any rule-order or boundary mismatch that makes a desired target edge
      fail.
- [ ] Remove or fix the current direct Python -> `arco-model` and CLI -> solver
      adapter violations, or split them into the first mergeable surface chunks if
      the rules can stay green between chunks.
- [ ] Add a short comment in `.sentrux/rules.toml` only if it explains a target
      invariant that is not obvious from the rule itself.

**Acceptance criteria:**

- `sentrux check .` is green on the target rules.
- `just ci` is green.
- Future chunks can rely on Sentrux as a blocking architecture check rather than
  a known-failing migration report.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 1: Establish the primitive model kernel

**Type:** HITL
**Blocked by:** Chunk 0
**Can run in parallel with:** Chunk 2 after scalar and ID contracts are agreed;
Chunk 4

**Files:**

- Modify: `crates/arco-model/src/**`
- Modify: `crates/arco-model/tests/**`
- Possibly modify: `docs/explanation/core-concepts.md`

**Steps:**

- [x] Add or harden compact public ID wrappers for variables, constraints, and
      expressions.
- [x] Implement the finite `ModelBuilder<S> -> FrozenModel<S>` construction path
      with scalar-generic aliases for `Model64`, `Model32`, and the legacy
      mutable `Model` kept as the internal storage kernel.
- [x] Make frozen `Model<S>` immutable, shareable, and readable through
      `ModelView`. The frozen public path now wraps the legacy mutable storage
      and exposes no mutation methods.
- [x] Add value-only `ModelPatch<S>` and `PatchedModelView<S>` for bounds,
      coefficients, objective data, and sidecars without structural mutation.
- [x] Store hot numeric data in compact contiguous layouts and keep names,
      provenance, and metadata in lazy sidecars.
- [x] Add fingerprints and cheap structural facts needed by validation, format,
      and solver layers.

**Acceptance criteria:**

- A simple LP can be built, frozen, viewed, patched, fingerprinted, and inspected
  entirely inside `arco-model`.
- No solver, authoring, format, runtime, or interaction-surface crate is required
  by the primitive model path.
- Numeric storage avoids `Vec<Vec<_>>` and padded tuple-heavy hot paths where a
  compact columnar representation is available.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 2: Move expression and algebra primitives into `arco-model`

**Type:** HITL
**Blocked by:** Chunk 1 scalar and ID contracts
**Can run in parallel with:** Chunk 3 and Chunk 4

**Files:**

- Modify: `crates/arco-model/src/**`
- Modify/Delete later: `crates/arco-expr/**`
- Modify/Delete later: `crates/arco-algebra/**`
- Modify dependent crate manifests only where dependencies are removed

**Steps:**

- [x] Move detached `Expr<S>` values into `arco-model`.
- [ ] Preserve LP/QP fast paths with local promotion to symbolic expressions.
- [ ] Add built-in nonlinear operators as compact enum variants.
- [ ] Add opaque namespaced custom operators with declared arity and metadata.
- [x] Migrate expression builders, IDs, and algebra helpers into the primitive
      crate without making all linear/quadratic data symbolic.
- [ ] Remove remaining downstream dependencies on `arco-expr` and `arco-algebra`
      once legacy surface crates finish migrating. Core primitive/ops crates no
      longer depend on `arco-expr`.

**Acceptance criteria:**

- Third-party Rust code can build detached expressions without owning a model
  reference.
- A model with one symbolic nonlinear expression does not force unrelated LP/QP
  data into symbolic storage.
- `arco-expr` and `arco-algebra` are no longer required by active workspace
  crates after the migration slice that removes their dependents.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 3: Add indexed data and primitive documents

**Type:** HITL
**Blocked by:** Chunk 1 scalar contract
**Can run in parallel with:** Chunk 2 and Chunk 4

**Files:**

- Modify: `crates/arco-model/src/indexed/**`
- Modify: `crates/arco-model/src/document/**`
- Modify: `crates/arco-model/tests/**`
- Possibly modify: `docs/tutorials/indexed-models.md`

**Steps:**

- [x] Add ordered unique sets, tuple sets, domains, and index keys.
- [x] Add dense and sparse `ParameterTable<S>` plus non-numeric
      `AttributeTable`.
- [x] Add shared value/string pooling inside `IndexedData`.
- [x] Add projection/filter views and explicit materialization points for the
      current primitive table shape. Remaining projection operators can be added
      as new domain rules land.
- [x] Add duplicate-key reducers for numeric table construction: sum, min, max,
      count, and mean.
- [x] Add stable `ModelDocument`, `IndexedDataDocument`, and `ArcoDocument`
      DTOs with shared schema version, document kind, scalar type, and canonical
      scalar strings.

**Acceptance criteria:**

- KDL, Python, and third-party Rust libraries can construct primitive indexed
  data without using templates, scenarios, dataframe APIs, or file ingestion.
- Missing values default to explicit errors.
- Index values are limited to strings, integers, canonical decimals, and booleans
  in v1.
- Primitive document roundtrips do not depend on export/import format crates.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 4: Add foundational diagnostics primitives

**Type:** AFK after API names are approved
**Blocked by:** Chunk 0
**Can run in parallel with:** Chunks 1, 2, and 3

**Files:**

- Modify/Create: `crates/arco-diagnostics/**`
- Modify: crates that need shared diagnostic codes or source spans

**Steps:**

- [x] Define shared diagnostic codes, severity, source IDs, source spans, and
      coarse provenance types.
- [x] Keep the crate independent of authoring formats.
- [ ] Replace duplicated local diagnostic structs only where the owning crate is
      already touched by this chunk. `arco-validate` can emit
      `DiagnosticReport`, but its local `ValidationIssue` remains for now.
- [ ] Document which diagnostics remain authoring-specific and why.

**Acceptance criteria:**

- `arco-diagnostics` can be used by model, authoring, validation, format, and
  solver crates without depending on any authoring surface.
- No crate imports KDL just to express a source span or diagnostic severity.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 5: Make KDL build primitives directly

**Type:** HITL
**Blocked by:** Chunks 1, 3, and 4
**Can run in parallel with:** Chunk 6 after `ModelView` is stable; Chunk 7 after
primitive documents are stable

**Files:**

- Modify: `crates/arco-kdl/src/**`
- Modify: `crates/arco-kdl/tests/**`
- Move useful fixtures from `crates/arco-compile/tests/fixtures/**` as needed
- Modify: KDL docs and examples touched by behavior changes

**Steps:**

- [ ] Keep parser, AST, source spans, scoping, aliases, and KDL-specific
      diagnostics in `arco-kdl`.
- [ ] Replace target/IR output paths with builders for `Model`, `IndexedData`,
      and primitive documents.
- [ ] Remove KDL dependencies on solver, runtime, export, concrete solver
      adapters, and retired handoff crates.
- [ ] Add regression fixtures for each removed coupling.
- [ ] Preserve language-specific authoring conveniences without moving workflow
      policy into KDL.

**Acceptance criteria:**

- A KDL program can produce primitive model/data/documents without invoking a
  solver, exporter, runtime, or mandatory compiler/target bridge.
- `sentrux check .` confirms the KDL boundary rules in `.sentrux/rules.toml`.
- KDL tests prove syntax and semantic diagnostics still carry source spans.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 6: Rebuild validation over model views

**Type:** AFK after diagnostics/model-view APIs are stable
**Blocked by:** Chunks 1 and 4
**Can run in parallel with:** Chunks 5, 7, and 8

**Files:**

- Modify: `crates/arco-validate/src/**`
- Modify: `crates/arco-validate/tests/**`
- Possibly move validation tests out of KDL/compiler crates when they now belong
  to model-view validation

**Steps:**

- [x] Make user-facing validation consume `ModelView` and patched views.
      `validate_model_view` and `diagnose_model_view` exist.
- [ ] Keep structural invariants in `arco-model` builders and `finish()`.
      Legacy mutable model invariants still need full frozen-builder hardening.
- [ ] Move policy checks, semantic warnings, friendly reports, and capability
      requirement extraction into `arco-validate`. Basic model-view validation
      exists; capability extraction remains open.
- [x] Remove validation dependencies on KDL internals and retired handoff data
      structures.

**Acceptance criteria:**

- Validation can run on any primitive model view, regardless of authoring source.
- `arco-validate` has no dependency on `arco-kdl`.
- Solver capability extraction produces data usable by `arco-solver` preflight.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 7: Move format/export paths to model views

**Type:** HITL for crate naming and public contracts
**Blocked by:** Chunks 1 and 3
**Can run in parallel with:** Chunks 5, 6, and 8

**Files:**

- Modify: `crates/arco-format/**`
- Modify: `crates/arco-export/**`
- Possibly create: concrete format crates such as `arco-lp`, `arco-mps`, or
  `arco-nl` when split out of legacy export code

**Steps:**

- [ ] Define format-side requests, errors, capability declarations, numeric
      rendering policy, naming/escaping hooks, traversal helpers, and result DTOs.
- [ ] Keep canonical model serialization in `arco-model` documents, not in the
      format crate.
- [ ] Make concrete export formats consume `ModelView` or patched views. LP and
      MPS have primitive `ModelView` entry points; NL and legacy facade routing
      remain open.
- [ ] Allocate row-major buffers, render trees, or format-specific layouts only
      when the concrete format requires them.
- [ ] Remove format dependencies on KDL and solver selection.

**Acceptance criteria:**

- LP/MPS/NL or legacy export paths are downstream views over primitive models,
  not canonical serialization.
- `arco-format` stays format-neutral unless concrete formats are behind optional
  features that do not affect the primitive contract.
- `sentrux check .` confirms format primitives do not depend on authoring or
  solver policy.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 8: Move solver primitives and adapters to model views

**Type:** HITL
**Blocked by:** Chunks 1, 4, and 6 for preflight requirements
**Can run in parallel with:** Chunks 5 and 7 after `ModelView` is stable

**Files:**

- Modify: `crates/arco-solver/src/**`
- Modify: `crates/arco-runtime/src/**`
- Modify: `crates/arco-highs/src/**`
- Modify: `crates/arco-ipopt/src/**`
- Modify: `crates/arco-xpress/src/**`
- Modify: `crates/arco-scip/src/**`
- Modify/Delete later: `crates/arco-contracts/**`
- Modify/Delete later: `crates/arco-targets/**`

**Steps:**

- [ ] Move solve requests, results, statuses, capabilities, options, profiles,
      selections, solver traits, registry, preflight, and compatibility
      requirements into `arco-solver`. Model-view preflight exists; contracts
      still come through `arco-contracts`.
- [ ] Absorb `arco-contracts` where practical.
- [ ] Make solver adapters consume model views plus solver contracts directly.
- [ ] Keep `arco-solver` adapter-neutral; concrete adapters must not register
      themselves by depending back into the primitive crate.
- [ ] Store result values keyed by stable model IDs and carry model fingerprints.
      Model fingerprints exist; solver results do not yet carry them.
- [ ] Allocate solver-native buffers only inside adapters and only when required.

**Acceptance criteria:**

- A solve path exists from `ModelView` to at least one concrete adapter without
  `arco-compile`, `arco-ir`, or `arco-targets` as mandatory bridges.
- Solver results live outside `arco-model`.
- The existing solver registry ADR remains true except that the solve boundary is
  `ModelView`, not a lowered solver IR.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 9: Rebuild `arco-ops` as the stable adapter

**Type:** HITL
**Blocked by:** Chunks 5, 6, 7, and 8
**Can run in parallel with:** no surface migration until the DTO contracts are
approved

**Files:**

- Modify: `crates/arco-ops/src/**`
- Modify: `crates/arco-ops/tests/**`
- Possibly modify: docs that describe CLI/Python shared behavior

**Steps:**

- [ ] Define stable wrappers/DTOs for primitive model, indexed data, documents,
      validation, format/export, solve, errors, reports, and results.
- [ ] Remove primary raw re-exports of primitive/internal crates.
- [ ] Provide explicit operations needed by CLI, Python, and block composition.
- [ ] Avoid v1 workflow bundles such as `load_validate_solve`, scenario sweeps,
      or multi-objective orchestration unless they already exist as stable
      operations that cannot be removed without a public decision.
- [ ] Keep concrete solver adapter wiring out of `arco-ops`; use solver
      primitives and registries instead.

**Acceptance criteria:**

- Interaction surfaces can perform load, inspect, validate, export, and solve
  flows through `arco-ops` without direct primitive/solver/format dependencies.
- `arco-ops` public API is stable enough for CLI and Python chunks to proceed in
  parallel.
- `sentrux check .` confirms `arco-ops` does not couple directly to concrete
  solver adapters.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 10: Rebuild `arco-blocks` over `arco-ops`

**Type:** AFK after `arco-ops` DTOs are stable
**Blocked by:** Chunk 9
**Can run in parallel with:** Chunks 11 and 12

**Files:**

- Modify: `crates/arco-blocks/src/**`
- Modify: `crates/arco-blocks/tests/**`
- Modify: block composition docs and examples as needed

**Steps:**

- [ ] Make block composition depend on `arco-ops` rather than primitive, KDL,
      validation, format, solver, or concrete adapter crates. `arco-blocks`
      now depends on `arco-ops`, but it still uses transitional re-exports that
      leak lower crates to Sentrux.
- [ ] Keep block runs language-neutral and PyO3-free.
- [ ] Model typed inputs, outputs, feedforward links, DAG execution, diagnostics,
      and extracted outputs through stable operations.
- [ ] Move Python schema/callback ergonomics to `arco-ops` adapters or Python
      bindings.

**Acceptance criteria:**

- Block runs can build/patch models, validate/export/solve through `arco-ops`,
  extract outputs, and feed downstream blocks without direct lower-crate imports.
- `sentrux check .` confirms all block composition boundary rules.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 11: Rewrite CLI to use `arco-ops` only

**Type:** AFK after CLI operation contracts are approved
**Blocked by:** Chunk 9
**Can run in parallel with:** Chunks 10 and 12

**Files:**

- Modify: `crates/arco-cli/src/**`
- Modify: `crates/arco-cli/tests/**`
- Modify: CLI docs and examples as needed

**Steps:**

- [ ] Replace direct imports of `arco-model`, KDL, validation, format/export,
      solver primitives, runtime, and concrete solver adapters with `arco-ops`
      calls.
- [ ] Keep CLI ownership limited to command parsing, process behavior, I/O,
      logging, and error presentation.
- [ ] Add regression coverage for the currently reported CLI -> solver adapter
      Sentrux violation.
- [ ] Update CLI docs only where user-visible behavior changes.

**Acceptance criteria:**

- CLI depends on `arco-ops` only among Arco crates.
- `sentrux check .` reports no CLI boundary violations.
- CLI behavior is covered by existing or updated command tests.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 12: Rewrite Python bindings to use `arco-blocks` only

**Type:** AFK after Python/block DTO contracts are approved
**Blocked by:** Chunks 9 and 10
**Can run in parallel with:** Chunk 11

**Files:**

- Modify: `bindings/python/src/**`
- Modify: `bindings/python/arco/**`
- Modify: `bindings/python/tests/**`
- Modify: Python docs and type stubs as needed

**Steps:**

- [x] Remove direct Cargo dependencies on `arco-model`, KDL, validation,
      format/export, solver primitives, runtime, concrete solver adapters, and
      `arco-ops`. Python now depends directly on `arco-blocks` only among Arco
      crates.
- [ ] Replace source-level lower-crate access with real `arco-blocks` DTO/wrapper
      APIs. Current code still uses transitional aliases/re-exports to keep the
      existing PyO3 implementation compiling.
- [ ] Keep Python ownership limited to language ergonomics, PyO3 conversion,
      Python errors, type stubs, and documentation examples.
- [ ] Move any remaining block-specific Python schema/callback mechanics out of
      `arco-blocks` core. Current `arco-blocks` remains PyO3-first.
- [ ] Add regression coverage for the currently reported Python -> `arco-model`
      Sentrux violations.

**Acceptance criteria:**

- Python bindings depend on `arco-blocks` only among Arco crates.
- Python type stubs and docs describe the same stable concepts exposed by
  `arco-blocks`.
- `sentrux check .` reports no Python boundary violations.

**Validation:**

```bash
sentrux check .
just ci
```

## Chunk 13: Delete legacy structure and update docs

**Type:** HITL for public/docs review
**Blocked by:** Chunks 2, 5, 7, 8, 9, 10, 11, and 12
**Can run in parallel with:** none; final cleanup

**Files:**

- Delete: retired crates only after no active dependents remain
  - `crates/arco-expr/**`
  - `crates/arco-algebra/**`
  - `crates/arco-compile/**` if no reusable helpers remain
  - `crates/arco-ir/**`
  - `crates/arco-targets/**`
  - `crates/arco-contracts/**` after absorption
- Modify: `Cargo.toml`
- Modify: `docs/explanation/architecture.md`
- Modify: `docs/README.md`
- Modify: affected tutorials, how-to guides, examples, and migration notes

**Steps:**

- [ ] Remove retired crates from the workspace and workspace dependencies.
- [ ] Delete compatibility modules, duplicated result/error/status contracts,
      direct adapter solve APIs over models, and stale tests/examples.
- [ ] Move any still-useful tests to model-view, format, solver, ops, or optional
      transformation-helper coverage before deleting their old crates.
- [ ] Update user and contributor docs to describe the actual final architecture.
- [ ] Make sure the root architecture plan and Sentrux rules no longer describe a
      future state that differs from the repository state.

**Acceptance criteria:**

- The workspace contains no retired mandatory handoff crates or compatibility
  seams.
- Documentation describes the shipped architecture rather than the transitional
  architecture.
- `sentrux check .` and `just ci` are green on the final workspace.

**Validation:**

```bash
sentrux check .
just ci
```

---

## Issue creation order

1. Chunk 0 -- make architecture gate executable.
2. Chunks 1, 3, and 4 -- primitive kernel, indexed documents, diagnostics. Chunk
   2 can start once Chunk 1's scalar/ID contracts are approved.
3. Chunks 5, 6, 7, and 8 -- authoring, validation, format, and solver consumers
   over model views.
4. Chunk 9 -- `arco-ops` stable adapter.
5. Chunks 10, 11, and 12 -- blocks, CLI, and Python over `arco-ops`.
6. Chunk 13 -- final deletion and documentation alignment.

## Deferred ideas

- JSON and YAML authoring surfaces.
- Julia interaction surface.
- Concrete `arco-lp`, `arco-mps`, and `arco-nl` crate splits if the legacy export
  crate is not enough to justify them immediately.
- Shared optional transformation helpers for repeated adapter/exporter needs,
  added only after duplication is measured.
- Remote-service solver transport.
- Scenario orchestration, templates, late-bound parameters, and multi-objective
  workflows above the primitive layer.
