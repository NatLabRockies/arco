//! Python wrappers for model view types.

use arco_model::Bounds;
use pyo3::prelude::*;

use crate::PyObject;
use crate::py_modules::bounds::PyBounds;
use crate::py_modules::serde_bridge;

fn pythonize_metadata(
    py: Python<'_>,
    value: Option<&serde_json::Value>,
) -> PyResult<Option<PyObject>> {
    match value {
        Some(v) => Ok(Some(serde_bridge::json_to_py(py, v)?)),
        None => Ok(None),
    }
}

/// View of a variable in a model snapshot.
#[pyo3_macros::pyclass(from_py_object, name = "VariableView")]
#[derive(Clone)]
pub struct PyVariableView {
    pub(crate) id: u32,
    pub(crate) name: Option<String>,
    pub(crate) bounds: Bounds,
    pub(crate) is_integer: bool,
    pub(crate) is_active: bool,
    pub(crate) metadata: Option<serde_json::Value>,
}

#[pyo3_macros::pymethods]
impl PyVariableView {
    #[getter]
    fn id(&self) -> u32 {
        self.id
    }

    #[getter]
    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[getter]
    fn bounds(&self) -> PyBounds {
        PyBounds::from_inner(self.bounds)
    }

    #[getter]
    fn is_integer(&self) -> bool {
        self.is_integer
    }

    #[getter]
    fn is_active(&self) -> bool {
        self.is_active
    }

    #[getter]
    fn metadata(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        pythonize_metadata(py, self.metadata.as_ref())
    }
}

/// View of a constraint in a model snapshot.
#[pyo3_macros::pyclass(from_py_object, name = "ConstraintView")]
#[derive(Clone)]
pub struct PyConstraintView {
    pub(crate) id: u32,
    pub(crate) name: Option<String>,
    pub(crate) bounds: Bounds,
    pub(crate) nnz: usize,
    pub(crate) metadata: Option<serde_json::Value>,
}

#[pyo3_macros::pymethods]
impl PyConstraintView {
    #[getter]
    fn id(&self) -> u32 {
        self.id
    }

    #[getter]
    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[getter]
    fn bounds(&self) -> PyBounds {
        PyBounds::from_inner(self.bounds)
    }

    #[getter]
    fn nnz(&self) -> usize {
        self.nnz
    }

    #[getter]
    fn metadata(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        pythonize_metadata(py, self.metadata.as_ref())
    }
}

/// View of a coefficient in a model snapshot.
#[pyo3_macros::pyclass(from_py_object, name = "CoefficientView")]
#[derive(Clone)]
pub struct PyCoefficientView {
    pub(crate) variable_id: u32,
    pub(crate) constraint_id: u32,
    pub(crate) value: f64,
}

#[pyo3_macros::pymethods]
impl PyCoefficientView {
    #[getter]
    fn variable_id(&self) -> u32 {
        self.variable_id
    }

    #[getter]
    fn constraint_id(&self) -> u32 {
        self.constraint_id
    }

    #[getter]
    fn value(&self) -> f64 {
        self.value
    }
}

/// View of the objective in a model snapshot.
#[pyo3_macros::pyclass(from_py_object, name = "ObjectiveView")]
#[derive(Clone)]
pub struct PyObjectiveView {
    pub(crate) sense: Option<String>,
    pub(crate) terms: Vec<(u32, f64)>,
    pub(crate) name: Option<String>,
}

#[pyo3_macros::pymethods]
impl PyObjectiveView {
    #[getter]
    fn sense(&self) -> Option<String> {
        self.sense.clone()
    }

    #[getter]
    fn terms(&self) -> Vec<(u32, f64)> {
        self.terms.clone()
    }

    #[getter]
    fn name(&self) -> Option<String> {
        self.name.clone()
    }
}

/// View of a slack variable in a model snapshot.
#[pyo3_macros::pyclass(from_py_object, name = "SlackView")]
#[derive(Clone)]
pub struct PySlackView {
    pub(crate) constraint_id: u32,
    pub(crate) bound: String,
    pub(crate) penalty: f64,
    pub(crate) lower_variable: Option<u32>,
    pub(crate) upper_variable: Option<u32>,
    pub(crate) name: Option<String>,
}

#[pyo3_macros::pymethods]
impl PySlackView {
    #[getter]
    fn constraint_id(&self) -> u32 {
        self.constraint_id
    }

    #[getter]
    fn bound(&self) -> String {
        self.bound.clone()
    }

    #[getter]
    fn penalty(&self) -> f64 {
        self.penalty
    }

    #[getter]
    fn lower_variable(&self) -> Option<u32> {
        self.lower_variable
    }

    #[getter]
    fn upper_variable(&self) -> Option<u32> {
        self.upper_variable
    }

    #[getter]
    fn name(&self) -> Option<String> {
        self.name.clone()
    }
}

/// Register view classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVariableView>()?;
    m.add_class::<PyConstraintView>()?;
    m.add_class::<PyCoefficientView>()?;
    m.add_class::<PyObjectiveView>()?;
    m.add_class::<PySlackView>()?;
    Ok(())
}
