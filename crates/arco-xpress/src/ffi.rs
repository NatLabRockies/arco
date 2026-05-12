//! FFI bindings to the FICO Xpress solver C library (`libxprs`).
//!
//! All `unsafe` interaction with the Xpress C API is isolated in this module.
//! Symbols are resolved at runtime so binaries can be built without the
//! proprietary SDK present.
#![allow(unsafe_code)]
#![allow(non_camel_case_types)]

use std::ffi::{CStr, CString, c_char, c_double, c_int, c_void};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Opaque Xpress problem handle.
pub type XPRSprob = *mut c_void;

pub const XPRS_PLUSINFINITY: f64 = 1.0e20;
pub const XPRS_MINUSINFINITY: f64 = -1.0e20;

// Objective sense
pub const XPRS_OBJ_MINIMIZE: c_int = 1;
pub const XPRS_OBJ_MAXIMIZE: c_int = -1;

// Integer control indices
pub const XPRS_THREADS: c_int = 8278;
pub const XPRS_PRESOLVE: c_int = 8011;
pub const XPRS_LPLOG: c_int = 8009;
pub const XPRS_MIPLOG: c_int = 8028;
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

type XPRSinitFn = unsafe extern "C" fn(*const c_char) -> c_int;
type XPRSfreeFn = unsafe extern "C" fn() -> c_int;
type XPRScreateprobFn = unsafe extern "C" fn(*mut XPRSprob) -> c_int;
type XPRSdestroyprobFn = unsafe extern "C" fn(XPRSprob) -> c_int;
type XPRSloadlpFn = unsafe extern "C" fn(
    XPRSprob,
    *const c_char,
    c_int,
    c_int,
    *const c_char,
    *const c_double,
    *const c_double,
    *const c_double,
    *const c_int,
    *const c_int,
    *const c_int,
    *const c_double,
    *const c_double,
    *const c_double,
) -> c_int;
type XPRSloadmipFn = unsafe extern "C" fn(
    XPRSprob,
    *const c_char,
    c_int,
    c_int,
    *const c_char,
    *const c_double,
    *const c_double,
    *const c_double,
    *const c_int,
    *const c_int,
    *const c_int,
    *const c_double,
    *const c_double,
    *const c_double,
    c_int,
    c_int,
    *const c_char,
    *const c_int,
    *const c_double,
    *const c_char,
    *const c_int,
    *const c_int,
    *const c_double,
) -> c_int;
type XPRSaddqmatrix64Fn = unsafe extern "C" fn(
    XPRSprob,
    c_int,
    i64,
    *const c_int,
    *const c_int,
    *const c_double,
) -> c_int;
type XPRSchgobjsenseFn = unsafe extern "C" fn(XPRSprob, c_int) -> c_int;
type XPRSlpoptimizeFn = unsafe extern "C" fn(XPRSprob, *const c_char) -> c_int;
type XPRSmipoptimizeFn = unsafe extern "C" fn(XPRSprob, *const c_char) -> c_int;
type XPRSgetlpsolFn = unsafe extern "C" fn(
    XPRSprob,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
) -> c_int;
type XPRSgetmipsolFn = unsafe extern "C" fn(XPRSprob, *mut c_double, *mut c_double) -> c_int;
type XPRSaddmipsolFn =
    unsafe extern "C" fn(XPRSprob, c_int, *const c_double, *const c_int, *const c_char) -> c_int;
type XPRSsetintcontrolFn = unsafe extern "C" fn(XPRSprob, c_int, c_int) -> c_int;
type XPRSgetintcontrolFn = unsafe extern "C" fn(XPRSprob, c_int, *mut c_int) -> c_int;
type XPRSsetdblcontrolFn = unsafe extern "C" fn(XPRSprob, c_int, c_double) -> c_int;
type XPRSgetdblcontrolFn = unsafe extern "C" fn(XPRSprob, c_int, *mut c_double) -> c_int;
type XPRSgetintattribFn = unsafe extern "C" fn(XPRSprob, c_int, *mut c_int) -> c_int;
type XPRSgetdblattribFn = unsafe extern "C" fn(XPRSprob, c_int, *mut c_double) -> c_int;
type XPRSlicenseFn = unsafe extern "C" fn(*mut c_int, *const c_char) -> c_int;
type XPRSgetlicerrmsgFn = unsafe extern "C" fn(*mut c_char, c_int) -> c_int;
type XPRSgetversionFn = unsafe extern "C" fn(*mut c_char) -> c_int;
type XPRSgetbannerFn = unsafe extern "C" fn(*mut c_char) -> c_int;
type XPRSmessageCallbackFn =
    unsafe extern "C" fn(XPRSprob, *mut c_void, *const c_char, c_int, c_int);
type XPRSsetcbmessageFn =
    unsafe extern "C" fn(XPRSprob, Option<XPRSmessageCallbackFn>, *mut c_void) -> c_int;

#[derive(Clone, Debug)]
enum LibraryTarget {
    Path(PathBuf),
    Name(&'static str),
}

impl LibraryTarget {
    fn display_name(&self) -> String {
        match self {
            Self::Path(path) => path.display().to_string(),
            Self::Name(name) => (*name).to_string(),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeLoadError {
    details: String,
}

impl RuntimeLoadError {
    fn new(details: String) -> Self {
        Self { details }
    }
}

impl Display for RuntimeLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.details)
    }
}

impl std::error::Error for RuntimeLoadError {}

pub struct Api {
    pub xprs_init: XPRSinitFn,
    pub xprs_free: XPRSfreeFn,
    pub xprs_createprob: XPRScreateprobFn,
    pub xprs_destroyprob: XPRSdestroyprobFn,
    pub xprs_loadlp: XPRSloadlpFn,
    pub xprs_loadmip: XPRSloadmipFn,
    pub xprs_addqmatrix64: XPRSaddqmatrix64Fn,
    pub xprs_chgobjsense: XPRSchgobjsenseFn,
    pub xprs_lpoptimize: XPRSlpoptimizeFn,
    pub xprs_mipoptimize: XPRSmipoptimizeFn,
    pub xprs_getlpsol: XPRSgetlpsolFn,
    pub xprs_getmipsol: XPRSgetmipsolFn,
    pub xprs_addmipsol: XPRSaddmipsolFn,
    pub xprs_setintcontrol: XPRSsetintcontrolFn,
    pub xprs_getintcontrol: XPRSgetintcontrolFn,
    pub xprs_setdblcontrol: XPRSsetdblcontrolFn,
    pub xprs_getdblcontrol: XPRSgetdblcontrolFn,
    pub xprs_getintattrib: XPRSgetintattribFn,
    pub xprs_getdblattrib: XPRSgetdblattribFn,
    pub xprs_license: XPRSlicenseFn,
    pub xprs_getlicerrmsg: XPRSgetlicerrmsgFn,
    pub xprs_getversion: XPRSgetversionFn,
    pub xprs_getbanner: XPRSgetbannerFn,
    pub xprs_setcbmessage: XPRSsetcbmessageFn,
}

static API: OnceLock<Result<Api, RuntimeLoadError>> = OnceLock::new();

macro_rules! load_symbol {
    ($handle:expr, $symbol:literal, $ty:ty) => {{
        let raw = lookup_symbol($handle, concat!($symbol, "\0").as_bytes())?;
        unsafe { std::mem::transmute::<*mut c_void, $ty>(raw) }
    }};
}

fn env_xpress_dir() -> Option<PathBuf> {
    std::env::var_os("XPRESSDIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn runtime_library_candidates(xpress_dir: Option<&Path>) -> Vec<LibraryTarget> {
    let mut candidates = Vec::new();

    if let Some(dir) = xpress_dir {
        #[cfg(target_os = "macos")]
        {
            candidates.push(LibraryTarget::Path(dir.join("lib").join("libxprs.dylib")));
        }
        #[cfg(target_os = "linux")]
        {
            candidates.push(LibraryTarget::Path(dir.join("lib").join("libxprs.so")));
            candidates.extend(versioned_unix_library_candidates(&dir.join("lib")));
        }
        #[cfg(windows)]
        {
            candidates.push(LibraryTarget::Path(dir.join("bin").join("xprs.dll")));
            candidates.push(LibraryTarget::Path(dir.join("lib").join("xprs.dll")));
        }
    }

    #[cfg(target_os = "macos")]
    {
        candidates.push(LibraryTarget::Name("libxprs.dylib"));
    }
    #[cfg(target_os = "linux")]
    {
        candidates.push(LibraryTarget::Name("libxprs.so"));
    }
    #[cfg(windows)]
    {
        candidates.push(LibraryTarget::Name("xprs.dll"));
    }

    candidates
}

#[cfg(target_os = "linux")]
fn versioned_unix_library_candidates(lib_dir: &Path) -> Vec<LibraryTarget> {
    let Ok(entries) = std::fs::read_dir(lib_dir) else {
        return Vec::new();
    };

    let mut candidates = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("libxprs.so."))
        })
        .map(LibraryTarget::Path)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.display_name().cmp(&right.display_name()));
    candidates
}

pub fn runtime_library_available(xpress_dir: Option<&Path>) -> bool {
    runtime_library_candidates(xpress_dir)
        .into_iter()
        .any(|target| open_library(&target).is_ok_and(|handle| close_library(handle).is_ok()))
}

pub fn api() -> Result<&'static Api, RuntimeLoadError> {
    API.get_or_init(load_api)
        .as_ref()
        .map_err(|error| RuntimeLoadError {
            details: error.details.clone(),
        })
}

fn load_api() -> Result<Api, RuntimeLoadError> {
    let candidates = runtime_library_candidates(env_xpress_dir().as_deref());
    let mut failures = Vec::new();

    for target in candidates {
        match open_library(&target) {
            Ok(handle) => {
                return Ok(Api {
                    xprs_init: load_symbol!(handle, "XPRSinit", XPRSinitFn),
                    xprs_free: load_symbol!(handle, "XPRSfree", XPRSfreeFn),
                    xprs_createprob: load_symbol!(handle, "XPRScreateprob", XPRScreateprobFn),
                    xprs_destroyprob: load_symbol!(handle, "XPRSdestroyprob", XPRSdestroyprobFn),
                    xprs_loadlp: load_symbol!(handle, "XPRSloadlp", XPRSloadlpFn),
                    xprs_loadmip: load_symbol!(handle, "XPRSloadmip", XPRSloadmipFn),
                    xprs_addqmatrix64: load_symbol!(handle, "XPRSaddqmatrix64", XPRSaddqmatrix64Fn),
                    xprs_chgobjsense: load_symbol!(handle, "XPRSchgobjsense", XPRSchgobjsenseFn),
                    xprs_lpoptimize: load_symbol!(handle, "XPRSlpoptimize", XPRSlpoptimizeFn),
                    xprs_mipoptimize: load_symbol!(handle, "XPRSmipoptimize", XPRSmipoptimizeFn),
                    xprs_getlpsol: load_symbol!(handle, "XPRSgetlpsol", XPRSgetlpsolFn),
                    xprs_getmipsol: load_symbol!(handle, "XPRSgetmipsol", XPRSgetmipsolFn),
                    xprs_addmipsol: load_symbol!(handle, "XPRSaddmipsol", XPRSaddmipsolFn),
                    xprs_setintcontrol: load_symbol!(
                        handle,
                        "XPRSsetintcontrol",
                        XPRSsetintcontrolFn
                    ),
                    xprs_getintcontrol: load_symbol!(
                        handle,
                        "XPRSgetintcontrol",
                        XPRSgetintcontrolFn
                    ),
                    xprs_setdblcontrol: load_symbol!(
                        handle,
                        "XPRSsetdblcontrol",
                        XPRSsetdblcontrolFn
                    ),
                    xprs_getdblcontrol: load_symbol!(
                        handle,
                        "XPRSgetdblcontrol",
                        XPRSgetdblcontrolFn
                    ),
                    xprs_getintattrib: load_symbol!(handle, "XPRSgetintattrib", XPRSgetintattribFn),
                    xprs_getdblattrib: load_symbol!(handle, "XPRSgetdblattrib", XPRSgetdblattribFn),
                    xprs_license: load_symbol!(handle, "XPRSlicense", XPRSlicenseFn),
                    xprs_getlicerrmsg: load_symbol!(handle, "XPRSgetlicerrmsg", XPRSgetlicerrmsgFn),
                    xprs_getversion: load_symbol!(handle, "XPRSgetversion", XPRSgetversionFn),
                    xprs_getbanner: load_symbol!(handle, "XPRSgetbanner", XPRSgetbannerFn),
                    xprs_setcbmessage: load_symbol!(handle, "XPRSsetcbmessage", XPRSsetcbmessageFn),
                });
            }
            Err(error) => failures.push(format!("{} ({error})", target.display_name())),
        }
    }

    Err(RuntimeLoadError::new(format!(
        "unable to load Xpress runtime library; tried: {}",
        failures.join(", ")
    )))
}

/// Check an Xpress return code. Returns `Ok(())` for 0 or `Err(code)` otherwise.
pub fn check_xprs(code: c_int) -> Result<(), c_int> {
    if code == 0 { Ok(()) } else { Err(code) }
}

type LibraryHandle = *mut c_void;

fn open_library(target: &LibraryTarget) -> Result<LibraryHandle, String> {
    let name = c_string_for_target(target)?;
    open_library_impl(name.as_c_str())
}

fn c_string_for_target(target: &LibraryTarget) -> Result<CString, String> {
    let raw = match target {
        LibraryTarget::Path(path) => path.to_string_lossy().into_owned(),
        LibraryTarget::Name(name) => (*name).to_string(),
    };
    CString::new(raw)
        .map_err(|_| format!("candidate contains interior NUL: {}", target.display_name()))
}

#[cfg(unix)]
fn open_library_impl(name: &CStr) -> Result<LibraryHandle, String> {
    const RTLD_NOW: c_int = 2;

    unsafe {
        let handle = dlopen(name.as_ptr(), RTLD_NOW);
        if handle.is_null() {
            Err(dl_error_message())
        } else {
            Ok(handle)
        }
    }
}

#[cfg(unix)]
fn lookup_symbol(handle: LibraryHandle, name: &[u8]) -> Result<*mut c_void, RuntimeLoadError> {
    unsafe {
        let _ = dlerror();
        let symbol = dlsym(handle, name.as_ptr().cast::<c_char>());
        if symbol.is_null() {
            Err(RuntimeLoadError::new(format!(
                "missing Xpress symbol {} ({})",
                String::from_utf8_lossy(&name[..name.len() - 1]),
                dl_error_message()
            )))
        } else {
            Ok(symbol)
        }
    }
}

#[cfg(unix)]
fn close_library(handle: LibraryHandle) -> Result<(), String> {
    unsafe {
        if dlclose(handle) == 0 {
            Ok(())
        } else {
            Err(dl_error_message())
        }
    }
}

#[cfg(unix)]
fn dl_error_message() -> String {
    unsafe {
        let error = dlerror();
        if error.is_null() {
            "dynamic loader returned an unknown error".to_string()
        } else {
            CStr::from_ptr(error).to_string_lossy().into_owned()
        }
    }
}

#[cfg(unix)]
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

#[cfg(windows)]
fn open_library_impl(name: &CStr) -> Result<LibraryHandle, String> {
    unsafe {
        let handle = LoadLibraryA(name.as_ptr().cast::<u8>());
        if handle.is_null() {
            Err(format!("LoadLibraryA failed with error {}", GetLastError()))
        } else {
            Ok(handle)
        }
    }
}

#[cfg(windows)]
fn lookup_symbol(handle: LibraryHandle, name: &[u8]) -> Result<*mut c_void, RuntimeLoadError> {
    unsafe {
        let symbol = GetProcAddress(handle, name.as_ptr());
        if symbol.is_null() {
            Err(RuntimeLoadError::new(format!(
                "missing Xpress symbol {} (GetProcAddress error {})",
                String::from_utf8_lossy(&name[..name.len() - 1]),
                GetLastError()
            )))
        } else {
            Ok(symbol.cast::<c_void>())
        }
    }
}

#[cfg(windows)]
fn close_library(handle: LibraryHandle) -> Result<(), String> {
    unsafe {
        if FreeLibrary(handle.cast()) != 0 {
            Ok(())
        } else {
            Err(format!("FreeLibrary failed with error {}", GetLastError()))
        }
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryA(lp_lib_file_name: *const u8) -> *mut c_void;
    fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const u8) -> *mut c_void;
    fn FreeLibrary(h_lib_module: *mut c_void) -> i32;
    fn GetLastError() -> u32;
}
