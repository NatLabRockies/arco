//! Python iterators for model constraints and variables.

use arco_expr::{ConstraintId, VariableId};
use pyo3::prelude::*;

use crate::PyModel;
use crate::constraint::PyConstraint;
use crate::variable::PyVariable;

/// Iterator over constraints in a model.
#[pyclass(name = "ConstraintIterator")]
pub struct PyConstraintIterator {
    model: Py<PyModel>,
    index: usize,
    total: usize,
}

impl PyConstraintIterator {
    pub fn new(model: Py<PyModel>, total: usize) -> Self {
        Self {
            model,
            index: 0,
            total,
        }
    }
}

#[pymethods]
impl PyConstraintIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyConstraint> {
        if slf.index >= slf.total {
            return None;
        }
        let i = slf.index;
        slf.index += 1;

        let py = slf.py();
        let model = slf.model.borrow(py);
        let con_id = ConstraintId::new(i as u32);
        let con = model.inner.get_constraint(con_id).ok()?;
        let name = model
            .inner
            .get_constraint_name(con_id)
            .map(|s| s.to_string());
        Some(PyConstraint::new(i as u32, name, con.bounds))
    }

    fn __len__(&self) -> usize {
        self.total.saturating_sub(self.index)
    }
}

/// Iterator over variables in a model.
#[pyclass(name = "VariableIterator")]
pub struct PyVariableIterator {
    model: Py<PyModel>,
    index: usize,
    total: usize,
}

impl PyVariableIterator {
    pub fn new(model: Py<PyModel>, total: usize) -> Self {
        Self {
            model,
            index: 0,
            total,
        }
    }
}

#[pymethods]
impl PyVariableIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyVariable> {
        if slf.index >= slf.total {
            return None;
        }
        let i = slf.index;
        slf.index += 1;

        let py = slf.py();
        let model = slf.model.borrow(py);
        let var_id = VariableId::new(i as u32);
        let var = model.inner.get_variable(var_id).ok()?;
        let name = model.inner.get_variable_name(var_id).map(|s| s.to_string());
        Some(PyVariable::from_model_variable(i as u32, name, &var))
    }

    fn __len__(&self) -> usize {
        self.total.saturating_sub(self.index)
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyConstraintIterator>()?;
    m.add_class::<PyVariableIterator>()?;
    Ok(())
}
