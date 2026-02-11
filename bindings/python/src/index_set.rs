//! Python wrapper for index sets.

use crate::PyObject;
use crate::errors::{IndexSetArgumentError, IndexSetEmptyError, IndexSetTypeError};
use pyo3::IntoPyObject;
use pyo3::prelude::*;

/// Internal representation of an index set member.
#[derive(Debug, Clone)]
pub enum IndexMember {
    Int(i64),
    Float(f64),
    Str(String),
}

impl IndexMember {
    fn to_pyobject(&self, py: Python<'_>) -> PyResult<PyObject> {
        let obj = match self {
            IndexMember::Int(v) => v.into_pyobject(py)?.into_any(),
            IndexMember::Float(v) => v.into_pyobject(py)?.into_any(),
            IndexMember::Str(v) => v.into_pyobject(py)?.into_any(),
        };
        Ok(obj.unbind())
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            IndexMember::Int(v) => Some(*v as f64),
            IndexMember::Float(v) => Some(*v),
            IndexMember::Str(_) => None,
        }
    }
}

/// A named set of indices for array dimensions.
#[pyclass(from_py_object, name = "IndexSet")]
#[derive(Debug, Clone)]
pub struct PyIndexSet {
    pub name: String,
    pub members: Vec<IndexMember>,
}

#[pymethods]
impl PyIndexSet {
    #[new]
    #[pyo3(signature = (name, *, size=None, members=None))]
    fn new(name: String, size: Option<usize>, members: Option<Vec<PyObject>>) -> PyResult<Self> {
        match (size, members) {
            (Some(size), None) => {
                if size == 0 {
                    return Err(IndexSetEmptyError::new_err("size must be >= 1"));
                }
                let members = (0..size)
                    .map(|value| IndexMember::Int(value as i64))
                    .collect();
                Ok(Self { name, members })
            }
            (None, Some(members)) => {
                if members.is_empty() {
                    return Err(IndexSetEmptyError::new_err("members must be non-empty"));
                }
                Python::attach(|py| {
                    let mut parsed = Vec::with_capacity(members.len());
                    for member in members {
                        let bound = member.bind(py);
                        if let Ok(value) = bound.extract::<i64>() {
                            parsed.push(IndexMember::Int(value));
                        } else if let Ok(value) = bound.extract::<f64>() {
                            parsed.push(IndexMember::Float(value));
                        } else if let Ok(value) = bound.extract::<String>() {
                            parsed.push(IndexMember::Str(value));
                        } else {
                            return Err(IndexSetTypeError::new_err(
                                "members must be int, float, or str",
                            ));
                        }
                    }
                    Ok(Self {
                        name,
                        members: parsed,
                    })
                })
            }
            (Some(_), Some(_)) => Err(IndexSetArgumentError::new_err(
                "provide size or members, not both",
            )),
            (None, None) => Err(IndexSetArgumentError::new_err(
                "size or members is required",
            )),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[getter]
    fn size(&self) -> usize {
        self.members.len()
    }

    #[getter]
    fn members(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.members
            .iter()
            .map(|member| member.to_pyobject(py))
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "IndexSet(name='{}', size={})",
            self.name,
            self.members.len()
        )
    }
}

/// Register index set class with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIndexSet>()?;
    Ok(())
}
