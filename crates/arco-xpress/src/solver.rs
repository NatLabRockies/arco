//! Xpress solver implementation over primitive model views.

use crate::ffi;
use crate::solution::Solution;
use crate::status;
use arco_model::{ConstraintId, ModelFingerprint, ModelView, Sense, VariableId};
use arco_solver::{
    ModelViewBackend, ModelViewSolveResult, Solve, SolverConfig, SolverDiagnostic, SolverError,
    SolverModelStats, validate_model_view_solve_result_with_config,
};
use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, warn};

pub type SolverErrorAlias = SolverError;

struct XpressGuard {
    api: &'static ffi::Api,
    original_xpressdir: Option<String>,
    original_xpauth: Option<String>,
}

impl Drop for XpressGuard {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            (self.api.xprs_free)();
        }
        restore_env("XPRESSDIR", self.original_xpressdir.as_deref());
        restore_env("XPAUTH_PATH", self.original_xpauth.as_deref());
    }
}

struct ProbGuard {
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
}

impl Drop for ProbGuard {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            (self.api.xprs_destroyprob)(self.prob);
        }
    }
}

const ERRMSG_BUF_LEN: c_int = 512;
const LAST_ERROR_BUF_LEN: c_int = 4096;

/// Return the most likely local Xpress installation directory.
pub fn detect_xpress_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var("XPRESSDIR")
        .ok()
        .filter(|value| !value.is_empty())
    {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.push(home.join("User Apps").join("FICO Xpress").join("xpressmp"));
        candidates.push(home.join("opt").join("xpressmp"));
    }
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let user_profile = PathBuf::from(user_profile);
        candidates.push(
            user_profile
                .join("AppData")
                .join("Local")
                .join("FICO Xpress")
                .join("xpressmp"),
        );
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let program_files = PathBuf::from(program_files);
        candidates.push(program_files.join("FICO Xpress").join("xpressmp"));
    }
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        let program_files_x86 = PathBuf::from(program_files_x86);
        candidates.push(program_files_x86.join("FICO Xpress").join("xpressmp"));
    }
    candidates.extend([
        PathBuf::from("/Applications/FICO Xpress/xpressmp"),
        PathBuf::from("/Volumes/FICO Xpress Installer/FICO Xpress/xpressmp"),
        PathBuf::from("/opt/xpressmp"),
        PathBuf::from("/Library/xpressmp"),
        PathBuf::from("C:\\xpressmp"),
    ]);

    candidates.into_iter().find(|path| path.exists())
}

pub fn xpress_runtime_available() -> bool {
    ffi::runtime_library_available(detect_xpress_dir().as_deref())
}

pub fn detect_xpress_license_path(xpress_dir: Option<&Path>) -> Option<PathBuf> {
    license_candidates(xpress_dir)
        .into_iter()
        .find(|path| path.exists())
}

#[allow(unsafe_code)]
fn xprs_lic_errmsg() -> String {
    let Ok(api) = ffi::api() else {
        return "unable to load Xpress runtime library".to_string();
    };
    let mut buf = [0 as c_char; ERRMSG_BUF_LEN as usize];
    unsafe {
        (api.xprs_getlicerrmsg)(buf.as_mut_ptr(), ERRMSG_BUF_LEN);
    }
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn xpress_api() -> Result<&'static ffi::Api, SolverError> {
    ffi::api().map_err(|error| {
        SolverError::SolverNotAvailable(format!(
            "Xpress runtime library is not available: {error}. Install the FICO Xpress runtime and set XPRESSDIR if needed."
        ))
    })
}

pub fn license_candidates(xpress_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("XPAUTH_PATH").filter(|value| !value.is_empty()) {
        candidates.push(PathBuf::from(path));
    }
    if let Some(dir) = xpress_dir {
        candidates.extend([
            dir.join("bin").join("community-xpauth.xpr"),
            dir.join("bin").join("xpauth.xpr"),
            dir.join("xpauth.xpr"),
        ]);
    }
    candidates
}

#[allow(unsafe_code)]
fn xprs_init() -> Result<XpressGuard, SolverError> {
    let original_xpressdir = std::env::var("XPRESSDIR").ok();
    let original_xpauth = std::env::var("XPAUTH_PATH").ok();
    let detected_dir = detect_xpress_dir();

    if original_xpressdir.is_none() {
        if let Some(dir) = &detected_dir {
            unsafe {
                std::env::set_var("XPRESSDIR", dir);
            }
        }
    }

    let api = xpress_api()?;

    for candidate in license_candidates(detected_dir.as_deref()) {
        if !candidate.exists() {
            continue;
        }
        let Ok(c_path) = CString::new(candidate.to_string_lossy().as_ref()) else {
            continue;
        };
        let mut lic_status: c_int = 0;
        if unsafe { (api.xprs_license)(&raw mut lic_status, c_path.as_ptr()) } != 0 {
            continue;
        }
        unsafe {
            std::env::set_var("XPAUTH_PATH", &candidate);
        }
        if unsafe { (api.xprs_init)(std::ptr::null()) } == 0 {
            return Ok(XpressGuard {
                api,
                original_xpressdir,
                original_xpauth,
            });
        }
        unsafe {
            (api.xprs_free)();
        }
    }

    restore_env("XPRESSDIR", original_xpressdir.as_deref());
    restore_env("XPAUTH_PATH", original_xpauth.as_deref());
    let dir_info = detected_dir.as_ref().map_or_else(
        || "(not found)".to_string(),
        |path| path.display().to_string(),
    );
    let message = xprs_lic_errmsg();
    Err(SolverError::SolverSpecific(format!(
        "Xpress license initialization failed: {message} [XPRESSDIR={dir_info}]. \
         If you are using the Community Edition, refresh or reinstall the SDK to \
         regenerate an unexpired community-xpauth.xpr file."
    )))
}

#[allow(unsafe_code)]
fn restore_env(key: &str, original: Option<&str>) {
    match original {
        Some(value) => unsafe { std::env::set_var(key, value) },
        None => unsafe { std::env::remove_var(key) },
    }
}

#[allow(unsafe_code)]
fn xprs_last_error(api: &'static ffi::Api, prob: ffi::XPRSprob) -> Option<String> {
    let mut buffer = [0 as c_char; LAST_ERROR_BUF_LEN as usize];
    let rc = unsafe { (api.xprs_getlasterror)(prob, buffer.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }

    let message = unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .trim()
        .to_string();
    if message.is_empty() {
        None
    } else {
        Some(message)
    }
}

fn xpress_model_stats(model: &(impl ModelView + ?Sized)) -> SolverModelStats {
    SolverModelStats {
        variables: model.num_variables(),
        constraints: model.num_constraints(),
        coefficients: model.num_coefficients(),
    }
}

fn normalize_xpress_last_error(last_error: &str) -> String {
    let trimmed = last_error.trim();
    let without_prefix = trimmed.strip_prefix('?').unwrap_or(trimmed).trim_start();
    let without_code = without_prefix
        .split_once(" Error:")
        .map_or(without_prefix, |(_, message)| message.trim());

    if without_code.contains("Problem has too many rows and columns. The maximum is 5000") {
        return "Xpress Community Edition size limit exceeded: this model is larger than the 5000 row/column maximum. Try a smaller instance or use HiGHS/full Xpress."
            .to_string();
    }

    without_code.to_string()
}

fn format_xpress_failure_message(action: &str, rc: c_int, last_error: Option<&str>) -> String {
    match last_error {
        Some(message) => {
            let normalized = normalize_xpress_last_error(message);
            format!("Xpress solve failed ({action}, rc={rc}): {normalized}")
        }
        None => format!("Xpress solve failed ({action}, rc={rc})"),
    }
}

fn xpress_failure_error_for_stats(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    action: &str,
    rc: c_int,
    stats: &SolverModelStats,
) -> SolverError {
    let last_error = xprs_last_error(api, prob);
    if last_error
        .as_deref()
        .is_some_and(|message| normalize_xpress_last_error(message).contains("size limit exceeded"))
    {
        return SolverError::Diagnostic(SolverDiagnostic::ModelSizeLimit {
            solver: "Xpress Community Edition".to_string(),
            operation: action.trim_start_matches("XPRS").to_ascii_lowercase(),
            return_code: rc,
            limit: 5000,
            model: stats.clone(),
        });
    }

    SolverError::SolverSpecific(format_xpress_failure_message(
        action,
        rc,
        last_error.as_deref(),
    ))
}

#[allow(unsafe_code)]
fn xprs_create_prob(api: &'static ffi::Api) -> Result<ProbGuard, SolverError> {
    let mut prob: ffi::XPRSprob = std::ptr::null_mut();
    ffi::check_xprs(unsafe { (api.xprs_createprob)(&raw mut prob) })
        .map_err(|rc| SolverError::SolverSpecific(format!("XPRScreateprob failed: {rc}")))?;
    Ok(ProbGuard { api, prob })
}

#[allow(unsafe_code)]
fn set_int_control(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    control: c_int,
    value: c_int,
) -> Result<(), SolverError> {
    ffi::check_xprs(unsafe { (api.xprs_setintcontrol)(prob, control, value) }).map_err(|rc| {
        SolverError::SolverSpecific(format!(
            "XPRSsetintcontrol({control}, {value}) failed: {rc}"
        ))
    })
}

#[allow(unsafe_code)]
fn set_dbl_control(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    control: c_int,
    value: f64,
) -> Result<(), SolverError> {
    ffi::check_xprs(unsafe { (api.xprs_setdblcontrol)(prob, control, value) }).map_err(|rc| {
        SolverError::SolverSpecific(format!(
            "XPRSsetdblcontrol({control}, {value}) failed: {rc}"
        ))
    })
}

#[allow(unsafe_code)]
fn get_int_attrib(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    attrib: c_int,
) -> Result<c_int, SolverError> {
    let mut value: c_int = 0;
    ffi::check_xprs(unsafe { (api.xprs_getintattrib)(prob, attrib, &raw mut value) }).map_err(
        |rc| SolverError::SolverSpecific(format!("XPRSgetintattrib({attrib}) failed: {rc}")),
    )?;
    Ok(value)
}

#[allow(unsafe_code)]
fn get_dbl_attrib(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    attrib: c_int,
) -> Result<f64, SolverError> {
    let mut value = 0.0;
    ffi::check_xprs(unsafe { (api.xprs_getdblattrib)(prob, attrib, &raw mut value) }).map_err(
        |rc| SolverError::SolverSpecific(format!("XPRSgetdblattrib({attrib}) failed: {rc}")),
    )?;
    Ok(value)
}

fn bounds_to_xpress_row(lower: f64, upper: f64) -> (u8, f64, f64) {
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

fn clamp_bound(value: f64) -> f64 {
    if value.is_infinite() {
        if value.is_sign_positive() {
            ffi::XPRS_PLUSINFINITY
        } else {
            ffi::XPRS_MINUSINFINITY
        }
    } else {
        value.clamp(ffi::XPRS_MINUSINFINITY, ffi::XPRS_PLUSINFINITY)
    }
}

fn ensure_non_negative_finite_setting(name: &str, value: Option<f64>) -> Result<(), SolverError> {
    if let Some(value) = value {
        if !value.is_finite() || value < 0.0 {
            return Err(SolverError::InvalidSettings(format!(
                "{name} must be finite and >= 0"
            )));
        }
    }
    Ok(())
}

fn validate_solver_config(config: &SolverConfig) -> Result<(), SolverError> {
    ensure_non_negative_finite_setting("time_limit", config.time_limit)?;
    ensure_non_negative_finite_setting("mip_gap", config.mip_gap)?;
    ensure_non_negative_finite_setting("tolerance", config.tolerance)?;
    let _ = lp_optimize_flags(config)?;

    if let Some(0) = config.threads {
        return Err(SolverError::InvalidSettings(
            "threads must be >= 1".to_string(),
        ));
    }
    Ok(())
}

fn lp_optimize_flags(config: &SolverConfig) -> Result<Option<&'static str>, SolverError> {
    let algorithm = config
        .parameters
        .get("xpress.lp_algorithm")
        .map_or("auto", String::as_str);
    match algorithm {
        "auto" | "" => Ok(None),
        "primal" | "primal_simplex" | "p" => Ok(Some("p")),
        "dual" | "dual_simplex" | "d" => Ok(Some("d")),
        "barrier" | "b" => Ok(Some("b")),
        "primal_barrier" | "primal+barrier" | "pb" | "bp" => Ok(Some("pb")),
        "dual_barrier" | "dual+barrier" | "db" | "bd" => Ok(Some("db")),
        "primal_dual" | "primal+dual" | "pd" | "dp" => Ok(Some("pd")),
        "all" | "pdb" | "pbd" | "dpb" | "dbp" | "bpd" | "bdp" => Ok(Some("pdb")),
        other => Err(SolverError::InvalidSettings(format!(
            "xpress.lp_algorithm must be one of auto, primal, dual, barrier, primal_barrier, dual_barrier, primal_dual, or all (got {other:?})"
        ))),
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn xpress_message_callback(
    _prob: ffi::XPRSprob,
    _data: *mut c_void,
    msg: *const c_char,
    msglen: c_int,
    _msgtype: c_int,
) {
    if msg.is_null() || msglen <= 0 {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(msg.cast::<u8>(), msglen as usize) };
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(bytes);
    if !bytes.ends_with(b"\n") {
        let _ = stdout.write_all(b"\n");
    }
    let _ = stdout.flush();
}

#[allow(unsafe_code)]
fn enable_console_logging(api: &'static ffi::Api, prob: ffi::XPRSprob) -> Result<(), SolverError> {
    ffi::check_xprs(unsafe {
        (api.xprs_setcbmessage)(prob, Some(xpress_message_callback), std::ptr::null_mut())
    })
    .map_err(|rc| SolverError::SolverSpecific(format!("XPRSsetcbmessage failed: {rc}")))?;
    set_int_control(api, prob, ffi::XPRS_LPLOG, 1)?;
    set_int_control(api, prob, ffi::XPRS_MIPLOG, 1)
}

fn apply_solver_config(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    config: &SolverConfig,
) -> Result<(), SolverError> {
    validate_solver_config(config)?;

    let log_to_console = config.log_to_console.unwrap_or(false);
    if log_to_console {
        enable_console_logging(api, prob)?;
    }
    set_int_control(api, prob, ffi::XPRS_OUTPUTLOG, i32::from(log_to_console))?;

    if let Some(limit) = config.time_limit {
        set_dbl_control(api, prob, ffi::XPRS_MAXTIME, -limit)?;
    }
    if let Some(gap) = config.mip_gap {
        set_dbl_control(api, prob, ffi::XPRS_MIPRELSTOP, gap)?;
    }
    if let Some(presolve) = config.presolve {
        set_int_control(api, prob, ffi::XPRS_PRESOLVE, i32::from(presolve))?;
    }
    if let Some(threads) = config.threads {
        set_int_control(api, prob, ffi::XPRS_THREADS, threads as c_int)?;
    }
    if let Some(tolerance) = config.tolerance {
        set_dbl_control(api, prob, ffi::XPRS_FEASTOL, tolerance)?;
        set_dbl_control(api, prob, ffi::XPRS_OPTIMALITYTOL, tolerance)?;
    }

    Ok(())
}

struct SolveArtifacts {
    solution: Solution,
    metadata: BTreeMap<String, f64>,
}

struct PreparedModelViewSolve {
    problem: PreparedXpressProblem,
    fingerprint: ModelFingerprint,
    fingerprint_seconds: f64,
    num_variables: usize,
    num_constraints: usize,
}

struct PreparedModelViewLoad {
    load_data: XpressLoadData,
    fingerprint: ModelFingerprint,
    fingerprint_seconds: f64,
    num_variables: usize,
    num_constraints: usize,
}

struct PreparedXpressProblem {
    prob_guard: ProbGuard,
    _env_guard: XpressGuard,
    api: &'static ffi::Api,
    ncols: usize,
    nrows: usize,
    has_integer: bool,
    stats: SolverModelStats,
    matrix_build_seconds: f64,
    solve_started: Instant,
}

struct XpressLoadData {
    ncols: usize,
    nrows: usize,
    has_integer: bool,
    stats: SolverModelStats,
    sense: Sense,
    objective_coefficients: XpressObjectiveCoefficients,
    variable_bounds: XpressVariableBounds,
    row_types: Vec<u8>,
    rhs: Vec<f64>,
    rng: Option<Vec<f64>>,
    matrix: XpressColumnMatrix,
    col_types: Vec<u8>,
    int_col_indices: Vec<c_int>,
    int_col_limits: Vec<f64>,
    matrix_build_seconds: f64,
    solve_started: Instant,
}

struct XpressColumnMatrix {
    mstart: Vec<c_int>,
    mrwind: Vec<c_int>,
    dmatval: Vec<f64>,
}

enum XpressObjectiveCoefficients {
    Dense(Vec<f64>),
    Sparse(Vec<(VariableId, f64)>),
}

enum XpressVariableBounds {
    Full {
        lower_bounds: Vec<f64>,
        upper_bounds: Vec<f64>,
    },
    DefaultNonnegativeWithUpperChanges {
        upper_indices: Vec<c_int>,
        upper_types: Vec<u8>,
        upper_values: Vec<f64>,
    },
}

impl XpressVariableBounds {
    fn load_ptrs(&self) -> (*const f64, *const f64) {
        match self {
            Self::Full {
                lower_bounds,
                upper_bounds,
            } => (lower_bounds.as_ptr(), upper_bounds.as_ptr()),
            Self::DefaultNonnegativeWithUpperChanges { .. } => (std::ptr::null(), std::ptr::null()),
        }
    }

    fn lower_bound_for_integer_column(&self, index: usize) -> Option<f64> {
        match self {
            Self::Full { lower_bounds, .. } => lower_bounds.get(index).copied(),
            Self::DefaultNonnegativeWithUpperChanges { .. } => Some(0.0),
        }
    }

    #[allow(unsafe_code)]
    fn apply_deferred_changes(
        self,
        api: &'static ffi::Api,
        prob: ffi::XPRSprob,
        stats: &SolverModelStats,
    ) -> Result<(), SolverError> {
        let Self::DefaultNonnegativeWithUpperChanges {
            upper_indices,
            upper_types,
            upper_values,
        } = self
        else {
            return Ok(());
        };
        if upper_indices.is_empty() {
            return Ok(());
        }

        ffi::check_xprs(unsafe {
            (api.xprs_chgbounds)(
                prob,
                upper_indices.len() as c_int,
                upper_indices.as_ptr(),
                upper_types.as_ptr().cast::<c_char>(),
                upper_values.as_ptr(),
            )
        })
        .map_err(|rc| xpress_failure_error_for_stats(api, prob, "XPRSchgbounds", rc, stats))
    }
}

impl XpressObjectiveCoefficients {
    fn into_dense(self, ncols: usize) -> Vec<f64> {
        match self {
            Self::Dense(coefficients) => coefficients,
            Self::Sparse(terms) => {
                let mut coefficients = vec![0.0; ncols];
                for (variable_id, coefficient) in terms {
                    let index = variable_id.inner() as usize;
                    if index < coefficients.len() {
                        coefficients[index] += coefficient;
                    }
                }
                coefficients
            }
        }
    }
}

fn build_xpress_column_matrix(
    model: &(impl ModelView + ?Sized),
    ncols: usize,
    nrows: usize,
    expected_coefficients: usize,
) -> Result<XpressColumnMatrix, SolverError> {
    let mut mstart = Vec::with_capacity(ncols + 1);
    let mut mrwind = Vec::with_capacity(expected_coefficients);
    let mut dmatval = Vec::with_capacity(expected_coefficients);
    for index in 0..ncols {
        mstart.push(mrwind.len() as c_int);
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;
        if !variable.is_active {
            continue;
        }
        if let Some(column) = model.column(variable_id) {
            for (constraint_id, coefficient) in column {
                let row_index = constraint_id.inner() as usize;
                if row_index < nrows {
                    mrwind.push(row_index as c_int);
                    dmatval.push(*coefficient);
                }
            }
        }
    }
    mstart.push(mrwind.len() as c_int);
    Ok(XpressColumnMatrix {
        mstart,
        mrwind,
        dmatval,
    })
}

fn build_xpress_variable_bounds(
    model: &(impl ModelView + ?Sized),
    ncols: usize,
) -> Result<XpressVariableBounds, SolverError> {
    let mut can_use_default_nonnegative_bounds = true;
    for index in 0..ncols {
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;
        if !variable.is_active {
            continue;
        }
        if variable.is_integer || variable.bounds.lower != 0.0 {
            can_use_default_nonnegative_bounds = false;
            break;
        }
    }

    if can_use_default_nonnegative_bounds {
        let mut upper_indices = Vec::new();
        let mut upper_values = Vec::new();
        for index in 0..ncols {
            let variable_id = VariableId::new(index as u32);
            let variable = model
                .variable(variable_id)
                .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;
            let upper = if variable.is_active {
                clamp_bound(variable.bounds.upper)
            } else {
                0.0
            };
            if upper < ffi::XPRS_PLUSINFINITY {
                upper_indices.push(index as c_int);
                upper_values.push(upper);
            }
        }
        return Ok(XpressVariableBounds::DefaultNonnegativeWithUpperChanges {
            upper_types: vec![b'U'; upper_indices.len()],
            upper_indices,
            upper_values,
        });
    }

    let mut lower_bounds = Vec::with_capacity(ncols);
    let mut upper_bounds = Vec::with_capacity(ncols);
    for index in 0..ncols {
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;
        if variable.is_active {
            lower_bounds.push(clamp_bound(variable.bounds.lower));
            upper_bounds.push(clamp_bound(variable.bounds.upper));
        } else {
            lower_bounds.push(0.0);
            upper_bounds.push(0.0);
        }
    }

    Ok(XpressVariableBounds::Full {
        lower_bounds,
        upper_bounds,
    })
}

fn dense_objective_coefficients(
    model: &(impl ModelView + ?Sized),
    terms: &[(VariableId, f64)],
    ncols: usize,
) -> Result<Vec<f64>, SolverError> {
    let mut objective_coefficients = vec![0.0; ncols];
    for (variable_id, coefficient) in terms {
        let index = variable_id.inner() as usize;
        let variable = model
            .variable(*variable_id)
            .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;
        if variable.is_active && index < objective_coefficients.len() {
            objective_coefficients[index] += *coefficient;
        }
    }
    Ok(objective_coefficients)
}

fn retain_active_objective_terms(
    model: &(impl ModelView + ?Sized),
    terms: &mut Vec<(VariableId, f64)>,
    ncols: usize,
) -> Result<(), SolverError> {
    let mut write_index = 0;
    for read_index in 0..terms.len() {
        let (variable_id, coefficient) = terms[read_index];
        let index = variable_id.inner() as usize;
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;
        if variable.is_active && index < ncols {
            terms[write_index] = (variable_id, coefficient);
            write_index += 1;
        }
    }
    terms.truncate(write_index);
    Ok(())
}

#[allow(unsafe_code)]
fn prepare_load_data(model: &(impl ModelView + ?Sized)) -> Result<XpressLoadData, SolverError> {
    let ncols = model.num_variables();
    let objective_coefficients =
        dense_objective_coefficients(model, model.objective().terms.as_slice(), ncols)?;
    prepare_load_data_with_objective(
        model,
        XpressObjectiveCoefficients::Dense(objective_coefficients),
    )
}

#[allow(unsafe_code)]
fn prepare_load_data_with_objective(
    model: &(impl ModelView + ?Sized),
    objective_coefficients: XpressObjectiveCoefficients,
) -> Result<XpressLoadData, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }

    let solve_started = Instant::now();
    let ncols = model.num_variables();
    let nrows = model.num_constraints();
    let stats = xpress_model_stats(model);
    let sense = model.objective().sense.unwrap_or(Sense::Minimize);

    debug!(
        component = "solver",
        operation = "solve",
        solver = "xpress",
        variables = ncols as u64,
        constraints = nrows as u64,
        "Starting Xpress solve"
    );

    let variable_bounds = build_xpress_variable_bounds(model, ncols)?;
    let mut col_types = Vec::new();
    let mut int_col_indices = Vec::new();
    let mut int_col_limits = Vec::new();
    let mut has_integer = false;

    for index in 0..ncols {
        let variable_id = VariableId::new(index as u32);
        let variable = model
            .variable(variable_id)
            .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;

        if variable.is_integer && variable.is_active {
            has_integer = true;
            col_types.push(
                if variable.bounds.lower >= -1e-12 && variable.bounds.upper <= 1.0 + 1e-12 {
                    b'B'
                } else {
                    b'I'
                },
            );
            int_col_indices.push(index as c_int);
            int_col_limits.push(
                variable_bounds
                    .lower_bound_for_integer_column(index)
                    .unwrap_or_else(|| clamp_bound(variable.bounds.lower)),
            );
        }
    }

    let mut row_types = Vec::with_capacity(nrows);
    let mut rhs = Vec::with_capacity(nrows);
    let mut rng: Option<Vec<f64>> = None;
    for index in 0..nrows {
        let constraint = model
            .constraint(ConstraintId::new(index as u32))
            .ok_or_else(|| SolverError::SolverSpecific(format!("missing constraint {index}")))?;
        let (row_type, rhs_value, range_value) =
            bounds_to_xpress_row(constraint.bounds.lower, constraint.bounds.upper);
        if row_type == b'R' && rng.is_none() {
            rng = Some(vec![0.0; index]);
        }
        if let Some(rng_values) = rng.as_mut() {
            rng_values.push(range_value);
        }
        row_types.push(row_type);
        rhs.push(rhs_value);
    }

    let matrix_build_start = Instant::now();
    let XpressColumnMatrix {
        mstart,
        mrwind,
        dmatval,
    } = build_xpress_column_matrix(model, ncols, nrows, stats.coefficients)?;
    let matrix_build_seconds = matrix_build_start.elapsed().as_secs_f64();

    Ok(XpressLoadData {
        ncols,
        nrows,
        has_integer,
        stats,
        sense,
        objective_coefficients,
        variable_bounds,
        row_types,
        rhs,
        rng,
        matrix: XpressColumnMatrix {
            mstart,
            mrwind,
            dmatval,
        },
        col_types,
        int_col_indices,
        int_col_limits,
        matrix_build_seconds,
        solve_started,
    })
}

#[allow(unsafe_code)]
fn load_prepared_problem(
    load_data: XpressLoadData,
    config: &SolverConfig,
) -> Result<PreparedXpressProblem, SolverError> {
    let XpressLoadData {
        ncols,
        nrows,
        has_integer,
        stats,
        sense,
        objective_coefficients,
        variable_bounds,
        row_types,
        rhs,
        rng,
        matrix:
            XpressColumnMatrix {
                mstart,
                mrwind,
                dmatval,
            },
        col_types,
        int_col_indices,
        int_col_limits,
        matrix_build_seconds,
        solve_started,
    } = load_data;
    let objective_coefficients = objective_coefficients.into_dense(ncols);
    let rng_ptr = rng
        .as_ref()
        .map_or(std::ptr::null(), |values| values.as_ptr());
    let (lower_bounds_ptr, upper_bounds_ptr) = variable_bounds.load_ptrs();

    let env_guard = xprs_init()?;
    let api = env_guard.api;
    let prob_guard = xprs_create_prob(api)?;
    let prob = prob_guard.prob;

    apply_solver_config(api, prob, config)?;

    if has_integer {
        ffi::check_xprs(unsafe {
            (api.xprs_loadmip)(
                prob,
                std::ptr::null(),
                ncols as c_int,
                nrows as c_int,
                row_types.as_ptr().cast::<c_char>(),
                rhs.as_ptr(),
                rng_ptr,
                objective_coefficients.as_ptr(),
                mstart.as_ptr(),
                std::ptr::null(),
                mrwind.as_ptr(),
                dmatval.as_ptr(),
                lower_bounds_ptr,
                upper_bounds_ptr,
                int_col_indices.len() as c_int,
                0,
                col_types.as_ptr().cast::<c_char>(),
                int_col_indices.as_ptr(),
                int_col_limits.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            )
        })
        .map_err(|rc| xpress_failure_error_for_stats(api, prob, "XPRSloadmip", rc, &stats))?;
    } else {
        ffi::check_xprs(unsafe {
            (api.xprs_loadlp)(
                prob,
                std::ptr::null(),
                ncols as c_int,
                nrows as c_int,
                row_types.as_ptr().cast::<c_char>(),
                rhs.as_ptr(),
                rng_ptr,
                objective_coefficients.as_ptr(),
                mstart.as_ptr(),
                std::ptr::null(),
                mrwind.as_ptr(),
                dmatval.as_ptr(),
                lower_bounds_ptr,
                upper_bounds_ptr,
            )
        })
        .map_err(|rc| xpress_failure_error_for_stats(api, prob, "XPRSloadlp", rc, &stats))?;
    }

    drop((
        objective_coefficients,
        row_types,
        rhs,
        rng,
        mstart,
        mrwind,
        dmatval,
        col_types,
        int_col_indices,
        int_col_limits,
    ));

    variable_bounds.apply_deferred_changes(api, prob, &stats)?;

    ffi::check_xprs(unsafe {
        (api.xprs_chgobjsense)(
            prob,
            match sense {
                Sense::Minimize => ffi::XPRS_OBJ_MINIMIZE,
                Sense::Maximize => ffi::XPRS_OBJ_MAXIMIZE,
            },
        )
    })
    .map_err(|rc| SolverError::SolverSpecific(format!("XPRSchgobjsense failed: {rc}")))?;

    Ok(PreparedXpressProblem {
        prob_guard,
        _env_guard: env_guard,
        api,
        ncols,
        nrows,
        has_integer,
        stats,
        matrix_build_seconds,
        solve_started,
    })
}

fn prepare_problem(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<PreparedXpressProblem, SolverError> {
    load_prepared_problem(prepare_load_data(model)?, config)
}

#[allow(unsafe_code)]
fn finish_prepared_problem(
    prepared: PreparedXpressProblem,
    config: &SolverConfig,
) -> Result<SolveArtifacts, SolverError> {
    let api = prepared.api;
    let prob = prepared.prob_guard.prob;
    let ncols = prepared.ncols;
    let nrows = prepared.nrows;
    let has_integer = prepared.has_integer;
    let lp_flags = lp_optimize_flags(config)?;
    let lp_flags = lp_flags.map(CString::new).transpose().map_err(|_| {
        SolverError::InvalidSettings("xpress.lp_algorithm contains NUL".to_string())
    })?;
    let lp_flags_ptr = lp_flags
        .as_ref()
        .map_or(std::ptr::null(), |flags| flags.as_ptr());

    let run_start = Instant::now();
    if has_integer {
        ffi::check_xprs(unsafe { (api.xprs_mipoptimize)(prob, std::ptr::null()) }).map_err(
            |rc| xpress_failure_error_for_stats(api, prob, "XPRSmipoptimize", rc, &prepared.stats),
        )?;
    } else {
        ffi::check_xprs(unsafe { (api.xprs_lpoptimize)(prob, lp_flags_ptr) }).map_err(|rc| {
            xpress_failure_error_for_stats(api, prob, "XPRSlpoptimize", rc, &prepared.stats)
        })?;
    }
    let run_seconds = run_start.elapsed().as_secs_f64();
    let solve_time_seconds = prepared.solve_started.elapsed().as_secs_f64();

    let (core_status, has_solution, status_string) = if has_integer {
        let raw = get_int_attrib(api, prob, ffi::XPRS_MIPSTATUS)?;
        (
            status::mip_status_to_core(raw),
            status::mip_has_solution(raw),
            status::mip_status_string(raw),
        )
    } else {
        let raw = get_int_attrib(api, prob, ffi::XPRS_LPSTATUS)?;
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
        solver_status = status_string,
        is_mip = has_integer,
        duration_ms = solve_time_seconds * 1000.0,
        "Xpress solve completed"
    );

    if !has_solution {
        warn!(
            component = "solver",
            operation = "solve",
            solver = "xpress",
            solver_status = status_string,
            duration_ms = solve_time_seconds * 1000.0,
            "Solver did not return a feasible solution"
        );
        return Err(SolverError::SolveFailure {
            status: core_status,
        });
    }

    let objective_value = if has_integer {
        get_dbl_attrib(api, prob, ffi::XPRS_MIPOBJVAL)?
    } else {
        get_dbl_attrib(api, prob, ffi::XPRS_LPOBJVAL)?
    };

    let extract_solution = config
        .parameters
        .get("arco.extract_solution")
        .is_none_or(|value| value != "false");
    let extract_start = Instant::now();
    let (primal_values, variable_duals, row_values, constraint_duals) = if extract_solution {
        let mut primal_values = vec![0.0; ncols];
        let mut variable_duals = vec![0.0; ncols];
        let mut row_values = vec![0.0; nrows];
        let mut constraint_duals = vec![0.0; nrows];
        if has_integer {
            ffi::check_xprs(unsafe {
                (api.xprs_getmipsol)(prob, primal_values.as_mut_ptr(), row_values.as_mut_ptr())
            })
            .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetmipsol failed: {rc}")))?;
            (primal_values, variable_duals, row_values, constraint_duals)
        } else {
            ffi::check_xprs(unsafe {
                (api.xprs_getlpsol)(
                    prob,
                    primal_values.as_mut_ptr(),
                    row_values.as_mut_ptr(),
                    constraint_duals.as_mut_ptr(),
                    variable_duals.as_mut_ptr(),
                )
            })
            .map_err(|rc| SolverError::SolverSpecific(format!("XPRSgetlpsol failed: {rc}")))?;
            (primal_values, variable_duals, row_values, constraint_duals)
        }
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };
    let solution_extract_seconds = extract_start.elapsed().as_secs_f64();

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "xpress_matrix_build_s".to_string(),
        prepared.matrix_build_seconds,
    );
    metadata.insert("xpress_run_s".to_string(), run_seconds);
    metadata.insert("solution_extract_s".to_string(), solution_extract_seconds);
    metadata.insert("num_variables".to_string(), prepared.stats.variables as f64);
    metadata.insert(
        "num_constraints".to_string(),
        prepared.stats.constraints as f64,
    );
    metadata.insert(
        "num_coefficients".to_string(),
        prepared.stats.coefficients as f64,
    );

    Ok(SolveArtifacts {
        solution: Solution {
            primal_values,
            variable_duals,
            constraint_duals,
            row_values,
            objective_value,
            core_status,
            is_mip: has_integer,
            solve_time_seconds,
        },
        metadata,
    })
}

fn solve_problem(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<SolveArtifacts, SolverError> {
    finish_prepared_problem(prepare_problem(model, config)?, config)
}

/// Xpress backend registration object for primitive model views.
#[derive(Debug, Default, Clone, Copy)]
pub struct XpressModelViewBackend;

impl ModelViewBackend for XpressModelViewBackend {
    fn family(&self) -> &'static str {
        "xpress"
    }

    fn solve_model_view(
        &self,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        solve_model_view(model, config)
    }
}

/// Solve a primitive model view with Xpress and return the solver-contract result envelope.
pub fn solve_model_view(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let prepared = prepare_model_view_solve(model, config)?;
    let result = finish_prepared_model_view_solve(prepared, config)?;
    validate_model_view_solve_result_with_config(model, &result, config)?;
    Ok(result)
}

/// Solve an owned core model with Xpress, allowing callers to release Arco
/// model memory after the Xpress problem has been loaded.
pub fn solve_owned_model(
    mut model: arco_model::Model,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let prepared_load = prepare_owned_model_view_load(&mut model, config)?;
    drop(model);
    let prepared = PreparedModelViewSolve {
        problem: load_prepared_problem(prepared_load.load_data, config)?,
        fingerprint: prepared_load.fingerprint,
        fingerprint_seconds: prepared_load.fingerprint_seconds,
        num_variables: prepared_load.num_variables,
        num_constraints: prepared_load.num_constraints,
    };
    finish_prepared_model_view_solve(prepared, config)
}

fn prepare_model_view_load(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<PreparedModelViewLoad, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let load_data = prepare_load_data(model)?;

    let fingerprint_start = Instant::now();
    let fingerprint = if config
        .parameters
        .get("arco.fingerprint")
        .is_none_or(|value| value != "false")
    {
        model.fingerprint()
    } else {
        ModelFingerprint(0)
    };
    let fingerprint_seconds = fingerprint_start.elapsed().as_secs_f64();

    Ok(PreparedModelViewLoad {
        num_variables: load_data.stats.variables,
        num_constraints: load_data.stats.constraints,
        load_data,
        fingerprint,
        fingerprint_seconds,
    })
}

fn prepare_owned_model_view_load(
    model: &mut arco_model::Model,
    config: &SolverConfig,
) -> Result<PreparedModelViewLoad, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let fingerprint_start = Instant::now();
    let fingerprint = if config
        .parameters
        .get("arco.fingerprint")
        .is_none_or(|value| value != "false")
    {
        model.fingerprint()
    } else {
        ModelFingerprint(0)
    };
    let fingerprint_seconds = fingerprint_start.elapsed().as_secs_f64();

    let mut objective_terms = model.take_objective_terms_for_consumed_solve();
    retain_active_objective_terms(model, &mut objective_terms, model.num_variables())?;
    let load_data = prepare_load_data_with_objective(
        model,
        XpressObjectiveCoefficients::Sparse(objective_terms),
    )?;

    Ok(PreparedModelViewLoad {
        num_variables: load_data.stats.variables,
        num_constraints: load_data.stats.constraints,
        load_data,
        fingerprint,
        fingerprint_seconds,
    })
}

fn prepare_model_view_solve(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<PreparedModelViewSolve, SolverError> {
    let prepared_load = prepare_model_view_load(model, config)?;
    Ok(PreparedModelViewSolve {
        problem: load_prepared_problem(prepared_load.load_data, config)?,
        fingerprint: prepared_load.fingerprint,
        fingerprint_seconds: prepared_load.fingerprint_seconds,
        num_variables: prepared_load.num_variables,
        num_constraints: prepared_load.num_constraints,
    })
}

fn finish_prepared_model_view_solve(
    prepared: PreparedModelViewSolve,
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    let fingerprint = prepared.fingerprint;
    let fingerprint_seconds = prepared.fingerprint_seconds;
    let num_variables = prepared.num_variables;
    let num_constraints = prepared.num_constraints;
    let SolveArtifacts {
        solution,
        mut metadata,
    } = finish_prepared_problem(prepared.problem, config)?;
    metadata.insert("fingerprint_s".to_string(), fingerprint_seconds);

    let result = ModelViewSolveResult {
        fingerprint,
        status: solution.core_status,
        objective_value: solution.objective_value,
        primal_values: solution.primal_values,
        variable_duals: solution.variable_duals,
        row_values: solution.row_values,
        constraint_duals: solution.constraint_duals,
        metadata,
    };
    validate_result_shape_counts(&result, config, num_variables, num_constraints)?;
    Ok(result)
}

fn validate_result_shape_counts(
    result: &ModelViewSolveResult,
    config: &SolverConfig,
    num_variables: usize,
    num_constraints: usize,
) -> Result<(), SolverError> {
    let allow_omitted_primal_values = config
        .parameters
        .get("arco.extract_solution")
        .is_some_and(|value| value == "false");
    if allow_omitted_primal_values {
        validate_optional_result_len("primal_values", result.primal_values.len(), num_variables)?;
    } else {
        validate_required_result_len("primal_values", result.primal_values.len(), num_variables)?;
    }
    validate_optional_result_len("variable_duals", result.variable_duals.len(), num_variables)?;
    validate_optional_result_len("row_values", result.row_values.len(), num_constraints)?;
    validate_optional_result_len(
        "constraint_duals",
        result.constraint_duals.len(),
        num_constraints,
    )?;
    Ok(())
}

fn validate_required_result_len(
    name: &str,
    actual: usize,
    expected: usize,
) -> Result<(), SolverError> {
    if actual == expected {
        return Ok(());
    }
    Err(SolverError::InvalidResultShape(format!(
        "{name} length {actual} does not match expected {expected}"
    )))
}

fn validate_optional_result_len(
    name: &str,
    actual: usize,
    expected: usize,
) -> Result<(), SolverError> {
    if actual == 0 || actual == expected {
        return Ok(());
    }
    Err(SolverError::InvalidResultShape(format!(
        "{name} length {actual} must be 0 or match expected {expected}"
    )))
}

pub struct Solver<'model> {
    model: &'model dyn ModelView,
    config: SolverConfig,
}

impl<'model> Solver<'model> {
    pub fn new(model: &'model impl ModelView) -> Result<Self, SolverError> {
        if model.num_variables() == 0 {
            return Err(SolverError::EmptyModel);
        }
        if model.objective().sense.is_none() && model.objective().terms.is_empty() {
            return Err(SolverError::NoObjective);
        }
        Ok(Self {
            model,
            config: SolverConfig::new(),
        })
    }

    fn update_config(&mut self, update: impl FnOnce(SolverConfig) -> SolverConfig) {
        self.config = update(std::mem::take(&mut self.config));
    }

    pub fn set_log_to_console(&mut self, enabled: bool) {
        self.update_config(|config| config.with_log_to_console(enabled));
    }

    pub fn set_time_limit(&mut self, seconds: f64) {
        self.update_config(|config| config.with_time_limit(seconds));
    }

    pub fn set_mip_gap(&mut self, gap: f64) {
        self.update_config(|config| config.with_mip_gap(gap));
    }

    pub fn set_verbosity(&mut self, level: u32) {
        self.update_config(|config| config.with_verbosity(level));
    }

    pub fn set_presolve(&mut self, enabled: bool) {
        self.update_config(|config| config.with_presolve(enabled));
    }

    pub fn set_threads(&mut self, threads: u32) {
        self.update_config(|config| config.with_threads(threads));
    }

    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.update_config(|config| config.with_tolerance(tolerance));
    }

    pub fn config(&self) -> &SolverConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: SolverConfig) {
        self.config = config;
    }

    pub fn solve(&mut self) -> Result<Solution, SolverError> {
        self.solve_with_config(&self.config)
    }

    pub fn solve_with_config(&self, config: &SolverConfig) -> Result<Solution, SolverError> {
        solve_problem(self.model, config).map(|artifacts| artifacts.solution)
    }
}

impl Solve for Solver<'_> {
    type Solution = Solution;

    fn solve(&mut self, config: &SolverConfig) -> Result<Self::Solution, SolverError> {
        self.solve_with_config(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_model::{Bounds, Constraint, Model, Objective, Variable};
    use arco_solver::{check_empty_model_rejected, check_no_objective_rejected};

    fn build_simple_model() -> Model {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("variable");
        let demand = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        model.set_coefficient(x, demand, 1.0).expect("coefficient");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 2.0)],
            })
            .expect("objective");
        model
    }

    #[test]
    fn detects_local_install_in_home_opt_directory() {
        let path = detect_xpress_dir();
        assert!(path.is_some() || !xpress_runtime_available());
    }

    #[test]
    fn model_view_backend_rejects_empty_model() {
        let backend = XpressModelViewBackend;
        check_empty_model_rejected(&backend).expect("Xpress should reject empty model");
    }

    #[test]
    fn model_view_backend_rejects_no_objective_model() {
        let backend = XpressModelViewBackend;
        check_no_objective_rejected(&backend).expect("Xpress should reject missing objective");
    }

    #[test]
    fn owned_model_solver_rejects_empty_model_before_runtime_setup() {
        let model = Model::new();
        let error = solve_owned_model(model, &SolverConfig::new())
            .expect_err("owned Xpress solve should reject an empty model");

        assert!(matches!(error, SolverError::EmptyModel));
    }

    #[test]
    fn owned_model_solver_rejects_no_objective_model_before_runtime_setup() {
        let mut model = Model::new();
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("variable");

        let error = solve_owned_model(model, &SolverConfig::new())
            .expect_err("owned Xpress solve should reject missing objective");

        assert!(matches!(error, SolverError::NoObjective));
    }

    #[test]
    fn solver_wrapper_rejects_empty_model() {
        let model = Model::new();
        assert!(matches!(Solver::new(&model), Err(SolverError::EmptyModel)));
    }

    #[test]
    fn solver_wrapper_rejects_no_objective_model() {
        let mut model = Model::new();
        model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .expect("variable");

        assert!(matches!(Solver::new(&model), Err(SolverError::NoObjective)));
    }

    #[test]
    fn solver_wrapper_tracks_configuration_updates() {
        let model = build_simple_model();
        let mut solver = Solver::new(&model).expect("solver");
        solver.set_threads(2);
        solver.set_time_limit(5.0);
        solver.set_log_to_console(false);

        assert_eq!(solver.config().threads, Some(2));
        assert_eq!(solver.config().time_limit, Some(5.0));
        assert_eq!(solver.config().log_to_console, Some(false));
    }

    #[test]
    fn xpress_column_matrix_reserves_known_coefficient_count() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("x variable");
        let y = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("y variable");
        let first = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("first constraint");
        let second = model
            .add_constraint(Constraint {
                bounds: Bounds::new(f64::NEG_INFINITY, 3.0),
            })
            .expect("second constraint");
        model.set_coefficient(x, first, 1.5).expect("x first");
        model.set_coefficient(x, second, -2.0).expect("x second");
        model.set_coefficient(y, second, 4.0).expect("y second");

        let matrix = build_xpress_column_matrix(
            &model,
            model.num_variables(),
            model.num_constraints(),
            model.num_coefficients(),
        )
        .expect("matrix build");

        assert!(matrix.mrwind.capacity() >= model.num_coefficients());
        assert!(matrix.dmatval.capacity() >= model.num_coefficients());
        assert_eq!(matrix.mstart, vec![0, 2, 3]);
        assert_eq!(matrix.mrwind, vec![0, 1, 1]);
        assert_eq!(matrix.dmatval, vec![1.5, -2.0, 4.0]);
    }

    #[test]
    fn xpress_load_data_builds_without_runtime_initialization() {
        let model = build_simple_model();

        let load_data = prepare_load_data(&model).expect("load data should build");

        assert_eq!(load_data.ncols, model.num_variables());
        assert_eq!(load_data.nrows, model.num_constraints());
        assert!(!load_data.has_integer);
        assert_eq!(load_data.stats.variables, 1);
        assert_eq!(load_data.stats.constraints, 1);
        assert_eq!(load_data.stats.coefficients, 1);
        assert_eq!(load_data.objective_coefficients.into_dense(1), vec![2.0]);
        match &load_data.variable_bounds {
            XpressVariableBounds::DefaultNonnegativeWithUpperChanges {
                upper_indices,
                upper_types,
                upper_values,
            } => {
                assert!(upper_indices.is_empty());
                assert!(upper_types.is_empty());
                assert!(upper_values.is_empty());
            }
            XpressVariableBounds::Full { .. } => {
                panic!("default nonnegative LP should not allocate full bound arrays")
            }
        }
        assert_eq!(load_data.row_types, vec![b'G']);
        assert_eq!(load_data.rhs, vec![1.0]);
        assert!(load_data.rng.is_none());
        assert_eq!(load_data.matrix.mstart, vec![0, 1]);
        assert_eq!(load_data.matrix.mrwind, vec![0]);
        assert_eq!(load_data.matrix.dmatval, vec![1.0]);
    }

    #[test]
    fn xpress_load_data_defers_only_finite_upper_bound_changes() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 7.0)))
            .expect("bounded variable");
        let y = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("unbounded variable");
        let demand = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        model
            .set_coefficient(x, demand, 1.0)
            .expect("x coefficient");
        model
            .set_coefficient(y, demand, 1.0)
            .expect("y coefficient");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.0), (y, 1.0)],
            })
            .expect("objective");

        let load_data = prepare_load_data(&model).expect("load data should build");

        match &load_data.variable_bounds {
            XpressVariableBounds::DefaultNonnegativeWithUpperChanges {
                upper_indices,
                upper_types,
                upper_values,
            } => {
                assert_eq!(upper_indices, &vec![0]);
                assert_eq!(upper_types, &vec![b'U']);
                assert_eq!(upper_values, &vec![7.0]);
            }
            XpressVariableBounds::Full { .. } => {
                panic!("finite upper bounds should be deferred for default nonnegative LPs")
            }
        }
    }

    #[test]
    fn xpress_load_data_uses_full_bounds_for_nonzero_lower_bounds() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(-1.0, 7.0)))
            .expect("bounded variable");
        let demand = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        model.set_coefficient(x, demand, 1.0).expect("coefficient");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.0)],
            })
            .expect("objective");

        let load_data = prepare_load_data(&model).expect("load data should build");

        match &load_data.variable_bounds {
            XpressVariableBounds::Full {
                lower_bounds,
                upper_bounds,
            } => {
                assert_eq!(lower_bounds, &vec![-1.0]);
                assert_eq!(upper_bounds, &vec![7.0]);
            }
            XpressVariableBounds::DefaultNonnegativeWithUpperChanges { .. } => {
                panic!("nonzero lower bounds require full bound arrays")
            }
        }
    }

    #[test]
    fn xpress_load_data_allocates_range_values_only_for_ranged_rows() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("variable");
        let lower_only = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("lower-only constraint");
        let ranged = model
            .add_constraint(Constraint {
                bounds: Bounds::new(2.0, 5.0),
            })
            .expect("ranged constraint");
        model
            .set_coefficient(x, lower_only, 1.0)
            .expect("lower coefficient");
        model
            .set_coefficient(x, ranged, 1.0)
            .expect("ranged coefficient");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 2.0)],
            })
            .expect("objective");

        let load_data = prepare_load_data(&model).expect("load data should build");

        assert_eq!(load_data.row_types, vec![b'G', b'R']);
        assert_eq!(load_data.rhs, vec![1.0, 2.0]);
        assert_eq!(load_data.rng, Some(vec![0.0, 3.0]));
    }

    #[test]
    fn owned_xpress_load_data_drains_sparse_objective_without_runtime_initialization() {
        let mut model = Model::new();
        let active = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .expect("active variable");
        let inactive = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, f64::INFINITY),
                is_integer: false,
                is_active: false,
            })
            .expect("inactive variable");
        let demand = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, f64::INFINITY),
            })
            .expect("constraint");
        model
            .set_coefficient(active, demand, 1.0)
            .expect("coefficient");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(active, 2.0), (inactive, 7.0)],
            })
            .expect("objective");

        let prepared = prepare_owned_model_view_load(
            &mut model,
            &SolverConfig::new().with_parameter("arco.fingerprint", "false"),
        )
        .expect("owned load data should build");

        assert!(model.objective().terms.is_empty());
        match &prepared.load_data.objective_coefficients {
            XpressObjectiveCoefficients::Sparse(terms) => {
                assert_eq!(terms.as_slice(), &[(active, 2.0)]);
            }
            XpressObjectiveCoefficients::Dense(_) => {
                panic!("owned load data should retain sparse objective terms")
            }
        }
        assert_eq!(
            prepared.load_data.objective_coefficients.into_dense(2),
            vec![2.0, 0.0]
        );
    }

    #[test]
    fn reject_invalid_numeric_solver_settings() {
        let error = validate_solver_config(&SolverConfig::new().with_time_limit(-1.0))
            .expect_err("negative time_limit should be rejected");
        assert!(matches!(
            error,
            SolverError::InvalidSettings(message) if message == "time_limit must be finite and >= 0"
        ));

        let error = validate_solver_config(&SolverConfig::new().with_mip_gap(f64::NAN))
            .expect_err("non-finite mip_gap should be rejected");
        assert!(matches!(
            error,
            SolverError::InvalidSettings(message) if message == "mip_gap must be finite and >= 0"
        ));

        let error = validate_solver_config(&SolverConfig::new().with_tolerance(-0.5))
            .expect_err("negative tolerance should be rejected");
        assert!(matches!(
            error,
            SolverError::InvalidSettings(message) if message == "tolerance must be finite and >= 0"
        ));
    }

    #[test]
    fn reject_zero_threads() {
        let error = validate_solver_config(&SolverConfig::new().with_threads(0))
            .expect_err("zero threads should be rejected");
        assert!(matches!(
            error,
            SolverError::InvalidSettings(message) if message == "threads must be >= 1"
        ));
    }

    #[test]
    fn maps_xpress_lp_algorithm_parameter_to_optimizer_flags() {
        for (value, expected) in [
            ("auto", None),
            ("", None),
            ("primal", Some("p")),
            ("p", Some("p")),
            ("dual", Some("d")),
            ("d", Some("d")),
            ("barrier", Some("b")),
            ("b", Some("b")),
            ("primal_barrier", Some("pb")),
            ("dual_barrier", Some("db")),
            ("primal_dual", Some("pd")),
            ("all", Some("pdb")),
        ] {
            let config = SolverConfig::new().with_parameter("xpress.lp_algorithm", value);

            let flags = lp_optimize_flags(&config).expect("algorithm should be accepted");

            assert_eq!(flags, expected);
        }
    }

    #[test]
    fn rejects_invalid_xpress_lp_algorithm_parameter_before_runtime_setup() {
        let config = SolverConfig::new().with_parameter("xpress.lp_algorithm", "not-an-algorithm");

        let error =
            validate_solver_config(&config).expect_err("invalid LP algorithm should be rejected");

        assert!(matches!(
            error,
            SolverError::InvalidSettings(message)
                if message.contains("xpress.lp_algorithm")
                    && message.contains("not-an-algorithm")
        ));
    }

    #[test]
    fn formats_xpress_failure_message_with_last_error() {
        let message =
            format_xpress_failure_message("XPRSlpoptimize", 120, Some("invalid identifier 'gen'"));

        assert_eq!(
            message,
            "Xpress solve failed (XPRSlpoptimize, rc=120): invalid identifier 'gen'"
        );
    }

    #[test]
    fn formats_xpress_failure_message_without_last_error() {
        let message = format_xpress_failure_message("XPRSloadlp", 1157, None);

        assert_eq!(message, "Xpress solve failed (XPRSloadlp, rc=1157)");
    }

    #[test]
    fn normalizes_xpress_community_size_limit_message() {
        let message = normalize_xpress_last_error(
            "?120 Error: Problem has too many rows and columns. The maximum is 5000",
        );

        assert_eq!(
            message,
            "Xpress Community Edition size limit exceeded: this model is larger than the 5000 row/column maximum. Try a smaller instance or use HiGHS/full Xpress."
        );
    }
}
