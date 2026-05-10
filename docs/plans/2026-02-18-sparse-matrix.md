# Sparse Matrix Export Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
> **Goal:** Implement trait-based sparse matrix exports (CSC/CRS/COO) in `arco-core`, migrate all internal consumers, remove the `rows()` allocation path, and lock in memory/perf regressions with benchmark gates.
> **Architecture:** Add a dedicated sparse export module in `arco-core` with a `Model` trait implementation that emits exact-sized SoA buffers for CSC/CRS/COO. Migrate callers (`pretty`, Python bindings, bench) to consume these exports directly so row materialization is eliminated from hot paths. Extend benchmark staging and gating to enforce total-stage and sparse-stage regressions.
> **Tech Stack:** Rust workspace crates (`arco-core`, `arco-bench`, `arco-python`), PyO3 bindings, `just` task runner, cargo test/clippy/fmt, bench JSONL compare tooling.

---

## Global execution rules

- Follow Red -> Green -> Refactor for every code-changing task below.
- Prefer `just` targets when available (`just fmt`, `just test`, `just clippy`, `just bench-run`, `just bench-compare`).
- Keep commits small and task-scoped.
- Do not batch unrelated file edits across tasks.

## Task 1: Add sparse export API tests in core (Red)

**Files:**

- Modify: `crates/arco-core/src/model/mod.rs`
- Create: `crates/arco-core/src/model/tests/sparse_export.rs`

**Step 1 (Red):** Add failing tests that call planned APIs:

- `model.export_csc()` shape/ptr/index/value assertions.
- `model.export_crs()` row pointer monotonicity and row grouping assertions.
- `model.export_coo()` `(row,col,value)` triplet assertions.

Run:

```bash
cargo test -p arco-core sparse_export
```

Expected:

- FAIL with missing methods/types (for example `no method named export_csc`).

**Step 2 (Commit suggestion):**

- `test(arco-core): add failing sparse export api coverage`

## Task 2: Implement sparse module skeleton + CSC (Green)

**Files:**

- Create: `crates/arco-core/src/model/sparse.rs`
- Modify: `crates/arco-core/src/model/mod.rs`

**Step 1 (Green):** Add minimal compile path:

- Define sparse structs (`CscMatrix`, `CrsMatrix`, `CooMatrix`) and `SparseMatrixExport` trait.
- Wire module in `mod.rs` and re-export public sparse types.
- Implement `export_csc()` with exact-size allocation (`nnz`, `num_variables + 1`).

Run:

```bash
cargo test -p arco-core sparse_export
```

Expected:

- Some tests pass (CSC), CRS/COO tests still fail.

**Step 2 (Refactor):**

- Extract shared shape/nnz helpers in `sparse.rs` without changing behavior.

Run:

```bash
cargo test -p arco-core sparse_export
```

Expected:

- Same pass/fail profile as before refactor, no new failures.

**Step 3 (Commit suggestion):**

- `feat(arco-core): add sparse export trait and csc implementation`

## Task 3: Implement CRS and COO exact-allocation builders (Green)

**Files:**

- Modify: `crates/arco-core/src/model/sparse.rs`
- Modify: `crates/arco-core/src/model/tests/sparse_export.rs`

**Step 1 (Red):** Extend tests for edge conditions:

- Empty model.
- Empty rows/columns interleaved with non-empty ones.
- Pointer last element equals `nnz`.

Run:

```bash
cargo test -p arco-core sparse_export
```

Expected:

- FAIL for missing/incorrect CRS/COO behavior.

**Step 2 (Green):** Implement:

- `export_crs()` via row-count -> prefix-sum -> fill cursor pass.
- `export_coo()` single pass over columns writing row/col/value arrays.

Run:

```bash
cargo test -p arco-core sparse_export
```

Expected:

- PASS for sparse export test module.

**Step 3 (Refactor):**

- Tighten integer conversions and internal helper names; keep no `unwrap`/`expect` in production paths.

Run:

```bash
cargo test -p arco-core sparse_export
```

Expected:

- PASS.

**Step 4 (Commit suggestion):**

- `feat(arco-core): implement crs and coo sparse exports`

## Task 4: Migrate pretty-printer off `rows()` (TDD)

**Files:**

- Modify: `crates/arco-core/src/model/pretty.rs`
- Modify: `crates/arco-core/src/model/tests/sparse_export.rs`

**Step 1 (Red):** Add a regression test asserting pretty output is unchanged for a model with sparse/non-contiguous coefficients while using new row representation semantics.

Run:

```bash
cargo test -p arco-core pretty::tests::format_ascii_supports_adapter_labels_and_sections
```

Expected:

- FAIL once test expectation includes sparse path assumptions.

**Step 2 (Green):** Replace `self.rows()` usage in `pretty.rs` with `self.export_crs()` and iterate row slices via `row_ptrs`/`col_indices`/`values`.

Run:

```bash
cargo test -p arco-core pretty::tests
```

Expected:

- PASS with identical rendered output.

**Step 3 (Refactor):**

- Small helper for row slice extraction to keep formatting loop readable.

Run:

```bash
cargo test -p arco-core pretty::tests
```

Expected:

- PASS.

**Step 4 (Commit suggestion):**

- `refactor(arco-core): switch pretty rendering to crs export`

## Task 5: Migrate Python `export_csc` and `export_crs`, add `export_coo` (TDD)

**Files:**

- Modify: `bindings/python/src/lib.rs`
- Modify: `bindings/python/scripts/test_docs_doctest.py` (if doctests cover export methods)

**Step 1 (Red):** Add/adjust doctest expectations for:

- `Model.export_csc()` schema unchanged.
- `Model.export_crs()` schema unchanged.
- New `Model.export_coo()` schema (`rows`, `cols`, `values`, `shape`).

Run:

```bash
just py-test
```

Expected:

- FAIL on missing `export_coo` or mismatched key names.

**Step 2 (Green):** Update PyO3 methods in `lib.rs` to call `self.inner.export_csc()/export_crs()/export_coo()` and marshal arrays to Python dicts.

Run:

```bash
just py-test
```

Expected:

- PASS doctests.

**Step 3 (Refactor):**

- Extract shared dict-assembly helper to avoid duplicate shape/key wiring.

Run:

```bash
just py-test
```

Expected:

- PASS.

**Step 4 (Commit suggestion):**

- `feat(arco-python): migrate sparse exports to core trait and add coo`

## Task 6: Migrate bench CSC extraction and add sparse stage measurements (TDD)

**Files:**

- Modify: `crates/arco-bench/src/main.rs`

**Step 1 (Red):** Add/extend bench unit tests for comparison/gating stage names to include:

- `export_csc`
- `export_crs`
- `export_coo`

Run:

```bash
cargo test -p arco-bench compare_detects_regressions
```

Expected:

- FAIL due to missing sparse stage records.

**Step 2 (Green):**

- Replace `extract_csc_matrix` manual loop with `model.export_csc()` mapping.
- In `execute_case`, record dedicated timed+RSS stages for `export_csc`, `export_crs`, `export_coo`.

Run:

```bash
cargo test -p arco-bench
```

Expected:

- PASS bench tests.

**Step 3 (Refactor):**

- Consolidate repeated export-stage measurement logic into helper function.

Run:

```bash
cargo test -p arco-bench
```

Expected:

- PASS.

**Step 4 (Commit suggestion):**

- `perf(arco-bench): add sparse export stages and core csc extraction`

## Task 7: Add benchmark gates for total + sparse stages (TDD)

**Files:**

- Modify: `justfile`

**Step 1 (Red):** Update bench gate recipe expectations to require sparse export stages and run compare per stage.

Run:

```bash
just --list
```

Expected:

- Existing targets visible; new/updated gate target not yet functional.

**Step 2 (Green):**

- Add or update a recipe that checks these stages: `total`, `export_csc`, `export_crs`, `export_coo`.
- Use `cargo run -p arco-bench -- compare --stage <stage> ...` for each stage.

Run:

```bash
just bench-gate artifacts/base.jsonl artifacts/candidate.jsonl 5 0
```

Expected:

- Command structure validates; fails only if files missing or thresholds violated.

**Step 3 (Refactor):**

- Keep stage list in one place in recipe for maintainability.

Run:

```bash
just --list
```

Expected:

- PASS, recipe still listed.

**Step 4 (Commit suggestion):**

- `ci(bench): gate total and sparse export stages`

## Task 8: Remove `rows()` allocation path (TDD)

**Files:**

- Modify: `crates/arco-core/src/model/storage.rs`
- Modify: `crates/arco-core/src/model/pretty.rs`
- Modify: `bindings/python/src/lib.rs`

**Step 1 (Red):** Search and assert no required runtime call sites remain for `rows()`.

Run:

```bash
rg "\.rows\(" crates/arco-core bindings/python crates/arco-bench
```

Expected:

- Shows remaining references before deletion.

**Step 2 (Green):**

- Remove `Model::rows()` from `storage.rs`.
- Resolve compile errors by fully switching remaining call sites to sparse exports.

Run:

```bash
cargo test -p arco-core
```

Expected:

- PASS.

**Step 3 (Refactor):**

- Trim any now-dead imports or helper code left by the migration.

Run:

```bash
just clippy
```

Expected:

- PASS with `-D warnings`.

**Step 4 (Commit suggestion):**

- `refactor(arco-core): remove dense rows allocation api`

## Task 9: Update chores tracking entry (non-code hygiene)

**Files:**

- Modify: `chores.md`

**Step 1:** Update Medium Priority item about row allocation to reflect completion (mark done or remove and renumber consistently).

Run:

```bash
rg "Reduce dense matrix allocation in row extraction|rows\(\)" chores.md
```

Expected:

- Updated text reflects sparse export migration and removal of `rows()` path.

**Step 2 (Commit suggestion):**

- `docs(chores): mark rows allocation chore complete`

## Task 10: Full validation and formatting gate

**Files:**

- Modify: `crates/arco-core/src/model/sparse.rs`
- Modify: `crates/arco-core/src/model/mod.rs`
- Modify: `crates/arco-core/src/model/storage.rs`
- Modify: `crates/arco-core/src/model/pretty.rs`
- Modify: `crates/arco-core/src/model/tests/sparse_export.rs`
- Modify: `bindings/python/src/lib.rs`
- Modify: `crates/arco-bench/src/main.rs`
- Modify: `justfile`
- Modify: `chores.md`

**Step 1:** Format code.

Run:

```bash
just fmt
```

Expected:

- PASS, no formatting diffs left.

**Step 2:** Run focused Rust tests first.

Run:

```bash
cargo test -p arco-core sparse_export && cargo test -p arco-core pretty::tests && cargo test -p arco-bench
```

Expected:

- PASS.

**Step 3:** Run Python doctest workflow.

Run:

```bash
just py-test
```

Expected:

- PASS.

**Step 4:** Run strict lint gate.

Run:

```bash
just clippy
```

Expected:

- PASS with no warnings.

**Step 5 (Commit suggestion):**

- `chore: run final fmt clippy and targeted tests for sparse export migration`

## Task 11: Benchmark baseline/candidate capture and regression gates

**Files:**

- No code changes required (artifacts only).

**Step 1:** Capture baseline on current main/reference branch.

Run:

```bash
cargo run -p arco-bench -- run --scenario model-build,fac25 --output artifacts/bench/sparse-baseline.jsonl
```

Expected:

- Produces `artifacts/bench/sparse-baseline.jsonl` with `total` stage records.

**Step 2:** Capture candidate on sparse-export branch.

Run:

```bash
cargo run -p arco-bench -- run --scenario model-build,fac25 --output artifacts/bench/sparse-candidate.jsonl
```

Expected:

- Produces `artifacts/bench/sparse-candidate.jsonl` with `total`, `export_csc`, `export_crs`, `export_coo` records.

**Step 3:** Enforce total-stage gate (duration <= +5%, memory <= +0%).

Run:

```bash
cargo run -p arco-bench -- compare --baseline artifacts/bench/sparse-baseline.jsonl --candidate artifacts/bench/sparse-candidate.jsonl --stage total --duration-threshold-pct 5 --memory-threshold-pct 0 --format table
```

Expected:

- PASS (exit 0); otherwise block merge.

**Step 4:** Enforce sparse-stage memory non-regression gates.

Run:

```bash
cargo run -p arco-bench -- compare --baseline artifacts/bench/sparse-baseline.jsonl --candidate artifacts/bench/sparse-candidate.jsonl --stage export_csc --duration-threshold-pct 5 --memory-threshold-pct 0 --format table && cargo run -p arco-bench -- compare --baseline artifacts/bench/sparse-baseline.jsonl --candidate artifacts/bench/sparse-candidate.jsonl --stage export_crs --duration-threshold-pct 5 --memory-threshold-pct 0 --format table && cargo run -p arco-bench -- compare --baseline artifacts/bench/sparse-baseline.jsonl --candidate artifacts/bench/sparse-candidate.jsonl --stage export_coo --duration-threshold-pct 5 --memory-threshold-pct 0 --format table
```

Expected:

- PASS (exit 0 for all); any failure blocks merge and requires investigation.

**Step 5:** Optional consolidated gate via `just` recipe.

Run:

```bash
just bench-gate artifacts/bench/sparse-baseline.jsonl artifacts/bench/sparse-candidate.jsonl 5 0
```

Expected:

- PASS across `total` and sparse stages.

**Step 6 (Commit suggestion):**

- `perf(bench): record and gate sparse export regression metrics`

---

## Final merge criteria checklist

- `Model` exposes trait-based `export_csc`, `export_crs`, `export_coo` from `arco-core`.
- `pretty.rs`, Python exports, and bench extraction use sparse exports (no `rows()` path).
- `Model::rows()` is removed.
- `justfile` benchmark gating covers `total` + sparse export stages.
- `chores.md` row-allocation entry updated to reflect completion.
- All required format/lint/tests pass.
- Benchmark thresholds pass: total duration <= +5%, memory <= +0%; sparse stages memory <= +0%.
