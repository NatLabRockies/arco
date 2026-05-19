//! Python wrapper for index sets.

use crate::PyObject;
use crate::py_modules::errors::{
    IndexSetArgumentError, IndexSetEmptyError, IndexSetIndexError, IndexSetTypeError,
};
use pyo3::IntoPyObject;
use pyo3::prelude::*;
use pyo3::types::{PyList, PySlice};

/// Internal representation of an index set member.
#[derive(Debug, Clone)]
pub enum IndexMember {
    Int(i64),
    Float(f64),
    Str(String),
    Tuple(Vec<IndexMember>),
}

impl IndexMember {
    fn from_bound(value: &Bound<'_, PyAny>) -> Option<Self> {
        if let Ok(parsed) = value.extract::<i64>() {
            return Some(Self::Int(parsed));
        }
        if let Ok(parsed) = value.extract::<f64>() {
            return Some(Self::Float(parsed));
        }
        if let Ok(parsed) = value.extract::<String>() {
            return Some(Self::Str(parsed));
        }
        if let Ok(tuple) = value.cast::<pyo3::types::PyTuple>() {
            let mut items = Vec::with_capacity(tuple.len());
            for item in tuple.iter() {
                let parsed = Self::from_bound(&item)?;
                items.push(parsed);
            }
            return Some(Self::Tuple(items));
        }
        None
    }

    fn to_pyobject(&self, py: Python<'_>) -> PyResult<PyObject> {
        let obj = match self {
            IndexMember::Int(v) => v.into_pyobject(py)?.into_any(),
            IndexMember::Float(v) => v.into_pyobject(py)?.into_any(),
            IndexMember::Str(v) => v.into_pyobject(py)?.into_any(),
            IndexMember::Tuple(items) => {
                let tuple_items = items
                    .iter()
                    .map(|item| item.to_pyobject(py))
                    .collect::<PyResult<Vec<PyObject>>>()?;
                pyo3::types::PyTuple::new(py, tuple_items)?.into_any()
            }
        };
        Ok(obj.unbind())
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            IndexMember::Int(v) => Some(*v as f64),
            IndexMember::Float(v) => Some(*v),
            IndexMember::Str(_) | IndexMember::Tuple(_) => None,
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
    #[pyo3(signature = (*, name, size=None, members=None))]
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
                        let parsed_member = IndexMember::from_bound(bound).ok_or_else(|| {
                            IndexSetTypeError::new_err(
                                "members must be int, float, str, or tuples of those",
                            )
                        })?;
                        parsed.push(parsed_member);
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

    fn alias(&self, name: String) -> Self {
        Self {
            name,
            members: self.members.clone(),
        }
    }

    fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(idx) = index.extract::<isize>() {
            let resolved = if idx < 0 {
                self.members.len() as isize + idx
            } else {
                idx
            };
            if resolved < 0 || resolved as usize >= self.members.len() {
                return Err(IndexSetIndexError::new_err(format!(
                    "index {} out of range for IndexSet of size {}",
                    idx,
                    self.members.len()
                )));
            }
            return self.members[resolved as usize].to_pyobject(py);
        }

        if let Ok(slice) = index.cast::<PySlice>() {
            let indices = slice.indices(self.members.len() as isize)?;
            let mut members = Vec::new();
            let mut current = indices.start;
            while (indices.step > 0 && current < indices.stop)
                || (indices.step < 0 && current > indices.stop)
            {
                members.push(self.members[current as usize].clone());
                current += indices.step;
            }
            if members.is_empty() {
                return Err(IndexSetEmptyError::new_err(
                    "IndexSet slices must contain at least one member",
                ));
            }
            return Ok(Self {
                name: self.name.clone(),
                members,
            }
            .into_pyobject(py)?
            .into_any()
            .unbind());
        }

        Err(IndexSetTypeError::new_err(
            "IndexSet indices must be integers or slices",
        ))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let items = self
            .members
            .iter()
            .map(|member| member.to_pyobject(py))
            .collect::<PyResult<Vec<PyObject>>>()?;
        Ok(PyList::new(py, items)?
            .call_method0("__iter__")?
            .unbind()
            .into())
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
