# Architecture

Arco ships the primitive-centered architecture described in
[`ARCHITECTURE_REFACTOR_PLAN.md`](../../ARCHITECTURE_REFACTOR_PLAN.md), with
remaining migration debt tracked in ADR chunk checklists. This page tracks the
**currently shipped** crate seams.

## Shipped core shape

```text
Interaction surfaces
  arco-cli
  arco-python ──► arco-blocks (block composition)
        │              │
        └──────┬───────┘
               ▼
arco-ops (stable interaction facade)
        │
        ├── arco-arrays  binding-agnostic labeled-array planning + sparse mask broadcast
        ├── arco-kdl      KDL parsing + source AST + primitive documents
        ├── arco-model    primitive model + indexed data + document DTOs
        ├── arco-validate model-view validation/reporting
        ├── arco-solver   solver selection/preflight/contracts
        ├── arco-format  portable DTOs + LP/MPS exports over model views
        └── concrete solver adapters (adapter-neutral registry wiring)
```

## Crate responsibilities (current)

- `arco-model`: canonical primitive model APIs (`ModelBuilder`, `FrozenModel`,
  `ModelView`, `ModelPatch`), indexed-data primitives, and primitive document
  DTOs.
- `arco-arrays`: reusable labeled-axis array primitives used by bindings for
  axis identity, broadcast planning, sparse active-mask expansion, and
  fail-fast shape validation.
- `arco-kdl`: KDL parser/AST/diagnostics plus document-only primitive builders
  for `IndexedData` and primitive document shells; it does not lower algebra to
  solve-ready models.
- `arco-validate`: user-facing validation over model views.
- `arco-solver`: solver-facing contracts, selection, and preflight.
- `arco-ops`: stable interaction facade used by CLI, Python core APIs,
  and block composition. LP/MPS problem export uses stable ops DTO boundaries;
  KDL document construction remains in `arco-kdl`.
- `arco-blocks`: block composition layer over `arco-ops`; Python imports it only
  for block-specific APIs.

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
