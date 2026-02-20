use arco_expr::ComparisonSense;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyExpr;
use crate::PyObject;
use crate::index_set::PyIndexSet;

/// A multi-dimensional array of constraint expressions.
#[pyclass(name = "ConstraintArray")]
pub struct PyConstraintArray {
    exprs: Vec<PyExpr>,
    sense: ComparisonSense,
    rhs: Vec<f64>,
    shape: Vec<usize>,
    index_sets: Vec<Py<PyIndexSet>>,
}

impl PyConstraintArray {
    pub fn new(
        exprs: Vec<PyExpr>,
        sense: ComparisonSense,
        rhs: Vec<f64>,
        shape: Vec<usize>,
        index_sets: Vec<Py<PyIndexSet>>,
    ) -> Self {
        Self {
            exprs,
            sense,
            rhs,
            shape,
            index_sets,
        }
    }

    pub fn exprs(&self) -> &[PyExpr] {
        &self.exprs
    }

    pub fn get_sense(&self) -> ComparisonSense {
        self.sense
    }

    pub fn get_rhs(&self) -> &[f64] {
        &self.rhs
    }
}

#[pymethods]
impl PyConstraintArray {
    #[getter]
    fn sense(&self) -> String {
        self.sense.as_str().to_string()
    }

    #[getter]
    fn rhs(&self) -> Vec<f64> {
        self.rhs.clone()
    }

    #[getter]
    fn shape(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(PyTuple::new(py, self.shape.clone())?.into())
    }

    #[getter]
    fn index_sets(&self, py: Python<'_>) -> PyResult<PyObject> {
        let sets = self
            .index_sets
            .iter()
            .map(|set| set.clone_ref(py))
            .collect::<Vec<_>>();
        Ok(PyTuple::new(py, sets)?.into())
    }

    fn __len__(&self) -> usize {
        self.exprs.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "ConstraintArray(shape={:?}, sense='{}')",
            self.shape,
            self.sense.as_str()
        )
    }
}
