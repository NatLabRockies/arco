//! Python wrapper for Constraint objects returned by add_constraint/add_constraints.

use arco_core::types::Bounds;
use pyo3::prelude::*;

use crate::bounds::PyBounds;

/// A constraint returned by `add_constraint()`.
///
/// Wraps a constraint ID with cached metadata (name, bounds).
#[pyclass(from_py_object, name = "Constraint")]
#[derive(Debug, Clone)]
pub struct PyConstraint {
    pub constraint_id: u32,
    pub name: Option<String>,
    pub constraint_bounds: Bounds,
}

impl PyConstraint {
    pub fn new(constraint_id: u32, name: Option<String>, constraint_bounds: Bounds) -> Self {
        Self {
            constraint_id,
            name,
            constraint_bounds,
        }
    }
}

#[pymethods]
impl PyConstraint {
    /// The constraint name, or None if unnamed.
    #[getter]
    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// The constraint bounds as a `Bounds` object.
    #[getter]
    fn bounds(&self) -> PyBounds {
        PyBounds::from_inner(self.constraint_bounds)
    }

    fn __repr__(&self) -> String {
        let name = self
            .name
            .as_deref()
            .map_or_else(|| self.constraint_id.to_string(), |n| format!("'{}'", n));
        format!(
            "Constraint({}, Bounds({}, {}))",
            name, self.constraint_bounds.lower, self.constraint_bounds.upper
        )
    }

    fn __int__(&self) -> u32 {
        self.constraint_id
    }

    fn __index__(&self) -> u32 {
        self.constraint_id
    }
}

/// Register Constraint class with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyConstraint>()?;
    Ok(())
}
