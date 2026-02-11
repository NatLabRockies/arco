//! Python wrapper for Variable objects returned by add_variable/add_variables.

use arco_expr::ComparisonSense;
use pyo3::prelude::*;

use crate::bounds::{BoundsSpec, PyBounds};
use crate::errors::ExprDivisionByZeroError;
use crate::expr::{PyConstraintExpr, PyExpr};

/// A decision variable returned by `add_variable()`.
///
/// Wraps a variable ID with cached metadata (name, bounds, integrality).
/// Participates in arithmetic to produce `Expr` objects and in
/// comparisons to produce `ConstraintExpr` objects.
#[pyclass(from_py_object, name = "Variable")]
#[derive(Debug, Clone)]
pub struct PyVariable {
    pub var_id: u32,
    pub name: Option<String>,
    pub bounds_spec: BoundsSpec,
}

impl PyVariable {
    pub fn new(var_id: u32, name: Option<String>, bounds_spec: BoundsSpec) -> Self {
        Self {
            var_id,
            name,
            bounds_spec,
        }
    }

    /// Create a PyVariable from a model Variable (handles is_binary detection).
    #[allow(clippy::float_cmp)]
    pub fn from_model_variable(
        var_id: u32,
        name: Option<String>,
        var: &arco_core::types::Variable,
    ) -> Self {
        let is_binary = var.is_integer && var.bounds.lower == 0.0 && var.bounds.upper == 1.0;
        Self {
            var_id,
            name,
            bounds_spec: BoundsSpec {
                bounds: var.bounds,
                is_integer: var.is_integer,
                is_binary,
            },
        }
    }

    /// Create the underlying [`PyExpr`] term for this variable.
    pub fn to_expr(&self) -> PyExpr {
        PyExpr::from_term(self.var_id, 1.0)
    }

    #[allow(clippy::float_cmp)]
    fn bounds_repr(&self) -> String {
        let spec = &self.bounds_spec;
        if spec.is_binary {
            return "Binary".to_string();
        }
        // Match known constant patterns
        let lo = spec.bounds.lower;
        let hi = spec.bounds.upper;
        let is_int = spec.is_integer;
        if is_int {
            if lo == 0.0 && hi == f64::INFINITY {
                return "NonNegativeInt".to_string();
            }
            if lo == f64::NEG_INFINITY && hi == 0.0 {
                return "NonPositiveInt".to_string();
            }
            if lo == 1.0 && hi == f64::INFINITY {
                return "PositiveInt".to_string();
            }
            if lo == f64::NEG_INFINITY && hi == -1.0 {
                return "NegativeInt".to_string();
            }
        } else {
            if lo == 0.0 && hi == f64::INFINITY {
                return "NonNegativeFloat".to_string();
            }
            if lo == f64::NEG_INFINITY && hi == 0.0 {
                return "NonPositiveFloat".to_string();
            }
            if lo == f64::MIN_POSITIVE && hi == f64::INFINITY {
                return "PositiveFloat".to_string();
            }
            if lo == f64::NEG_INFINITY && hi == -f64::MIN_POSITIVE {
                return "NegativeFloat".to_string();
            }
        }
        format!("Bounds({}, {})", lo, hi)
    }
}

#[pymethods]
impl PyVariable {
    /// The variable name, or None if unnamed.
    #[getter]
    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// The variable bounds as a `Bounds` object.
    #[getter]
    fn bounds(&self) -> PyBounds {
        PyBounds::from_inner(self.bounds_spec.bounds)
    }

    /// Whether this variable is integer-constrained.
    #[getter]
    fn is_integer(&self) -> bool {
        self.bounds_spec.is_integer
    }

    /// Whether this variable is binary.
    #[getter]
    fn is_binary(&self) -> bool {
        self.bounds_spec.is_binary
    }

    fn __repr__(&self) -> String {
        let name = self
            .name
            .as_deref()
            .map_or_else(|| self.var_id.to_string(), |n| format!("'{}'", n));
        format!("Variable({}, {})", name, self.bounds_repr())
    }

    // ── Arithmetic operators ───────────────────────────────────────────

    fn __add__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExpr> {
        self.to_expr().add_any(other)
    }

    fn __radd__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExpr> {
        self.to_expr().add_any(other)
    }

    fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExpr> {
        self.to_expr().sub_any(other)
    }

    fn __rsub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExpr> {
        self.to_expr().rsub_any(other)
    }

    fn __mul__(&self, other: f64) -> PyExpr {
        self.to_expr().scale(other)
    }

    fn __rmul__(&self, other: f64) -> PyExpr {
        self.to_expr().scale(other)
    }

    fn __neg__(&self) -> PyExpr {
        self.to_expr().scale(-1.0)
    }

    fn __truediv__(&self, other: f64) -> PyResult<PyExpr> {
        if other == 0.0 {
            return Err(ExprDivisionByZeroError::new_err("division by zero"));
        }
        Ok(self.to_expr().scale(1.0 / other))
    }

    // ── Comparison operators ───────────────────────────────────────────

    fn __ge__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintExpr> {
        self.to_expr()
            .compare_py(rhs, ComparisonSense::GreaterEqual)
    }

    fn __le__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintExpr> {
        self.to_expr().compare_py(rhs, ComparisonSense::LessEqual)
    }

    fn __eq__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintExpr> {
        self.to_expr().compare_py(rhs, ComparisonSense::Equal)
    }

    // ── int() conversion for backwards compat with raw var ID usage ──

    fn __int__(&self) -> u32 {
        self.var_id
    }

    fn __index__(&self) -> u32 {
        self.var_id
    }
}

/// Register Variable class with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVariable>()?;
    Ok(())
}
