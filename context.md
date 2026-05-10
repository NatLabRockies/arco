# Code Context

## Files Retrieved

1. `ARCHITECTURE_REFACTOR_PLAN.md` (lines 1-260) - target-state crate architecture, seam responsibilities, and migration phases.
2. `Cargo.toml` (lines 1-60) - actual workspace members/default-members and shared workspace dependency aliases.

## Key Code

- Workspace members: `crates/*` plus `bindings/python`.
- Default members in `Cargo.toml`: `arco-algebra`, `arco-blocks`, `arco-cli`, `arco-compile`, `arco-contracts`, `arco-core`, `arco-exchange`, `arco-export`, `arco-expr`, `arco-highs`, `arco-ir`, `arco-kdl`, `arco-model`, `arco-ops`, `arco-runtime`, `arco-scip`, `arco-solver`, `arco-solver-types`, `arco-targets`, `arco-tools`, `arco-validate`, `arco-xpress`.
- Workspace crates found under `crates/`: `arco-algebra`, `arco-blocks`, `arco-cli`, `arco-compile`, `arco-contracts`, `arco-core`, `arco-exchange`, `arco-export`, `arco-expr`, `arco-highs`, `arco-ipopt`, `arco-ir`, `arco-kdl`, `arco-model`, `arco-ops`, `arco-runtime`, `arco-scip`, `arco-solver`, `arco-solver-types`, `arco-targets`, `arco-tools`, `arco-validate`, `arco-xpress`.
- Plan-only target crates not present in the workspace list: `arco-json`, `arco-yaml`, `arco-python`.
- Plan highlights missing/important seams: `arco-targets`, `arco-contracts`, `arco-ops`, `arco-validate`.

## Architecture

- The plan centers the canonical model (`arco-model`, `arco-algebra`, `arco-blocks`, `arco-validate`) and makes compilation the only bridge out (`arco-compile` -> `arco-targets` / `arco-ir`).
- Exchange is intended to consume portable IR only (`arco-exchange` -> `arco-ir`).
- Solver adapters (`arco-highs`, `arco-ipopt`, `arco-xpress`, `arco-scip`) should depend on shared contracts/targets/runtime rather than on each other.
- Interaction surfaces (`arco-cli`, planned `arco-python`) should go through `arco-ops`.

## Start Here

`Cargo.toml` first, because it defines the current workspace shape; then `ARCHITECTURE_REFACTOR_PLAN.md` to compare that shape with the intended target architecture.

## Planning Terms

- KDL file composition: textual inclusion of declarations from other `.kdl` files so users can split model constraints and related declarations across files instead of keeping a whole model in one file. This is distinct from scenario `use` (scenario-to-model reference) and ergonomic `use_data` (model-to-data-block import).
- Model fragment: a `.kdl` file whose declarations are target-agnostic. The including site decides where the fragment lands, so reusable fragments do not name the model they extend.
- Composition should support both top-level includes for shared data/schema declarations and model-scope includes for constraint/control/expression/objective fragments.
- Include paths should resolve relative to the entrypoint KDL file's parent directory, matching existing CSV `source` path resolution.
- Nested includes are intentionally not supported: only the entrypoint KDL file may contain include declarations.
- Include composition preserves existing duplicate-name validation: included declarations are merged into the entrypoint program/model, and duplicate constraints or other duplicate declaration names remain validation errors rather than overrides.
- Include expansion should process entrypoint include declarations in source order and splice parsed declarations at the include site for deterministic diagnostics and O(total declarations) behavior.
- Included fragments are for reusable schema/model pieces, not scenario selection. Fragment files must not contribute `scenario` declarations.
- Fragment files should be normal `.kdl` documents with declarations directly, not wrapped in a special `fragment { ... }` node.
