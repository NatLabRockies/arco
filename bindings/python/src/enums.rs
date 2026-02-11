//! Python enum wrappers for Arco types.

use arco_core::{Sense, SimplifyLevel};
use pyo3::prelude::*;

/// Python enum for optimization sense
#[pyclass(from_py_object, name = "Sense", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PySense {
    /// Minimize objective function
    #[pyo3(name = "MINIMIZE")]
    Minimize,
    /// Maximize objective function
    #[pyo3(name = "MAXIMIZE")]
    Maximize,
}

impl From<PySense> for Sense {
    fn from(sense: PySense) -> Self {
        match sense {
            PySense::Minimize => Sense::Minimize,
            PySense::Maximize => Sense::Maximize,
        }
    }
}

impl From<Sense> for PySense {
    fn from(sense: Sense) -> Self {
        match sense {
            Sense::Minimize => PySense::Minimize,
            Sense::Maximize => PySense::Maximize,
        }
    }
}

/// Python enum for expression simplification.
#[pyclass(from_py_object, name = "SimplifyLevel", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PySimplifyLevel {
    #[pyo3(name = "NONE")]
    None,
    #[pyo3(name = "LIGHT")]
    Light,
}

impl From<PySimplifyLevel> for SimplifyLevel {
    fn from(level: PySimplifyLevel) -> Self {
        match level {
            PySimplifyLevel::None => SimplifyLevel::None,
            PySimplifyLevel::Light => SimplifyLevel::Light,
        }
    }
}

impl From<SimplifyLevel> for PySimplifyLevel {
    fn from(level: SimplifyLevel) -> Self {
        match level {
            SimplifyLevel::None => PySimplifyLevel::None,
            SimplifyLevel::Light => PySimplifyLevel::Light,
        }
    }
}

/// Register enum classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySense>()?;
    m.add_class::<PySimplifyLevel>()?;
    Ok(())
}
