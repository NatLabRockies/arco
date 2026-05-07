//! Python enum wrappers for Arco types.

use arco_ops::expression::ComparisonSense;
use arco_ops::modeling::{Sense, SimplifyLevel};
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

/// Python enum for constraint comparison sense.
///
/// Accepts enum variants (`ComparisonSense.GE`) or string aliases
/// (`"ge"`, `">="`, `"le"`, `"<="`, `"eq"`, `"=="`).
#[pyclass(name = "ComparisonSense", eq, eq_int, skip_from_py_object)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyComparisonSense {
    /// Greater than or equal (`>=`)
    #[pyo3(name = "GE")]
    GreaterEqual,
    /// Less than or equal (`<=`)
    #[pyo3(name = "LE")]
    LessEqual,
    /// Exactly equal (`==`)
    #[pyo3(name = "EQ")]
    Equal,
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyComparisonSense {
    type Error = PyErr;

    fn extract(ob: pyo3::Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let ob = ob.to_owned();
        // Try extracting as the enum variant first
        if let Ok(val) = ob.extract::<i64>() {
            // PyO3 eq_int enum: variant ordinals
            return match val {
                0 => Ok(PyComparisonSense::GreaterEqual),
                1 => Ok(PyComparisonSense::LessEqual),
                2 => Ok(PyComparisonSense::Equal),
                _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid ComparisonSense value: {val}"
                ))),
            };
        }
        // Try extracting as a string alias
        if let Ok(s) = ob.extract::<String>() {
            return match s.to_lowercase().as_str() {
                "ge" | ">=" => Ok(PyComparisonSense::GreaterEqual),
                "le" | "<=" => Ok(PyComparisonSense::LessEqual),
                "eq" | "==" => Ok(PyComparisonSense::Equal),
                _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid sense '{s}' (expected 'ge', 'le', 'eq', '>=', '<=', or '==')",
                ))),
            };
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "expected ComparisonSense enum or string ('ge', 'le', 'eq', '>=', '<=', or '==')",
        ))
    }
}

impl From<PyComparisonSense> for ComparisonSense {
    fn from(sense: PyComparisonSense) -> Self {
        match sense {
            PyComparisonSense::GreaterEqual => ComparisonSense::GreaterEqual,
            PyComparisonSense::LessEqual => ComparisonSense::LessEqual,
            PyComparisonSense::Equal => ComparisonSense::Equal,
        }
    }
}

impl From<ComparisonSense> for PyComparisonSense {
    fn from(sense: ComparisonSense) -> Self {
        match sense {
            ComparisonSense::GreaterEqual => PyComparisonSense::GreaterEqual,
            ComparisonSense::LessEqual => PyComparisonSense::LessEqual,
            ComparisonSense::Equal => PyComparisonSense::Equal,
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
    m.add_class::<PyComparisonSense>()?;
    m.add_class::<PySimplifyLevel>()?;
    Ok(())
}
