# Defensive Programming Patterns Assessment - Arco Rust Codebase

## Summary

This assessment identified defensive programming patterns that hide errors and reduce code clarity. The following changes have been implemented to replace defensive patterns with explicit error handling where appropriate.

## Changes Implemented

### 1. Presolve Stats Functions - `unwrap_or(0)` → `Option<u64>`

**Files Modified:**

- `crates/arco-highs/src/ffi.rs`
- `crates/arco-highs/src/solver.rs`

**Change:** Changed `presolved_num_rows()`, `presolved_num_cols()`, and `presolved_num_nz()` to return `Option<u64>` instead of `u64` with `unwrap_or(0)`.

**Rationale:** Returning 0 when presolve info is unavailable makes it indistinguishable from actual zero rows/columns. Using `Option<u64>` properly signals unavailable data.

### 2. Filter Evaluation - `_ => false` → Explicit Error Handling

**Files Modified:**

- `crates/arco-cli/src/execution.rs`

**Change:** Renamed `eval_filter` to `try_eval_filter` and changed return type from `bool` to `Option<bool>`. Callers now handle `None` explicitly with warning logs.

**Rationale:** Silently failing closed when filter expressions can't be evaluated hides bugs. Now unsupported expressions are logged as warnings before filtering out the row.

### 3. Status Mapping - `_ => Unknown` → Logging for Unknown Codes

**Files Modified:**

- `crates/arco-xpress/src/status.rs`
- `crates/arco-ipopt/src/status.rs`
- `crates/arco-highs/src/ffi.rs`

**Change:** Replaced `_ =>` catch-all arms with named captures (`unknown =>`) that log debug messages before returning `Unknown`.

**Rationale:** Unknown status codes from external solvers could indicate bugs or new statuses. Logging them helps with debugging without breaking production.

## Patterns Kept (Acceptable Defensive Programming)

These patterns were identified but kept as they serve a legitimate purpose:

### 1. `catch_unwind` in Tests

**Location:** `crates/arco-highs/src/solver.rs`

Kept because these are tests specifically verifying that the solver doesn't panic with invalid config. This is intentional testing of panic boundaries.

### 2. `unwrap_or(f64::NAN)` for Logging

**Location:** `crates/arco-highs/src/solver.rs`

Kept because NaN is an appropriate sentinel for missing objective values in logging contexts.

### 3. `unwrap_or(f64::INFINITY)` for Bounds

**Locations:** `crates/arco-cli/src/debug.rs`, `crates/arco-cli/src/execution.rs`

Kept because infinity is the mathematically correct representation of unbounded variables.

### 4. Auto-Generated Constraint Names

**Location:** `crates/arco-kdl/src/source/parser_constraints.rs`

Kept because auto-generated names like `constraint_1` are a documented feature for unnamed constraints.

### 5. Default Version Strings

**Location:** `crates/arco-blocks/src/resolve.rs`

Kept because `0.0.0` is a sensible default for optional version fields.

### 6. Test-Only unwrap()/expect()

Kept because test code is allowed to panic on assertion failures - that's what tests do.

## Summary of Philosophy

The key principle applied: **Error handling should be used for unknown/unsanitized input, IO operations, and external calls - but not to mask internal bugs.**

- External solver status codes → Logged but handled gracefully (expected unknowns)
- Missing presolve stats → Return Option (proper signal of unavailable data)
- Invalid filter expressions → Log warning (likely user error, should be visible)
- Internal invariant violations → These should propagate, not be caught with defaults
