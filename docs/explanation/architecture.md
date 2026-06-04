# Architecture

Arco ships the primitive-centered architecture described in
[`ARCHITECTURE_REFACTOR_PLAN.md`](../../ARCHITECTURE_REFACTOR_PLAN.md), with
remaining migration debt tracked in ADR chunk checklists. This page tracks the
**currently shipped** crate seams.

## Shipped core shape

```text
User surfaces
  bindings/python (arco-python)
  crates/arco-cli
        │
        ├──────────────► crates/arco-blocks
        │                         │
        └──────────────┬──────────┘
                       ▼
Runtime facade
  crates/arco-ops
        │
        ├── Semantic/data construction: crates/arco-kdl
        ├── Validation/reporting:      crates/arco-validate
        ├── Labeled axes/arrays:       crates/arco-arrays
        ├── Primitive model contract:  crates/arco-model
        ├── Portable/export DTOs:      crates/arco-format
        ├── Solver contracts:          crates/arco-solver
        └── Solver registry/adapters:  crates/arco-builtin-solvers
                                      crates/arco-highs
                                      crates/arco-ipopt
                                      crates/arco-xpress
                                      crates/arco-scip
```

## Crate responsibilities (current)

- `arco-model`: primitive model contract. Owns variables/controls, bounds,
  expressions, constraints, objectives, model snapshots, sparse import/export,
  `ModelView`, `ModelPatch`, indexed-data primitives, and primitive document
  DTOs. This layer stays independent of Python, KDL, CLI, blocks, and
  solver-specific behavior.
- `arco-arrays`: labeled-axis and sparse-array semantics. Owns axis identity,
  parameter alignment, broadcast planning, reductions, sparse active-mask
  expansion, tuple-domain planning, and fail-fast shape validation.
- `arco-kdl`: semantic/data construction for KDL. Owns parsing, source
  diagnostics, primitive document construction, data binding, and scenario
  inference. It should produce normalized artifacts instead of leaking parser
  structures into user surfaces or solvers.
- `arco-validate`: user-facing validation and reporting over model views.
- `arco-solver`: solver contracts, capability metadata, selection, preflight,
  shared configuration, status mapping, and backend traits.
- `arco-ops`: stable runtime facade used by CLI, Python core APIs, and block
  composition. It owns solve orchestration, validation routing, inspection
  routing, solver selection, result mapping, and stable DTO boundaries.
- `arco-blocks`: block composition, typed input/output contracts, stage
  diagnostics, and swappability checks. Blocks compose public model/runtime
  APIs instead of reaching into primitive storage.
- `arco-builtin-solvers`: built-in solver family wiring. Registers shipped
  solver families (HiGHS, SCIP, Xpress). Does not include IPOPT as a shipped
  built-in.
- `arco-highs`/`arco-scip`/`arco-xpress`: shipped solver adapters included in
  default product artifacts.
- `arco-ipopt`: portable facade crate. The default build provides solver
  selection and a clear unavailable diagnostic for solve attempts. Native
  IPOPT solve execution requires the `ipopt` feature on `arco-ops`, which is
  intentionally outside the normal workspace `--all-features` path so CI
  remains portable without native IPOPT libraries.
- Solver adapters consume `ModelView`/DTO contracts so adding a backend does
  not change Python, KDL, block, or primitive model APIs.

## User-facing API architecture rules

- User surfaces own ergonomics, examples, command/API naming, and error
  presentation. They should stay thin over shared contracts.
- Reusable blocks and domain-specific helpers must build on public model APIs
  rather than mutating internals.
- Advanced raw IDs, sparse matrix import/export, and solver-specific settings
  remain available as expert APIs, but beginner paths should use named model
  objects, axes, inspection, and result accessors.
- Memory behavior is part of the public contract. Sparse active masks and tuple
  domains should remain visible through shape/count inspection before solve,
  and convenience layers must not hide accidental dense expansion.
- Adding a solver should stay confined to solver contracts, runtime
  registration, and a concrete backend crate unless a shared contract is
  genuinely missing.

## Transitional seams still present

The old compile/target handoff crates are retired from the active workspace.
`arco-expr` and `arco-contracts` have been removed after their APIs were
absorbed into `arco-model` and `arco-solver`.
`arco-algebra`, `arco-ir`, `arco-export`, `arco-compile`, and `arco-targets`
have also been removed after expression, portable export, and legacy lowering
APIs moved into active primitive/model-view seams.

## Legacy crate retirement boundary (2026-05-06)

Active-workspace dependency snapshot from `cargo metadata --no-deps`:

| Crate                      | Active workspace dependents | Retirement status                                             |
| -------------------------- | --------------------------- | ------------------------------------------------------------- |
| `arco-expr` (deleted)      | none                        | Removed; expression and ID primitives live in `arco-model`.   |
| `arco-contracts` (deleted) | none                        | Removed; solver contracts live in `arco-solver`.              |
| `arco-algebra` (deleted)   | none                        | Removed; expression APIs live in `arco-model`.                |
| `arco-compile` (deleted)   | none                        | Removed; legacy lowering internals now live under `arco-ops`. |
| `arco-ir` (deleted)        | none                        | Removed; portable export DTOs live in `arco-format`.          |
| `arco-targets` (deleted)   | none                        | Removed; target DTOs are no longer a standalone crate seam.   |
| `arco-export` (deleted)    | none                        | Removed; LP/MPS writers live in `arco-format`.                |

## KDL boundary status

`arco-kdl` now builds primitive document artifacts directly and does not depend
on solver, runtime, export, concrete adapter, or retired handoff crates. Its
primitive path is document/indexed-data only and does not compile KDL algebra to
a `FrozenModel`.
