//! Python wrappers for slack and elastic handles.

use arco_core::{ElasticHandle, SlackHandle};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::PyObject;

/// Python wrapper for a slack handle.
#[pyclass(name = "SlackHandle")]
pub struct PySlackHandle {
    inner: SlackHandle,
}

impl PySlackHandle {
    pub fn from_handle(handle: SlackHandle) -> Self {
        Self { inner: handle }
    }
}

#[pymethods]
impl PySlackHandle {
    #[getter]
    fn constraint_id(&self) -> u32 {
        self.inner.constraint_id.inner()
    }

    #[getter]
    fn bound(&self) -> String {
        self.inner.bound.as_str().to_string()
    }

    #[getter]
    fn penalty(&self) -> f64 {
        self.inner.penalty
    }

    #[getter]
    fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    #[getter]
    fn var_ids(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("lower", self.inner.var_ids.lower.map(|id| id.inner()))?;
        dict.set_item("upper", self.inner.var_ids.upper.map(|id| id.inner()))?;
        Ok(dict.unbind().into())
    }
}

/// Python wrapper for an elastic handle.
#[pyclass(name = "ElasticHandle")]
pub struct PyElasticHandle {
    lower: Option<SlackHandle>,
    upper: Option<SlackHandle>,
}

impl PyElasticHandle {
    pub fn from_handle(handle: ElasticHandle) -> Self {
        Self {
            lower: handle.lower,
            upper: handle.upper,
        }
    }
}

#[pymethods]
impl PyElasticHandle {
    #[getter]
    fn lower(&self) -> Option<PySlackHandle> {
        self.lower
            .as_ref()
            .map(|handle| PySlackHandle::from_handle(handle.clone()))
    }

    #[getter]
    fn upper(&self) -> Option<PySlackHandle> {
        self.upper
            .as_ref()
            .map(|handle| PySlackHandle::from_handle(handle.clone()))
    }
}

/// Register handle classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySlackHandle>()?;
    m.add_class::<PyElasticHandle>()?;
    Ok(())
}
