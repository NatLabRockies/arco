//! Python wrapper for bounds and bound constant types.

use crate::PyObject;
use crate::py_modules::errors::BoundsInvalidError;
use arco_model::Bounds;
use pyo3::Borrowed;
use pyo3::prelude::*;

/// Resolved bound specification extracted from any Python bounds argument.
/// Used internally by add_variable/add_variables to unpack metadata.
#[derive(Debug, Clone, Copy)]
pub struct BoundsSpec {
    pub bounds: Bounds,
    pub is_integer: bool,
    pub is_binary: bool,
}

impl<'a, 'py> FromPyObject<'a, 'py> for BoundsSpec {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let ob = ob.to_owned();
        // Try the BoundType enum first (the primary way to specify bound constants)
        if let Ok(bt) = ob.extract::<PyBoundType>() {
            return Ok(bt.spec());
        }
        // Fall back to regular Bounds (scalar only; array bounds are handled separately)
        if let Ok(b) = ob.extract::<PyRef<'py, PyBounds>>() {
            if !b.is_array_bounds() {
                return Ok(BoundsSpec {
                    bounds: b.inner,
                    is_integer: false,
                    is_binary: false,
                });
            }
            // Array bounds cannot convert to scalar BoundsSpec
        }
        Err(BoundsInvalidError::new_err(
            "ARCO_BOUNDS_002: expected a Bounds or bound constant (e.g. NonNegativeFloat, Binary)",
        ))
    }
}

/// Python wrapper for bounds (scalar or per-element array bounds).
#[pyo3_macros::pyclass(from_py_object, name = "Bounds")]
pub struct PyBounds {
    pub inner: Bounds,
    /// Per-element lower bound source object (None for scalar bounds)
    pub array_lower: Option<PyObject>,
    /// Per-element upper bound source object (None for scalar bounds)
    pub array_upper: Option<PyObject>,
}

impl Clone for PyBounds {
    fn clone(&self) -> Self {
        Python::attach(|py| Self {
            inner: self.inner,
            array_lower: self.array_lower.as_ref().map(|o| o.clone_ref(py)),
            array_upper: self.array_upper.as_ref().map(|o| o.clone_ref(py)),
        })
    }
}

impl std::fmt::Debug for PyBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PyBounds")
            .field("inner", &self.inner)
            .field("array_lower", &self.array_lower.is_some())
            .field("array_upper", &self.array_upper.is_some())
            .finish()
    }
}

impl PyBounds {
    pub(crate) fn from_inner(inner: Bounds) -> Self {
        Self {
            inner,
            array_lower: None,
            array_upper: None,
        }
    }

    pub(crate) fn is_array_bounds(&self) -> bool {
        self.array_lower.is_some()
    }
}

#[pyo3_macros::pymethods]
impl PyBounds {
    #[new]
    #[pyo3(signature = (lower=None, upper=None))]
    fn new(
        _py: Python<'_>,
        lower: Option<&Bound<'_, PyAny>>,
        upper: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let low_obj =
            lower.ok_or_else(|| BoundsInvalidError::new_err("Bounds() requires lower= bound"))?;
        let high_obj =
            upper.ok_or_else(|| BoundsInvalidError::new_err("Bounds() requires upper= bound"))?;

        // Try scalar f64 first
        if let (Ok(low), Ok(high)) = (low_obj.extract::<f64>(), high_obj.extract::<f64>()) {
            if low > high {
                return Err(BoundsInvalidError::new_err(format!(
                    "Bounds invalid: lower ({low}) > upper ({high})"
                )));
            }
            return Ok(Self {
                inner: Bounds::new(low, high),
                array_lower: None,
                array_upper: None,
            });
        }
        // Preserve the original Python objects so labeled params can still
        // broadcast by axis identity when the model builder consumes them.
        Ok(Self {
            inner: Bounds::new(f64::NEG_INFINITY, f64::INFINITY),
            array_lower: Some(low_obj.clone().unbind()),
            array_upper: Some(high_obj.clone().unbind()),
        })
    }

    #[getter]
    fn lower(&self, py: Python<'_>) -> PyResult<PyObject> {
        if let Some(ref arr) = self.array_lower {
            Ok(arr.clone_ref(py))
        } else {
            Ok(self.inner.lower.into_pyobject(py)?.into_any().unbind())
        }
    }

    #[getter]
    fn upper(&self, py: Python<'_>) -> PyResult<PyObject> {
        if let Some(ref arr) = self.array_upper {
            Ok(arr.clone_ref(py))
        } else {
            Ok(self.inner.upper.into_pyobject(py)?.into_any().unbind())
        }
    }

    fn __repr__(&self) -> String {
        if self.is_array_bounds() {
            "Bounds(lower=<array>, upper=<array>)".to_string()
        } else {
            format!(
                "Bounds(lower={}, upper={})",
                self.inner.lower, self.inner.upper
            )
        }
    }
}

/// Enum of common bound types for decision variables.
///
/// Use these directly without parentheses:
/// ```python
/// x = model.add_variable(bounds=arco.NonNegativeFloat)
/// b = model.add_variable(bounds=arco.Binary)
/// ```
#[pyo3_macros::pyclass(from_py_object, name = "BoundType", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum PyBoundType {
    /// (0, +inf) — strictly positive float
    PositiveFloat,
    /// (-inf, 0) — strictly negative float
    NegativeFloat,
    /// [0, +inf) — non-negative float (most common for LP)
    NonNegativeFloat,
    /// (-inf, 0] — non-positive float
    NonPositiveFloat,
    /// {1, 2, 3, ...} — strictly positive integer
    PositiveInt,
    /// {..., -3, -2, -1} — strictly negative integer
    NegativeInt,
    /// {0, 1, 2, ...} — non-negative integer
    NonNegativeInt,
    /// {..., -2, -1, 0} — non-positive integer
    NonPositiveInt,
    /// {0, 1} — binary decision variable
    Binary,
}

impl PyBoundType {
    /// Convert the bound type to a full BoundsSpec for internal use.
    pub(crate) fn spec(&self) -> BoundsSpec {
        match self {
            PyBoundType::PositiveFloat => BoundsSpec {
                bounds: Bounds::new(f64::MIN_POSITIVE, f64::INFINITY),
                is_integer: false,
                is_binary: false,
            },
            PyBoundType::NegativeFloat => BoundsSpec {
                bounds: Bounds::new(f64::NEG_INFINITY, -f64::MIN_POSITIVE),
                is_integer: false,
                is_binary: false,
            },
            PyBoundType::NonNegativeFloat => BoundsSpec {
                bounds: Bounds::new(0.0, f64::INFINITY),
                is_integer: false,
                is_binary: false,
            },
            PyBoundType::NonPositiveFloat => BoundsSpec {
                bounds: Bounds::new(f64::NEG_INFINITY, 0.0),
                is_integer: false,
                is_binary: false,
            },
            PyBoundType::PositiveInt => BoundsSpec {
                bounds: Bounds::new(1.0, f64::INFINITY),
                is_integer: true,
                is_binary: false,
            },
            PyBoundType::NegativeInt => BoundsSpec {
                bounds: Bounds::new(f64::NEG_INFINITY, -1.0),
                is_integer: true,
                is_binary: false,
            },
            PyBoundType::NonNegativeInt => BoundsSpec {
                bounds: Bounds::new(0.0, f64::INFINITY),
                is_integer: true,
                is_binary: false,
            },
            PyBoundType::NonPositiveInt => BoundsSpec {
                bounds: Bounds::new(f64::NEG_INFINITY, 0.0),
                is_integer: true,
                is_binary: false,
            },
            PyBoundType::Binary => BoundsSpec {
                bounds: Bounds::new(0.0, 1.0),
                is_integer: true,
                is_binary: true,
            },
        }
    }
}

/// Register bounds classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBounds>()?;
    m.add_class::<PyBoundType>()?;
    Ok(())
}

pub fn export_bound_constants(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Re-export enum members at module top-level for ergonomic imports:
    // `from arco import NonNegativeFloat, Binary`
    m.add("PositiveFloat", PyBoundType::PositiveFloat)?;
    m.add("NegativeFloat", PyBoundType::NegativeFloat)?;
    m.add("NonNegativeFloat", PyBoundType::NonNegativeFloat)?;
    m.add("NonPositiveFloat", PyBoundType::NonPositiveFloat)?;
    m.add("PositiveInt", PyBoundType::PositiveInt)?;
    m.add("NegativeInt", PyBoundType::NegativeInt)?;
    m.add("NonNegativeInt", PyBoundType::NonNegativeInt)?;
    m.add("NonPositiveInt", PyBoundType::NonPositiveInt)?;
    m.add("Binary", PyBoundType::Binary)?;

    Ok(())
}
