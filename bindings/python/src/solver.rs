//! Python wrappers for solver configuration and instances.

use crate::py_modules::errors::SolverInvalidSettingError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::BTreeMap;

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
    pub presolve: Option<bool>,
    pub threads: Option<u32>,
    pub tolerance: Option<f64>,
    pub time_limit: Option<f64>,
    pub mip_gap: Option<f64>,
    pub verbosity: Option<u32>,
    pub log_to_console: Option<bool>,
    pub parameters: SolverParameters,
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

    pub fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
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
            self.parameters.clone(),
        )
    }

    /// Convert these settings into a generic `SolverConfig`.
    pub fn to_solver_config(&self) -> arco_ops::solve::SolverConfig {
        let mut config = arco_ops::solve::SolverConfig::new();
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
        for (key, value) in &self.parameters {
            config = config.with_parameter(key.clone(), value.clone());
        }
        config
    }
}

fn extract_optional<T: for<'a, 'py> FromPyObject<'a, 'py, Error = PyErr>>(
    value: &Bound<'_, PyAny>,
) -> PyResult<Option<T>> {
    if value.is_none() {
        return Ok(None);
    }
    value.extract().map(Some)
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

pub fn apply_solver_updates(
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
        settings.parameters,
    )
}

fn solver_repr(label: &str, settings: &SolverSettings) -> String {
    format!(
        "{label}(presolve={:?}, threads={:?}, tolerance={:?}, time_limit={:?}, mip_gap={:?}, verbosity={:?}, log_to_console={:?}, parameters={:?})",
        settings.presolve,
        settings.threads,
        settings.tolerance,
        settings.time_limit,
        settings.mip_gap,
        settings.verbosity,
        settings.log_to_console,
        settings.parameters,
    )
}

#[pyclass(from_py_object, subclass, name = "Solver")]
#[derive(Debug, Clone)]
pub struct PySolver {
    pub settings: SolverSettings,
}

#[pyclass(from_py_object, name = "SolverSelection")]
#[derive(Debug, Clone)]
pub struct PySolverSelection {
    pub token: String,
    pub family_hint: Option<String>,
}

#[pymethods]
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

#[pyclass(from_py_object, name = "SolverProfile")]
#[derive(Debug, Clone)]
pub struct PySolverProfile {
    pub name: String,
    pub family: String,
    pub transport: String,
}

#[pymethods]
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

#[pymethods]
impl PySolver {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, parameters=None, solver=None)
    )]
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
    ) -> PyResult<Self> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
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

    #[pyo3(signature = (*, update=None))]
    fn copy(&self, py: Python<'_>, update: Option<&Bound<'_, PyDict>>) -> PyResult<Py<Self>> {
        let settings = apply_solver_updates(self.settings.clone(), update)?;
        Py::new(py, PySolver { settings })
    }

    fn __repr__(&self) -> String {
        solver_repr("Solver", &self.settings)
    }
}

#[pyclass(from_py_object, extends = PySolver, name = "HiGHS")]
#[derive(Debug, Clone)]
pub struct PyHiGHS;

#[pymethods]
impl PyHiGHS {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, parameters=None, solver=None)
    )]
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
    ) -> PyResult<(Self, PySolver)> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            merge_solver_parameter(parameters, solver),
        )?;
        Ok((PyHiGHS, PySolver { settings }))
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        update: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let base = slf.into_super();
        let settings = apply_solver_updates(base.settings.clone(), update)?;
        Py::new(py, (PyHiGHS, PySolver { settings }))
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        let base = slf.into_super();
        solver_repr("HiGHS", &base.settings)
    }
}

#[pyclass(from_py_object, extends = PySolver, name = "Xpress")]
#[derive(Debug, Clone)]
pub struct PyXpress;

#[pymethods]
impl PyXpress {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, parameters=None, solver=None)
    )]
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
    ) -> PyResult<(Self, PySolver)> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            merge_solver_parameter(parameters, solver),
        )?;
        Ok((PyXpress, PySolver { settings }))
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        update: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let base = slf.into_super();
        let settings = apply_solver_updates(base.settings.clone(), update)?;
        Py::new(py, (PyXpress, PySolver { settings }))
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        let base = slf.into_super();
        solver_repr("Xpress", &base.settings)
    }
}

#[cfg(feature = "ipopt")]
#[pyclass(from_py_object, extends = PySolver, name = "Ipopt")]
#[derive(Debug, Clone)]
pub struct PyIpopt;

#[cfg(feature = "ipopt")]
#[pymethods]
impl PyIpopt {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None, parameters=None, solver=None)
    )]
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
    ) -> PyResult<(Self, PySolver)> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
            merge_solver_parameter(parameters, solver),
        )?;
        Ok((PyIpopt, PySolver { settings }))
    }

    #[pyo3(signature = (*, update=None))]
    fn copy(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        update: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let base = slf.into_super();
        let settings = apply_solver_updates(base.settings.clone(), update)?;
        Py::new(py, (PyIpopt, PySolver { settings }))
    }

    fn __repr__(slf: PyRef<'_, Self>) -> String {
        let base = slf.into_super();
        solver_repr("Ipopt", &base.settings)
    }
}

/// Register solver classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySolver>()?;
    m.add_class::<PySolverSelection>()?;
    m.add_class::<PySolverProfile>()?;
    m.add_class::<PyHiGHS>()?;
    m.add_class::<PyXpress>()?;
    #[cfg(feature = "ipopt")]
    m.add_class::<PyIpopt>()?;
    Ok(())
}

/// Create a Solution for a solve failure (infeasible, unbounded, etc.).
pub(crate) fn solve_failure_solution(
    status: arco_ops::solve::SolverStatus,
) -> arco_ops::solve::Solution {
    arco_ops::solve::Solution {
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
pub(crate) fn detect_default_backend(solver: Option<&Bound<'_, PyAny>>) -> String {
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
    if solver.cast::<PyXpress>().is_ok() {
        return "xpress".to_string();
    }
    "highs".to_string()
}

/// Extract `SolverSettings` from an optional Python solver object (`HiGHS`, `Ipopt`, `Xpress`, or `Solver`).
pub(crate) fn extract_solver_settings(
    solver: Option<&Bound<'_, PyAny>>,
) -> PyResult<SolverSettings> {
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
    if let Ok(xpress) = solver.cast::<PyXpress>() {
        return Ok(xpress.borrow().into_super().settings.clone());
    }
    if let Ok(base) = solver.cast::<PySolver>() {
        return Ok(base.borrow().settings.clone());
    }
    Err(crate::py_modules::errors::SolverTypeError::new_err(
        "solver must be a SolverSelection, SolverProfile, Solver, HiGHS, Ipopt, or Xpress instance",
    ))
}
