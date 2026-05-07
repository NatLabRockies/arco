# Architecture

Arco is in an incremental migration to the primitive-centered architecture in
[`ARCHITECTURE_REFACTOR_PLAN.md`](../../ARCHITECTURE_REFACTOR_PLAN.md). This page
tracks the **currently shipped** crate seams.

## Shipped core shape

```text
Interaction surfaces
  arco-cli
  arco-python
        │
        ▼
arco-ops (transitional facade)
        │
        ├── arco-kdl      KDL parsing + source AST + primitive builders
        ├── arco-model    primitive model + indexed data + document DTOs
        ├── arco-validate model-view validation/reporting
        ├── arco-solver   solver selection/preflight/contracts
        ├── arco-format / arco-export (format-neutral view + concrete exports)
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
- `arco-ops`: transitional interaction facade used by surface crates.

## Transitional seams still present

The workspace still contains migration-debt crates and couplings from the older
compile/target architecture (`arco-compile`, `arco-targets`, `arco-algebra`,
and legacy export seams). `arco-expr` and `arco-contracts` are excluded from
active workspace membership after their APIs were absorbed into `arco-model` and
`arco-solver`. Remaining seams are being removed chunk-by-chunk per the ARD/plan
and `.sentrux` rules.

## KDL boundary status

`arco-kdl` now builds primitive artifacts directly and does not depend on solver,
runtime, export, concrete adapter, or retired handoff crates.
