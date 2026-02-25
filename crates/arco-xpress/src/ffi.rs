//! FFI bindings to the FICO Xpress solver C library (`libxprs`).
//!
//! All `unsafe` interaction with the Xpress C API is isolated in this module.
//! Each `extern "C"` function maps directly to the corresponding Xpress C function
//! documented in the Xpress Optimizer Reference Manual.
#![allow(unsafe_code)]
#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_double, c_int};

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
pub const XPRS_PRESOLVE: c_int = 8011;
pub const XPRS_OUTPUTLOG: c_int = 8035;

// Double control indices
pub const XPRS_MAXTIME: c_int = 8020;
pub const XPRS_MIPRELSTOP: c_int = 7020;
pub const XPRS_FEASTOL: c_int = 7003;
pub const XPRS_OPTIMALITYTOL: c_int = 7006;

// Integer attribute indices
pub const XPRS_LPSTATUS: c_int = 1010;
pub const XPRS_MIPSTATUS: c_int = 1011;

// Double attribute indices
pub const XPRS_LPOBJVAL: c_int = 2001;
pub const XPRS_MIPOBJVAL: c_int = 2003;

unsafe extern "C" {
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

    pub fn XPRSgetmipsol(prob: XPRSprob, x: *mut c_double, slack: *mut c_double) -> c_int;

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
    /// Caller must provide a buffer of at least 16 bytes.
    pub fn XPRSgetversion(version: *mut c_char) -> c_int;
    /// Caller must provide a buffer of at least 512 bytes.
    pub fn XPRSgetbanner(banner: *mut c_char) -> c_int;
}

/// Check an Xpress return code. Returns `Ok(())` for 0 or `Err(code)` otherwise.
pub fn check_xprs(code: c_int) -> Result<(), c_int> {
    if code == 0 { Ok(()) } else { Err(code) }
}
