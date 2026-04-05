# Reduce Solver Allocations Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate all redundant copies of solution vectors (going from 3 copies to 1) and remove the unnecessary `SolverConfig` clone on the default solve path.

**Architecture:** Bypass the `highs` crate's `get_solution()` method by calling `highs_sys::Highs_getSolution` directly in our FFI layer, allocating solution vectors exactly once into a `SolutionSnapshot` that exposes ownership transfer. Refactor `Solver::solve()` to call `solve_model()` directly to avoid the borrow-conflict-driven config clone. Both changes are internal refactors with no public API changes.

**Tech Stack:** Rust (`arco-highs` crate, `highs-sys` FFI), `cargo test`, `cargo clippy`, `cargo fmt`, `just` task runner.

---

## Global execution rules

- Follow Red -> Green -> Refactor for every code-changing task below.
- Prefer `just` targets when available (`just fmt`, `just test`, `just clippy`).
- Keep commits small and task-scoped.
- Do not batch unrelated file edits across tasks.

## Context

### Issue 7: Solution vector copies

The solution data currently flows through three copies:

| Copy | Location | What | Avoidable? |
|------|----------|------|------------|
| #1 | `highs::SolvedModel::get_solution()` | Allocates 4 `Vec<f64>`, calls `Highs_getSolution` FFI to fill them | See below |
| #2 | `ffi.rs:551-555` `solution_snapshot()` | `.to_vec()` from `highs::Solution` slice accessors into `SolutionSnapshot` | **Yes** |
| #3 | `solver.rs:671-674` `solve_model()` | `.to_vec()` from `SolutionSnapshot` slice accessors into local vars | **Yes** |

Copy #1 is the only *necessary* step -- data must cross the C-to-Rust boundary. But the `highs` crate wraps those Vecs in a `Solution` struct with private fields and slice-only accessors, forcing Copy #2. And our `SolutionSnapshot` repeats the same pattern, forcing Copy #3.

**Fix:** Call `highs_sys::Highs_getSolution` directly in `HighsModel::solution_snapshot()`, allocating our Vecs once and filling them in-place. Then add `SolutionSnapshot::into_vecs()` to transfer ownership into `solve_model()`. Result: **1 allocation, 1 FFI memcpy, 0 Rust-to-Rust copies**.

The `ffi.rs` module already uses `highs_sys` directly in 4 other places (`Highs_getIntInfoValue`, `Highs_version`) following the same `solved.as_ptr()` pattern. This is consistent with the existing approach.

### Issue 8: SolverConfig clone

At `solver.rs:171`, `Solver::solve()` clones `self.config` to work around a borrow conflict:
```rust
pub fn solve(&mut self) -> Result<Solution, SolverError> {
    self.solve_with_config(&self.config.clone()) // clone to avoid &mut self + &self.config conflict
}
```

The clone is ~80 bytes of stack scalars. The `arco_core::Solver` trait impl at line 190 already avoids this by calling `solve_model()` directly. The inherent `solve()` method should do the same.

### Upstream `highs` crate improvement (out of scope, tracked)

The `highs` crate (v1.12.0, also v2.0.0) wraps solution vectors in private fields with slice-only accessors. A small upstream PR to `rust-or/highs` adding consuming accessors would eliminate the need for our direct `highs-sys` call:

```rust
// Proposed addition to highs::Solution
impl Solution {
    pub fn into_columns(self) -> Vec<f64> { self.colvalue }
    pub fn into_dual_columns(self) -> Vec<f64> { self.coldual }
    pub fn into_rows(self) -> Vec<f64> { self.rowvalue }
    pub fn into_dual_rows(self) -> Vec<f64> { self.rowdual }

    /// Consume the solution and return all vectors.
    /// Returns (col_values, col_duals, row_values, row_duals).
    pub fn into_parts(self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        (self.colvalue, self.coldual, self.rowvalue, self.rowdual)
    }
}
```

This is non-breaking, trivial, and semantically correct (the Vecs are freshly allocated in `get_solution()`). File as a GitHub issue or PR against `rust-or/highs` as a follow-up. If accepted, we can revert our direct `highs-sys` call and use the safe API.

---

## Task 1: Rewrite `solution_snapshot()` to call `highs-sys` directly

**Files:**
- Modify: `crates/arco-highs/src/ffi.rs:92-121` (add `new` and `into_vecs` to `SolutionSnapshot`)
- Modify: `crates/arco-highs/src/ffi.rs:540-557` (rewrite `solution_snapshot`)
- Modify: `crates/arco-highs/src/ffi.rs:611-629` (add tests)

**Step 1: Write failing tests (Red)**

Add to the existing `#[cfg(test)] mod tests` block at `crates/arco-highs/src/ffi.rs:611`:

```rust
#[test]
fn test_solution_snapshot_into_vecs() {
    use crate::ffi::SolutionSnapshot;

    let snapshot = SolutionSnapshot::new(
        vec![1.0, 2.0],
        vec![3.0, 4.0],
        vec![5.0],
        vec![6.0],
    );

    // Verify slice accessors still work before consuming
    assert_eq!(snapshot.col_values(), &[1.0, 2.0]);
    assert_eq!(snapshot.col_duals(), &[3.0, 4.0]);
    assert_eq!(snapshot.row_values(), &[5.0]);
    assert_eq!(snapshot.row_duals(), &[6.0]);

    // Consume and verify ownership transfer
    let (col_values, col_duals, row_values, row_duals) = snapshot.into_vecs();
    assert_eq!(col_values, vec![1.0, 2.0]);
    assert_eq!(col_duals, vec![3.0, 4.0]);
    assert_eq!(row_values, vec![5.0]);
    assert_eq!(row_duals, vec![6.0]);
}
```

**Step 2: Run the test to verify it fails**

Run:
```bash
cargo test -p arco-highs ffi::tests::test_solution_snapshot_into_vecs
```
Expected: FAIL -- `SolutionSnapshot::new` and `into_vecs` do not exist yet.

**Step 3: Add `new()` and `into_vecs()` methods (Green)**

In `crates/arco-highs/src/ffi.rs`, add to the `impl SolutionSnapshot` block (after the existing slice accessors at line 120, before the closing `}`):

```rust
/// Create a new SolutionSnapshot from owned vectors.
#[cfg(test)]
pub(crate) fn new(
    col_values: Vec<f64>,
    col_duals: Vec<f64>,
    row_values: Vec<f64>,
    row_duals: Vec<f64>,
) -> Self {
    Self {
        col_values,
        col_duals,
        row_values,
        row_duals,
    }
}

/// Consume the snapshot and return owned vectors.
///
/// Returns `(col_values, col_duals, row_values, row_duals)`.
pub fn into_vecs(self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    (self.col_values, self.col_duals, self.row_values, self.row_duals)
}
```

**Step 4: Run the test to verify it passes**

Run:
```bash
cargo test -p arco-highs ffi::tests::test_solution_snapshot_into_vecs
```
Expected: PASS

**Step 5: Rewrite `solution_snapshot()` to use `highs-sys` directly**

Replace the existing `solution_snapshot` method at `crates/arco-highs/src/ffi.rs:540-557`:

From:

```rust
pub fn solution_snapshot(&self) -> Result<SolutionSnapshot, HighsModelError> {
    let solution = self.solved.as_ref().ok_or(HighsModelError::SolveRequired {
        operation: "solution_snapshot",
    })?;
    let solution = solution.get_solution();

    Ok(SolutionSnapshot {
        col_values: solution.columns().to_vec(),
        col_duals: solution.dual_columns().to_vec(),
        row_values: solution.rows().to_vec(),
        row_duals: solution.dual_rows().to_vec(),
    })
}
```

To:

```rust
pub fn solution_snapshot(&self) -> Result<SolutionSnapshot, HighsModelError> {
    let solved = self.solved.as_ref().ok_or(HighsModelError::SolveRequired {
        operation: "solution_snapshot",
    })?;

    let ptr = solved.as_ptr();

    // Query dimensions directly from HiGHS to allocate exact-sized buffers.
    let num_cols =
        usize::try_from(unsafe { highs_sys::Highs_getNumCol(ptr) }).unwrap_or(0);
    let num_rows =
        usize::try_from(unsafe { highs_sys::Highs_getNumRow(ptr) }).unwrap_or(0);

    let mut col_values = vec![0.0_f64; num_cols];
    let mut col_duals = vec![0.0_f64; num_cols];
    let mut row_values = vec![0.0_f64; num_rows];
    let mut row_duals = vec![0.0_f64; num_rows];

    // Safety: ptr is a valid HiGHS model pointer from SolvedModel.
    // Buffers are sized to num_cols/num_rows as required by Highs_getSolution.
    unsafe {
        highs_sys::Highs_getSolution(
            ptr,
            col_values.as_mut_ptr(),
            col_duals.as_mut_ptr(),
            row_values.as_mut_ptr(),
            row_duals.as_mut_ptr(),
        );
    }

    Ok(SolutionSnapshot {
        col_values,
        col_duals,
        row_values,
        row_duals,
    })
}
```

Note: `Highs_getSolution` and `Highs_getNumCol`/`Highs_getNumRow` all take `*const c_void` in the C API. `solved.as_ptr()` returns `*const c_void`. This matches the pattern already used by `simplex_iteration_count()` and other methods in this file.

**Step 6: Run all existing tests**

Run:
```bash
cargo test -p arco-highs --all-features
```
Expected: All tests PASS. The `ffi_smoke` integration tests exercise `solution_snapshot()` end-to-end and will catch any regression.

**Step 7: Run clippy**

Run:
```bash
cargo clippy -p arco-highs --all-features --tests -- -D warnings
```
Expected: No warnings.

**Step 8: Commit**

```
perf(arco-highs): bypass highs crate to allocate solution vectors once

Call highs_sys::Highs_getSolution directly instead of going through
highs::SolvedModel::get_solution(), which allocates 4 Vec<f64> that
we could only access via slice copies. This eliminates one full
redundant copy of all solution vectors per solve.
```

---

## Task 2: Use `into_vecs()` in `solve_model()` to eliminate the remaining copy

**Files:**
- Modify: `crates/arco-highs/src/solver.rs:664-674`

**Step 1: Replace `.to_vec()` calls with `into_vecs()`**

Change `crates/arco-highs/src/solver.rs:664-674` from:

```rust
// Extract solution
let snapshot = highs_model
    .solution_snapshot()
    .map_err(highs_model_error_to_solver_error)?;
let objective_value = highs_model
    .objective_value()
    .map_err(highs_model_error_to_solver_error)?;
let primal_values = snapshot.col_values().to_vec();
let variable_duals = snapshot.col_duals().to_vec();
let constraint_duals = snapshot.row_duals().to_vec();
let row_values = snapshot.row_values().to_vec();
```

To:

```rust
// Extract solution
let snapshot = highs_model
    .solution_snapshot()
    .map_err(highs_model_error_to_solver_error)?;
let objective_value = highs_model
    .objective_value()
    .map_err(highs_model_error_to_solver_error)?;
let (primal_values, variable_duals, row_values, constraint_duals) = snapshot.into_vecs();
```

**Important:** `into_vecs()` returns `(col_values, col_duals, row_values, row_duals)`. The mapping is:
- `col_values` -> `primal_values`
- `col_duals` -> `variable_duals`
- `row_values` -> `row_values`
- `row_duals` -> `constraint_duals`

So the destructuring must be: `(primal_values, variable_duals, row_values, constraint_duals)`.

**Step 2: Run existing tests**

Run:
```bash
cargo test -p arco-highs --all-features
```
Expected: All tests PASS.

**Step 3: Run clippy**

Run:
```bash
cargo clippy -p arco-highs --all-features --tests -- -D warnings
```
Expected: No warnings.

**Step 4: Commit**

```
perf(arco-highs): transfer solution vector ownership instead of cloning

Use SolutionSnapshot::into_vecs() in solve_model() to move solution
vectors directly instead of copying via .to_vec(). Combined with the
direct highs-sys call, solution data now goes from C memory to the
final Solution struct with exactly one allocation and zero Rust copies.
```

---

## Task 3: Remove `SolverConfig` clone in `Solver::solve()`

**Files:**
- Modify: `crates/arco-highs/src/solver.rs:169-172`

**Step 1: Refactor `solve()` to call `solve_model()` directly**

Change `crates/arco-highs/src/solver.rs:169-172` from:

```rust
/// Solve the model and return the solution
pub fn solve(&mut self) -> Result<Solution, SolverError> {
    self.solve_with_config(&self.config.clone())
}
```

To:

```rust
/// Solve the model and return the solution
pub fn solve(&mut self) -> Result<Solution, SolverError> {
    solve_model(
        &self.model,
        &self.config,
        self.primal_start.as_deref(),
        self.use_async_crs,
    )
}
```

This matches the pattern already used in the `arco_core::Solver` trait impl at line 190-195. By calling the free function `solve_model()` directly instead of going through `self.solve_with_config()`, there is no `&mut self` receiver borrow to conflict with `&self.config`.

**Step 2: Run existing tests**

Run:
```bash
cargo test -p arco-highs --all-features
```
Expected: All tests PASS.

**Step 3: Run clippy**

Run:
```bash
cargo clippy -p arco-highs --all-features --tests -- -D warnings
```
Expected: No warnings.

**Step 4: Run integration tests specifically**

Run:
```bash
cargo test -p arco-highs --all-features --test integration --test ffi_smoke
```
Expected: All PASS.

**Step 5: Commit**

```
refactor(arco-highs): remove unnecessary SolverConfig clone in Solver::solve()

Call solve_model() directly instead of going through solve_with_config(),
avoiding the borrow conflict that required cloning self.config. This
matches the pattern already used in the trait implementations.
```

---

## Task 4: Final validation

**Step 1: Run full crate test suite**

Run:
```bash
just test
```
Expected: All workspace tests PASS.

**Step 2: Run full lint suite**

Run:
```bash
just clippy
```
Expected: No warnings.

**Step 3: Format check**

Run:
```bash
cargo fmt --check
```
Expected: No formatting issues.

---

## Summary of changes

| Change | File | Effect |
|--------|------|--------|
| Rewrite `solution_snapshot()` to call `highs-sys` directly | `ffi.rs:540-557` | Eliminates Copy #1 + #2 (allocate once, fill via FFI) |
| Add `SolutionSnapshot::into_vecs()` | `ffi.rs:101-121` | Enables ownership transfer of solution vectors |
| Use `into_vecs()` in `solve_model()` | `solver.rs:671-674` | Eliminates Copy #3 (move instead of clone) |
| Refactor `Solver::solve()` | `solver.rs:169-172` | Removes config clone, matches existing trait impl pattern |

**Before:** 3 allocations + 3 memcpys of `(2C + 2R) * 8` bytes per solve.
**After:** 1 allocation + 1 FFI memcpy. Zero Rust-to-Rust copies.

**Upstream follow-up:** File issue/PR on `rust-or/highs` to add `Solution::into_parts()`. If accepted, we can replace the direct `highs-sys` call with the safe API.
