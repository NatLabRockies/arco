# Architecture

Arco is in an incremental migration to the primitive-centered architecture in
[`ARCHITECTURE_REFACTOR_PLAN.md`](../../ARCHITECTURE_REFACTOR_PLAN.md). This page
tracks the **currently shipped** crate seams.

## Shipped core shape

```text
Interaction surfaces
  arco-cli
  arco-python ──► arco-blocks (block composition)
        │              │
        └──────┬───────┘
               ▼
arco-ops (transitional facade)
        │
        ├── arco-kdl      KDL parsing + source AST + primitive builders
        ├── arco-model    primitive model + indexed data + document DTOs
        ├── arco-validate model-view validation/reporting
        ├── arco-solver   solver selection/preflight/contracts
        ├── arco-format  portable DTOs + LP/MPS exports over model views
        └── concrete solver adapters (transitional direct wiring)
```

## Crate responsibilities (current)

- `arco-model`: canonical primitive model APIs (`ModelBuilder`, `FrozenModel`,
  `ModelView`, `ModelPatch`), indexed-data primitives, and primitive document
  DTOs.
- `arco-kdl`: KDL parser/AST/diagnostics plus direct primitive builders for
  `FrozenModel`, `IndexedData`, and primitive documents.
- `arco-validate`: user-facing validation over model views.
- `arco-solver`: solver-facing contracts, selection, and preflight.
- `arco-ops`: transitional interaction facade used by CLI, Python core APIs,
  and block composition.
- `arco-blocks`: block composition layer over `arco-ops`; Python imports it only
  for block-specific APIs.

## Transitional seams still present

The old compile/target handoff crates are retired from the active workspace.
`arco-expr` and `arco-contracts` are excluded from active workspace membership
after their APIs were absorbed into `arco-model` and `arco-solver`.
`arco-algebra`, `arco-ir`, `arco-export`, `arco-compile`, and `arco-targets`
have been removed after expression, portable export, and legacy lowering APIs
moved into active primitive/model-view seams.

## Legacy crate retirement boundary (2026-05-06)

Active-workspace dependency snapshot from `cargo metadata --no-deps`:

| Crate                               | Active workspace dependents | Retirement status                                             |
| ----------------------------------- | --------------------------- | ------------------------------------------------------------- |
| `arco-expr` (excluded on disk)      | none                        | Safe boundary: no active dependents.                          |
| `arco-contracts` (excluded on disk) | none                        | Safe boundary: no active dependents.                          |
| `arco-algebra` (deleted)            | none                        | Removed; expression APIs live in `arco-model`.                |
| `arco-compile` (deleted)            | none                        | Removed; legacy lowering internals now live under `arco-ops`. |
| `arco-ir` (deleted)                 | none                        | Removed; portable export DTOs live in `arco-format`.          |
| `arco-targets` (deleted)            | none                        | Removed; target DTOs are no longer a standalone crate seam.   |
| `arco-export` (deleted)             | none                        | Removed; LP/MPS writers live in `arco-format`.                |

## KDL boundary status

`arco-kdl` now builds primitive artifacts directly and does not depend on solver,
runtime, export, concrete adapter, or retired handoff crates.
