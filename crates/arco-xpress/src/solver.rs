//! Xpress solver wrapper and `SolverBackend` implementation.
//!
//! This module provides the main [`Solver`] struct for direct use and the
//! [`XpressBackend`] zero-sized struct for trait-based dispatch via
//! [`arco_solver::SolverBackend`].

use crate::ffi;
use crate::solution::Solution;
use crate::status;
use arco_core::solver::SolverError as CoreSolverError;
use arco_core::{Model, Sense};
use arco_expr::{ConstraintId, VariableId};
use arco_solver::{Solve, SolverBackend, SolverConfig, SolverError as GenericSolverError};
use std::collections::BTreeMap;
use std::ffi::c_int;
use std::time::Instant;
use tracing::{debug, warn};

/// Re-export of [`arco_core::solver::SolverError`] for backward compatibility.
pub type SolverError = CoreSolverError;

/// RAII guard that calls [`ffi::XPRSfree`] on drop to release the Xpress
/// global environment.
struct XpressGuard;

impl Drop for XpressGuard {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: XPRSfree only requires that XPRSinit was called previously,
        // which is guaranteed by the guard's construction pattern in `xprs_init`.
        unsafe {
            ffi::XPRSfree();
        }
    }
}

/// RAII guard that calls [`ffi::XPRSdestroyprob`] on drop to destroy an
/// Xpress problem handle.
struct ProbGuard(ffi::XPRSprob);

impl Drop for ProbGuard {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: self.0 was created by XPRScreateprob and has not been
        // destroyed yet (single owner via this guard).
        unsafe {
            ffi::XPRSdestroyprob(self.0);
        }
    }
}

const ERRMSG_BUF_LEN: c_int = 512;

/// Retrieve the current Xpress license error message.
#[allow(unsafe_code)]
fn xprs_lic_errmsg() -> String {
    let mut buf = [0 as std::ffi::c_char; ERRMSG_BUF_LEN as usize];
    // SAFETY: buf is a valid, zeroed buffer of the declared length.
    unsafe { ffi::XPRSgetlicerrmsg(buf.as_mut_ptr(), ERRMSG_BUF_LEN) };
    unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// Initialize the Xpress environment, returning an RAII guard that frees it
/// on drop.
///
/// Xpress SDK 9+ requires `XPRSlicense` before `XPRSinit`. Additionally,
/// `XPRSinit` independently searches `XPRESSDIR/bin/xpauth.xpr` for a
/// license file, ignoring whatever `XPRSlicense` loaded. The only way to
/// override this is to set the `XPAUTH_PATH` env var before calling
/// `XPRSinit`.
///
/// License candidates are tried in this order:
/// 1. Explicit `XPAUTH_PATH` (if already set by the user)
/// 2. `$XPRESSDIR/bin/community-xpauth.xpr` (community license)
/// 3. `$XPRESSDIR/bin/xpauth.xpr` (commercial license)
/// 4. `$XPRESSDIR/xpauth.xpr`
///
/// The community license (`hostid="any"`) is tried before the commercial
/// one so that a stale commercial license for a different machine does not
/// poison the Xpress init state.
#[allow(unsafe_code)]
fn xprs_init() -> Result<XpressGuard, SolverError> {
    let xpress_dir = std::env::var("XPRESSDIR").ok();
    let original_xpauth = std::env::var("XPAUTH_PATH").ok();

    // Build candidate license file paths in priority order.
    let mut candidates: Vec<String> = Vec::new();
    if let Some(path) = &original_xpauth {
        candidates.push(path.clone());
    }
    if let Some(dir) = &xpress_dir {
        for suffix in &["bin/community-xpauth.xpr", "bin/xpauth.xpr", "xpauth.xpr"] {
            candidates.push(format!("{dir}/{suffix}"));
        }
    }

    // Try each candidate: call XPRSlicense to load the file, then set
    // XPAUTH_PATH so XPRSinit uses it (instead of auto-discovering a
    // different file in XPRESSDIR). Clean up with XPRSfree on failure so
    // the next candidate can be tried.
    for candidate in &candidates {
        let Ok(c_path) = std::ffi::CString::new(candidate.as_str()) else {
            continue;
        };
        let mut lic_status: std::ffi::c_int = 0;
        // SAFETY: c_path is a valid, null-terminated C string.
        if unsafe { ffi::XPRSlicense(&raw mut lic_status, c_path.as_ptr()) } != 0 {
            continue;
        }
        // SAFETY: single-threaded init path; restored after attempt.
        unsafe { std::env::set_var("XPAUTH_PATH", candidate) };
        // SAFETY: XPRSlicense succeeded.
        let init_rc = unsafe { ffi::XPRSinit(std::ptr::null()) };
        if init_rc == 0 {
            // Success — restore XPAUTH_PATH and return.
            restore_env("XPAUTH_PATH", original_xpauth.as_deref());
            return Ok(XpressGuard);
        }
        // Failed — clean up so the next candidate can be tried.
        unsafe { ffi::XPRSfree() };
    }

    // All candidates exhausted — restore env and report failure.
    restore_env("XPAUTH_PATH", original_xpauth.as_deref());
    let dir_info = xpress_dir.as_deref().unwrap_or("(not set)");
    let msg = xprs_lic_errmsg();
    Err(SolverError::SolverSpecific(format!(
        "Xpress license initialization failed: {msg} [XPRESSDIR={dir_info}]\n\
         \n\
         To use the Xpress community edition, download and install it from\n\
         https://www.fico.com/en/products/fico-xpress-optimization\n\
         The installer generates a machine-specific xpauth.xpr license file."
    )))
}

/// Restore an environment variable to its original value, or remove it if it
/// was originally unset.
#[allow(unsafe_code)]
fn restore_env(key: &str, original: Option<&str>) {
    // SAFETY: called from single-threaded init/cleanup path.
    match original {
        Some(val) => unsafe { std::env::set_var(key, val) },
        None => unsafe { std::env::remove_var(key) },
    }
}

/// Create a new Xpress problem, returning an RAII guard that destroys it on
/// drop.
#[allow(unsafe_code)]
fn xprs_create_prob() -> Result<ProbGuard, SolverError> {
    let mut prob: ffi::XPRSprob = std::ptr::null_mut();
    // SAFETY: prob is a valid pointer-to-null that XPRScreateprob will fill.
    ffi::check_xprs(unsafe { ffi::XPRScreateprob(&raw mut prob) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRScreateprob failed: {rc}")))?;
    Ok(ProbGuard(prob))
}

/// Set an integer control on the Xpress problem.
#[allow(unsafe_code)]
fn set_int_control(prob: ffi::XPRSprob, control: c_int, value: c_int) -> Result<(), SolverError> {
    // SAFETY: prob is a valid Xpress problem handle from XPRScreateprob.
    ffi::check_xprs(unsafe { ffi::XPRSsetintcontrol(prob, control, value) }).map_err(|rc| {
        SolverError::SolverSpecific(format!(
            "XPRSsetintcontrol({control}, {value}) failed: {rc}"
        ))
    })
}

/// Set a double control on the Xpress problem.
#[allow(unsafe_code)]
fn set_dbl_control(prob: ffi::XPRSprob, control: c_int, value: f64) -> Result<(), SolverError> {
    // SAFETY: prob is a valid Xpress problem handle from XPRScreateprob.
    ffi::check_xprs(unsafe { ffi::XPRSsetdblcontrol(prob, control, value) }).map_err(|rc| {
        SolverError::SolverSpecific(format!(
            "XPRSsetdblcontrol({control}, {value}) failed: {rc}"
        ))
    })
}

/// Get an integer attribute from the Xpress problem.
#[allow(unsafe_code)]
fn get_int_attrib(prob: ffi::XPRSprob, attrib: c_int) -> Result<c_int, SolverError> {
    let mut value: c_int = 0;
    // SAFETY: prob is a valid handle; value is a valid pointer to receive output.
    ffi::check_xprs(unsafe { ffi::XPRSgetintattrib(prob, attrib, &raw mut value) }).map_err(
        |rc| SolverError::SolverSpecific(format!("XPRSgetintattrib({attrib}) failed: {rc}")),
    )?;
    Ok(value)
}

/// Get a double attribute from the Xpress problem.
#[allow(unsafe_code)]
fn get_dbl_attrib(prob: ffi::XPRSprob, attrib: c_int) -> Result<f64, SolverError> {
    let mut value: f64 = 0.0;
    // SAFETY: prob is a valid handle; value is a valid pointer to receive output.
    ffi::check_xprs(unsafe { ffi::XPRSgetdblattrib(prob, attrib, &raw mut value) }).map_err(
        |rc| SolverError::SolverSpecific(format!("XPRSgetdblattrib({attrib}) failed: {rc}")),
    )?;
    Ok(value)
}

/// Convert arco (lower, upper) bounds to an Xpress row type, rhs, and range
/// value.
///
/// Xpress encodes constraint bounds differently from arco:
///
/// | Arco bounds          | Xpress row type | rhs   | range         |
/// |----------------------|-----------------|-------|---------------|
/// | lower == upper       | `E` (equality)  | lower | 0.0           |
/// | both finite, l < u   | `R` (range)     | lower | upper - lower |
/// | only upper finite    | `L` (<=)        | upper | 0.0           |
/// | only lower finite    | `G` (>=)        | lower | 0.0           |
/// | neither finite       | `N` (free)      | 0.0   | 0.0           |
fn bounds_to_xpress_row(lower: f64, upper: f64) -> (u8, f64, f64) {
    debug_assert!(
        lower <= upper || !lower.is_finite() || !upper.is_finite(),
        "bounds_to_xpress_row: lower ({lower}) > upper ({upper})"
    );
    let lo_finite = lower.is_finite();
    let up_finite = upper.is_finite();

    if lo_finite && up_finite {
        if (lower - upper).abs() < 1e-12 {
            (b'E', lower, 0.0)
        } else {
            (b'R', lower, upper - lower)
        }
    } else if up_finite {
        (b'L', upper, 0.0)
    } else if lo_finite {
        (b'G', lower, 0.0)
    } else {
        (b'N', 0.0, 0.0)
    }
}

/// Clamp a bound value to the Xpress representable range.
fn clamp_bound(value: f64) -> f64 {
    value.clamp(ffi::XPRS_MINUSINFINITY, ffi::XPRS_PLUSINFINITY)
}

/// Validate that a model is ready for solving.
fn validate_model(model: &Model) -> Result<(), SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    Ok(())
}

/// Validate solver configuration values.
fn validate_solver_config(config: &SolverConfig) -> Result<(), SolverError> {
    if let Some(limit) = config.time_limit {
        if !limit.is_finite() || limit < 0.0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: time_limit must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(gap) = config.mip_gap {
        if !gap.is_finite() || gap < 0.0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: mip_gap must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(tolerance) = config.tolerance {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: tolerance must be finite and >= 0".to_string(),
            ));
        }
    }
    if let Some(threads) = config.threads {
        if threads == 0 {
            return Err(SolverError::SolverSpecific(
                "invalid solver setting: threads must be >= 1".to_string(),
            ));
        }
    }
    Ok(())
}

/// Collect objective coefficients from the model.
///
/// Returns the optimization sense and a map from variable ID to summed
/// coefficient.
fn collect_objective_coefficients(
    model: &Model,
) -> Result<(Sense, BTreeMap<VariableId, f64>), SolverError> {
    let objective = model.objective();
    let Some(sense) = objective.sense else {
        return Err(SolverError::NoObjective);
    };

    let mut objective_coeffs: BTreeMap<VariableId, f64> = BTreeMap::new();
    for (var_id, coeff) in &objective.terms {
        let var = model
            .get_variable(*var_id)
            .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;
        if !var.is_active {
            continue;
        }
        *objective_coeffs.entry(*var_id).or_insert(0.0) += *coeff;
    }

    Ok((sense, objective_coeffs))
}

/// Apply [`SolverConfig`] settings to an Xpress problem handle.
fn apply_solver_config(prob: ffi::XPRSprob, config: &SolverConfig) -> Result<(), SolverError> {
    validate_solver_config(config)?;

    // Logging: off by default
    let log_value = i32::from(config.log_to_console.unwrap_or(false));
    set_int_control(prob, ffi::XPRS_OUTPUTLOG, log_value)?;

    if let Some(limit) = config.time_limit {
        // Xpress MAXTIME is in seconds (negative means wall-clock)
        set_dbl_control(prob, ffi::XPRS_MAXTIME, -limit)?;
    }
    if let Some(gap) = config.mip_gap {
        set_dbl_control(prob, ffi::XPRS_MIPRELSTOP, gap)?;
    }
    if let Some(presolve) = config.presolve {
        let val = i32::from(presolve);
        set_int_control(prob, ffi::XPRS_PRESOLVE, val)?;
    }
    if let Some(threads) = config.threads {
        set_int_control(prob, ffi::XPRS_THREADS, threads as c_int)?;
    }
    if let Some(tolerance) = config.tolerance {
        set_dbl_control(prob, ffi::XPRS_FEASTOL, tolerance)?;
        set_dbl_control(prob, ffi::XPRS_OPTIMALITYTOL, tolerance)?;
    }

    Ok(())
}

/// Solve a model with the given configuration.
#[allow(unsafe_code)]
fn solve_model(
    model: &Model,
    config: &SolverConfig,
    primal_start: Option<&[(VariableId, f64)]>,
) -> Result<Solution, SolverError> {
    validate_model(model)?;

    let solve_started = Instant::now();

    let ncols = model.num_variables();
    let nrows = model.num_constraints();

    debug!(
        component = "solver",
        operation = "solve",
        solver = "xpress",
        variables = ncols as u64,
        constraints = nrows as u64,
        "Starting Xpress solve"
    );

    let (sense, objective_coeffs) = collect_objective_coefficients(model)?;

    let mut obj_coeffs = Vec::with_capacity(ncols);
    let mut lower_bounds = Vec::with_capacity(ncols);
    let mut upper_bounds = Vec::with_capacity(ncols);
    let mut col_types: Vec<u8> = Vec::new();
    let mut has_integer = false;
    let mut int_col_indices: Vec<c_int> = Vec::new();
    let mut int_col_limits: Vec<f64> = Vec::new();

    for index in 0..ncols {
        let var_id = VariableId::new(index as u32);
        let var = model
            .get_variable(var_id)
            .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;

        let obj_coeff = if var.is_active {
            objective_coeffs.get(&var_id).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        obj_coeffs.push(obj_coeff);

        let (lb, ub) = if var.is_active {
            (clamp_bound(var.bounds.lower), clamp_bound(var.bounds.upper))
        } else {
            (0.0, 0.0)
        };
        lower_bounds.push(lb);
        upper_bounds.push(ub);

        if var.is_integer && var.is_active {
            has_integer = true;
            col_types.push(if ub <= 1.0 + 1e-12 && lb >= -1e-12 {
                b'B'
            } else {
                b'I'
            });
            int_col_indices.push(index as c_int);
            int_col_limits.push(lb);
        }
    }

    let mut row_types: Vec<u8> = Vec::with_capacity(nrows);
    let mut rhs: Vec<f64> = Vec::with_capacity(nrows);
    let mut rng: Vec<f64> = Vec::with_capacity(nrows);

    for index in 0..nrows {
        let cid = ConstraintId::new(index as u32);
        let constraint = model
            .get_constraint(cid)
            .map_err(|_| SolverError::SolverSpecific(format!("missing constraint {index}")))?;
        let (rtype, rhs_val, rng_val) =
            bounds_to_xpress_row(constraint.bounds.lower, constraint.bounds.upper);
        row_types.push(rtype);
        rhs.push(rhs_val);
        rng.push(rng_val);
    }

    let mut mstart: Vec<c_int> = Vec::with_capacity(ncols + 1);
    let mut mrwind: Vec<c_int> = Vec::new();
    let mut dmatval: Vec<f64> = Vec::new();

    for (var_id, column) in model.columns() {
        mstart.push(mrwind.len() as c_int);

        let var = model.get_variable(var_id).ok();
        let is_active = var.is_some_and(|v| v.is_active);
        if !is_active {
            continue;
        }

        for (constraint_id, coeff) in column {
            let row_idx = constraint_id.inner() as usize;
            if row_idx < nrows {
                mrwind.push(row_idx as c_int);
                dmatval.push(*coeff);
            }
        }
    }
    // Final sentinel entry
    mstart.push(mrwind.len() as c_int);

    let _env_guard = xprs_init()?;
    let prob_guard = xprs_create_prob()?;
    let prob = prob_guard.0;

    apply_solver_config(prob, config)?;

    let ncols_i = ncols as c_int;
    let nrows_i = nrows as c_int;

    if has_integer {
        let ngents = int_col_indices.len() as c_int;
        // SAFETY: prob is valid; all arrays have correct lengths per Xpress API.
        ffi::check_xprs(unsafe {
            ffi::XPRSloadmip(
                prob,
                std::ptr::null(),                // probname
                ncols_i,                         // ncols
                nrows_i,                         // nrows
                row_types.as_ptr().cast::<i8>(), // rowtype
                rhs.as_ptr(),                    // rhs
                rng.as_ptr(),                    // rng
                obj_coeffs.as_ptr(),             // objcoef
                mstart.as_ptr(),                 // mstart
                std::ptr::null(),                // mnel (NULL = use mstart diffs)
                mrwind.as_ptr(),                 // mrwind
                dmatval.as_ptr(),                // dmatval
                lower_bounds.as_ptr(),           // dlb
                upper_bounds.as_ptr(),           // dub
                ngents,                          // ngents
                0,                               // nsets
                col_types.as_ptr().cast::<i8>(), // coltype
                int_col_indices.as_ptr(),        // mgcols
                int_col_limits.as_ptr(),         // dlim
                std::ptr::null(),                // stype
                std::ptr::null(),                // msstart
                std::ptr::null(),                // mscols
                std::ptr::null(),                // dref
            )
        })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSloadmip failed: {rc}")))?;
    } else {
        // SAFETY: prob is valid; all arrays have correct lengths per Xpress API.
        ffi::check_xprs(unsafe {
            ffi::XPRSloadlp(
                prob,
                std::ptr::null(),                // probname
                ncols_i,                         // ncols
                nrows_i,                         // nrows
                row_types.as_ptr().cast::<i8>(), // rowtype
                rhs.as_ptr(),                    // rhs
                rng.as_ptr(),                    // rng
                obj_coeffs.as_ptr(),             // objcoef
                mstart.as_ptr(),                 // mstart
                std::ptr::null(),                // mnel
                mrwind.as_ptr(),                 // mrwind
                dmatval.as_ptr(),                // dmatval
                lower_bounds.as_ptr(),           // dlb
                upper_bounds.as_ptr(),           // dub
            )
        })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSloadlp failed: {rc}")))?;
    }

    let xprs_sense = match sense {
        Sense::Minimize => ffi::XPRS_OBJ_MINIMIZE,
        Sense::Maximize => ffi::XPRS_OBJ_MAXIMIZE,
    };
    // SAFETY: prob is a valid handle.
    ffi::check_xprs(unsafe { ffi::XPRSchgobjsense(prob, xprs_sense) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSchgobjsense failed: {rc}")))?;

    if has_integer {
        if let Some(hints) = primal_start {
            let mut sol_cols: Vec<c_int> = Vec::with_capacity(hints.len());
            let mut sol_vals: Vec<f64> = Vec::with_capacity(hints.len());
            for (var_id, value) in hints {
                sol_cols.push(var_id.inner() as c_int);
                sol_vals.push(*value);
            }
            let n = sol_cols.len() as c_int;
            // SAFETY: prob is valid; arrays have length n.
            ffi::check_xprs(unsafe {
                ffi::XPRSaddmipsol(
                    prob,
                    n,
                    sol_vals.as_ptr(),
                    sol_cols.as_ptr(),
                    std::ptr::null(),
                )
            })
            .map_err(|rc| SolverError::SolverSpecific(format!("XPRSaddmipsol failed: {rc}")))?;
            debug!(
                component = "solver",
                operation = "warm_start",
                solver = "xpress",
                num_hints = hints.len(),
                "Applied MIP warm-start"
            );
        }
    }

    // SAFETY: prob is a valid handle; null flags means default behavior.
    if has_integer {
        ffi::check_xprs(unsafe { ffi::XPRSmipoptimize(prob, std::ptr::null()) })
            .map_err(|rc| SolverError::SolverSpecific(format!("XPRSmipoptimize failed: {rc}")))?;
    } else {
        ffi::check_xprs(unsafe { ffi::XPRSlpoptimize(prob, std::ptr::null()) })
            .map_err(|rc| SolverError::SolverSpecific(format!("XPRSlpoptimize failed: {rc}")))?;
    }

    let solve_time = solve_started.elapsed().as_secs_f64();

    let (core_status, has_sol, status_str) = if has_integer {
        let raw = get_int_attrib(prob, ffi::XPRS_MIPSTATUS)?;
        (
            status::mip_status_to_core(raw),
            status::mip_has_solution(raw),
            status::mip_status_string(raw),
        )
    } else {
        let raw = get_int_attrib(prob, ffi::XPRS_LPSTATUS)?;
        (
            status::lp_status_to_core(raw),
            status::lp_has_solution(raw),
            status::lp_status_string(raw),
        )
    };

    debug!(
        component = "solver",
        operation = "solve",
        solver = "xpress",
        solver_status = status_str,
        is_mip = has_integer,
        duration_ms = solve_time * 1000.0,
        "Xpress solve completed"
    );

    if !has_sol {
        warn!(
            component = "solver",
            operation = "solve",
            status = "warn",
            solver = "xpress",
            solver_status = status_str,
            duration_ms = solve_time * 1000.0,
            "Solver did not find a feasible solution"
        );
        return Err(SolverError::SolveFailure {
            status: core_status,
        });
    }

    let objective_value = if has_integer {
        get_dbl_attrib(prob, ffi::XPRS_MIPOBJVAL)?
    } else {
        get_dbl_attrib(prob, ffi::XPRS_LPOBJVAL)?
    };

    let mut primal_values = vec![0.0_f64; ncols];
    let mut variable_duals = vec![0.0_f64; ncols];
    let mut constraint_duals = vec![0.0_f64; nrows];
    let mut row_values = vec![0.0_f64; nrows];

    // SAFETY: prob is valid; all output arrays have the correct lengths.
    if has_integer {
        ffi::check_xprs(unsafe {
            ffi::XPRSgetmipsol(prob, primal_values.as_mut_ptr(), row_values.as_mut_ptr())
        })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetmipsol failed: {rc}")))?;
    } else {
        ffi::check_xprs(unsafe {
            ffi::XPRSgetlpsol(
                prob,
                primal_values.as_mut_ptr(),
                row_values.as_mut_ptr(),
                constraint_duals.as_mut_ptr(),
                variable_duals.as_mut_ptr(),
            )
        })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetlpsol failed: {rc}")))?;
    }

    debug!(
        component = "solver",
        operation = "extract_solution",
        solver = "xpress",
        objective_value,
        num_primal_values = primal_values.len(),
        is_mip = has_integer,
        duration_ms = solve_time * 1000.0,
        "Solution extracted"
    );

    Ok(Solution {
        primal_values,
        variable_duals,
        constraint_duals,
        row_values,
        objective_value,
        core_status,
        is_mip: has_integer,
        solve_time_seconds: solve_time,
    })
}

/// Xpress solver wrapper.
///
/// Holds an arco [`Model`], the current [`SolverConfig`], and optional primal
/// start hints.
pub struct Solver {
    model: Model,
    config: SolverConfig,
    primal_start: Option<Vec<(VariableId, f64)>>,
}

impl Solver {
    /// Create a new Xpress solver from a [`Model`].
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::EmptyModel`] if the model has no variables.
    pub fn new(model: Model) -> Result<Self, SolverError> {
        validate_model(&model)?;

        debug!(
            component = "solver",
            operation = "init",
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
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::InvalidVariableId`] if any variable ID in
    /// `hints` does not exist in the model.
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
            solver = "xpress",
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

// Implement the Solve trait from arco-solver
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
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use arco_core::types::Bounds;
    use arco_core::{Objective, Variable};

    #[test]
    fn test_bounds_to_xpress_row_less_equal() {
        let (rtype, rhs_val, rng_val) = bounds_to_xpress_row(f64::NEG_INFINITY, 10.0);
        assert_eq!(rtype, b'L');
        assert_eq!(rhs_val, 10.0);
        assert_eq!(rng_val, 0.0);
    }

    #[test]
    fn test_bounds_to_xpress_row_greater_equal() {
        let (rtype, rhs_val, rng_val) = bounds_to_xpress_row(5.0, f64::INFINITY);
        assert_eq!(rtype, b'G');
        assert_eq!(rhs_val, 5.0);
        assert_eq!(rng_val, 0.0);
    }

    #[test]
    fn test_bounds_to_xpress_row_equality() {
        let (rtype, rhs_val, rng_val) = bounds_to_xpress_row(7.0, 7.0);
        assert_eq!(rtype, b'E');
        assert_eq!(rhs_val, 7.0);
        assert_eq!(rng_val, 0.0);
    }

    #[test]
    fn test_bounds_to_xpress_row_range() {
        let (rtype, rhs_val, rng_val) = bounds_to_xpress_row(2.0, 8.0);
        assert_eq!(rtype, b'R');
        assert_eq!(rhs_val, 2.0);
        assert_eq!(rng_val, 6.0);
    }

    #[test]
    fn test_bounds_to_xpress_row_free() {
        let (rtype, rhs_val, rng_val) = bounds_to_xpress_row(f64::NEG_INFINITY, f64::INFINITY);
        assert_eq!(rtype, b'N');
        assert_eq!(rhs_val, 0.0);
        assert_eq!(rng_val, 0.0);
    }

    #[test]
    fn test_clamp_bound_infinity() {
        assert_eq!(clamp_bound(f64::INFINITY), ffi::XPRS_PLUSINFINITY);
    }

    #[test]
    fn test_clamp_bound_neg_infinity() {
        assert_eq!(clamp_bound(f64::NEG_INFINITY), ffi::XPRS_MINUSINFINITY);
    }

    #[test]
    fn test_clamp_bound_finite() {
        assert_eq!(clamp_bound(42.0), 42.0);
        assert_eq!(clamp_bound(-42.5), -42.5);
        assert_eq!(clamp_bound(0.0), 0.0);
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
    fn test_solver_new_rejects_empty_model() {
        let model = Model::new();
        assert!(matches!(Solver::new(model), Err(SolverError::EmptyModel)));
    }

    #[test]
    fn test_primal_start_storage() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).expect("solver should be created");
        let hints = vec![(VariableId::new(0), 5.0)];
        assert!(solver.set_primal_start(&hints).is_ok());
        assert_eq!(solver.get_primal_start(), Some(hints.as_slice()));
    }

    #[test]
    fn test_primal_start_validation_rejects_bad_ids() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).expect("solver should be created");
        let invalid_hints = vec![(VariableId::new(9999), 0.5)];
        assert!(solver.set_primal_start(&invalid_hints).is_err());
    }

    #[test]
    fn test_primal_start_clear() {
        let model = build_single_variable_model();
        let mut solver = Solver::new(model).expect("solver should be created");
        let hints = vec![(VariableId::new(0), 5.0)];
        solver
            .set_primal_start(&hints)
            .expect("valid hints should succeed");
        assert!(solver.get_primal_start().is_some());
        solver.clear_primal_start();
        assert!(solver.get_primal_start().is_none());
    }

    #[test]
    fn test_validate_solver_config_rejects_negative_time_limit() {
        let config = SolverConfig::new().with_time_limit(-1.0);
        assert!(validate_solver_config(&config).is_err());
    }

    #[test]
    fn test_validate_solver_config_rejects_negative_mip_gap() {
        let config = SolverConfig::new().with_mip_gap(-0.1);
        assert!(validate_solver_config(&config).is_err());
    }

    #[test]
    fn test_validate_solver_config_rejects_zero_threads() {
        let config = SolverConfig::new().with_threads(0);
        assert!(validate_solver_config(&config).is_err());
    }

    #[test]
    fn test_validate_solver_config_rejects_negative_tolerance() {
        let config = SolverConfig::new().with_tolerance(-1e-9);
        assert!(validate_solver_config(&config).is_err());
    }

    #[test]
    fn test_validate_solver_config_accepts_valid_config() {
        let config = SolverConfig::new()
            .with_time_limit(60.0)
            .with_mip_gap(0.01)
            .with_threads(4)
            .with_tolerance(1e-6)
            .with_presolve(true)
            .with_log_to_console(false);
        assert!(validate_solver_config(&config).is_ok());
    }

    #[test]
    fn test_validate_solver_config_accepts_defaults() {
        let config = SolverConfig::new();
        assert!(validate_solver_config(&config).is_ok());
    }

    #[test]
    fn test_collect_objective_rejects_no_objective() {
        let mut model = Model::new();
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("variable");
        let result = collect_objective_coefficients(&model);
        assert!(matches!(result, Err(SolverError::NoObjective)));
    }

    #[test]
    fn test_collect_objective_returns_sense_and_coeffs() {
        let model = build_single_variable_model();
        let (sense, coeffs) = collect_objective_coefficients(&model).expect("objective");
        assert!(matches!(sense, Sense::Minimize));
        assert_eq!(coeffs.get(&VariableId::new(0)).copied(), Some(1.0));
    }
}
