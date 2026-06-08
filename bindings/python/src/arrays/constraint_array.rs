use arco_ops::expression::ComparisonSense;
use arco_ops::modeling::types::Bounds;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};

use crate::PyExpr;
use crate::PyObject;
use crate::py_modules::constraint::PyConstraint;
use crate::py_modules::errors::{ArrayIndexError, ArrayTypeError};
use crate::py_modules::index_set::PyIndexSet;

use super::CompactTerm;
use super::LinearArrayCore;

pub(crate) type SparseConstraintRows<'a> = (&'a [PyExpr], &'a [f64], &'a [usize], ComparisonSense);

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
    SparseRows {
        exprs: Vec<PyExpr>,
        sense: ComparisonSense,
        rhs: Vec<f64>,
        active_indices: Vec<usize>,
    },
    /// Lazy array-vs-array comparison used to apply active masks before
    /// allocating per-row terms.
    LazyCompare {
        left: LinearArrayCore,
        right: LinearArrayCore,
        sense: ComparisonSense,
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
    /// Set after constraints are inserted into the model (via `from_batch`).
    first_constraint_id: Option<u32>,
    /// Base name applied when this array was inserted into a model.
    name: Option<String>,
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
            first_constraint_id: None,
            name: None,
        }
    }

    pub(crate) fn from_sparse_rows(
        exprs: Vec<PyExpr>,
        sense: ComparisonSense,
        rhs: Vec<f64>,
        active_indices: Vec<usize>,
        shape: Vec<usize>,
        index_sets: Vec<Py<PyIndexSet>>,
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::SparseRows {
                exprs,
                sense,
                rhs,
                active_indices,
            },
            shape,
            index_sets,
            first_constraint_id: None,
            name: None,
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
            first_constraint_id: None,
            name: None,
        }
    }

    pub(crate) fn from_lazy_compare(
        left: LinearArrayCore,
        right: LinearArrayCore,
        sense: ComparisonSense,
    ) -> Self {
        let shape = left.shape.clone();
        let index_sets = left.clone_index_sets();
        Self {
            storage: ConstraintArrayStorage::LazyCompare { left, right, sense },
            shape,
            index_sets,
            first_constraint_id: None,
            name: None,
        }
    }

    pub fn exprs(&self) -> &[PyExpr] {
        match &self.storage {
            ConstraintArrayStorage::Full { exprs, .. }
            | ConstraintArrayStorage::SparseRows { exprs, .. } => exprs,
            ConstraintArrayStorage::LazyCompare { .. } => &[],
            ConstraintArrayStorage::Compact(_) => &[], // Should not be called on compact
        }
    }

    pub fn get_sense(&self) -> ComparisonSense {
        match &self.storage {
            ConstraintArrayStorage::Full { sense, .. }
            | ConstraintArrayStorage::SparseRows { sense, .. } => *sense,
            ConstraintArrayStorage::LazyCompare { sense, .. } => *sense,
            ConstraintArrayStorage::Compact(c) => c.sense,
        }
    }

    pub fn get_rhs(&self) -> Vec<f64> {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. }
            | ConstraintArrayStorage::SparseRows { rhs, .. } => rhs.clone(),
            ConstraintArrayStorage::LazyCompare { left, right, .. } => left
                .values
                .iter()
                .zip(right.values.iter())
                .map(|(left_expr, right_expr)| {
                    let diff = left_expr.inner().add(&right_expr.inner().scale(-1.0));
                    -PyExpr::from_expr(diff).constant()
                })
                .collect(),
            ConstraintArrayStorage::Compact(c) => c.rhs_vec(),
        }
    }

    pub(crate) fn shape_ref(&self) -> &[usize] {
        &self.shape
    }

    pub(crate) fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
        Python::attach(|py| {
            self.index_sets
                .iter()
                .map(|set| set.clone_ref(py))
                .collect()
        })
    }

    /// Get compact storage if available.
    pub(crate) fn as_compact(&self) -> Option<&CompactConstraintStorage> {
        match &self.storage {
            ConstraintArrayStorage::Compact(c) => Some(c),
            ConstraintArrayStorage::Full { .. } | ConstraintArrayStorage::SparseRows { .. } => None,
            ConstraintArrayStorage::LazyCompare { .. } => None,
        }
    }

    pub(crate) fn as_lazy_compare(
        &self,
    ) -> Option<(&LinearArrayCore, &LinearArrayCore, ComparisonSense)> {
        match &self.storage {
            ConstraintArrayStorage::LazyCompare { left, right, sense } => {
                Some((left, right, *sense))
            }
            ConstraintArrayStorage::Full { .. }
            | ConstraintArrayStorage::SparseRows { .. }
            | ConstraintArrayStorage::Compact(_) => None,
        }
    }

    pub(crate) fn as_sparse_rows(&self) -> Option<SparseConstraintRows<'_>> {
        match &self.storage {
            ConstraintArrayStorage::SparseRows {
                exprs,
                sense,
                rhs,
                active_indices,
            } => Some((exprs, rhs, active_indices, *sense)),
            ConstraintArrayStorage::Full { .. }
            | ConstraintArrayStorage::LazyCompare { .. }
            | ConstraintArrayStorage::Compact(_) => None,
        }
    }

    /// Create a lightweight ConstraintArray from batch insertion results.
    /// The exprs are empty since constraints have already been added to the model.
    pub fn from_batch(
        first_constraint_id: u32,
        count: usize,
        sense: ComparisonSense,
        rhs: &[f64],
        name: Option<String>,
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::Full {
                exprs: Vec::new(),
                sense,
                rhs: rhs.to_vec(),
            },
            shape: vec![count],
            index_sets: Vec::new(),
            first_constraint_id: Some(first_constraint_id),
            name,
        }
    }

    pub(crate) fn from_batch_shaped(
        first_constraint_id: u32,
        shape: Vec<usize>,
        index_sets: Vec<Py<PyIndexSet>>,
        sense: ComparisonSense,
        rhs: &[f64],
        name: Option<String>,
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::Full {
                exprs: Vec::new(),
                sense,
                rhs: rhs.to_vec(),
            },
            shape,
            index_sets,
            first_constraint_id: Some(first_constraint_id),
            name,
        }
    }

    fn len(&self) -> usize {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. }
            | ConstraintArrayStorage::SparseRows { rhs, .. } => rhs.len(),
            ConstraintArrayStorage::LazyCompare { left, .. } => left.values.len(),
            ConstraintArrayStorage::Compact(c) => c.count,
        }
    }

    /// Get the rhs value at a specific index without allocating.
    fn rhs_at(&self, index: usize) -> f64 {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. }
            | ConstraintArrayStorage::SparseRows { rhs, .. } => rhs[index],
            ConstraintArrayStorage::LazyCompare { left, right, .. } => {
                let diff = left.values[index]
                    .inner()
                    .add(&right.values[index].inner().scale(-1.0));
                -PyExpr::from_expr(diff).constant()
            }
            ConstraintArrayStorage::Compact(c) => match &c.rhs {
                CompactRhs::Scalar(v) => *v,
                CompactRhs::Vec(v) => v[index],
            },
        }
    }

    fn constraint_at(&self, index: usize) -> PyResult<PyConstraint> {
        let first_id = self.first_constraint_id.ok_or_else(|| {
            ArrayTypeError::new_err(
                "this ConstraintArray has not been added to a model yet and is not subscriptable",
            )
        })?;
        let rhs_val = self.rhs_at(index);
        let bounds = match self.get_sense() {
            ComparisonSense::LessEqual => Bounds::new(f64::NEG_INFINITY, rhs_val),
            ComparisonSense::GreaterEqual => Bounds::new(rhs_val, f64::INFINITY),
            ComparisonSense::Equal => Bounds::new(rhs_val, rhs_val),
        };
        let name = self.name.as_ref().map(|base| {
            if self.len() == 1 {
                base.clone()
            } else {
                format!("{base}[{index}]")
            }
        });
        Ok(PyConstraint::new(first_id + index as u32, name, bounds))
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

    fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let constraints = (0..self.len())
            .map(|index| self.constraint_at(index))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyList::new(py, constraints)?
            .call_method0("__iter__")?
            .unbind())
    }

    fn __getitem__(&self, index: usize) -> PyResult<PyConstraint> {
        if index >= self.len() {
            return Err(ArrayIndexError::new_err(format!(
                "index {} out of range for ConstraintArray of size {}",
                index,
                self.len()
            )));
        }
        self.constraint_at(index)
    }

    fn __repr__(&self) -> String {
        format!(
            "ConstraintArray(shape={:?}, sense='{}')",
            self.shape,
            self.get_sense().as_str()
        )
    }
}
