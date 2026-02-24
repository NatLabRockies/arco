# Xpress Solver Backend Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add FICO Xpress as a third solver backend for arco, supporting LP, MIP, and QP problems.

**Architecture:** New `arco-xpress` crate with hand-written FFI bindings to `libxprs` via `XPRESSDIR` env var. Mirrors the `arco-ipopt` pattern: `ffi.rs` for C bindings, `solver.rs` for `Solver`+`XpressBackend`, `solution.rs` for `SolutionView`, `status.rs` for status mapping. Python bindings wire up via the existing `PyXpress` class and feature-gated `resolve_backend()`.

**Tech Stack:** Rust FFI (`extern "C"`), `libxprs` shared library, PyO3, arco-solver traits

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

```rust
fn main() {
    let xpress_dir = std::env::var("XPRESSDIR").unwrap_or_else(|_| {
        // Try common default locations
        for path in &[
            "/opt/xpressmp",
            "/Library/xpressmp",
            "C:\\xpressmp",
        ] {
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

**Step 3: Create `crates/arco-xpress/src/lib.rs`** (minimal placeholder)

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

**Step 4: Add workspace dependency to root `Cargo.toml`**

Add after `arco-ipopt` line:
```toml
arco-xpress = { path = "crates/arco-xpress" }
```

**Step 5: Create stub source files so the crate compiles**

Create empty `crates/arco-xpress/src/ffi.rs`, `crates/arco-xpress/src/solver.rs`, `crates/arco-xpress/src/solution.rs`, `crates/arco-xpress/src/status.rs` with minimal content to compile. These will be filled in subsequent tasks.

**Step 6: Verify it compiles**

Run: `cargo check -p arco-xpress 2>&1 | tail -5`

Note: This will only work if `XPRESSDIR` is set. If not available, skip this check and verify later.

**Step 7: Commit**

```bash
git add crates/arco-xpress/ Cargo.toml
git commit -m "feat(xpress): add arco-xpress crate skeleton with build script"
```

---

### Task 2: Implement FFI bindings (`ffi.rs`)

**Files:**
- Create: `crates/arco-xpress/src/ffi.rs`

**Step 1: Write the FFI declarations**

Reference: The Xpress C API uses `XPRSprob` as an opaque pointer to a problem instance. All functions follow the convention `XPRSfunction(prob, ...)` returning `c_int` (0 = success).

```rust
//! FFI bindings to the FICO Xpress solver C library (`libxprs`).
//!
//! This module provides hand-written `extern "C"` declarations for the
//! subset of the Xpress API needed by arco. The bindings are linked
//! against `libxprs` found via the `XPRESSDIR` environment variable.
#![allow(unsafe_code)]
#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_double, c_int};

/// Opaque Xpress problem handle.
pub type XPRSprob = *mut std::ffi::c_void;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Xpress "plus infinity" sentinel.
pub const XPRS_PLUSINFINITY: f64 = 1.0e20;
/// Xpress "minus infinity" sentinel.
pub const XPRS_MINUSINFINITY: f64 = -1.0e20;

// Objective sense values for XPRSchgobjsense
pub const XPRS_OBJ_MINIMIZE: c_int = 1;
pub const XPRS_OBJ_MAXIMIZE: c_int = -1;

// LP status attribute values (XPRS_LPSTATUS)
pub const XPRS_LP_UNSTARTED: c_int = 0;
pub const XPRS_LP_OPTIMAL: c_int = 1;
pub const XPRS_LP_INFEAS: c_int = 2;
pub const XPRS_LP_CUTOFF: c_int = 3;
pub const XPRS_LP_UNFINISHED: c_int = 4;
pub const XPRS_LP_UNBOUNDED: c_int = 5;
pub const XPRS_LP_CUTOFF_IN_DUAL: c_int = 6;
pub const XPRS_LP_UNSOLVED: c_int = 7;
pub const XPRS_LP_NONCONVEX: c_int = 8;

// MIP status attribute values (XPRS_MIPSTATUS)
pub const XPRS_MIP_NOT_LOADED: c_int = 0;
pub const XPRS_MIP_LP_NOT_OPTIMAL: c_int = 1;
pub const XPRS_MIP_LP_OPTIMAL: c_int = 2;
pub const XPRS_MIP_NO_SOL_FOUND: c_int = 3;
pub const XPRS_MIP_SOLUTION: c_int = 4;
pub const XPRS_MIP_INFEAS: c_int = 5;
pub const XPRS_MIP_OPTIMAL: c_int = 6;
pub const XPRS_MIP_UNBOUNDED: c_int = 7;

// Integer control indices
pub const XPRS_THREADS: c_int = 8278;
pub const XPRS_PRESOLVE: c_int = 8229;
pub const XPRS_MIPLOG: c_int = 8209;
pub const XPRS_LPLOG: c_int = 8208;
pub const XPRS_OUTPUTLOG: c_int = 8369;

// Double control indices
pub const XPRS_MAXTIME: c_int = 8020;
pub const XPRS_MIPRELSTOP: c_int = 8019;
pub const XPRS_FEASTOL: c_int = 7003;
pub const XPRS_OPTIMALITYTOL: c_int = 7004;

// Integer attribute indices
pub const XPRS_LPSTATUS: c_int = 1010;
pub const XPRS_MIPSTATUS: c_int = 1011;
pub const XPRS_COLS: c_int = 1018;
pub const XPRS_ROWS: c_int = 1019;
pub const XPRS_SIMPLEXITER: c_int = 1009;
pub const XPRS_BARITER: c_int = 1065;
pub const XPRS_MIPSOLNODE: c_int = 1031;

// Double attribute indices
pub const XPRS_LPOBJVAL: c_int = 2001;
pub const XPRS_MIPOBJVAL: c_int = 2003;
pub const XPRS_BESTBOUND: c_int = 2005;

// Column types for XPRSloadglobal
pub const XPRS_TYPE_CONTINUOUS: c_char = b'C' as c_char;
pub const XPRS_TYPE_INTEGER: c_char = b'I' as c_char;
pub const XPRS_TYPE_BINARY: c_char = b'B' as c_char;

// Row types
pub const XPRS_ROW_LESS_EQUAL: c_char = b'L' as c_char;
pub const XPRS_ROW_GREATER_EQUAL: c_char = b'G' as c_char;
pub const XPRS_ROW_EQUAL: c_char = b'E' as c_char;
pub const XPRS_ROW_RANGE: c_char = b'R' as c_char;
pub const XPRS_ROW_NONBINDING: c_char = b'N' as c_char;

extern "C" {
    // -----------------------------------------------------------------------
    // Initialization and cleanup
    // -----------------------------------------------------------------------

    /// Initialize the Xpress library. Must be called before any other function.
    /// Pass null for `xpress` to use the default path.
    pub fn XPRSinit(xpress: *const c_char) -> c_int;

    /// Free all Xpress resources. Call once at shutdown.
    pub fn XPRSfree() -> c_int;

    /// Create a new Xpress problem instance.
    pub fn XPRScreateprob(prob: *mut XPRSprob) -> c_int;

    /// Destroy a Xpress problem instance.
    pub fn XPRSdestroyprob(prob: XPRSprob) -> c_int;

    // -----------------------------------------------------------------------
    // Problem loading
    // -----------------------------------------------------------------------

    /// Load an LP problem into the Xpress problem.
    ///
    /// # Arguments
    /// - `prob` - problem handle
    /// - `probname` - problem name (can be empty string)
    /// - `ncols` - number of columns (variables)
    /// - `nrows` - number of rows (constraints)
    /// - `rowtype` - array of row types ('L', 'G', 'E', 'R', 'N')
    /// - `rhs` - array of RHS values
    /// - `rng` - array of range values (NULL if not used)
    /// - `objcoef` - array of objective coefficients
    /// - `mstart` - column start indices in the constraint matrix (CSC)
    /// - `mnel` - (can be NULL) number of non-zeros per column
    /// - `mrwind` - row indices of non-zero elements
    /// - `dmatval` - values of non-zero elements
    /// - `dlb` - lower bounds on variables
    /// - `dub` - upper bounds on variables
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

    /// Load a MIP (global) problem. Extends XPRSloadlp with column types.
    ///
    /// # Additional arguments
    /// - `ngents` - number of entities (integer/binary variables)
    /// - `nsets` - number of SOS sets (0 for basic MIP)
    /// - `coltype` - array of column types for integer variables ('I', 'B', 'C')
    /// - `mgcols` - indices of integer columns
    /// - `dlim` - limit values for integer columns (can be NULL)
    /// - `stype` - SOS set types (NULL if nsets=0)
    /// - `msstart` - SOS set start indices (NULL if nsets=0)
    /// - `mscols` - SOS set column indices (NULL if nsets=0)
    /// - `dref` - SOS reference row values (NULL if nsets=0)
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

    /// Add a quadratic matrix to the objective.
    ///
    /// # Arguments
    /// - `prob` - problem handle
    /// - `irow` - row index (-1 for objective)
    /// - `nqtr` - number of quadratic terms
    /// - `mqcol1` - first column indices
    /// - `mqcol2` - second column indices
    /// - `dqe` - quadratic coefficients
    pub fn XPRSaddqmatrix64(
        prob: XPRSprob,
        irow: c_int,
        nqtr: i64,
        mqcol1: *const c_int,
        mqcol2: *const c_int,
        dqe: *const c_double,
    ) -> c_int;

    // -----------------------------------------------------------------------
    // Objective sense
    // -----------------------------------------------------------------------

    /// Change the objective sense (XPRS_OBJ_MINIMIZE or XPRS_OBJ_MAXIMIZE).
    pub fn XPRSchgobjsense(prob: XPRSprob, objsense: c_int) -> c_int;

    // -----------------------------------------------------------------------
    // Optimization
    // -----------------------------------------------------------------------

    /// Solve the LP relaxation.
    pub fn XPRSlpoptimize(prob: XPRSprob, flags: *const c_char) -> c_int;

    /// Solve the MIP.
    pub fn XPRSmipoptimize(prob: XPRSprob, flags: *const c_char) -> c_int;

    // -----------------------------------------------------------------------
    // Solution retrieval
    // -----------------------------------------------------------------------

    /// Get the LP solution (primal values, slack, duals, reduced costs).
    pub fn XPRSgetlpsol(
        prob: XPRSprob,
        x: *mut c_double,
        slack: *mut c_double,
        dual: *mut c_double,
        dj: *mut c_double,
    ) -> c_int;

    /// Get the MIP solution.
    pub fn XPRSgetmipsol(
        prob: XPRSprob,
        x: *mut c_double,
        slack: *mut c_double,
    ) -> c_int;

    /// Add a MIP solution (warm-start).
    pub fn XPRSaddmipsol(
        prob: XPRSprob,
        ilength: c_int,
        mipsolval: *const c_double,
        mipsolcol: *const c_int,
        name: *const c_char,
    ) -> c_int;

    // -----------------------------------------------------------------------
    // Controls (configuration)
    // -----------------------------------------------------------------------

    /// Set an integer control parameter.
    pub fn XPRSsetintcontrol(prob: XPRSprob, ipar: c_int, isval: c_int) -> c_int;

    /// Get an integer control parameter.
    pub fn XPRSgetintcontrol(prob: XPRSprob, ipar: c_int, p_value: *mut c_int) -> c_int;

    /// Set a double control parameter.
    pub fn XPRSsetdblcontrol(prob: XPRSprob, ipar: c_int, dsval: c_double) -> c_int;

    /// Get a double control parameter.
    pub fn XPRSgetdblcontrol(prob: XPRSprob, ipar: c_int, p_value: *mut c_double) -> c_int;

    // -----------------------------------------------------------------------
    // Attributes (problem info / solution info)
    // -----------------------------------------------------------------------

    /// Get an integer attribute.
    pub fn XPRSgetintattrib(prob: XPRSprob, ipar: c_int, p_value: *mut c_int) -> c_int;

    /// Get a double attribute.
    pub fn XPRSgetdblattrib(prob: XPRSprob, ipar: c_int, p_value: *mut c_double) -> c_int;

    // -----------------------------------------------------------------------
    // Version
    // -----------------------------------------------------------------------

    /// Get the Xpress version string.
    pub fn XPRSgetversion(version: *mut c_char) -> c_int;

    /// Get the Xpress banner string.
    pub fn XPRSgetbanner(banner: *mut c_char) -> c_int;
}

/// Check an Xpress return code, returning Ok(()) for 0 or Err with the code.
pub fn check_xprs(code: c_int) -> Result<(), c_int> {
    if code == 0 {
        Ok(())
    } else {
        Err(code)
    }
}
```

**Step 2: Verify the module compiles**

Run: `cargo check -p arco-xpress 2>&1 | tail -5`

**Step 3: Commit**

```bash
git add crates/arco-xpress/src/ffi.rs
git commit -m "feat(xpress): add FFI bindings to Xpress C API"
```

---

### Task 3: Implement status mapping (`status.rs`)

**Files:**
- Create: `crates/arco-xpress/src/status.rs`

**Step 1: Write the status mapping**

```rust
//! Xpress status to Arco status mapping.

use crate::ffi;
use arco_core::solver::SolverStatus as CoreSolverStatus;
use arco_solver::SolverStatus;

/// Map an Xpress LP status integer to a core solver status.
pub(crate) fn lp_status_to_core(status: i32) -> CoreSolverStatus {
    match status {
        ffi::XPRS_LP_OPTIMAL => CoreSolverStatus::Optimal,
        ffi::XPRS_LP_INFEAS => CoreSolverStatus::Infeasible,
        ffi::XPRS_LP_UNBOUNDED => CoreSolverStatus::Unbounded,
        ffi::XPRS_LP_UNFINISHED => CoreSolverStatus::TimeLimit,
        _ => CoreSolverStatus::Unknown,
    }
}

/// Map an Xpress MIP status integer to a core solver status.
pub(crate) fn mip_status_to_core(status: i32) -> CoreSolverStatus {
    match status {
        ffi::XPRS_MIP_OPTIMAL => CoreSolverStatus::Optimal,
        ffi::XPRS_MIP_SOLUTION => CoreSolverStatus::Optimal,
        ffi::XPRS_MIP_INFEAS => CoreSolverStatus::Infeasible,
        ffi::XPRS_MIP_UNBOUNDED => CoreSolverStatus::Unbounded,
        ffi::XPRS_MIP_NO_SOL_FOUND => CoreSolverStatus::Unknown,
        _ => CoreSolverStatus::Unknown,
    }
}

/// Convert a core status to the generic solver status.
pub(crate) fn core_to_generic(status: CoreSolverStatus) -> SolverStatus {
    status.into()
}

/// Check whether an LP status indicates a usable solution is available.
pub(crate) fn lp_has_solution(status: i32) -> bool {
    matches!(status, ffi::XPRS_LP_OPTIMAL | ffi::XPRS_LP_UNFINISHED)
}

/// Check whether a MIP status indicates a usable solution is available.
pub(crate) fn mip_has_solution(status: i32) -> bool {
    matches!(
        status,
        ffi::XPRS_MIP_OPTIMAL | ffi::XPRS_MIP_SOLUTION
    )
}

/// Get a human-readable string for an LP status.
pub(crate) fn lp_status_string(status: i32) -> &'static str {
    match status {
        ffi::XPRS_LP_UNSTARTED => "unstarted",
        ffi::XPRS_LP_OPTIMAL => "optimal",
        ffi::XPRS_LP_INFEAS => "infeasible",
        ffi::XPRS_LP_CUTOFF => "cutoff",
        ffi::XPRS_LP_UNFINISHED => "unfinished",
        ffi::XPRS_LP_UNBOUNDED => "unbounded",
        ffi::XPRS_LP_CUTOFF_IN_DUAL => "cutoff_in_dual",
        ffi::XPRS_LP_UNSOLVED => "unsolved",
        ffi::XPRS_LP_NONCONVEX => "nonconvex",
        _ => "unknown",
    }
}

/// Get a human-readable string for a MIP status.
pub(crate) fn mip_status_string(status: i32) -> &'static str {
    match status {
        ffi::XPRS_MIP_NOT_LOADED => "not_loaded",
        ffi::XPRS_MIP_LP_NOT_OPTIMAL => "lp_not_optimal",
        ffi::XPRS_MIP_LP_OPTIMAL => "lp_optimal",
        ffi::XPRS_MIP_NO_SOL_FOUND => "no_solution_found",
        ffi::XPRS_MIP_SOLUTION => "solution_found",
        ffi::XPRS_MIP_INFEAS => "infeasible",
        ffi::XPRS_MIP_OPTIMAL => "optimal",
        ffi::XPRS_MIP_UNBOUNDED => "unbounded",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lp_status_mapping() {
        assert_eq!(lp_status_to_core(ffi::XPRS_LP_OPTIMAL), CoreSolverStatus::Optimal);
        assert_eq!(lp_status_to_core(ffi::XPRS_LP_INFEAS), CoreSolverStatus::Infeasible);
        assert_eq!(lp_status_to_core(ffi::XPRS_LP_UNBOUNDED), CoreSolverStatus::Unbounded);
        assert_eq!(lp_status_to_core(ffi::XPRS_LP_UNFINISHED), CoreSolverStatus::TimeLimit);
        assert_eq!(lp_status_to_core(999), CoreSolverStatus::Unknown);
    }

    #[test]
    fn test_mip_status_mapping() {
        assert_eq!(mip_status_to_core(ffi::XPRS_MIP_OPTIMAL), CoreSolverStatus::Optimal);
        assert_eq!(mip_status_to_core(ffi::XPRS_MIP_SOLUTION), CoreSolverStatus::Optimal);
        assert_eq!(mip_status_to_core(ffi::XPRS_MIP_INFEAS), CoreSolverStatus::Infeasible);
        assert_eq!(mip_status_to_core(ffi::XPRS_MIP_UNBOUNDED), CoreSolverStatus::Unbounded);
        assert_eq!(mip_status_to_core(ffi::XPRS_MIP_NO_SOL_FOUND), CoreSolverStatus::Unknown);
    }

    #[test]
    fn test_lp_has_solution() {
        assert!(lp_has_solution(ffi::XPRS_LP_OPTIMAL));
        assert!(lp_has_solution(ffi::XPRS_LP_UNFINISHED));
        assert!(!lp_has_solution(ffi::XPRS_LP_INFEAS));
        assert!(!lp_has_solution(ffi::XPRS_LP_UNBOUNDED));
    }

    #[test]
    fn test_mip_has_solution() {
        assert!(mip_has_solution(ffi::XPRS_MIP_OPTIMAL));
        assert!(mip_has_solution(ffi::XPRS_MIP_SOLUTION));
        assert!(!mip_has_solution(ffi::XPRS_MIP_INFEAS));
        assert!(!mip_has_solution(ffi::XPRS_MIP_NO_SOL_FOUND));
    }

    #[test]
    fn test_status_strings() {
        assert_eq!(lp_status_string(ffi::XPRS_LP_OPTIMAL), "optimal");
        assert_eq!(lp_status_string(ffi::XPRS_LP_INFEAS), "infeasible");
        assert_eq!(mip_status_string(ffi::XPRS_MIP_OPTIMAL), "optimal");
        assert_eq!(mip_status_string(ffi::XPRS_MIP_INFEAS), "infeasible");
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p arco-xpress -- status 2>&1 | tail -10`

**Step 3: Commit**

```bash
git add crates/arco-xpress/src/status.rs
git commit -m "feat(xpress): add Xpress status to arco-core status mapping"
```

---

### Task 4: Implement Solution type (`solution.rs`)

**Files:**
- Create: `crates/arco-xpress/src/solution.rs`

**Step 1: Write the Solution struct**

```rust
//! Xpress solution type and trait implementations.

use crate::status::{core_to_generic, lp_status_to_core, lp_status_string, mip_status_to_core, mip_status_string};
use arco_core::solver::{Solution as CoreSolution, SolverStatus as CoreSolverStatus};
use arco_solver::{SolutionView, SolverStatus};
use std::collections::BTreeMap;

/// Solution from the Xpress solver.
#[derive(Debug, Clone)]
pub struct Solution {
    pub(crate) primal_values: Vec<f64>,
    pub(crate) variable_duals: Vec<f64>,
    pub(crate) constraint_duals: Vec<f64>,
    pub(crate) row_values: Vec<f64>,
    pub(crate) objective_value: f64,
    pub(crate) core_status: CoreSolverStatus,
    pub(crate) solve_time_seconds: f64,
    /// Whether this was a MIP solve (vs pure LP).
    pub(crate) is_mip: bool,
}

impl Solution {
    /// Get the primal value of a variable at the given index.
    pub fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    /// Get the dual value (reduced cost) of a variable at the given index.
    pub fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    /// Get the dual value (shadow price) of a constraint at the given index.
    pub fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    /// Get the objective value.
    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    /// Get all primal values.
    pub fn primal_values(&self) -> &[f64] {
        &self.primal_values
    }

    /// Get all variable dual values.
    pub fn variable_duals(&self) -> &[f64] {
        &self.variable_duals
    }

    /// Get all constraint dual values.
    pub fn constraint_duals(&self) -> &[f64] {
        &self.constraint_duals
    }

    /// Get solve time in seconds.
    pub fn solve_time_seconds(&self) -> f64 {
        self.solve_time_seconds
    }

    /// Check if solution is optimal.
    pub fn is_optimal(&self) -> bool {
        self.core_status.is_optimal()
    }

    /// Check if solution is feasible.
    pub fn is_feasible(&self) -> bool {
        self.core_status.is_feasible()
    }

    /// Check if solution is infeasible.
    pub fn is_infeasible(&self) -> bool {
        self.core_status.is_infeasible()
    }

    /// Check if solution is unbounded.
    pub fn is_unbounded(&self) -> bool {
        self.core_status.is_unbounded()
    }

    /// Get the core status.
    pub fn core_status(&self) -> CoreSolverStatus {
        self.core_status
    }

    /// Convert this Xpress-specific solution into a solver-agnostic `arco_core::Solution`.
    pub fn into_core_solution(self) -> CoreSolution {
        CoreSolution {
            primal_values: self.primal_values,
            variable_duals: self.variable_duals,
            constraint_duals: self.constraint_duals,
            row_values: self.row_values,
            objective_value: self.objective_value,
            status: self.core_status,
            solve_time_seconds: self.solve_time_seconds,
            metadata: BTreeMap::new(),
        }
    }
}

impl SolutionView for Solution {
    fn objective_value(&self) -> f64 {
        self.objective_value
    }

    fn status(&self) -> SolverStatus {
        core_to_generic(self.core_status)
    }

    fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    fn primal_values(&self) -> &[f64] {
        &self.primal_values
    }

    fn variable_duals(&self) -> &[f64] {
        &self.variable_duals
    }

    fn constraint_duals(&self) -> &[f64] {
        &self.constraint_duals
    }

    fn solve_time_seconds(&self) -> f64 {
        self.solve_time_seconds
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p arco-xpress 2>&1 | tail -5`

**Step 3: Commit**

```bash
git add crates/arco-xpress/src/solution.rs
git commit -m "feat(xpress): add Solution type with SolutionView trait impl"
```

---

### Task 5: Implement Solver and XpressBackend (`solver.rs`)

**Files:**
- Create: `crates/arco-xpress/src/solver.rs`

This is the main implementation. It converts an `arco_core::Model` into the Xpress CSC format, calls the Xpress C API, and extracts the solution.

**Step 1: Write the solver implementation**

```rust
//! Xpress solver implementation.

use crate::ffi;
use crate::solution::Solution;
use crate::status::{lp_has_solution, lp_status_to_core, mip_has_solution, mip_status_to_core};
use arco_core::solver::SolverError as CoreSolverError;
use arco_core::{Model, Sense};
use arco_expr::{ConstraintId, VariableId};
use arco_solver::{Solve, SolverBackend, SolverConfig, SolverError as GenericSolverError};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::os::raw::c_int;
use std::time::Instant;
use tracing::{debug, warn};

/// Re-export of [`arco_core::solver::SolverError`] for backward compatibility.
pub type SolverError = CoreSolverError;

/// Xpress solver wrapper.
pub struct Solver {
    model: Model,
    config: SolverConfig,
    primal_start: Option<Vec<(VariableId, f64)>>,
}

impl Solver {
    /// Create a new Xpress solver from a Model.
    pub fn new(model: Model) -> Result<Self, SolverError> {
        validate_model(&model)?;

        debug!(
            component = "solver",
            operation = "init",
            status = "success",
            solver = "xpress",
            variables = model.num_variables() as u64,
            constraints = model.num_constraints() as u64,
            "Creating Xpress solver from model"
        );

        Ok(Solver {
            model,
            config: SolverConfig::new(),
            primal_start: None,
        })
    }

    fn update_config(&mut self, update: impl FnOnce(SolverConfig) -> SolverConfig) {
        self.config = update(std::mem::take(&mut self.config));
    }

    /// Enable or disable Xpress logging to console for the next solve.
    pub fn set_log_to_console(&mut self, enabled: bool) {
        self.update_config(|config| config.with_log_to_console(enabled));
    }

    /// Set a time limit in seconds for the next solve.
    pub fn set_time_limit(&mut self, seconds: f64) {
        self.update_config(|config| config.with_time_limit(seconds));
    }

    /// Set a relative MIP gap for the next solve.
    pub fn set_mip_gap(&mut self, gap: f64) {
        self.update_config(|config| config.with_mip_gap(gap));
    }

    /// Set verbosity level for the next solve.
    pub fn set_verbosity(&mut self, level: u32) {
        self.update_config(|config| config.with_verbosity(level));
    }

    /// Enable or disable presolve for the next solve.
    pub fn set_presolve(&mut self, enabled: bool) {
        self.update_config(|config| config.with_presolve(enabled));
    }

    /// Set thread count for the next solve.
    pub fn set_threads(&mut self, threads: u32) {
        self.update_config(|config| config.with_threads(threads));
    }

    /// Set feasibility tolerance for the next solve.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.update_config(|config| config.with_tolerance(tolerance));
    }

    /// Set primal start values (warm-start hints).
    pub fn set_primal_start(&mut self, hints: &[(VariableId, f64)]) -> Result<(), SolverError> {
        for (var_id, _) in hints {
            if self.model.get_variable(*var_id).is_err() {
                return Err(SolverError::InvalidVariableId(var_id.inner()));
            }
        }
        self.primal_start = Some(hints.to_vec());
        debug!(
            component = "solver",
            operation = "set_primal_start",
            status = "success",
            num_hints = hints.len(),
            "Stored warm-start hints"
        );
        Ok(())
    }

    /// Clear primal start hints.
    pub fn clear_primal_start(&mut self) {
        self.primal_start = None;
    }

    /// Get current primal start hints.
    pub fn get_primal_start(&self) -> Option<&[(VariableId, f64)]> {
        self.primal_start.as_deref()
    }

    /// Get access to the current solver configuration.
    pub fn config(&self) -> &SolverConfig {
        &self.config
    }

    /// Set the solver configuration.
    pub fn set_config(&mut self, config: SolverConfig) {
        self.config = config;
    }

    /// Solve the model and return the solution.
    pub fn solve(&mut self) -> Result<Solution, SolverError> {
        solve_model(&self.model, &self.config, self.primal_start.as_deref())
    }

    /// Solve the model with a specific configuration.
    pub fn solve_with_config(&mut self, config: &SolverConfig) -> Result<Solution, SolverError> {
        solve_model(&self.model, config, self.primal_start.as_deref())
    }
}

impl Solve for Solver {
    type Solution = Solution;

    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, GenericSolverError> {
        self.solve_with_config(config).map_err(Into::into)
    }
}

/// Zero-sized backend for trait-based dispatch from the Python bindings.
pub struct XpressBackend;

impl SolverBackend for XpressBackend {
    fn solve(
        &self,
        model: &Model,
        config: &SolverConfig,
        primal_start: Option<&[(VariableId, f64)]>,
    ) -> Result<arco_core::solver::Solution, GenericSolverError> {
        solve_model(model, config, primal_start)
            .map(|s| s.into_core_solution())
            .map_err(Into::into)
    }

    fn name(&self) -> &'static str {
        "Xpress"
    }

    fn supports_integer(&self) -> bool {
        true
    }
}

/// Validate that a model is ready for solving.
fn validate_model(model: &Model) -> Result<(), SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    Ok(())
}

/// RAII guard for Xpress initialization. Calls XPRSfree on drop.
struct XpressGuard;

impl XpressGuard {
    fn init() -> Result<Self, SolverError> {
        let rc = unsafe { ffi::XPRSinit(std::ptr::null()) };
        if rc != 0 {
            return Err(SolverError::SolverNotAvailable(format!(
                "XPRSinit failed with code {rc}. Check XPRESSDIR and license."
            )));
        }
        Ok(XpressGuard)
    }
}

impl Drop for XpressGuard {
    fn drop(&mut self) {
        unsafe {
            ffi::XPRSfree();
        }
    }
}

/// RAII guard for an Xpress problem handle.
struct ProbGuard(ffi::XPRSprob);

impl ProbGuard {
    fn create() -> Result<Self, SolverError> {
        let mut prob: ffi::XPRSprob = std::ptr::null_mut();
        let rc = unsafe { ffi::XPRScreateprob(&mut prob) };
        if rc != 0 {
            return Err(SolverError::SolverSpecific(format!(
                "XPRScreateprob failed with code {rc}"
            )));
        }
        Ok(ProbGuard(prob))
    }

    fn as_ptr(&self) -> ffi::XPRSprob {
        self.0
    }
}

impl Drop for ProbGuard {
    fn drop(&mut self) {
        unsafe {
            ffi::XPRSdestroyprob(self.0);
        }
    }
}

/// Helper to call XPRSsetintcontrol and return SolverError on failure.
fn set_int_control(prob: ffi::XPRSprob, control: c_int, value: c_int) -> Result<(), SolverError> {
    ffi::check_xprs(unsafe { ffi::XPRSsetintcontrol(prob, control, value) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSsetintcontrol({control}) failed: {rc}")))
}

/// Helper to call XPRSsetdblcontrol and return SolverError on failure.
fn set_dbl_control(prob: ffi::XPRSprob, control: c_int, value: f64) -> Result<(), SolverError> {
    ffi::check_xprs(unsafe { ffi::XPRSsetdblcontrol(prob, control, value) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSsetdblcontrol({control}) failed: {rc}")))
}

/// Helper to get an integer attribute.
fn get_int_attrib(prob: ffi::XPRSprob, attrib: c_int) -> Result<c_int, SolverError> {
    let mut value: c_int = 0;
    ffi::check_xprs(unsafe { ffi::XPRSgetintattrib(prob, attrib, &mut value) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetintattrib({attrib}) failed: {rc}")))?;
    Ok(value)
}

/// Helper to get a double attribute.
fn get_dbl_attrib(prob: ffi::XPRSprob, attrib: c_int) -> Result<f64, SolverError> {
    let mut value: f64 = 0.0;
    ffi::check_xprs(unsafe { ffi::XPRSgetdblattrib(prob, attrib, &mut value) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetdblattrib({attrib}) failed: {rc}")))?;
    Ok(value)
}

/// Apply SolverConfig to the Xpress problem via control parameters.
fn apply_config(prob: ffi::XPRSprob, config: &SolverConfig) -> Result<(), SolverError> {
    // Logging: suppress by default
    let log_enabled = config.log_to_console.unwrap_or(false);
    if !log_enabled {
        set_int_control(prob, ffi::XPRS_OUTPUTLOG, 0)?;
    }

    if let Some(limit) = config.time_limit {
        // Xpress MAXTIME: negative = wall-clock seconds, positive = deterministic
        set_dbl_control(prob, ffi::XPRS_MAXTIME, -limit)?;
    }
    if let Some(gap) = config.mip_gap {
        set_dbl_control(prob, ffi::XPRS_MIPRELSTOP, gap)?;
    }
    if let Some(threads) = config.threads {
        set_int_control(prob, ffi::XPRS_THREADS, threads as c_int)?;
    }
    if let Some(presolve) = config.presolve {
        set_int_control(prob, ffi::XPRS_PRESOLVE, if presolve { -1 } else { 0 })?;
    }
    if let Some(tolerance) = config.tolerance {
        set_dbl_control(prob, ffi::XPRS_FEASTOL, tolerance)?;
        set_dbl_control(prob, ffi::XPRS_OPTIMALITYTOL, tolerance)?;
    }
    if let Some(level) = config.verbosity {
        if level == 0 && !log_enabled {
            set_int_control(prob, ffi::XPRS_OUTPUTLOG, 0)?;
        }
    }
    Ok(())
}

/// Clamp a bound to Xpress's finite range.
fn clamp_bound(val: f64) -> f64 {
    if val >= f64::INFINITY || val > ffi::XPRS_PLUSINFINITY {
        ffi::XPRS_PLUSINFINITY
    } else if val <= f64::NEG_INFINITY || val < ffi::XPRS_MINUSINFINITY {
        ffi::XPRS_MINUSINFINITY
    } else {
        val
    }
}

/// Convert arco constraint bounds (lower, upper) into Xpress row type + rhs + range.
///
/// Xpress uses a different convention:
/// - 'L': row_activity <= rhs
/// - 'G': row_activity >= rhs
/// - 'E': row_activity == rhs
/// - 'R': rhs <= row_activity <= rhs + range (range >= 0)
fn bounds_to_xpress_row(lower: f64, upper: f64) -> (u8, f64, f64) {
    let lo = clamp_bound(lower);
    let hi = clamp_bound(upper);

    let lo_finite = lo > ffi::XPRS_MINUSINFINITY;
    let hi_finite = hi < ffi::XPRS_PLUSINFINITY;

    if lo_finite && hi_finite {
        if (lo - hi).abs() < 1e-12 {
            // Equality
            (b'E', lo, 0.0)
        } else {
            // Range: rhs = lower, range = upper - lower
            (b'R', lo, hi - lo)
        }
    } else if hi_finite {
        (b'L', hi, 0.0)
    } else if lo_finite {
        (b'G', lo, 0.0)
    } else {
        (b'N', 0.0, 0.0)
    }
}

/// Core solve implementation shared by `Solver` and `XpressBackend`.
fn solve_model(
    model: &Model,
    config: &SolverConfig,
    primal_start: Option<&[(VariableId, f64)]>,
) -> Result<Solution, SolverError> {
    validate_model(model)?;

    let solve_started = Instant::now();

    debug!(
        component = "solver",
        operation = "solve",
        solver = "xpress",
        variables = model.num_variables() as u64,
        constraints = model.num_constraints() as u64,
        "Starting Xpress solve"
    );

    // Initialize Xpress
    let _xpress = XpressGuard::init()?;
    let prob = ProbGuard::create()?;

    // Apply config
    apply_config(prob.as_ptr(), config)?;

    // Collect objective
    let objective = model.objective();
    let Some(sense) = objective.sense else {
        return Err(SolverError::NoObjective);
    };

    let num_vars = model.num_variables();
    let num_constraints = model.num_constraints();

    // Build variable data
    let mut var_id_to_col: BTreeMap<VariableId, usize> = BTreeMap::new();
    let mut obj_coeffs = vec![0.0; num_vars];
    let mut lower_bounds = Vec::with_capacity(num_vars);
    let mut upper_bounds = Vec::with_capacity(num_vars);
    let mut has_integers = false;
    let mut col_types: Vec<u8> = Vec::with_capacity(num_vars);

    for index in 0..num_vars {
        let var_id = VariableId::new(index as u32);
        let var = model
            .get_variable(var_id)
            .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;

        let (lo, hi) = if var.is_active {
            (clamp_bound(var.bounds.lower), clamp_bound(var.bounds.upper))
        } else {
            (0.0, 0.0)
        };

        lower_bounds.push(lo);
        upper_bounds.push(hi);
        var_id_to_col.insert(var_id, index);

        if var.is_integer {
            has_integers = true;
            // Binary: integer with bounds [0,1]
            if (lo - 0.0).abs() < 1e-12 && (hi - 1.0).abs() < 1e-12 {
                col_types.push(b'B');
            } else {
                col_types.push(b'I');
            }
        } else {
            col_types.push(b'C');
        }
    }

    // Build objective coefficients
    for (var_id, coeff) in &objective.terms {
        let var = model
            .get_variable(*var_id)
            .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;
        if !var.is_active {
            continue;
        }
        if let Some(&col) = var_id_to_col.get(var_id) {
            obj_coeffs[col] += coeff;
        }
    }

    // Build constraint data
    let mut row_types: Vec<u8> = Vec::with_capacity(num_constraints);
    let mut rhs: Vec<f64> = Vec::with_capacity(num_constraints);
    let mut rng: Vec<f64> = Vec::with_capacity(num_constraints);

    for con_index in 0..num_constraints {
        let con_id = ConstraintId::new(con_index as u32);
        let constraint = model.get_constraint(con_id).map_err(|_| {
            SolverError::SolverSpecific(format!("constraint {con_index} not found"))
        })?;
        let (rtype, r, range) = bounds_to_xpress_row(constraint.bounds.lower, constraint.bounds.upper);
        row_types.push(rtype);
        rhs.push(r);
        rng.push(range);
    }

    // Build CSC constraint matrix
    // Xpress expects the matrix in CSC (compressed sparse column) format:
    //   mstart[j] = start index for column j in mrwind/dmatval
    //   mrwind[k] = row index of k-th non-zero
    //   dmatval[k] = value of k-th non-zero
    let mut mstart: Vec<c_int> = Vec::with_capacity(num_vars + 1);
    let mut mrwind: Vec<c_int> = Vec::new();
    let mut dmatval: Vec<f64> = Vec::new();

    for col_index in 0..num_vars {
        mstart.push(mrwind.len() as c_int);
        let var_id = VariableId::new(col_index as u32);

        if let Ok(var) = model.get_variable(var_id) {
            if !var.is_active {
                continue;
            }
        }

        if let Some(column) = model.get_column(var_id) {
            for (con_id, coeff) in column {
                mrwind.push(con_id.inner() as c_int);
                dmatval.push(*coeff);
            }
        }
    }
    // Sentinel: mstart has ncols+1 entries (last one = total nnz)
    mstart.push(mrwind.len() as c_int);

    let prob_name = CString::new("").expect("empty string");

    // Load problem
    if has_integers {
        // Build integer column data
        let mut int_cols: Vec<c_int> = Vec::new();
        let mut int_types: Vec<u8> = Vec::new();
        for (i, &ct) in col_types.iter().enumerate() {
            if ct != b'C' {
                int_cols.push(i as c_int);
                int_types.push(ct);
            }
        }
        let ngents = int_cols.len() as c_int;

        let rc = unsafe {
            ffi::XPRSloadglobal(
                prob.as_ptr(),
                prob_name.as_ptr(),
                num_vars as c_int,
                num_constraints as c_int,
                row_types.as_ptr() as *const i8,
                rhs.as_ptr(),
                rng.as_ptr(),
                obj_coeffs.as_ptr(),
                mstart.as_ptr(),
                std::ptr::null(), // mnel (not needed with mstart sentinel)
                mrwind.as_ptr(),
                dmatval.as_ptr(),
                lower_bounds.as_ptr(),
                upper_bounds.as_ptr(),
                ngents,
                0,                     // nsets
                int_types.as_ptr() as *const i8,
                int_cols.as_ptr(),
                std::ptr::null(),      // dlim
                std::ptr::null(),      // stype
                std::ptr::null(),      // msstart
                std::ptr::null(),      // mscols
                std::ptr::null(),      // dref
            )
        };
        if rc != 0 {
            return Err(SolverError::SolverSpecific(format!(
                "XPRSloadglobal failed with code {rc}"
            )));
        }
    } else {
        let rc = unsafe {
            ffi::XPRSloadlp(
                prob.as_ptr(),
                prob_name.as_ptr(),
                num_vars as c_int,
                num_constraints as c_int,
                row_types.as_ptr() as *const i8,
                rhs.as_ptr(),
                rng.as_ptr(),
                obj_coeffs.as_ptr(),
                mstart.as_ptr(),
                std::ptr::null(), // mnel
                mrwind.as_ptr(),
                dmatval.as_ptr(),
                lower_bounds.as_ptr(),
                upper_bounds.as_ptr(),
            )
        };
        if rc != 0 {
            return Err(SolverError::SolverSpecific(format!(
                "XPRSloadlp failed with code {rc}"
            )));
        }
    }

    // Set objective sense
    let obj_sense = match sense {
        Sense::Minimize => ffi::XPRS_OBJ_MINIMIZE,
        Sense::Maximize => ffi::XPRS_OBJ_MAXIMIZE,
    };
    ffi::check_xprs(unsafe { ffi::XPRSchgobjsense(prob.as_ptr(), obj_sense) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSchgobjsense failed: {rc}")))?;

    // Apply warm-start hints for MIP
    if has_integers {
        if let Some(hints) = primal_start {
            let mut sol_vals: Vec<f64> = Vec::with_capacity(hints.len());
            let mut sol_cols: Vec<c_int> = Vec::with_capacity(hints.len());
            for (var_id, value) in hints {
                if let Some(&col) = var_id_to_col.get(var_id) {
                    sol_cols.push(col as c_int);
                    sol_vals.push(*value);
                }
            }
            if !sol_vals.is_empty() {
                let sol_name = CString::new("arco_start").expect("static string");
                let rc = unsafe {
                    ffi::XPRSaddmipsol(
                        prob.as_ptr(),
                        sol_vals.len() as c_int,
                        sol_vals.as_ptr(),
                        sol_cols.as_ptr(),
                        sol_name.as_ptr(),
                    )
                };
                if rc != 0 {
                    warn!(
                        component = "solver",
                        operation = "set_primal_start",
                        status = "warn",
                        rc,
                        "Failed to set MIP start solution; continuing without hints"
                    );
                }
            }
        }
    }

    // Solve
    if has_integers {
        let rc = unsafe { ffi::XPRSmipoptimize(prob.as_ptr(), std::ptr::null()) };
        if rc != 0 {
            return Err(SolverError::SolverSpecific(format!(
                "XPRSmipoptimize failed with code {rc}"
            )));
        }
    } else {
        let rc = unsafe { ffi::XPRSlpoptimize(prob.as_ptr(), std::ptr::null()) };
        if rc != 0 {
            return Err(SolverError::SolverSpecific(format!(
                "XPRSlpoptimize failed with code {rc}"
            )));
        }
    }

    let solve_time = solve_started.elapsed().as_secs_f64();

    // Get status and extract solution
    if has_integers {
        let mip_status = get_int_attrib(prob.as_ptr(), ffi::XPRS_MIPSTATUS)?;
        let core_status = mip_status_to_core(mip_status);

        debug!(
            component = "solver",
            operation = "solve",
            solver = "xpress",
            mip_status,
            ?core_status,
            duration_ms = solve_time * 1000.0,
            "Xpress MIP solve completed"
        );

        if !mip_has_solution(mip_status) {
            return Err(SolverError::SolveFailure { status: core_status });
        }

        // Extract MIP solution
        let mut primal_values = vec![0.0; num_vars];
        let mut slack_values = vec![0.0; num_constraints];
        ffi::check_xprs(unsafe {
            ffi::XPRSgetmipsol(
                prob.as_ptr(),
                primal_values.as_mut_ptr(),
                slack_values.as_mut_ptr(),
            )
        })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetmipsol failed: {rc}")))?;

        let objective_value = get_dbl_attrib(prob.as_ptr(), ffi::XPRS_MIPOBJVAL)?;

        // MIP duals are not available; return empty vectors
        let variable_duals = vec![0.0; num_vars];
        let constraint_duals = vec![0.0; num_constraints];

        // Compute row values from slack: row_value = rhs - slack (for 'L' type)
        // For simplicity, we report slack as row_values here
        let row_values = slack_values;

        Ok(Solution {
            primal_values,
            variable_duals,
            constraint_duals,
            row_values,
            objective_value,
            core_status,
            solve_time_seconds: solve_time,
            is_mip: true,
        })
    } else {
        let lp_status = get_int_attrib(prob.as_ptr(), ffi::XPRS_LPSTATUS)?;
        let core_status = lp_status_to_core(lp_status);

        debug!(
            component = "solver",
            operation = "solve",
            solver = "xpress",
            lp_status,
            ?core_status,
            duration_ms = solve_time * 1000.0,
            "Xpress LP solve completed"
        );

        if !lp_has_solution(lp_status) {
            return Err(SolverError::SolveFailure { status: core_status });
        }

        // Extract LP solution: primal, slack, dual, reduced costs
        let mut primal_values = vec![0.0; num_vars];
        let mut slack_values = vec![0.0; num_constraints];
        let mut constraint_duals = vec![0.0; num_constraints];
        let mut variable_duals = vec![0.0; num_vars];
        ffi::check_xprs(unsafe {
            ffi::XPRSgetlpsol(
                prob.as_ptr(),
                primal_values.as_mut_ptr(),
                slack_values.as_mut_ptr(),
                constraint_duals.as_mut_ptr(),
                variable_duals.as_mut_ptr(),
            )
        })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetlpsol failed: {rc}")))?;

        let objective_value = get_dbl_attrib(prob.as_ptr(), ffi::XPRS_LPOBJVAL)?;

        let row_values = slack_values;

        Ok(Solution {
            primal_values,
            variable_duals,
            constraint_duals,
            row_values,
            objective_value,
            core_status,
            solve_time_seconds: solve_time,
            is_mip: false,
        })
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_core::types::Bounds;
    use arco_core::{Objective, Variable};

    #[test]
    fn test_solver_new_rejects_empty_model() {
        let model = Model::new();
        assert!(matches!(Solver::new(model), Err(SolverError::EmptyModel)));
    }

    fn build_single_variable_model() -> Model {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("variable");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.0)],
            })
            .expect("objective");
        model
    }

    #[test]
    fn test_primal_start_storage() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).unwrap();
        let hints = vec![(VariableId::new(0), 5.0)];
        assert!(solver.set_primal_start(&hints).is_ok());
        assert_eq!(solver.get_primal_start(), Some(hints.as_slice()));
    }

    #[test]
    fn test_primal_start_validation() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).unwrap();
        let invalid_hints = vec![(VariableId::new(9999), 0.5)];
        assert!(solver.set_primal_start(&invalid_hints).is_err());
    }

    #[test]
    fn test_primal_start_clear() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).unwrap();
        let hints = vec![(VariableId::new(0), 5.0)];
        solver.set_primal_start(&hints).unwrap();
        assert!(solver.get_primal_start().is_some());
        solver.clear_primal_start();
        assert!(solver.get_primal_start().is_none());
    }

    #[test]
    fn test_bounds_to_xpress_row_less_equal() {
        let (rtype, rhs, rng) = bounds_to_xpress_row(f64::NEG_INFINITY, 10.0);
        assert_eq!(rtype, b'L');
        assert!((rhs - 10.0).abs() < 1e-12);
        assert!((rng - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_bounds_to_xpress_row_greater_equal() {
        let (rtype, rhs, rng) = bounds_to_xpress_row(5.0, f64::INFINITY);
        assert_eq!(rtype, b'G');
        assert!((rhs - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_bounds_to_xpress_row_equality() {
        let (rtype, rhs, _) = bounds_to_xpress_row(7.0, 7.0);
        assert_eq!(rtype, b'E');
        assert!((rhs - 7.0).abs() < 1e-12);
    }

    #[test]
    fn test_bounds_to_xpress_row_range() {
        let (rtype, rhs, rng) = bounds_to_xpress_row(3.0, 8.0);
        assert_eq!(rtype, b'R');
        assert!((rhs - 3.0).abs() < 1e-12);
        assert!((rng - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_bounds_to_xpress_row_free() {
        let (rtype, _, _) = bounds_to_xpress_row(f64::NEG_INFINITY, f64::INFINITY);
        assert_eq!(rtype, b'N');
    }

    #[test]
    fn test_clamp_bound() {
        assert_eq!(clamp_bound(f64::INFINITY), ffi::XPRS_PLUSINFINITY);
        assert_eq!(clamp_bound(f64::NEG_INFINITY), ffi::XPRS_MINUSINFINITY);
        assert_eq!(clamp_bound(5.0), 5.0);
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p arco-xpress 2>&1 | tail -5`

**Step 3: Run unit tests (those that don't require Xpress installed)**

Run: `cargo test -p arco-xpress 2>&1 | tail -15`

The `test_solver_new_rejects_empty_model`, `test_primal_start_*`, `test_bounds_to_xpress_row_*`, and `test_clamp_bound` tests should pass without Xpress installed.

**Step 4: Commit**

```bash
git add crates/arco-xpress/src/solver.rs
git commit -m "feat(xpress): add Solver and XpressBackend with full LP/MIP support"
```

---

### Task 6: Update lib.rs and finalize crate

**Files:**
- Modify: `crates/arco-xpress/src/lib.rs`

**Step 1: Update lib.rs with final exports**

The lib.rs was already created in Task 1. Verify it matches:

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

**Step 2: Verify full crate compiles and tests pass**

Run: `cargo test -p arco-xpress 2>&1 | tail -20`

**Step 3: Commit if any changes**

```bash
git add crates/arco-xpress/
git commit -m "feat(xpress): finalize arco-xpress crate structure"
```

---

### Task 7: Wire up Python bindings

**Files:**
- Modify: `bindings/python/Cargo.toml` (line ~25, ~32)
- Modify: `bindings/python/src/lib.rs` (lines ~1554-1561)

**Step 1: Add arco-xpress as optional dependency**

In `bindings/python/Cargo.toml`, add after the `arco-ipopt` line:

```toml
arco-xpress = { workspace = true, optional = true }
```

And add the feature:

```toml
[features]
default = []
ipopt = ["arco-ipopt"]
xpress = ["arco-xpress"]
```

**Step 2: Update `resolve_backend()` in `bindings/python/src/lib.rs`**

Replace the Xpress error stub (around lines 1558-1561):

```rust
    if solver.is_some_and(|s| s.cast::<PyXpress>().is_ok()) || default_backend == "xpress" {
        return Err(errors::SolverInternalError::new_err(
            "Xpress backend is not enabled in this build",
        ));
    }
```

With feature-gated dispatch:

```rust
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

**Step 3: Verify compilation**

Run: `cargo check -p arco-python 2>&1 | tail -5`

This should compile without the xpress feature (Xpress stub still returns error).

**Step 4: Commit**

```bash
git add bindings/python/Cargo.toml bindings/python/src/lib.rs
git commit -m "feat(xpress): wire up Xpress backend in Python bindings"
```

---

### Task 8: Add integration tests

**Files:**
- Create: `crates/arco-xpress/tests/integration.rs`

These tests require Xpress to be installed. They mirror the `arco-ipopt` integration tests.

**Step 1: Write integration tests**

```rust
//! Integration tests for arco-xpress.
//!
//! These tests require FICO Xpress to be installed and a valid license.
//! They will be skipped in CI unless Xpress is available.
#![allow(clippy::float_cmp)]

use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_expr::VariableId;
use arco_xpress::Solver;

/// Test: minimize 2x + 3y subject to x + y >= 5, x,y >= 0
#[test]
fn test_simple_lp() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .unwrap();

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(5.0, f64::INFINITY),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();
    model.set_coefficient(y, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 2.0), (y, 3.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).expect("Failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("Failed to solve");

    assert!(
        (solution.objective_value() - 10.0).abs() < 1e-4,
        "Expected objective value 10.0, got {}",
        solution.objective_value()
    );
}

/// Test: maximize x subject to x <= 10
#[test]
fn test_maximize_lp() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 10.0),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).expect("Failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("Failed to solve");

    assert!(
        (solution.objective_value() - 10.0).abs() < 1e-4,
        "Expected objective value 10.0, got {}",
        solution.objective_value()
    );
}

/// Test: integer variable (MIP)
#[test]
fn test_integer_variable() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::integer(Bounds::new(0.0, 10.0)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(f64::NEG_INFINITY, 1.5),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Maximize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).expect("Failed to create solver");
    solver.set_log_to_console(false);
    let solution = solver.solve().expect("Failed to solve");

    assert!(
        (solution.objective_value() - 1.0).abs() < 1e-4,
        "Expected integer solution 1.0, got {}",
        solution.objective_value()
    );
}

/// Test: dual values have correct lengths and are finite
#[test]
fn test_dual_values() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(0.0, 5.0),
        })
        .unwrap();

    model.set_coefficient(x, constraint, 1.0).unwrap();
    model.set_coefficient(y, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0), (y, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let num_variables = model.num_variables();
    let num_constraints = model.num_constraints();

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);
    let solution = solver.solve().unwrap();

    assert_eq!(solution.variable_duals().len(), num_variables);
    assert_eq!(solution.constraint_duals().len(), num_constraints);
    assert!(solution.variable_duals().iter().all(|v| v.is_finite()));
    assert!(solution.constraint_duals().iter().all(|v| v.is_finite()));
}

/// Test: infeasible model returns error
#[test]
fn test_infeasible() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    // x >= 20 AND x <= 10 (infeasible)
    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(20.0, f64::INFINITY),
        })
        .unwrap();
    model.set_coefficient(x, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);
    let result = solver.solve();
    assert!(result.is_err(), "Infeasible problem should fail to solve");
}

/// Test: primal start (warm-start hints) storage, validation, clear
#[test]
fn test_primal_start() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    let y = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(0.0, 5.0),
        })
        .unwrap();
    model.set_coefficient(x, constraint, 1.0).unwrap();
    model.set_coefficient(y, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0), (y, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);

    // Storage
    let hints = vec![(VariableId::new(0), 2.0), (VariableId::new(1), 1.0)];
    assert!(solver.set_primal_start(&hints).is_ok());
    assert_eq!(solver.get_primal_start(), Some(hints.as_slice()));

    // Validation
    let bad_hints = vec![(VariableId::new(9999), 0.5)];
    assert!(solver.set_primal_start(&bad_hints).is_err());

    // Clear
    solver.set_primal_start(&hints).unwrap();
    solver.clear_primal_start();
    assert!(solver.get_primal_start().is_none());

    // Solve
    let solution = solver.solve().unwrap();
    assert!(
        (solution.objective_value() - 0.0).abs() < 1e-4,
        "Expected objective value 0.0, got {}",
        solution.objective_value()
    );
}

/// Test: solution metadata (solve_time > 0)
#[test]
fn test_solution_metadata() {
    let mut model = Model::new();

    let x = model
        .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
        .unwrap();

    let constraint = model
        .add_constraint(Constraint {
            bounds: Bounds::new(0.0, 5.0),
        })
        .unwrap();
    model.set_coefficient(x, constraint, 1.0).unwrap();

    let objective = Objective {
        sense: Some(Sense::Minimize),
        terms: vec![(x, 1.0)],
    };
    model.set_objective(objective).unwrap();

    let mut solver = Solver::new(model).unwrap();
    solver.set_log_to_console(false);
    let solution = solver.solve().unwrap();

    assert!(
        solution.solve_time_seconds() >= 0.0,
        "Solve time should be non-negative"
    );
}
```

**Step 2: Commit**

```bash
git add crates/arco-xpress/tests/integration.rs
git commit -m "test(xpress): add integration tests for LP, MIP, duals, infeasibility"
```

---

### Task 9: Run full test suite and verify

**Step 1: Run status and unit tests (no Xpress required)**

Run: `cargo test -p arco-xpress -- --skip test_simple_lp --skip test_maximize --skip test_integer --skip test_dual_values --skip test_infeasible --skip test_primal_start --skip test_solution_metadata 2>&1 | tail -20`

Expected: All status mapping tests and bounds conversion tests pass.

**Step 2: Verify existing crates still compile and pass**

Run: `cargo test -p arco-solver -p arco-core 2>&1 | tail -10`

Expected: All existing tests pass, no regressions.

**Step 3: Verify Python bindings compile without xpress feature**

Run: `cargo check -p arco-python 2>&1 | tail -5`

Expected: Compiles successfully.

**Step 4: Final commit if anything was adjusted**

```bash
git add -A && git status
# Only commit if there are changes
```
