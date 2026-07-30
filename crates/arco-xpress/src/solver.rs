//! Xpress solver implementation over primitive model views.

use crate::ffi;
use crate::solution::Solution;
use crate::status;
use arco_model::{ConstraintId, ModelFingerprint, ModelView, Sense, VariableId};
use arco_solver::{
    LpAlgorithm, ModelViewBackend, ModelViewSolveResult, Solve, SolverConfig, SolverDiagnostic,
    SolverError, SolverModelStats, validate_model_view_solve_result,
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

pub(crate) fn license_candidates(xpress_dir: Option<&Path>) -> Vec<PathBuf> {
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

fn xpress_failure_error(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    action: &str,
    rc: c_int,
    model: &(impl ModelView + ?Sized),
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
            model: xpress_model_stats(model),
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

fn lp_optimizer_flags(config: &SolverConfig) -> Result<Option<&'static CStr>, SolverError> {
    match config.lp_algorithm {
        None | Some(LpAlgorithm::Automatic) => Ok(None),
        Some(LpAlgorithm::PrimalSimplex) => Ok(Some(c"p")),
        Some(LpAlgorithm::DualSimplex) => Ok(Some(c"d")),
        Some(LpAlgorithm::Barrier | LpAlgorithm::BarrierWithCrossover) => Ok(Some(c"b")),
        Some(LpAlgorithm::Concurrent) => Ok(Some(c"pdb")),
        Some(LpAlgorithm::PrimalDualFirstOrder) => Err(SolverError::InvalidSettings(
            "lp_algorithm 'primal_dual_first_order' is not supported by the Xpress backend"
                .to_string(),
        )),
    }
}

fn lp_crossover_setting(config: &SolverConfig) -> Option<c_int> {
    match config.lp_algorithm {
        Some(LpAlgorithm::Barrier) => Some(0),
        Some(LpAlgorithm::BarrierWithCrossover) => Some(1),
        _ => None,
    }
}

fn validate_solver_config(config: &SolverConfig) -> Result<(), SolverError> {
    ensure_non_negative_finite_setting("time_limit", config.time_limit)?;
    ensure_non_negative_finite_setting("mip_gap", config.mip_gap)?;
    ensure_non_negative_finite_setting("tolerance", config.tolerance)?;
    let _ = lp_optimizer_flags(config)?;

    if let Some(0) = config.threads {
        return Err(SolverError::InvalidSettings(
            "threads must be >= 1".to_string(),
        ));
    }
    Ok(())
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
    let callback_name = api
        .message_callback_symbol()
        .unwrap_or("Xpress message callback registration");
    let callback_result = unsafe {
        api.register_message_callback(prob, Some(xpress_message_callback), std::ptr::null_mut())
    }
    .map_err(|error| SolverError::SolverSpecific(error.to_string()))?;
    ffi::check_xprs(callback_result)
        .map_err(|rc| SolverError::SolverSpecific(format!("{callback_name} failed: {rc}")))?;
    set_int_control(api, prob, ffi::XPRS_LPLOG, 1)?;
    set_int_control(api, prob, ffi::XPRS_MIPLOG, 1)
}

fn apply_solver_config(
    api: &'static ffi::Api,
    prob: ffi::XPRSprob,
    config: &SolverConfig,
) -> Result<(), SolverError> {
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
    if let Some(crossover) = lp_crossover_setting(config) {
        set_int_control(api, prob, ffi::XPRS_CROSSOVER, crossover)?;
    }

    Ok(())
}

struct SolveArtifacts {
    solution: Solution,
    metadata: BTreeMap<String, f64>,
}

#[allow(unsafe_code)]
fn solve_problem(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<SolveArtifacts, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    validate_solver_config(config)?;
    let optimizer_flags_ptr = lp_optimizer_flags(config)?.map_or(std::ptr::null(), CStr::as_ptr);

    let solve_started = Instant::now();
    let ncols = model.num_variables();
    let nrows = model.num_constraints();
    let sense = model.objective().sense.unwrap_or(Sense::Minimize);

    debug!(
        component = "solver",
        operation = "solve",
        solver = "xpress",
        variables = ncols as u64,
        constraints = nrows as u64,
        "Starting Xpress solve"
    );

    let mut objective_coefficients = vec![0.0; ncols];
    for (variable_id, coefficient) in &model.objective().terms {
        let index = variable_id.inner() as usize;
        let variable = model
            .variable(*variable_id)
            .ok_or(SolverError::InvalidVariableId(variable_id.inner()))?;
        if variable.is_active && index < objective_coefficients.len() {
            objective_coefficients[index] += *coefficient;
        }
    }

    let mut lower_bounds = Vec::with_capacity(ncols);
    let mut upper_bounds = Vec::with_capacity(ncols);
    let mut col_types = Vec::new();
    let mut int_col_indices = Vec::new();
    let mut int_col_limits = Vec::new();
    let mut has_integer = false;

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
            int_col_limits.push(lower_bounds[index]);
        }
    }

    let mut row_types = Vec::with_capacity(nrows);
    let mut rhs = Vec::with_capacity(nrows);
    let mut rng = Vec::with_capacity(nrows);
    for index in 0..nrows {
        let constraint = model
            .constraint(ConstraintId::new(index as u32))
            .ok_or_else(|| SolverError::SolverSpecific(format!("missing constraint {index}")))?;
        let (row_type, rhs_value, range_value) =
            bounds_to_xpress_row(constraint.bounds.lower, constraint.bounds.upper);
        row_types.push(row_type);
        rhs.push(rhs_value);
        rng.push(range_value);
    }

    let matrix_build_start = Instant::now();
    let mut mstart = Vec::with_capacity(ncols + 1);
    let mut mrwind = Vec::new();
    let mut dmatval = Vec::new();
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
    let matrix_build_seconds = matrix_build_start.elapsed().as_secs_f64();

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
                rng.as_ptr(),
                objective_coefficients.as_ptr(),
                mstart.as_ptr(),
                std::ptr::null(),
                mrwind.as_ptr(),
                dmatval.as_ptr(),
                lower_bounds.as_ptr(),
                upper_bounds.as_ptr(),
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
        .map_err(|rc| xpress_failure_error(api, prob, api.mip_loader_symbol(), rc, model))?;
    } else {
        ffi::check_xprs(unsafe {
            (api.xprs_loadlp)(
                prob,
                std::ptr::null(),
                ncols as c_int,
                nrows as c_int,
                row_types.as_ptr().cast::<c_char>(),
                rhs.as_ptr(),
                rng.as_ptr(),
                objective_coefficients.as_ptr(),
                mstart.as_ptr(),
                std::ptr::null(),
                mrwind.as_ptr(),
                dmatval.as_ptr(),
                lower_bounds.as_ptr(),
                upper_bounds.as_ptr(),
            )
        })
        .map_err(|rc| xpress_failure_error(api, prob, "XPRSloadlp", rc, model))?;
    }

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

    let run_start = Instant::now();
    if has_integer {
        ffi::check_xprs(unsafe { (api.xprs_mipoptimize)(prob, optimizer_flags_ptr) })
            .map_err(|rc| xpress_failure_error(api, prob, "XPRSmipoptimize", rc, model))?;
    } else {
        ffi::check_xprs(unsafe { (api.xprs_lpoptimize)(prob, optimizer_flags_ptr) })
            .map_err(|rc| xpress_failure_error(api, prob, "XPRSlpoptimize", rc, model))?;
    }
    let run_seconds = run_start.elapsed().as_secs_f64();
    let solve_time_seconds = solve_started.elapsed().as_secs_f64();

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

    drop(prob_guard);
    drop(env_guard);

    let mut metadata = BTreeMap::new();
    metadata.insert("xpress_matrix_build_s".to_string(), matrix_build_seconds);
    metadata.insert("xpress_run_s".to_string(), run_seconds);
    metadata.insert("solution_extract_s".to_string(), solution_extract_seconds);
    metadata.insert("num_variables".to_string(), ncols as f64);
    metadata.insert("num_constraints".to_string(), nrows as f64);
    metadata.insert(
        "num_coefficients".to_string(),
        model.num_coefficients() as f64,
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
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let SolveArtifacts {
        solution,
        mut metadata,
    } = solve_problem(model, config)?;

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
    metadata.insert(
        "fingerprint_s".to_string(),
        fingerprint_start.elapsed().as_secs_f64(),
    );

    let result = ModelViewSolveResult {
        fingerprint,
        status: solution.core_status(),
        objective_value: solution.objective_value(),
        primal_values: solution.primal_values().to_vec(),
        variable_duals: solution.variable_duals().to_vec(),
        row_values: solution.row_values.clone(),
        constraint_duals: solution.constraint_duals().to_vec(),
        metadata,
    };
    validate_model_view_solve_result(model, &result)?;
    Ok(result)
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

    pub(crate) fn set_time_limit(&mut self, seconds: f64) {
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

    pub(crate) fn set_threads(&mut self, threads: u32) {
        self.update_config(|config| config.with_threads(threads));
    }

    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.update_config(|config| config.with_tolerance(tolerance));
    }

    /// Select the solver-independent LP algorithm preference.
    pub fn set_lp_algorithm(&mut self, algorithm: LpAlgorithm) {
        self.update_config(|config| config.with_lp_algorithm(algorithm));
    }

    pub(crate) fn config(&self) -> &SolverConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: SolverConfig) {
        self.config = config;
    }

    pub fn solve(&mut self) -> Result<Solution, SolverError> {
        self.solve_with_config(&self.config)
    }

    pub(crate) fn solve_with_config(&self, config: &SolverConfig) -> Result<Solution, SolverError> {
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
        solver.set_lp_algorithm(LpAlgorithm::Barrier);

        assert_eq!(solver.config().threads, Some(2));
        assert_eq!(solver.config().time_limit, Some(5.0));
        assert_eq!(solver.config().log_to_console, Some(false));
        assert_eq!(solver.config().lp_algorithm, Some(LpAlgorithm::Barrier));
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
    fn maps_lp_algorithms_to_xpress_optimizer_flags() {
        for (algorithm, expected_flag) in [
            (LpAlgorithm::Automatic, None),
            (LpAlgorithm::PrimalSimplex, Some("p")),
            (LpAlgorithm::DualSimplex, Some("d")),
            (LpAlgorithm::Barrier, Some("b")),
            (LpAlgorithm::BarrierWithCrossover, Some("b")),
            (LpAlgorithm::Concurrent, Some("pdb")),
        ] {
            let config = SolverConfig::new().with_lp_algorithm(algorithm);
            assert_eq!(
                lp_optimizer_flags(&config)
                    .expect("supported LP algorithm should be accepted")
                    .map(CStr::to_str)
                    .transpose()
                    .expect("algorithm flags should be UTF-8"),
                expected_flag,
                "unexpected optimizer flags for {algorithm:?}"
            );
        }
    }

    #[test]
    fn maps_barrier_crossover_modes_to_xpress_control_values() {
        assert_eq!(
            lp_crossover_setting(&SolverConfig::new().with_lp_algorithm(LpAlgorithm::Barrier)),
            Some(0)
        );
        assert_eq!(
            lp_crossover_setting(
                &SolverConfig::new().with_lp_algorithm(LpAlgorithm::BarrierWithCrossover)
            ),
            Some(1)
        );
        assert_eq!(lp_crossover_setting(&SolverConfig::new()), None);
    }

    #[test]
    fn rejects_unsupported_xpress_lp_algorithm() {
        let config = SolverConfig::new().with_lp_algorithm(LpAlgorithm::PrimalDualFirstOrder);
        let error = lp_optimizer_flags(&config)
            .expect_err("unsupported LP algorithm should be rejected before solving");

        assert!(matches!(
            error,
            SolverError::InvalidSettings(message)
                if message.contains("primal_dual_first_order")
                    && message.contains("Xpress backend")
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
