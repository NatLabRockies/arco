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
pub(crate) type XPRSprob = *mut c_void;

pub(crate) const XPRS_PLUSINFINITY: f64 = 1.0e20;
pub(crate) const XPRS_MINUSINFINITY: f64 = -1.0e20;

// Objective sense
pub(crate) const XPRS_OBJ_MINIMIZE: c_int = 1;
pub(crate) const XPRS_OBJ_MAXIMIZE: c_int = -1;

// Integer control indices
pub(crate) const XPRS_THREADS: c_int = 8278;
pub(crate) const XPRS_PRESOLVE: c_int = 8011;
pub(crate) const XPRS_LPLOG: c_int = 8009;
pub(crate) const XPRS_MIPLOG: c_int = 8028;
pub(crate) const XPRS_OUTPUTLOG: c_int = 8035;
pub(crate) const XPRS_CROSSOVER: c_int = 8044;

// Double control indices
pub(crate) const XPRS_MAXTIME: c_int = 8020;
pub(crate) const XPRS_MIPRELSTOP: c_int = 7020;
pub(crate) const XPRS_FEASTOL: c_int = 7003;
pub(crate) const XPRS_OPTIMALITYTOL: c_int = 7006;

// Integer attribute indices
pub(crate) const XPRS_LPSTATUS: c_int = 1010;
pub(crate) const XPRS_MIPSTATUS: c_int = 1011;

// Double attribute indices
pub(crate) const XPRS_LPOBJVAL: c_int = 2001;
pub(crate) const XPRS_MIPOBJVAL: c_int = 2003;

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
type XPRSgetlasterrorFn = unsafe extern "C" fn(XPRSprob, *mut c_char) -> c_int;
type XPRSlicenseFn = unsafe extern "C" fn(*mut c_int, *const c_char) -> c_int;
type XPRSgetlicerrmsgFn = unsafe extern "C" fn(*mut c_char, c_int) -> c_int;
type XPRSgetversionFn = unsafe extern "C" fn(*mut c_char) -> c_int;
type XPRSgetbannerFn = unsafe extern "C" fn(*mut c_char) -> c_int;
type XPRSmessageCallbackFn =
    unsafe extern "C" fn(XPRSprob, *mut c_void, *const c_char, c_int, c_int);
type XPRSsetcbmessageFn =
    unsafe extern "C" fn(XPRSprob, Option<XPRSmessageCallbackFn>, *mut c_void) -> c_int;
type XPRSaddcbmessageFn =
    unsafe extern "C" fn(XPRSprob, Option<XPRSmessageCallbackFn>, *mut c_void, c_int) -> c_int;

#[derive(Clone, Copy)]
enum MessageCallbackRegistration {
    Add(XPRSaddcbmessageFn),
    Set(XPRSsetcbmessageFn),
}

impl MessageCallbackRegistration {
    fn symbol_name(self) -> &'static str {
        match self {
            Self::Add(_) => "XPRSaddcbmessage",
            Self::Set(_) => "XPRSsetcbmessage",
        }
    }

    unsafe fn register(
        self,
        prob: XPRSprob,
        callback: Option<XPRSmessageCallbackFn>,
        data: *mut c_void,
    ) -> c_int {
        match self {
            Self::Add(add_callback) => unsafe { add_callback(prob, callback, data, 0) },
            Self::Set(set_callback) => unsafe { set_callback(prob, callback, data) },
        }
    }
}

const MISSING_OPTIONAL_SYMBOL_RC: c_int = -1;

unsafe extern "C" fn missing_xprs_addqmatrix64(
    _prob: XPRSprob,
    _row: c_int,
    _ncoefs: i64,
    _rowqcol1: *const c_int,
    _rowqcol2: *const c_int,
    _rowqcoef: *const c_double,
) -> c_int {
    MISSING_OPTIONAL_SYMBOL_RC
}

unsafe extern "C" fn missing_xprs_addmipsol(
    _prob: XPRSprob,
    _length: c_int,
    _solval: *const c_double,
    _colind: *const c_int,
    _name: *const c_char,
) -> c_int {
    MISSING_OPTIONAL_SYMBOL_RC
}

unsafe extern "C" fn missing_xprs_getintcontrol(
    _prob: XPRSprob,
    _control: c_int,
    _value: *mut c_int,
) -> c_int {
    MISSING_OPTIONAL_SYMBOL_RC
}

unsafe extern "C" fn missing_xprs_getdblcontrol(
    _prob: XPRSprob,
    _control: c_int,
    _value: *mut c_double,
) -> c_int {
    MISSING_OPTIONAL_SYMBOL_RC
}

unsafe extern "C" fn missing_xprs_getversion(_version: *mut c_char) -> c_int {
    MISSING_OPTIONAL_SYMBOL_RC
}

unsafe extern "C" fn missing_xprs_getbanner(_banner: *mut c_char) -> c_int {
    MISSING_OPTIONAL_SYMBOL_RC
}

unsafe extern "C" fn missing_xprs_setcbmessage(
    _prob: XPRSprob,
    _callback: Option<XPRSmessageCallbackFn>,
    _data: *mut c_void,
) -> c_int {
    MISSING_OPTIONAL_SYMBOL_RC
}

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

pub(crate) struct Api {
    pub(crate) xprs_init: XPRSinitFn,
    pub(crate) xprs_free: XPRSfreeFn,
    pub(crate) xprs_createprob: XPRScreateprobFn,
    pub(crate) xprs_destroyprob: XPRSdestroyprobFn,
    pub(crate) xprs_loadlp: XPRSloadlpFn,
    pub(crate) xprs_loadmip: XPRSloadmipFn,
    pub(crate) xprs_addqmatrix64: XPRSaddqmatrix64Fn,
    pub(crate) xprs_chgobjsense: XPRSchgobjsenseFn,
    pub(crate) xprs_lpoptimize: XPRSlpoptimizeFn,
    pub(crate) xprs_mipoptimize: XPRSmipoptimizeFn,
    pub(crate) xprs_getlpsol: XPRSgetlpsolFn,
    pub(crate) xprs_getmipsol: XPRSgetmipsolFn,
    pub(crate) xprs_addmipsol: XPRSaddmipsolFn,
    pub(crate) xprs_setintcontrol: XPRSsetintcontrolFn,
    pub(crate) xprs_getintcontrol: XPRSgetintcontrolFn,
    pub(crate) xprs_setdblcontrol: XPRSsetdblcontrolFn,
    pub(crate) xprs_getdblcontrol: XPRSgetdblcontrolFn,
    pub(crate) xprs_getintattrib: XPRSgetintattribFn,
    pub(crate) xprs_getdblattrib: XPRSgetdblattribFn,
    pub(crate) xprs_getlasterror: XPRSgetlasterrorFn,
    pub(crate) xprs_license: XPRSlicenseFn,
    pub(crate) xprs_getlicerrmsg: XPRSgetlicerrmsgFn,
    pub(crate) xprs_getversion: XPRSgetversionFn,
    pub(crate) xprs_getbanner: XPRSgetbannerFn,
    pub(crate) xprs_setcbmessage: XPRSsetcbmessageFn,
    mip_loader_symbol: &'static str,
    message_callback: Option<MessageCallbackRegistration>,
}

impl Api {
    pub(crate) fn mip_loader_symbol(&self) -> &'static str {
        self.mip_loader_symbol
    }

    pub(crate) fn message_callback_symbol(&self) -> Option<&'static str> {
        self.message_callback
            .map(MessageCallbackRegistration::symbol_name)
    }

    pub(crate) unsafe fn register_message_callback(
        &self,
        prob: XPRSprob,
        callback: Option<XPRSmessageCallbackFn>,
        data: *mut c_void,
    ) -> Result<c_int, RuntimeLoadError> {
        let Some(message_callback) = self.message_callback else {
            return Err(RuntimeLoadError::new(
                "missing Xpress message callback registration symbol \
                 (tried XPRSaddcbmessage, XPRSsetcbmessage)"
                    .to_string(),
            ));
        };

        Ok(unsafe { message_callback.register(prob, callback, data) })
    }
}

static API: OnceLock<Result<Api, RuntimeLoadError>> = OnceLock::new();

const REQUIRED_SYMBOLS: &[&[u8]] = &[
    b"XPRSinit\0",
    b"XPRSfree\0",
    b"XPRScreateprob\0",
    b"XPRSdestroyprob\0",
    b"XPRSloadlp\0",
    b"XPRSchgobjsense\0",
    b"XPRSlpoptimize\0",
    b"XPRSmipoptimize\0",
    b"XPRSgetlpsol\0",
    b"XPRSgetmipsol\0",
    b"XPRSsetintcontrol\0",
    b"XPRSsetdblcontrol\0",
    b"XPRSgetintattrib\0",
    b"XPRSgetdblattrib\0",
    b"XPRSgetlasterror\0",
    b"XPRSlicense\0",
    b"XPRSgetlicerrmsg\0",
];

const MIP_LOADER_SYMBOLS: &[(&str, &[u8])] = &[
    ("XPRSloadmip", b"XPRSloadmip\0"),
    ("XPRSloadglobal", b"XPRSloadglobal\0"),
];

macro_rules! load_symbol {
    ($handle:expr, $symbol:literal, $ty:ty) => {{
        let raw = lookup_symbol($handle, concat!($symbol, "\0").as_bytes())?;
        unsafe { std::mem::transmute::<*mut c_void, $ty>(raw) }
    }};
}

macro_rules! load_optional_symbol {
    ($handle:expr, $symbol:literal, $ty:ty, $fallback:expr) => {{
        lookup_symbol($handle, concat!($symbol, "\0").as_bytes())
            .map(|raw| unsafe { std::mem::transmute::<*mut c_void, $ty>(raw) })
            .unwrap_or($fallback)
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
    candidates.sort_by_key(|left| left.display_name());
    candidates
}

pub(crate) fn runtime_library_available(xpress_dir: Option<&Path>) -> bool {
    runtime_library_candidates(xpress_dir)
        .into_iter()
        .any(|target| {
            open_library(&target).is_ok_and(|handle| {
                let has_required_symbols = required_symbols_available(handle).is_ok();
                close_library(handle).is_ok() && has_required_symbols
            })
        })
}

pub(crate) fn api() -> Result<&'static Api, RuntimeLoadError> {
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
            Ok(handle) => match load_api_from_handle(handle) {
                Ok(api) => return Ok(api),
                Err(error) => {
                    let close_result = close_library(handle);
                    let close_details = close_result
                        .err()
                        .map(|close_error| format!("; close failed: {close_error}"))
                        .unwrap_or_default();
                    failures.push(format!(
                        "{} ({error}{close_details})",
                        target.display_name()
                    ));
                }
            },
            Err(error) => failures.push(format!("{} ({error})", target.display_name())),
        }
    }

    Err(RuntimeLoadError::new(format!(
        "unable to load Xpress runtime library; tried: {}",
        failures.join(", ")
    )))
}

fn load_api_from_handle(handle: LibraryHandle) -> Result<Api, RuntimeLoadError> {
    let (xprs_loadmip, mip_loader_symbol) = load_mip_loader(handle)?;
    let message_callback = load_message_callback_registration(handle);

    Ok(Api {
        xprs_init: load_symbol!(handle, "XPRSinit", XPRSinitFn),
        xprs_free: load_symbol!(handle, "XPRSfree", XPRSfreeFn),
        xprs_createprob: load_symbol!(handle, "XPRScreateprob", XPRScreateprobFn),
        xprs_destroyprob: load_symbol!(handle, "XPRSdestroyprob", XPRSdestroyprobFn),
        xprs_loadlp: load_symbol!(handle, "XPRSloadlp", XPRSloadlpFn),
        xprs_loadmip,
        xprs_addqmatrix64: load_optional_symbol!(
            handle,
            "XPRSaddqmatrix64",
            XPRSaddqmatrix64Fn,
            missing_xprs_addqmatrix64 as XPRSaddqmatrix64Fn
        ),
        xprs_chgobjsense: load_symbol!(handle, "XPRSchgobjsense", XPRSchgobjsenseFn),
        xprs_lpoptimize: load_symbol!(handle, "XPRSlpoptimize", XPRSlpoptimizeFn),
        xprs_mipoptimize: load_symbol!(handle, "XPRSmipoptimize", XPRSmipoptimizeFn),
        xprs_getlpsol: load_symbol!(handle, "XPRSgetlpsol", XPRSgetlpsolFn),
        xprs_getmipsol: load_symbol!(handle, "XPRSgetmipsol", XPRSgetmipsolFn),
        xprs_addmipsol: load_optional_symbol!(
            handle,
            "XPRSaddmipsol",
            XPRSaddmipsolFn,
            missing_xprs_addmipsol as XPRSaddmipsolFn
        ),
        xprs_setintcontrol: load_symbol!(handle, "XPRSsetintcontrol", XPRSsetintcontrolFn),
        xprs_getintcontrol: load_optional_symbol!(
            handle,
            "XPRSgetintcontrol",
            XPRSgetintcontrolFn,
            missing_xprs_getintcontrol as XPRSgetintcontrolFn
        ),
        xprs_setdblcontrol: load_symbol!(handle, "XPRSsetdblcontrol", XPRSsetdblcontrolFn),
        xprs_getdblcontrol: load_optional_symbol!(
            handle,
            "XPRSgetdblcontrol",
            XPRSgetdblcontrolFn,
            missing_xprs_getdblcontrol as XPRSgetdblcontrolFn
        ),
        xprs_getintattrib: load_symbol!(handle, "XPRSgetintattrib", XPRSgetintattribFn),
        xprs_getdblattrib: load_symbol!(handle, "XPRSgetdblattrib", XPRSgetdblattribFn),
        xprs_getlasterror: load_symbol!(handle, "XPRSgetlasterror", XPRSgetlasterrorFn),
        xprs_license: load_symbol!(handle, "XPRSlicense", XPRSlicenseFn),
        xprs_getlicerrmsg: load_symbol!(handle, "XPRSgetlicerrmsg", XPRSgetlicerrmsgFn),
        xprs_getversion: load_optional_symbol!(
            handle,
            "XPRSgetversion",
            XPRSgetversionFn,
            missing_xprs_getversion as XPRSgetversionFn
        ),
        xprs_getbanner: load_optional_symbol!(
            handle,
            "XPRSgetbanner",
            XPRSgetbannerFn,
            missing_xprs_getbanner as XPRSgetbannerFn
        ),
        xprs_setcbmessage: load_optional_symbol!(
            handle,
            "XPRSsetcbmessage",
            XPRSsetcbmessageFn,
            missing_xprs_setcbmessage as XPRSsetcbmessageFn
        ),
        mip_loader_symbol,
        message_callback,
    })
}

fn load_mip_loader(
    handle: LibraryHandle,
) -> Result<(XPRSloadmipFn, &'static str), RuntimeLoadError> {
    let (symbol_name, raw) = resolve_alternative_symbol_with(
        |symbol| lookup_symbol(handle, symbol),
        "MIP loader",
        MIP_LOADER_SYMBOLS,
    )?;
    Ok((
        unsafe { std::mem::transmute::<*mut c_void, XPRSloadmipFn>(raw) },
        symbol_name,
    ))
}

fn load_message_callback_registration(
    handle: LibraryHandle,
) -> Option<MessageCallbackRegistration> {
    lookup_symbol(handle, b"XPRSaddcbmessage\0")
        .map(|raw| {
            MessageCallbackRegistration::Add(unsafe {
                std::mem::transmute::<*mut c_void, XPRSaddcbmessageFn>(raw)
            })
        })
        .or_else(|_| {
            lookup_symbol(handle, b"XPRSsetcbmessage\0").map(|raw| {
                MessageCallbackRegistration::Set(unsafe {
                    std::mem::transmute::<*mut c_void, XPRSsetcbmessageFn>(raw)
                })
            })
        })
        .ok()
}

fn resolve_alternative_symbol_with(
    mut lookup: impl FnMut(&[u8]) -> Result<*mut c_void, RuntimeLoadError>,
    label: &str,
    candidates: &[(&'static str, &'static [u8])],
) -> Result<(&'static str, *mut c_void), RuntimeLoadError> {
    let mut failures = Vec::with_capacity(candidates.len());
    for (name, symbol) in candidates {
        match lookup(symbol) {
            Ok(raw) => return Ok((*name, raw)),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }

    Err(RuntimeLoadError::new(format!(
        "missing Xpress {label} symbol (tried {})",
        failures.join("; ")
    )))
}

fn required_symbols_available(handle: LibraryHandle) -> Result<(), RuntimeLoadError> {
    required_symbols_available_with(|symbol| lookup_symbol(handle, symbol))
}

fn required_symbols_available_with(
    mut lookup: impl FnMut(&[u8]) -> Result<*mut c_void, RuntimeLoadError>,
) -> Result<(), RuntimeLoadError> {
    for symbol in REQUIRED_SYMBOLS {
        lookup(symbol)?;
    }
    resolve_alternative_symbol_with(|symbol| lookup(symbol), "MIP loader", MIP_LOADER_SYMBOLS)?;
    Ok(())
}

/// Check an Xpress return code. Returns `Ok(())` for 0 or `Err(code)` otherwise.
pub(crate) fn check_xprs(code: c_int) -> Result<(), c_int> {
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

#[cfg(test)]
mod tests {
    use crate::ffi::{RuntimeLoadError, required_symbols_available_with};
    use std::ffi::c_void;
    use std::ptr::NonNull;

    fn present_symbol() -> *mut c_void {
        NonNull::<c_void>::dangling().as_ptr()
    }

    fn missing_symbol_error(symbol: &[u8]) -> RuntimeLoadError {
        RuntimeLoadError::new(format!(
            "missing Xpress symbol {}",
            String::from_utf8_lossy(&symbol[..symbol.len() - 1])
        ))
    }

    #[test]
    fn runtime_probe_accepts_xpress8_loadglobal_mip_loader_alias() {
        required_symbols_available_with(|symbol| {
            if symbol == b"XPRSloadmip\0" {
                return Err(missing_symbol_error(symbol));
            }

            Ok(present_symbol())
        })
        .expect("XPRSloadglobal should satisfy the MIP loader requirement");
    }

    #[test]
    fn runtime_probe_rejects_library_missing_both_mip_loader_symbols() {
        let error = required_symbols_available_with(|symbol| {
            if symbol == b"XPRSloadmip\0" || symbol == b"XPRSloadglobal\0" {
                return Err(missing_symbol_error(symbol));
            }

            Ok(present_symbol())
        })
        .expect_err("one Xpress MIP loader symbol is required");

        let message = error.to_string();
        assert!(message.contains("XPRSloadmip"));
        assert!(message.contains("XPRSloadglobal"));
    }

    #[test]
    fn runtime_probe_does_not_require_unused_or_logging_symbols() {
        let optional_symbols: &[&[u8]] = &[
            b"XPRSaddqmatrix64\0",
            b"XPRSaddmipsol\0",
            b"XPRSgetintcontrol\0",
            b"XPRSgetdblcontrol\0",
            b"XPRSgetversion\0",
            b"XPRSgetbanner\0",
            b"XPRSsetcbmessage\0",
        ];

        required_symbols_available_with(|symbol| {
            if optional_symbols.contains(&symbol) {
                return Err(missing_symbol_error(symbol));
            }

            Ok(present_symbol())
        })
        .expect("unused and log-only symbols should not block runtime detection");
    }
}
