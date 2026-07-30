//! Python wrappers for solver configuration and instances.

use crate::py_modules::enums::PyLpAlgorithm;
use crate::py_modules::errors::SolverInvalidSettingError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::BTreeMap;
use std::path::PathBuf;

type SolverParameters = BTreeMap<String, String>;

/// Overrides for solve() calls that don't modify the solver's base settings.
#[derive(Debug, Clone, Default)]
pub struct SolveOverrides {
    pub log_to_console: Option<bool>,
    pub time_limit: Option<f64>,
    pub mip_gap: Option<f64>,
    pub verbosity: Option<u32>,
}

/// Base solver settings that persist across solve() calls.
#[derive(Debug, Clone, Default)]
pub struct SolverSettings {
    pub(crate) presolve: Option<bool>,
    pub(crate) threads: Option<u32>,
    pub(crate) tolerance: Option<f64>,
    pub(crate) time_limit: Option<f64>,
    pub(crate) mip_gap: Option<f64>,
    pub(crate) verbosity: Option<u32>,
    pub(crate) log_to_console: Option<bool>,
    pub(crate) lp_algorithm: Option<PyLpAlgorithm>,
    pub(crate) parameters: SolverParameters,
}

impl SolverSettings {
    fn ensure_non_negative_finite(name: &str, value: f64) -> PyResult<()> {
        if !value.is_finite() || value < 0.0 {
            return Err(SolverInvalidSettingError::new_err(format!(
                "{name} must be finite and >= 0"
            )));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
        lp_algorithm: Option<PyLpAlgorithm>,
        parameters: SolverParameters,
    ) -> PyResult<Self> {
        if let Some(threads) = threads {
            if threads == 0 {
                return Err(SolverInvalidSettingError::new_err("threads must be >= 1"));
            }
        }
        if let Some(tolerance) = tolerance {
            Self::ensure_non_negative_finite("tolerance", tolerance)?;
        }
        if let Some(limit) = time_limit {
            Self::ensure_non_negative_finite("time_limit", limit)?;
        }
        if let Some(gap) = mip_gap {
            Self::ensure_non_negative_finite("mip_gap", gap)?;
        }
        Ok(Self {
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            lp_algorithm,
            parameters,
        })
    }

    pub fn with_overrides(&self, overrides: SolveOverrides) -> PyResult<Self> {
        Self::new(
            self.presolve,
            self.threads,
            self.tolerance,
            overrides.time_limit.or(self.time_limit),
            overrides.mip_gap.or(self.mip_gap),
            overrides.verbosity.or(self.verbosity),
            overrides.log_to_console.or(self.log_to_console),
            self.lp_algorithm,
            self.parameters.clone(),
        )
    }

    /// Convert these settings into a generic `SolverConfig`.
    pub fn to_solver_config(&self) -> arco_solver::SolverConfig {
        let mut config = arco_solver::SolverConfig::new();
        if let Some(presolve) = self.presolve {
            config = config.with_presolve(presolve);
        }
        if let Some(threads) = self.threads {
            config = config.with_threads(threads);
        }
        if let Some(tolerance) = self.tolerance {
            config = config.with_tolerance(tolerance);
        }
        if let Some(time_limit) = self.time_limit {
            config = config.with_time_limit(time_limit);
        }
        if let Some(mip_gap) = self.mip_gap {
            config = config.with_mip_gap(mip_gap);
        }
        if let Some(verbosity) = self.verbosity {
            config = config.with_verbosity(verbosity);
        }
        if let Some(log_to_console) = self.log_to_console {
            config = config.with_log_to_console(log_to_console);
        }
        if let Some(lp_algorithm) = self.lp_algorithm {
            config = config.with_lp_algorithm(lp_algorithm.into());
        }
        for (key, value) in &self.parameters {
            config = config.with_parameter(key.clone(), value.clone());
        }
        config
    }
}

pub fn validate_backend_settings(backend: &str, settings: &SolverSettings) -> PyResult<()> {
    if backend == "xpress" && settings.verbosity.is_some() {
        return Err(SolverInvalidSettingError::new_err(
            "verbosity is not supported by the Xpress backend",
        ));
    }
    if backend == "ipopt" && settings.lp_algorithm.is_some() {
        return Err(SolverInvalidSettingError::new_err(
            "lp_algorithm is not supported by the IPOPT backend",
        ));
    }
    Ok(())
}

fn extract_optional<T: for<'a, 'py> FromPyObject<'a, 'py>>(
    value: &Bound<'_, PyAny>,
) -> PyResult<Option<T>>
where
    for<'a, 'py> <T as FromPyObject<'a, 'py>>::Error: Into<PyErr>,
{
    if value.is_none() {
        return Ok(None);
    }
    value.extract().map(Some).map_err(Into::into)
}

fn merge_solver_parameter(
    parameters: Option<SolverParameters>,
    solver: Option<String>,
) -> SolverParameters {
    let mut parameters = parameters.unwrap_or_default();
    if let Some(solver) = solver {
        parameters.insert("solver".to_string(), solver);
    }
    parameters
}

pub(crate) fn apply_solver_updates(
    settings: SolverSettings,
    update: Option<&Bound<'_, PyDict>>,
) -> PyResult<SolverSettings> {
    let Some(update) = update else {
        return Ok(settings);
    };
    let mut settings = settings;
    for (key, value) in update.iter() {
        let key: String = key.extract()?;
        match key.as_str() {
            "presolve" => settings.presolve = extract_optional(&value)?,
            "threads" => settings.threads = extract_optional(&value)?,
            "tolerance" => settings.tolerance = extract_optional(&value)?,
            "time_limit" => settings.time_limit = extract_optional(&value)?,
            "mip_gap" => settings.mip_gap = extract_optional(&value)?,
            "verbosity" => settings.verbosity = extract_optional(&value)?,
            "log_to_console" => settings.log_to_console = extract_optional(&value)?,
            "lp_algorithm" => settings.lp_algorithm = extract_optional(&value)?,
            "parameters" => settings.parameters = value.extract()?,
            "solver" => {
                let value: Option<String> = extract_optional(&value)?;
                if let Some(value) = value {
                    settings.parameters.insert("solver".to_string(), value);
                } else {
                    settings.parameters.remove("solver");
                }
            }
            _ => {
                return Err(SolverInvalidSettingError::new_err(format!(
                    "Unknown solver setting '{key}'",
                )));
            }
        }
    }
    SolverSettings::new(
        settings.presolve,
        settings.threads,
        settings.tolerance,
        settings.time_limit,
        settings.mip_gap,
        settings.verbosity,
        settings.log_to_console,
        settings.lp_algorithm,
        settings.parameters,
    )
}

fn solver_repr(label: &str, settings: &SolverSettings) -> String {
    format!(
        "{label}(presolve={:?}, threads={:?}, tolerance={:?}, time_limit={:?}, mip_gap={:?}, verbosity={:?}, log_to_console={:?}, lp_algorithm={:?}, parameters={:?})",
        settings.presolve,
        settings.threads,
        settings.tolerance,
        settings.time_limit,
        settings.mip_gap,
        settings.verbosity,
        settings.log_to_console,
        settings.lp_algorithm,
        settings.parameters,
    )
}

#[pyo3_macros::pyclass(from_py_object, subclass, name = "Solver")]
#[derive(Debug, Clone)]
pub struct PySolver {
    pub(crate) settings: SolverSettings,
}

#[pyo3_macros::pyclass(from_py_object, name = "SolverSelection")]
#[derive(Debug, Clone)]
pub struct PySolverSelection {
    pub(crate) token: String,
    pub(crate) family_hint: Option<String>,
}

#[pyo3_macros::pymethods]
impl PySolverSelection {
    #[new]
    fn new(token: String) -> Self {
        Self {
            token,
            family_hint: None,
        }
    }

    #[staticmethod]
    fn family(name: String) -> Self {
        Self {
            token: name,
            family_hint: None,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (name, *, family=None))]
    fn profile(name: String, family: Option<String>) -> Self {
        Self {
            token: name,
            family_hint: family,
        }
    }

    #[getter]
    fn token(&self) -> String {
        self.token.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "SolverSelection(token={:?}, family_hint={:?})",
            self.token, self.family_hint
        )
    }
}

#[pyo3_macros::pyclass(from_py_object, name = "SolverProfile")]
#[derive(Debug, Clone)]
pub struct PySolverProfile {
    pub(crate) name: String,
    pub(crate) family: String,
    pub(crate) transport: String,
}

#[pyo3_macros::pymethods]
impl PySolverProfile {
    #[new]
    fn new(name: String, family: String, transport: Option<String>) -> Self {
        Self {
            name,
            family,
            transport: transport.unwrap_or_else(|| "embedded".to_string()),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[getter]
    fn family(&self) -> String {
        self.family.clone()
    }

    #[getter]
    fn transport(&self) -> String {
        self.transport.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "SolverProfile(name={:?}, family={:?}, transport={:?})",
            self.name, self.family, self.transport
        )
    }
}

#[pyo3_macros::pymethods]
impl PySolver {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, lp_algorithm=None, parameters=None, solver=None)
    )]
    #[allow(clippy::too_many_arguments)]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
        lp_algorithm: Option<PyLpAlgorithm>,
        parameters: Option<SolverParameters>,
        solver: Option<String>,
    ) -> PyResult<Self> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            lp_algorithm,
            merge_solver_parameter(parameters, solver),
        )?;
        Ok(Self { settings })
    }

    #[getter]
    fn presolve(&self) -> Option<bool> {
        self.settings.presolve
    }

    #[getter]
    fn threads(&self) -> Option<u32> {
        self.settings.threads
    }

    #[getter]
    fn tolerance(&self) -> Option<f64> {
        self.settings.tolerance
    }

    #[getter]
    fn time_limit(&self) -> Option<f64> {
        self.settings.time_limit
    }

    #[getter]
    fn mip_gap(&self) -> Option<f64> {
        self.settings.mip_gap
    }

    #[getter]
    fn verbosity(&self) -> Option<u32> {
        self.settings.verbosity
    }

    #[getter]
    fn log_to_console(&self) -> Option<bool> {
        self.settings.log_to_console
    }

    #[getter]
    fn lp_algorithm(&self) -> Option<PyLpAlgorithm> {
        self.settings.lp_algorithm
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(&self, py: Python<'_>, update: Option<&Bound<'_, PyDict>>) -> PyResult<Py<Self>> {
        let settings = apply_solver_updates(self.settings.clone(), update)?;
        Py::new(py, PySolver { settings })
    }

    fn __repr__(&self) -> String {
        solver_repr("Solver", &self.settings)
    }
}

#[pyo3_macros::pyclass(from_py_object, extends = PySolver, name = "HiGHS")]
#[derive(Debug, Clone)]
pub struct PyHiGHS;

#[pyo3_macros::pymethods]
impl PyHiGHS {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, lp_algorithm=None, parameters=None, solver=None)
    )]
    #[allow(clippy::too_many_arguments)]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
        lp_algorithm: Option<PyLpAlgorithm>,
        parameters: Option<SolverParameters>,
        solver: Option<String>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            lp_algorithm,
            merge_solver_parameter(parameters, solver),
        )?;
        Ok(PyClassInitializer::from(PySolver { settings }).add_subclass(PyHiGHS))
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        update: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let base = slf.into_super();
        let settings = apply_solver_updates(base.settings.clone(), update)?;
        let initializer = PyClassInitializer::from(PySolver { settings }).add_subclass(PyHiGHS);
        Py::new(py, initializer)
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        let base = slf.into_super();
        solver_repr("HiGHS", &base.settings)
    }
}

#[pyo3_macros::pyclass(from_py_object, extends = PySolver, name = "Xpress")]
#[derive(Debug, Clone)]
pub struct PyXpress;

#[pyo3_macros::pymethods]
impl PyXpress {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, lp_algorithm=None, parameters=None, solver=None)
    )]
    #[allow(clippy::too_many_arguments)]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
        lp_algorithm: Option<PyLpAlgorithm>,
        parameters: Option<SolverParameters>,
        solver: Option<String>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            lp_algorithm,
            merge_solver_parameter(parameters, solver),
        )?;
        validate_backend_settings("xpress", &settings)?;
        Ok(PyClassInitializer::from(PySolver { settings }).add_subclass(PyXpress))
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        update: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let base = slf.into_super();
        let settings = apply_solver_updates(base.settings.clone(), update)?;
        validate_backend_settings("xpress", &settings)?;
        let initializer = PyClassInitializer::from(PySolver { settings }).add_subclass(PyXpress);
        Py::new(py, initializer)
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        let base = slf.into_super();
        solver_repr("Xpress", &base.settings)
    }
}

#[pyo3_macros::pyclass(from_py_object, extends = PySolver, name = "Scip")]
#[derive(Debug, Clone)]
pub struct PyScip;

#[pyo3_macros::pymethods]
impl PyScip {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, lp_algorithm=None, parameters=None, solver=None)
    )]
    #[allow(clippy::too_many_arguments)]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
        lp_algorithm: Option<PyLpAlgorithm>,
        parameters: Option<SolverParameters>,
        solver: Option<String>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            lp_algorithm,
            merge_solver_parameter(parameters, solver),
        )?;
        Ok(PyClassInitializer::from(PySolver { settings }).add_subclass(PyScip))
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        update: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let base = slf.into_super();
        let settings = apply_solver_updates(base.settings.clone(), update)?;
        let initializer = PyClassInitializer::from(PySolver { settings }).add_subclass(PyScip);
        Py::new(py, initializer)
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        let base = slf.into_super();
        solver_repr("Scip", &base.settings)
    }
}

#[cfg(feature = "ipopt")]
#[pyo3_macros::pyclass(from_py_object, extends = PySolver, name = "Ipopt")]
#[derive(Debug, Clone)]
pub struct PyIpopt;

#[cfg(feature = "ipopt")]
#[pyo3_macros::pymethods]
impl PyIpopt {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, parameters=None, solver=None)
    )]
    #[allow(clippy::too_many_arguments)]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
        parameters: Option<SolverParameters>,
        solver: Option<String>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            None,
            merge_solver_parameter(parameters, solver),
        )?;
        Ok(PyClassInitializer::from(PySolver { settings }).add_subclass(PyIpopt))
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        update: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let base = slf.into_super();
        let settings = apply_solver_updates(base.settings.clone(), update)?;
        let initializer = PyClassInitializer::from(PySolver { settings }).add_subclass(PyIpopt);
        Py::new(py, initializer)
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        let base = slf.into_super();
        solver_repr("Ipopt", &base.settings)
    }
}

fn detect_xpress_dir_for_python() -> Option<PathBuf> {
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
        PathBuf::from("C:/xpressmp"),
    ]);

    candidates.into_iter().find(|path| path.exists())
}

pub fn xpress_backend_enabled() -> bool {
    cfg!(feature = "xpress")
}

fn solver_runtime_info_for_family(py: Python<'_>, family: &str) -> PyResult<Py<PyDict>> {
    let info = PyDict::new(py);
    info.set_item("family", family)?;

    match family {
        "xpress" => {
            info.set_item("requires_license", true)?;
            info.set_item("license_env_var", "XPAUTH_PATH")?;
            info.set_item("runtime_env_var", "XPRESSDIR")?;
            let xpress_dir = detect_xpress_dir_for_python().map(|path| path.display().to_string());
            info.set_item("runtime_dir", xpress_dir)?;
            let configured = std::env::var("XPAUTH_PATH").ok();
            info.set_item("configured_license_path", configured)?;
            info.set_item("backend_enabled", xpress_backend_enabled())?;
        }
        "highs" | "scip" | "ipopt" => {
            info.set_item("requires_license", false)?;
            info.set_item("license_env_var", Option::<String>::None)?;
            info.set_item("runtime_env_var", Option::<String>::None)?;
            info.set_item("runtime_dir", Option::<String>::None)?;
            info.set_item("configured_license_path", Option::<String>::None)?;
            info.set_item(
                "backend_enabled",
                family != "ipopt" || cfg!(feature = "ipopt"),
            )?;
        }
        _ => {
            return Err(
                crate::py_modules::errors::SolverInvalidSettingError::new_err(format!(
                    "Unknown solver family '{family}'"
                )),
            );
        }
    }

    Ok(info.unbind())
}

#[pyo3_macros::pyfunction]
#[pyo3(signature = (*, family=None))]
fn solver_runtime_info(py: Python<'_>, family: Option<String>) -> PyResult<Py<PyAny>> {
    let Some(family) = family else {
        let out = PyDict::new(py);
        for item in ["highs", "scip", "ipopt", "xpress"] {
            let info = solver_runtime_info_for_family(py, item)?;
            out.set_item(item, info.bind(py))?;
        }
        return Ok(out.unbind().into());
    };

    let normalized = family.to_lowercase();
    let info = solver_runtime_info_for_family(py, &normalized)?;
    Ok(info.into_any())
}

/// Register solver classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySolver>()?;
    m.add_class::<PySolverSelection>()?;
    m.add_class::<PySolverProfile>()?;
    m.add_class::<PyHiGHS>()?;
    m.add_class::<PyScip>()?;
    m.add_class::<PyXpress>()?;
    #[cfg(feature = "ipopt")]
    m.add_class::<PyIpopt>()?;
    m.add_function(wrap_pyfunction!(solver_runtime_info, m)?)?;
    Ok(())
}

/// Create a Solution for a solve failure (infeasible, unbounded, etc.).
pub fn solve_failure_solution(status: arco_solver::SolverStatus) -> arco_solver::Solution {
    arco_solver::Solution {
        primal_values: Vec::new(),
        variable_duals: Vec::new(),
        constraint_duals: Vec::new(),
        row_values: Vec::new(),
        objective_value: f64::NAN,
        status,
        solve_time_seconds: 0.0,
        metadata: std::collections::BTreeMap::new(),
    }
}

fn detect_default_backend_from_selection(selection: &PySolverSelection) -> String {
    selection.family_hint.as_ref().map_or_else(
        || selection.token.to_lowercase(),
        |family| family.to_lowercase(),
    )
}

/// Detect which backend name a solver object represents.
pub fn detect_default_backend(solver: Option<&Bound<'_, PyAny>>) -> String {
    let Some(solver) = solver else {
        return "highs".to_string();
    };
    if let Ok(selection) = solver.cast::<PySolverSelection>() {
        return detect_default_backend_from_selection(&selection.borrow());
    }
    if let Ok(profile) = solver.cast::<PySolverProfile>() {
        return profile.borrow().family.to_lowercase();
    }
    #[cfg(feature = "ipopt")]
    if solver.cast::<PyIpopt>().is_ok() {
        return "ipopt".to_string();
    }
    if solver.cast::<PyScip>().is_ok() {
        return "scip".to_string();
    }
    if solver.cast::<PyXpress>().is_ok() {
        return "xpress".to_string();
    }
    "highs".to_string()
}

/// Extract `SolverSettings` from an optional Python solver object (`HiGHS`, `Ipopt`, `Xpress`, or `Solver`).
pub fn extract_solver_settings(solver: Option<&Bound<'_, PyAny>>) -> PyResult<SolverSettings> {
    let Some(solver) = solver else {
        return Ok(SolverSettings::default());
    };
    if solver.cast::<PySolverSelection>().is_ok() || solver.cast::<PySolverProfile>().is_ok() {
        return Ok(SolverSettings::default());
    }
    if let Ok(highs) = solver.cast::<PyHiGHS>() {
        return Ok(highs.borrow().into_super().settings.clone());
    }
    #[cfg(feature = "ipopt")]
    if let Ok(ipopt) = solver.cast::<PyIpopt>() {
        return Ok(ipopt.borrow().into_super().settings.clone());
    }
    if let Ok(scip) = solver.cast::<PyScip>() {
        return Ok(scip.borrow().into_super().settings.clone());
    }
    if let Ok(xpress) = solver.cast::<PyXpress>() {
        return Ok(xpress.borrow().into_super().settings.clone());
    }
    if let Ok(base) = solver.cast::<PySolver>() {
        return Ok(base.borrow().settings.clone());
    }
    Err(crate::py_modules::errors::SolverTypeError::new_err(
        "solver must be a SolverSelection, SolverProfile, Solver, HiGHS, Scip, Ipopt, or Xpress instance",
    ))
}
