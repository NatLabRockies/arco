use arco_expr::ComparisonSense;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyExpr;
use crate::PyObject;
use crate::index_set::PyIndexSet;

use super::CompactTerm;

/// Right-hand side for compact constraints.
#[derive(Clone, Debug)]
pub(crate) enum CompactRhs {
    /// All elements share the same rhs value.
    Scalar(f64),
    /// Per-element rhs values.
    Vec(Vec<f64>),
}

/// Compact representation for constraints from compact expressions.
/// Terms and rhs are already adjusted (constant subtracted from rhs).
#[derive(Clone, Debug)]
pub(crate) struct CompactConstraintStorage {
    pub terms: Vec<CompactTerm>,
    pub sense: ComparisonSense,
    pub rhs: CompactRhs,
    pub count: usize,
}

impl CompactConstraintStorage {
    /// Get the rhs as a Vec<f64> (expanding scalar if needed).
    pub fn rhs_vec(&self) -> Vec<f64> {
        match &self.rhs {
            CompactRhs::Scalar(v) => vec![*v; self.count],
            CompactRhs::Vec(v) => v.clone(),
        }
    }

    /// Get the term patterns as (start_var_id, coefficient) pairs.
    pub fn term_patterns(&self) -> Vec<(u32, f64)> {
        self.terms
            .iter()
            .map(|t| (t.start_var_id, t.coefficient))
            .collect()
    }
}

/// Internal storage enum for ConstraintArray.
pub(crate) enum ConstraintArrayStorage {
    /// Full materialized storage with per-element expressions.
    Full {
        exprs: Vec<PyExpr>,
        sense: ComparisonSense,
        rhs: Vec<f64>,
    },
    /// Compact: constraints from compact expression patterns (not yet inserted).
    Compact(CompactConstraintStorage),
}

/// A multi-dimensional array of constraint expressions.
#[pyclass(name = "ConstraintArray")]
pub struct PyConstraintArray {
    storage: ConstraintArrayStorage,
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
            storage: ConstraintArrayStorage::Full { exprs, sense, rhs },
            shape,
            index_sets,
        }
    }

    /// Create a ConstraintArray from compact constraint storage.
    pub(crate) fn from_compact(
        compact: CompactConstraintStorage,
        shape: Vec<usize>,
        index_sets: Vec<Py<PyIndexSet>>,
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::Compact(compact),
            shape,
            index_sets,
        }
    }

    pub fn exprs(&self) -> &[PyExpr] {
        match &self.storage {
            ConstraintArrayStorage::Full { exprs, .. } => exprs,
            ConstraintArrayStorage::Compact(_) => &[], // Should not be called on compact
        }
    }

    pub fn get_sense(&self) -> ComparisonSense {
        match &self.storage {
            ConstraintArrayStorage::Full { sense, .. } => *sense,
            ConstraintArrayStorage::Compact(c) => c.sense,
        }
    }

    pub fn get_rhs(&self) -> Vec<f64> {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. } => rhs.clone(),
            ConstraintArrayStorage::Compact(c) => c.rhs_vec(),
        }
    }

    /// Get compact storage if available.
    pub(crate) fn as_compact(&self) -> Option<&CompactConstraintStorage> {
        match &self.storage {
            ConstraintArrayStorage::Compact(c) => Some(c),
            _ => None,
        }
    }

    /// Create a lightweight ConstraintArray from batch insertion results.
    /// The exprs are empty since constraints have already been added to the model.
    pub fn from_batch(
        _first_constraint_id: u32,
        count: usize,
        sense: ComparisonSense,
        rhs: &[f64],
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::Full {
                exprs: Vec::new(),
                sense,
                rhs: rhs.to_vec(),
            },
            shape: vec![count],
            index_sets: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. } => rhs.len(),
            ConstraintArrayStorage::Compact(c) => c.count,
        }
    }
}

#[pymethods]
impl PyConstraintArray {
    #[getter]
    fn sense(&self) -> String {
        self.get_sense().as_str().to_string()
    }

    #[getter]
    fn rhs(&self) -> Vec<f64> {
        self.get_rhs()
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
        self.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "ConstraintArray(shape={:?}, sense='{}')",
            self.shape,
            self.get_sense().as_str()
        )
    }
}
