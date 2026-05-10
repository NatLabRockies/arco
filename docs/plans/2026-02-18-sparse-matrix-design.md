# Sparse Matrix Abstractions Design

## Context and Pain Point

`Model::rows()` in `crates/arco-core/src/model/storage.rs` currently materializes
`Vec<Vec<(VariableId, f64)>>` for every row-oriented consumer. This has three
memory costs that show up on large models:

1. Dense outer allocation: one `Vec` per constraint even when many rows are
   empty.
2. Nested allocator overhead: many small row allocations and growth events.
3. Tuple storage overhead: `(VariableId, f64)` arrays introduce per-element
   padding/alignment waste compared with split arrays.

This path is used by `pretty.rs` and Python `export_crs`, and it duplicates work
already implied by column-first model storage.

## Goals

- Make sparse exports memory-efficient first, while keeping APIs practical.
- Support three sparse formats from a shared abstraction: CSC, CRS, COO.
- Use exact-sized allocations for export buffers (no geometric growth).
- Prefer borrowing where possible to avoid copying source model data.
- Remove `Model::rows()` from hot/large export paths.

## Non-goals

- Replacing the model's internal storage format in this change.
- Redesigning solver integration beyond export boundary changes.
- Optimizing pretty-print rendering output itself (only its matrix extraction).

## Chosen Approach: Trait-Based Sparse Exports

Add a sparse export layer in `arco-core` with trait-based format views/builders.

Proposed shape (naming can follow repo conventions):

- `SparseMatrixExport` trait implemented for `Model`.
- Format structs:
  - `CscMatrix { ptrs: Vec<usize>, indices: Vec<u32>, values: Vec<f64>, shape }`
  - `CrsMatrix { ptrs: Vec<usize>, indices: Vec<u32>, values: Vec<f64>, shape }`
  - `CooMatrix { rows: Vec<u32>, cols: Vec<u32>, values: Vec<f64>, shape }`
- Methods:
  - `export_csc(&self) -> CscMatrix`
  - `export_crs(&self) -> CrsMatrix`
  - `export_coo(&self) -> CooMatrix`
  - optional `*_into(&self, workspace: &mut SparseWorkspace)` variants for
    allocation reuse in repeated calls.

This keeps format-specific code explicit while sharing counting, shape, and
validation logic.

## Memory Layout Decisions

Use struct-of-arrays layout for all exported formats:

- Index arrays are `u32` (`row_indices`, `col_indices`, `rows`, `cols`).
- Value arrays are `f64`.
- Pointer arrays are `usize` (`col_ptrs`/`row_ptrs`) for native indexing.
- Do not store sparse entries as tuple arrays (for example, `Vec<(u32, f64)>`)
  because alignment padding wastes memory and hurts cache density.

Expected benefit: tighter packed arrays, fewer allocations, simpler FFI handoff
to Python/solver bindings.

## Borrowing Strategy

- Source data is borrowed from model columns during export passes; we do not
  clone per-column vectors up front.
- CSC export iterates `self.columns()` directly and writes into owned output
  buffers.
- CRS/COO exports read borrowed columns and build owned arrays via counting +
  fill passes.
- Add workspace-based APIs for repeated exports to reuse allocated capacity in
  long-running processes (bench tools, Python workflows).

## Exact-Allocation Strategy

For every export, derive exact sizes before writing payload arrays:

- `nnz = self.num_coefficients()`.
- CSC:
  - allocate `col_ptrs` with exact length `num_variables + 1`.
  - allocate `row_indices` and `values` with exact length/capacity `nnz`.
- CRS:
  1. Count entries per row into `row_counts: Vec<usize>` (length
     `num_constraints`).
  2. Prefix-sum into `row_ptrs` (length `num_constraints + 1`).
  3. Allocate `col_indices` and `values` at exact `nnz`.
  4. Fill via `next = row_ptrs[..num_constraints].to_vec()` cursors.
- COO:
  - allocate `rows`, `cols`, `values` at exact `nnz` and append once.

No `push` into under-capacity vectors that may reallocate; use exact target
length or exact reserved capacity from precomputed counts.

## Migration Plan

1. `arco-core` sparse layer
   - Add module (for example `crates/arco-core/src/model/sparse.rs`) with trait,
     matrix structs, and builders.
   - Keep `Model::columns()` as the borrowed source iterator.
   - Mark `Model::rows()` as deprecated (temporary) after consumers migrate.

2. `pretty.rs` migration
   - Replace `let rows = self.rows();` with CRS export usage.
   - Consume row pointers + index/value arrays directly when formatting each
     constraint line.
   - Preserve formatting behavior and truncation semantics.

3. Python `export_csc` / `export_crs`
   - Replace ad-hoc buffer construction in `bindings/python/src/lib.rs` with
     calls to core sparse export APIs.
   - Keep dict schema unchanged (`col_ptrs`/`row_indices`/`values` and
     `row_ptrs`/`col_indices`/`values`).
   - Add `export_coo` in the same style once trait is in place.

4. Bench extraction
   - Replace `extract_csc_matrix` manual loop in
     `crates/arco-bench/src/main.rs` with `model.export_csc()`.
   - Add optional CRS/COO extraction stage measurements for regression tracking.

5. Cleanup
   - Remove `Model::rows()` after all internal callers are migrated and tests
     are updated.

## Benchmarking and Regression Gates

Use `arco-bench` artifacts as the acceptance gate.

Benchmark strategy:

- Capture baseline artifact from main for `model-build` (all default cases) and
  `fac25`.
- Capture candidate artifact with sparse-export changes.
- Compare both runtime and memory for total + sparse export stages.

Required gates:

- No memory regression on `model-build total` and `fac25 total`
  (`rss_delta_bytes` must not increase).
- Sparse export stages (`export_csc`, `export_crs`, `export_coo` once added)
  must show lower or equal RSS delta than baseline.
- Duration regression cap: at most +5% on `total` stage; if exceeded, block
  merge unless memory win is significant and explicitly approved.

Suggested commands:

- `just bench-run` (or `cargo run -p arco-bench -- run --output <artifact.jsonl>`).
- `cargo run -p arco-bench -- compare --baseline <base.jsonl> --candidate <cand.jsonl> --stage total --duration-threshold-pct 5 --memory-threshold-pct 0`

## Implementation Notes for This Repo

- Keep this change centered in `arco-core` and make bindings/bench consume the
  shared API to avoid format drift.
- Preserve Python output contracts and pretty-printer output text exactly.
- Add regression tests for sparse array shapes and pointer monotonicity in
  `arco-core`.
- Prefer small, reviewable commits: core export API first, then each consumer
  migration.
