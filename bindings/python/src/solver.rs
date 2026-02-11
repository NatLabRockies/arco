//! Python wrappers for solver configuration and instances.

use crate::errors::SolverInvalidSettingError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

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
        )
    }

    pub fn apply_highs(&self, solver: &mut arco_highs::Solver) {
        if let Some(enabled) = self.log_to_console {
            solver.set_log_to_console(enabled);
        }
        if let Some(limit) = self.time_limit {
            solver.set_time_limit(limit);
        }
        if let Some(gap) = self.mip_gap {
            solver.set_mip_gap(gap);
        }
        if let Some(level) = self.verbosity {
            solver.set_verbosity(level);
        }
        if let Some(presolve) = self.presolve {
            solver.set_presolve(presolve);
        }
        if let Some(threads) = self.threads {
            solver.set_threads(threads);
        }
        if let Some(tolerance) = self.tolerance {
            solver.set_tolerance(tolerance);
        }
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
    )
}

fn solver_repr(label: &str, settings: &SolverSettings) -> String {
    format!(
        "{label}(presolve={:?}, threads={:?}, tolerance={:?}, time_limit={:?}, mip_gap={:?}, verbosity={:?}, log_to_console={:?})",
        settings.presolve,
        settings.threads,
        settings.tolerance,
        settings.time_limit,
        settings.mip_gap,
        settings.verbosity,
        settings.log_to_console,
    )
}

#[pyclass(from_py_object, subclass, name = "Solver")]
#[derive(Debug, Clone)]
pub struct PySolver {
    pub settings: SolverSettings,
}

#[pymethods]
impl PySolver {
    #[new]
    #[pyo3(
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None)
    )]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
    ) -> PyResult<Self> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
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
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None)
    )]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
    ) -> PyResult<(Self, PySolver)> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
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
        signature = (*, presolve=None, threads=None, tolerance=None, time_limit=None, mip_gap=None, verbosity=None, log_to_console=None)
    )]
    fn new(
        presolve: Option<bool>,
        threads: Option<u32>,
        tolerance: Option<f64>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        log_to_console: Option<bool>,
    ) -> PyResult<(Self, PySolver)> {
        let settings = SolverSettings::new(
            presolve,
            threads,
            tolerance,
            time_limit,
            mip_gap,
            verbosity,
            log_to_console,
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

/// Register solver classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySolver>()?;
    m.add_class::<PyHiGHS>()?;
    m.add_class::<PyXpress>()?;
    Ok(())
}
