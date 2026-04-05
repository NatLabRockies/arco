# Chores

## High Priority

1. Harden FFI solution extraction error handling
   - `crates/arco-highs/src/ffi.rs:582-598` silently falls back with `unwrap_or(0)` for row/column counts.
   - `highs_sys::Highs_getSolution` return status is not checked.
   - Return explicit `HighsModelError` on invalid dimensions/status instead of silently producing empty/partial snapshots.

## Medium Priority

2. Avoid silent fallback when extracting keys
   - `crates/arco-blocks/src/lib.rs:623` and `crates/arco-blocks/src/lib.rs:631` use `unwrap_or_default()` when extracting `String` keys.
   - Return an explicit error instead of defaulting to an empty key.

3. Add missing public API docs
   - Document currently under-documented public items across:
     - `crates/arco-expr` (for example `Expr`, `ConstraintExpr`, `ComparisonSense`, `LinearExprError`)
     - `crates/arco-tools` (for example `MemorySnapshot`, `MemoryProbe`, `MeasurementRecorder`)
     - `crates/arco-highs` (for example `Solution` fields)
     - `crates/arco-blocks` (for example `BlockDag`, `BlockError`)

4. Split large Python binding files
     - `bindings/python/src/arrays.rs` is large and combines multiple array wrappers.
     - Split by type (`variable`, `expr`, `constraint`) to improve maintainability.

5. Add direct tests for basic core types
     - `crates/arco-core/src/types.rs` and `crates/arco-core/src/slack.rs` rely mostly on indirect coverage.
     - Add explicit unit tests for constructors, enum string mappings, and helper methods.

## Low Priority

6. Consolidate repeated error/logging boilerplate in blocks
    - `crates/arco-blocks/src/lib.rs` repeats the same trace-and-return-error pattern many times.
    - Add a small helper or macro for consistent, less noisy error handling.

7. Clarify or remove empty feature flags
    - `arrow` feature flags exist in `arco-core` and `arco-python` but appear empty.
    - Either implement behavior behind these features or remove them to avoid confusion.

8. Modularize benchmark binary
    - `crates/arco-bench/src/main.rs` is large and can be split into modules for scenarios, comparison logic, and reporting.
