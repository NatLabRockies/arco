# Xpress Solver Backend Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add FICO Xpress as a third solver backend for arco, supporting LP, MIP, and QP problems.

**Architecture:** New `arco-xpress` crate with hand-written FFI bindings to `libxprs` via `XPRESSDIR` env var. Mirrors the `arco-ipopt` pattern: `ffi.rs` for C bindings, `solver.rs` for `Solver`+`XpressBackend`, `solution.rs` for `SolutionView`, `status.rs` for status mapping. Python bindings wire up via the existing `PyXpress` class and feature-gated `resolve_backend()`.

**Tech Stack:** Rust FFI (`extern "C"`), `libxprs` shared library, PyO3, arco-solver traits

**Validation workflow (run after every task):**
1. `just fmt`
2. `just clippy-solver arco-xpress` (or `just clippy` for full workspace)
3. `just test-solver arco-xpress` (or relevant `cargo test` subset)

**Style rules (from AGENTS.md):**
- Red-green-refactor: failing test first, then minimal implementation, then clean up
- No `unwrap`/`expect` in production code; proper `Result` propagation
- `unsafe` blocks isolated, minimal, and documented with `// SAFETY:` comments
- Performance-conscious: avoid unnecessary allocations, prefer zero-copy when possible
- Conventional Commits (`feat:`, `fix:`, `test:`, `docs:`)
- Use `just` targets for build/lint/test

---

### Task 1: Create arco-xpress crate skeleton

**Files:**
- Create: `crates/arco-xpress/Cargo.toml`
- Create: `crates/arco-xpress/build.rs`
- Create: `crates/arco-xpress/src/lib.rs`
- Modify: `Cargo.toml` (workspace root, line ~17)

**Step 1: Create `crates/arco-xpress/Cargo.toml`**

```toml
[package]
name = "arco-xpress"
version = { workspace = true }
description = "FICO Xpress solver backend for Arco optimization"
edition = { workspace = true }
rust-version = { workspace = true }
homepage = { workspace = true }
repository = { workspace = true }
authors = { workspace = true }
license-file = { workspace = true }
publish = false

[lints]
workspace = true

[dependencies]
arco-core = { workspace = true }
arco-expr = { workspace = true }
arco-solver = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tracing-subscriber = { workspace = true }
```

**Step 2: Create `crates/arco-xpress/build.rs`**

No `unwrap`/`expect` — use `panic!` with clear messages only in the build script (build scripts are not production code and `panic!` is the standard error reporting mechanism).

```rust
fn main() {
    let xpress_dir = std::env::var("XPRESSDIR").unwrap_or_else(|_| {
        for path in &["/opt/xpressmp", "/Library/xpressmp", "C:\\xpressmp"] {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }
        panic!(
            "XPRESSDIR environment variable not set and Xpress not found in default locations. \
             Set XPRESSDIR to your Xpress installation directory."
        );
    });

    let lib_dir = format!("{xpress_dir}/lib");
    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-lib=dylib=xprs");
    println!("cargo:rerun-if-env-changed=XPRESSDIR");
}
```

**Step 3: Create stub source files**

Create `crates/arco-xpress/src/lib.rs`:
```rust
//! FICO Xpress solver backend for Arco optimization.
//!
//! This crate provides a bridge from `arco-core::Model` to the FICO Xpress
//! solver. It supports LP, MIP, and QP problems.
//!
//! Xpress is a commercial solver and requires a valid license.

pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

pub use solution::Solution;
pub use solver::{Solver, XpressBackend};
```

Create empty stubs for `ffi.rs`, `solver.rs`, `solution.rs`, `status.rs` with enough content to compile (empty structs, type aliases, etc.).

**Step 4: Add workspace dependency to root `Cargo.toml`**

Add after `arco-ipopt` line (~17):
```toml
arco-xpress = { path = "crates/arco-xpress" }
```

**Step 5: Validate**

```bash
just fmt
just clippy-solver arco-xpress
```

Note: Will only work if `XPRESSDIR` is set. If not available, verify with `cargo check -p arco-xpress` later.

**Step 6: Commit**

```bash
git add crates/arco-xpress/ Cargo.toml Cargo.lock
git commit -m "feat(xpress): add arco-xpress crate skeleton with build script"
```

---

### Task 2: Implement status mapping with tests first (`status.rs`)

Red-green-refactor: write status mapping tests first, then implement.

**Files:**
- Create: `crates/arco-xpress/src/status.rs`

**Step 1 (Red): Write failing tests**

```rust
//! Xpress status to Arco status mapping.

use arco_core::solver::SolverStatus as CoreSolverStatus;
use arco_solver::SolverStatus;

// LP status constants (from Xpress C API)
const XPRS_LP_UNSTARTED: i32 = 0;
const XPRS_LP_OPTIMAL: i32 = 1;
const XPRS_LP_INFEAS: i32 = 2;
const XPRS_LP_CUTOFF: i32 = 3;
const XPRS_LP_UNFINISHED: i32 = 4;
const XPRS_LP_UNBOUNDED: i32 = 5;
const XPRS_LP_CUTOFF_IN_DUAL: i32 = 6;
const XPRS_LP_UNSOLVED: i32 = 7;
const XPRS_LP_NONCONVEX: i32 = 8;

// MIP status constants
const XPRS_MIP_NOT_LOADED: i32 = 0;
const XPRS_MIP_LP_NOT_OPTIMAL: i32 = 1;
const XPRS_MIP_LP_OPTIMAL: i32 = 2;
const XPRS_MIP_NO_SOL_FOUND: i32 = 3;
const XPRS_MIP_SOLUTION: i32 = 4;
const XPRS_MIP_INFEAS: i32 = 5;
const XPRS_MIP_OPTIMAL: i32 = 6;
const XPRS_MIP_UNBOUNDED: i32 = 7;

pub(crate) fn lp_status_to_core(status: i32) -> CoreSolverStatus {
    todo!()
}

pub(crate) fn mip_status_to_core(status: i32) -> CoreSolverStatus {
    todo!()
}

pub(crate) fn core_to_generic(status: CoreSolverStatus) -> SolverStatus {
    todo!()
}

pub(crate) fn lp_has_solution(status: i32) -> bool {
    todo!()
}

pub(crate) fn mip_has_solution(status: i32) -> bool {
    todo!()
}

pub(crate) fn lp_status_string(status: i32) -> &'static str {
    todo!()
}

pub(crate) fn mip_status_string(status: i32) -> &'static str {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lp_optimal_maps_to_core_optimal() {
        assert_eq!(lp_status_to_core(XPRS_LP_OPTIMAL), CoreSolverStatus::Optimal);
    }

    #[test]
    fn lp_infeas_maps_to_core_infeasible() {
        assert_eq!(lp_status_to_core(XPRS_LP_INFEAS), CoreSolverStatus::Infeasible);
    }

    #[test]
    fn lp_unbounded_maps_to_core_unbounded() {
        assert_eq!(lp_status_to_core(XPRS_LP_UNBOUNDED), CoreSolverStatus::Unbounded);
    }

    #[test]
    fn lp_unfinished_maps_to_time_limit() {
        assert_eq!(lp_status_to_core(XPRS_LP_UNFINISHED), CoreSolverStatus::TimeLimit);
    }

    #[test]
    fn lp_unknown_status_maps_to_unknown() {
        assert_eq!(lp_status_to_core(999), CoreSolverStatus::Unknown);
    }

    #[test]
    fn mip_optimal_maps_to_core_optimal() {
        assert_eq!(mip_status_to_core(XPRS_MIP_OPTIMAL), CoreSolverStatus::Optimal);
    }

    #[test]
    fn mip_solution_maps_to_core_optimal() {
        assert_eq!(mip_status_to_core(XPRS_MIP_SOLUTION), CoreSolverStatus::Optimal);
    }

    #[test]
    fn mip_infeas_maps_to_core_infeasible() {
        assert_eq!(mip_status_to_core(XPRS_MIP_INFEAS), CoreSolverStatus::Infeasible);
    }

    #[test]
    fn mip_unbounded_maps_to_core_unbounded() {
        assert_eq!(mip_status_to_core(XPRS_MIP_UNBOUNDED), CoreSolverStatus::Unbounded);
    }

    #[test]
    fn mip_no_sol_maps_to_unknown() {
        assert_eq!(mip_status_to_core(XPRS_MIP_NO_SOL_FOUND), CoreSolverStatus::Unknown);
    }

    #[test]
    fn lp_has_solution_accepts_optimal_and_unfinished() {
        assert!(lp_has_solution(XPRS_LP_OPTIMAL));
        assert!(lp_has_solution(XPRS_LP_UNFINISHED));
        assert!(!lp_has_solution(XPRS_LP_INFEAS));
        assert!(!lp_has_solution(XPRS_LP_UNBOUNDED));
        assert!(!lp_has_solution(XPRS_LP_UNSTARTED));
    }

    #[test]
    fn mip_has_solution_accepts_optimal_and_solution() {
        assert!(mip_has_solution(XPRS_MIP_OPTIMAL));
        assert!(mip_has_solution(XPRS_MIP_SOLUTION));
        assert!(!mip_has_solution(XPRS_MIP_INFEAS));
        assert!(!mip_has_solution(XPRS_MIP_NO_SOL_FOUND));
    }

    #[test]
    fn status_strings_return_expected_labels() {
        assert_eq!(lp_status_string(XPRS_LP_OPTIMAL), "optimal");
        assert_eq!(lp_status_string(XPRS_LP_INFEAS), "infeasible");
        assert_eq!(lp_status_string(XPRS_LP_UNBOUNDED), "unbounded");
        assert_eq!(mip_status_string(XPRS_MIP_OPTIMAL), "optimal");
        assert_eq!(mip_status_string(XPRS_MIP_INFEAS), "infeasible");
        assert_eq!(mip_status_string(999), "unknown");
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test -p arco-xpress -- status 2>&1 | tail -15
```

Expected: Tests fail with `todo!()` panics.

**Step 3 (Green): Implement the functions**

Replace each `todo!()` with the implementation:

```rust
pub(crate) fn lp_status_to_core(status: i32) -> CoreSolverStatus {
    match status {
        XPRS_LP_OPTIMAL => CoreSolverStatus::Optimal,
        XPRS_LP_INFEAS => CoreSolverStatus::Infeasible,
        XPRS_LP_UNBOUNDED => CoreSolverStatus::Unbounded,
        XPRS_LP_UNFINISHED => CoreSolverStatus::TimeLimit,
        _ => CoreSolverStatus::Unknown,
    }
}

pub(crate) fn mip_status_to_core(status: i32) -> CoreSolverStatus {
    match status {
        XPRS_MIP_OPTIMAL | XPRS_MIP_SOLUTION => CoreSolverStatus::Optimal,
        XPRS_MIP_INFEAS => CoreSolverStatus::Infeasible,
        XPRS_MIP_UNBOUNDED => CoreSolverStatus::Unbounded,
        _ => CoreSolverStatus::Unknown,
    }
}

pub(crate) fn core_to_generic(status: CoreSolverStatus) -> SolverStatus {
    status.into()
}

pub(crate) fn lp_has_solution(status: i32) -> bool {
    matches!(status, XPRS_LP_OPTIMAL | XPRS_LP_UNFINISHED)
}

pub(crate) fn mip_has_solution(status: i32) -> bool {
    matches!(status, XPRS_MIP_OPTIMAL | XPRS_MIP_SOLUTION)
}

pub(crate) fn lp_status_string(status: i32) -> &'static str {
    match status {
        XPRS_LP_UNSTARTED => "unstarted",
        XPRS_LP_OPTIMAL => "optimal",
        XPRS_LP_INFEAS => "infeasible",
        XPRS_LP_CUTOFF => "cutoff",
        XPRS_LP_UNFINISHED => "unfinished",
        XPRS_LP_UNBOUNDED => "unbounded",
        XPRS_LP_CUTOFF_IN_DUAL => "cutoff_in_dual",
        XPRS_LP_UNSOLVED => "unsolved",
        XPRS_LP_NONCONVEX => "nonconvex",
        _ => "unknown",
    }
}

pub(crate) fn mip_status_string(status: i32) -> &'static str {
    match status {
        XPRS_MIP_NOT_LOADED => "not_loaded",
        XPRS_MIP_LP_NOT_OPTIMAL => "lp_not_optimal",
        XPRS_MIP_LP_OPTIMAL => "lp_optimal",
        XPRS_MIP_NO_SOL_FOUND => "no_solution_found",
        XPRS_MIP_SOLUTION => "solution_found",
        XPRS_MIP_INFEAS => "infeasible",
        XPRS_MIP_OPTIMAL => "optimal",
        XPRS_MIP_UNBOUNDED => "unbounded",
        _ => "unknown",
    }
}
```

**Step 4: Run tests to verify they pass**

```bash
cargo test -p arco-xpress -- status 2>&1 | tail -15
```

Expected: All status tests pass.

**Step 5: Validate**

```bash
just fmt
just clippy-solver arco-xpress
```

**Step 6: Commit**

```bash
git add crates/arco-xpress/src/status.rs
git commit -m "feat(xpress): add status mapping with tests (red-green)"
```

---

### Task 3: Implement FFI bindings (`ffi.rs`)

**Files:**
- Create: `crates/arco-xpress/src/ffi.rs`

This module contains `unsafe` `extern "C"` declarations. All `unsafe` usage is isolated here.

**Step 1: Write FFI declarations**

```rust
//! FFI bindings to the FICO Xpress solver C library (`libxprs`).
//!
//! All `unsafe` interaction with the Xpress C API is isolated in this module.
//! Each `extern "C"` function maps directly to the corresponding Xpress C function
//! documented in the Xpress Optimizer Reference Manual.
#![allow(unsafe_code)]
#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_double, c_int};

/// Opaque Xpress problem handle.
pub type XPRSprob = *mut std::ffi::c_void;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const XPRS_PLUSINFINITY: f64 = 1.0e20;
pub const XPRS_MINUSINFINITY: f64 = -1.0e20;

// Objective sense
pub const XPRS_OBJ_MINIMIZE: c_int = 1;
pub const XPRS_OBJ_MAXIMIZE: c_int = -1;

// Integer control indices
pub const XPRS_THREADS: c_int = 8278;
pub const XPRS_PRESOLVE: c_int = 8229;
pub const XPRS_OUTPUTLOG: c_int = 8369;

// Double control indices
pub const XPRS_MAXTIME: c_int = 8020;
pub const XPRS_MIPRELSTOP: c_int = 8019;
pub const XPRS_FEASTOL: c_int = 7003;
pub const XPRS_OPTIMALITYTOL: c_int = 7004;

// Integer attribute indices
pub const XPRS_LPSTATUS: c_int = 1010;
pub const XPRS_MIPSTATUS: c_int = 1011;

// Double attribute indices
pub const XPRS_LPOBJVAL: c_int = 2001;
pub const XPRS_MIPOBJVAL: c_int = 2003;

extern "C" {
    // Initialization and cleanup
    pub fn XPRSinit(xpress: *const c_char) -> c_int;
    pub fn XPRSfree() -> c_int;
    pub fn XPRScreateprob(prob: *mut XPRSprob) -> c_int;
    pub fn XPRSdestroyprob(prob: XPRSprob) -> c_int;

    // Problem loading (CSC format)
    pub fn XPRSloadlp(
        prob: XPRSprob,
        probname: *const c_char,
        ncols: c_int,
        nrows: c_int,
        rowtype: *const c_char,
        rhs: *const c_double,
        rng: *const c_double,
        objcoef: *const c_double,
        mstart: *const c_int,
        mnel: *const c_int,
        mrwind: *const c_int,
        dmatval: *const c_double,
        dlb: *const c_double,
        dub: *const c_double,
    ) -> c_int;

    pub fn XPRSloadglobal(
        prob: XPRSprob,
        probname: *const c_char,
        ncols: c_int,
        nrows: c_int,
        rowtype: *const c_char,
        rhs: *const c_double,
        rng: *const c_double,
        objcoef: *const c_double,
        mstart: *const c_int,
        mnel: *const c_int,
        mrwind: *const c_int,
        dmatval: *const c_double,
        dlb: *const c_double,
        dub: *const c_double,
        ngents: c_int,
        nsets: c_int,
        coltype: *const c_char,
        mgcols: *const c_int,
        dlim: *const c_double,
        stype: *const c_char,
        msstart: *const c_int,
        mscols: *const c_int,
        dref: *const c_double,
    ) -> c_int;

    // QP objective
    pub fn XPRSaddqmatrix64(
        prob: XPRSprob,
        irow: c_int,
        nqtr: i64,
        mqcol1: *const c_int,
        mqcol2: *const c_int,
        dqe: *const c_double,
    ) -> c_int;

    // Objective sense
    pub fn XPRSchgobjsense(prob: XPRSprob, objsense: c_int) -> c_int;

    // Optimization
    pub fn XPRSlpoptimize(prob: XPRSprob, flags: *const c_char) -> c_int;
    pub fn XPRSmipoptimize(prob: XPRSprob, flags: *const c_char) -> c_int;

    // Solution retrieval
    pub fn XPRSgetlpsol(
        prob: XPRSprob,
        x: *mut c_double,
        slack: *mut c_double,
        dual: *mut c_double,
        dj: *mut c_double,
    ) -> c_int;

    pub fn XPRSgetmipsol(
        prob: XPRSprob,
        x: *mut c_double,
        slack: *mut c_double,
    ) -> c_int;

    // MIP warm-start
    pub fn XPRSaddmipsol(
        prob: XPRSprob,
        ilength: c_int,
        mipsolval: *const c_double,
        mipsolcol: *const c_int,
        name: *const c_char,
    ) -> c_int;

    // Controls
    pub fn XPRSsetintcontrol(prob: XPRSprob, ipar: c_int, isval: c_int) -> c_int;
    pub fn XPRSgetintcontrol(prob: XPRSprob, ipar: c_int, p_value: *mut c_int) -> c_int;
    pub fn XPRSsetdblcontrol(prob: XPRSprob, ipar: c_int, dsval: c_double) -> c_int;
    pub fn XPRSgetdblcontrol(prob: XPRSprob, ipar: c_int, p_value: *mut c_double) -> c_int;

    // Attributes
    pub fn XPRSgetintattrib(prob: XPRSprob, ipar: c_int, p_value: *mut c_int) -> c_int;
    pub fn XPRSgetdblattrib(prob: XPRSprob, ipar: c_int, p_value: *mut c_double) -> c_int;

    // Version
    pub fn XPRSgetversion(version: *mut c_char) -> c_int;
    pub fn XPRSgetbanner(banner: *mut c_char) -> c_int;
}

/// Check an Xpress return code. Returns `Ok(())` for 0 or `Err(code)` otherwise.
pub fn check_xprs(code: c_int) -> Result<(), c_int> {
    if code == 0 { Ok(()) } else { Err(code) }
}
```

**Step 2: Validate**

```bash
just fmt
just clippy-solver arco-xpress
```

**Step 3: Commit**

```bash
git add crates/arco-xpress/src/ffi.rs
git commit -m "feat(xpress): add hand-written FFI bindings to libxprs"
```

---

### Task 4: Implement Solution type with tests first (`solution.rs`)

**Files:**
- Create: `crates/arco-xpress/src/solution.rs`

**Step 1 (Red): Write Solution struct and SolutionView impl with tests**

Write the full `Solution` struct, `SolutionView` impl, and `into_core_solution()`. Add unit tests that verify:
- `SolutionView` accessors return correct values
- `into_core_solution()` preserves all fields
- Status helper methods (`is_optimal`, `is_feasible`, etc.) work correctly

Pattern: mirror `crates/arco-ipopt/src/solution.rs` exactly, but use `CoreSolverStatus` directly (no IPOPT-specific status type).

Key differences from IPOPT:
- Store `core_status: CoreSolverStatus` directly (not a solver-specific enum)
- Store `is_mip: bool` to distinguish LP/MIP solves
- No objective sign correction needed (Xpress handles maximize natively)
- No `unwrap`/`expect` in any method

**Step 2: Run tests**

```bash
cargo test -p arco-xpress -- solution 2>&1 | tail -15
```

**Step 3: Validate**

```bash
just fmt
just clippy-solver arco-xpress
```

**Step 4: Commit**

```bash
git add crates/arco-xpress/src/solution.rs
git commit -m "feat(xpress): add Solution type with SolutionView impl and tests"
```

---

### Task 5: Implement Solver and XpressBackend with tests (`solver.rs`)

This is the main task. It converts `arco_core::Model` to the Xpress CSC format and calls the C API.

**Files:**
- Create: `crates/arco-xpress/src/solver.rs`

**Step 1 (Red): Write unit tests for pure functions first**

These tests do NOT require Xpress installed — they test bounds conversion, model validation, and primal-start storage:

```rust
#[cfg(test)]
mod tests {
    // test_bounds_to_xpress_row_less_equal
    // test_bounds_to_xpress_row_greater_equal
    // test_bounds_to_xpress_row_equality
    // test_bounds_to_xpress_row_range
    // test_bounds_to_xpress_row_free
    // test_clamp_bound_infinity
    // test_clamp_bound_neg_infinity
    // test_clamp_bound_finite
    // test_solver_new_rejects_empty_model
    // test_primal_start_storage
    // test_primal_start_validation_rejects_bad_ids
    // test_primal_start_clear
}
```

**Step 2: Run to verify they fail**

```bash
cargo test -p arco-xpress -- solver 2>&1 | tail -15
```

**Step 3 (Green): Implement the Solver**

Key implementation details:

1. **RAII guards** for `XPRSinit`/`XPRSfree` and `XPRScreateprob`/`XPRSdestroyprob`:
   ```rust
   struct XpressGuard;  // calls XPRSfree on Drop
   struct ProbGuard(XPRSprob);  // calls XPRSdestroyprob on Drop
   ```

2. **`unsafe` blocks** must be:
   - Isolated in helper functions (`set_int_control`, `set_dbl_control`, `get_int_attrib`, `get_dbl_attrib`)
   - Each annotated with `// SAFETY:` comment explaining why the call is sound
   - Example:
     ```rust
     fn set_int_control(prob: XPRSprob, control: c_int, value: c_int) -> Result<(), SolverError> {
         // SAFETY: prob is a valid handle created by XPRScreateprob and not yet destroyed.
         // control and value are valid c_int values. Xpress documents this function as safe
         // to call on any valid problem handle.
         ffi::check_xprs(unsafe { ffi::XPRSsetintcontrol(prob, control, value) })
             .map_err(|rc| SolverError::SolverSpecific(format!("XPRSsetintcontrol({control}) failed: {rc}")))
     }
     ```

3. **`solve_model` function** (shared between `Solver::solve` and `XpressBackend::solve`):
   - Validate model
   - Initialize Xpress via RAII guard
   - Build variable data: bounds, objective coefficients, column types
   - Build constraint data: convert arco bounds to Xpress row types (L/G/E/R/N) + rhs + range
   - Build CSC matrix by iterating `model.columns()`
   - Call `XPRSloadlp` (pure LP) or `XPRSloadglobal` (MIP)
   - Set objective sense via `XPRSchgobjsense`
   - Apply `SolverConfig` controls
   - Apply warm-start via `XPRSaddmipsol` for MIP
   - Call `XPRSlpoptimize` or `XPRSmipoptimize`
   - Extract solution via `XPRSgetlpsol` or `XPRSgetmipsol`
   - Return `Solution`

4. **No `unwrap`/`expect`** anywhere in production code. All FFI calls return `Result`.

5. **Memory-conscious**: pre-allocate vectors with `Vec::with_capacity` based on known sizes.

6. **`bounds_to_xpress_row(lower, upper) -> (u8, f64, f64)`** converts arco's (lower, upper) bounds into Xpress row type + rhs + range:
   - Both finite and equal → `'E'`, rhs = value
   - Both finite and different → `'R'`, rhs = lower, range = upper - lower
   - Only upper finite → `'L'`, rhs = upper
   - Only lower finite → `'G'`, rhs = lower
   - Neither finite → `'N'`, rhs = 0

**Step 4: Run unit tests**

```bash
cargo test -p arco-xpress -- solver 2>&1 | tail -15
```

Expected: All pure-function tests pass. Tests that call Xpress C API will only pass with Xpress installed.

**Step 5: Validate**

```bash
just fmt
just clippy-solver arco-xpress
```

**Step 6: Commit**

```bash
git add crates/arco-xpress/src/solver.rs
git commit -m "feat(xpress): add Solver and XpressBackend with LP/MIP support"
```

---

### Task 6: Finalize lib.rs and verify crate

**Files:**
- Modify: `crates/arco-xpress/src/lib.rs` (if needed)

**Step 1: Verify lib.rs exports are correct**

Ensure it matches:
```rust
//! FICO Xpress solver backend for Arco optimization.
//!
//! This crate provides a bridge from `arco-core::Model` to the FICO Xpress
//! solver. It supports LP, MIP, and QP problems.
//!
//! Xpress is a commercial solver and requires a valid license.

pub mod ffi;
pub mod solution;
pub mod solver;
mod status;

pub use solution::Solution;
pub use solver::{Solver, XpressBackend};
```

**Step 2: Run full crate validation**

```bash
just fmt
just clippy-solver arco-xpress
cargo test -p arco-xpress --all-features -- --test-threads=1 2>&1 | tail -20
```

**Step 3: Commit if any changes**

```bash
git add crates/arco-xpress/
git commit -m "refactor(xpress): finalize crate exports and clean up"
```

---

### Task 7: Wire up Python bindings

**Files:**
- Modify: `bindings/python/Cargo.toml` (add optional dep + feature)
- Modify: `bindings/python/src/lib.rs` (update `resolve_backend`)

**Step 1: Add optional dependency**

In `bindings/python/Cargo.toml`, add after the `arco-ipopt` line:
```toml
arco-xpress = { workspace = true, optional = true }
```

Add the feature:
```toml
[features]
default = []
ipopt = ["arco-ipopt"]
xpress = ["arco-xpress"]
```

**Step 2: Update `resolve_backend()` in `bindings/python/src/lib.rs`**

Replace the Xpress error stub (~lines 1558-1561):

```rust
    // Before:
    if solver.is_some_and(|s| s.cast::<PyXpress>().is_ok()) || default_backend == "xpress" {
        return Err(errors::SolverInternalError::new_err(
            "Xpress backend is not enabled in this build",
        ));
    }

    // After:
    #[cfg(feature = "xpress")]
    if solver.is_some_and(|s| s.cast::<PyXpress>().is_ok()) || default_backend == "xpress" {
        return Ok(Box::new(arco_xpress::XpressBackend));
    }
    #[cfg(not(feature = "xpress"))]
    if solver.is_some_and(|s| s.cast::<PyXpress>().is_ok()) || default_backend == "xpress" {
        return Err(errors::SolverInternalError::new_err(
            "Xpress backend is not enabled in this build",
        ));
    }
```

**Step 3: Validate (without xpress feature — should still compile)**

```bash
cargo check -p arco-python 2>&1 | tail -5
just fmt
just clippy
```

**Step 4: Commit**

```bash
git add bindings/python/Cargo.toml bindings/python/src/lib.rs
git commit -m "feat(xpress): wire Xpress backend into Python bindings"
```

---

### Task 8: Add integration tests (require Xpress installed)

**Files:**
- Create: `crates/arco-xpress/tests/integration.rs`

**Step 1: Write integration tests**

Mirror `crates/arco-ipopt/tests/integration.rs` but add MIP tests since Xpress supports integers:

- `test_simple_lp`: minimize 2x + 3y s.t. x + y >= 5 → objective = 10.0
- `test_maximize_lp`: maximize x s.t. x <= 10 → objective = 10.0
- `test_integer_variable`: maximize x s.t. x <= 1.5, x integer → objective = 1.0
- `test_dual_values`: verify lengths and finiteness of duals
- `test_infeasible`: x >= 20 with x <= 10 → returns error
- `test_primal_start`: storage, validation, clear, solve
- `test_solution_metadata`: solve_time >= 0

No `unwrap`/`expect` in test helper code where possible; use `.expect("description")` only in tests since test panics are the error reporting mechanism.

**Step 2: Run integration tests (only if Xpress is installed)**

```bash
just test-solver arco-xpress
```

**Step 3: Commit**

```bash
git add crates/arco-xpress/tests/integration.rs
git commit -m "test(xpress): add integration tests for LP, MIP, duals, infeasibility"
```

---

### Task 9: Final validation and documentation

**Step 1: Run full workspace validation**

```bash
just fmt
just clippy
just test-core
cargo check -p arco-python
```

Verify no regressions in existing crates.

**Step 2: Verify `just` core-packages variable**

Check if `arco-xpress` should be added to the `core-packages` list in `justfile` line 23. If Xpress requires system libraries (it does), it should NOT be in `core-packages` — instead use `just test-solver arco-xpress` like IPOPT.

**Step 3: Update docs if needed**

Check `docs/how-to/configure-solver.md` — verify Xpress is already documented there (it may already be since the `PyXpress` class existed). If not, add a section matching the existing HiGHS/Ipopt pattern.

**Step 4: Final commit**

```bash
git add -A
git status
# Only commit if there are changes
git commit -m "docs(xpress): update solver configuration docs for Xpress"
```

**Step 5: Summary**

List what changed, tests run, docs updated, and any follow-ups per AGENTS.md final handoff requirements.
