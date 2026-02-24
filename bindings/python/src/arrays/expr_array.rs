use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyObject;
use crate::errors::{ArrayDimensionError, ArrayIndexError, ExprDivisionByZeroError};
use crate::expr::PyExpr;
use crate::index_set::PyIndexSet;

use super::indexing::{
    AxisIndex, maybe_boolean_mask_indices, resolve_axis_index, slice_indices, sliced_2d_index_sets,
};
use super::{
    CompactExprStorage, ExprArrayStorage, LinearArrayCore, try_extract_compact,
    try_make_compact_constraint,
};

/// A multi-dimensional array of linear expressions.
/// This is the result of any operation on VariableArray or ExprArray.
#[pyclass(name = "ExprArray")]
pub struct PyExprArray {
    pub(crate) storage: ExprArrayStorage,
}

impl PyExprArray {
    pub fn new(index_sets: Vec<Py<PyIndexSet>>, shape: Vec<usize>, values: Vec<PyExpr>) -> Self {
        Self {
            storage: ExprArrayStorage::Full(LinearArrayCore::new(index_sets, shape, values)),
        }
    }

    /// Create from compact expression storage.
    pub(crate) fn from_compact(
        compact: CompactExprStorage,
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
    ) -> Self {
        Self {
            storage: ExprArrayStorage::Compact {
                storage: compact,
                index_sets,
                shape,
            },
        }
    }

    /// Wrap a compact result using this array's index sets and shape.
    fn wrap_compact(&self, compact: CompactExprStorage) -> Self {
        Self::from_compact(
            compact,
            self.storage.clone_index_sets(),
            self.storage.shape().to_vec(),
        )
    }

    /// Materialize the LinearArrayCore on demand.
    pub(crate) fn to_core(&self) -> LinearArrayCore {
        self.storage.to_core()
    }

    /// Get compact storage if available.
    pub(crate) fn as_compact(&self) -> Option<&CompactExprStorage> {
        self.storage.as_compact()
    }

    fn getitem_tuple(&self, py: Python<'_>, tuple: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
        let shape = self.storage.shape();
        if shape.len() != 2 || tuple.len() != 2 {
            return Err(ArrayDimensionError::new_err(
                "tuple indexing requires a 2D array and exactly 2 indices",
            ));
        }
        // Materialize for indexing (indexing is rare in the hot path)
        let core = self.to_core();
        let nrows = core.shape[0];
        let ncols = core.shape[1];
        let idx0 = tuple.get_item(0)?;
        let idx1 = tuple.get_item(1)?;

        let rows = resolve_axis_index(&idx0, nrows)?;
        let cols = resolve_axis_index(&idx1, ncols)?;

        match (&rows, &cols) {
            (AxisIndex::Single(r), AxisIndex::Single(c)) => {
                let flat_idx = r * ncols + c;
                let expr = core.values.get(flat_idx).cloned().ok_or_else(|| {
                    ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
                })?;
                Ok(expr.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Single(r), AxisIndex::Range(col_indices)) => {
                let mut vals = Vec::with_capacity(col_indices.len());
                for &c in col_indices {
                    let flat_idx = r * ncols + c;
                    vals.push(core.values[flat_idx].clone());
                }
                let n = vals.len();
                let new_index_sets =
                    if col_indices.len() == ncols && core.index_sets.len() == 2 {
                        vec![core.index_sets[1].clone_ref(py)]
                    } else {
                        Vec::new()
                    };
                let result = PyExprArray::new(new_index_sets, vec![n], vals);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Range(row_indices), AxisIndex::Single(c)) => {
                let mut vals = Vec::with_capacity(row_indices.len());
                for &r in row_indices {
                    let flat_idx = r * ncols + c;
                    vals.push(core.values[flat_idx].clone());
                }
                let n = vals.len();
                let new_index_sets =
                    if row_indices.len() == nrows && core.index_sets.len() == 2 {
                        vec![core.index_sets[0].clone_ref(py)]
                    } else {
                        Vec::new()
                    };
                let result = PyExprArray::new(new_index_sets, vec![n], vals);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Range(row_indices), AxisIndex::Range(col_indices)) => {
                let new_nrows = row_indices.len();
                let new_ncols = col_indices.len();
                let mut vals = Vec::with_capacity(new_nrows * new_ncols);
                for &r in row_indices {
                    for &c in col_indices {
                        let flat_idx = r * ncols + c;
                        vals.push(core.values[flat_idx].clone());
                    }
                }
                let new_index_sets = sliced_2d_index_sets(
                    py,
                    &core.index_sets,
                    nrows,
                    ncols,
                    row_indices,
                    col_indices,
                )?;
                let result = PyExprArray::new(new_index_sets, vec![new_nrows, new_ncols], vals);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
        }
    }

    /// Get values, materializing if needed.
    pub(crate) fn get_values(&self) -> Vec<PyExpr> {
        match &self.storage {
            ExprArrayStorage::Full(core) => core.values.clone(),
            ExprArrayStorage::Compact {
                storage,
                index_sets,
                shape,
            } => storage.to_core(index_sets, shape).values,
        }
    }
}

// Explicit #[pymethods] with compact fast paths (replaces impl_array_ops! macro usage).
#[pymethods]
impl PyExprArray {
    fn __add__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact() {
            if let Some(other_compact) = try_extract_compact(other) {
                if self_compact.count == other_compact.count {
                    return Ok(self.wrap_compact(self_compact.add_compact(&other_compact)));
                }
            }
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self.wrap_compact(self_compact.add_constant(value)));
            }
        }
        let core = self.to_core();
        super::array_add(&core, other)
    }

    fn __radd__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        self.__add__(other)
    }

    fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact() {
            if let Some(other_compact) = try_extract_compact(other) {
                if self_compact.count == other_compact.count {
                    return Ok(self.wrap_compact(self_compact.sub_compact(&other_compact)));
                }
            }
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self.wrap_compact(self_compact.add_constant(-value)));
            }
        }
        let core = self.to_core();
        super::array_sub(&core, other)
    }

    fn __rsub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact() {
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self.wrap_compact(self_compact.scale(-1.0).add_constant(value)));
            }
            if let Some(other_compact) = try_extract_compact(other) {
                if other_compact.count == self_compact.count {
                    return Ok(self.wrap_compact(other_compact.sub_compact(self_compact)));
                }
            }
        }
        let core = self.to_core();
        super::array_rsub(&core, other)
    }

    fn __mul__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact() {
            if let Ok(scalar) = other.extract::<f64>() {
                return Ok(self.wrap_compact(self_compact.scale(scalar)));
            }
        }
        let core = self.to_core();
        super::array_mul(&core, other)
    }

    fn __rmul__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        self.__mul__(other)
    }

    fn __truediv__(&self, other: f64) -> PyResult<PyExprArray> {
        if other == 0.0 {
            return Err(ExprDivisionByZeroError::new_err("division by zero"));
        }
        if let Some(self_compact) = self.as_compact() {
            return Ok(self.wrap_compact(self_compact.scale(1.0 / other)));
        }
        let core = self.to_core();
        super::array_truediv(&core, other)
    }

    fn __neg__(&self) -> PyExprArray {
        if let Some(self_compact) = self.as_compact() {
            return self.wrap_compact(self_compact.scale(-1.0));
        }
        let core = self.to_core();
        super::array_neg(&core)
    }

    fn __ge__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<super::PyConstraintArray> {
        // Fast path: compact >= scalar/index_set/vec
        if let Some(self_compact) = self.as_compact() {
            if let Some(compact_con) = try_make_compact_constraint(
                self_compact,
                rhs,
                super::ComparisonSense::GreaterEqual,
            )? {
                return Ok(super::PyConstraintArray::from_compact(
                    compact_con,
                    self.storage.shape().to_vec(),
                    self.storage.clone_index_sets(),
                ));
            }
        }
        let core = self.to_core();
        super::compare_array_rhs(&core, rhs, super::ComparisonSense::GreaterEqual)
    }

    fn __le__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<super::PyConstraintArray> {
        if let Some(self_compact) = self.as_compact() {
            if let Some(compact_con) = try_make_compact_constraint(
                self_compact,
                rhs,
                super::ComparisonSense::LessEqual,
            )? {
                return Ok(super::PyConstraintArray::from_compact(
                    compact_con,
                    self.storage.shape().to_vec(),
                    self.storage.clone_index_sets(),
                ));
            }
        }
        let core = self.to_core();
        super::compare_array_rhs(&core, rhs, super::ComparisonSense::LessEqual)
    }

    fn __eq__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<super::PyConstraintArray> {
        if let Some(self_compact) = self.as_compact() {
            if let Some(compact_con) =
                try_make_compact_constraint(self_compact, rhs, super::ComparisonSense::Equal)?
            {
                return Ok(super::PyConstraintArray::from_compact(
                    compact_con,
                    self.storage.shape().to_vec(),
                    self.storage.clone_index_sets(),
                ));
            }
        }
        let core = self.to_core();
        super::compare_array_rhs(&core, rhs, super::ComparisonSense::Equal)
    }

    #[pyo3(signature = (*, over=None))]
    fn sum(&self, py: Python<'_>, over: Option<&Bound<'_, PyAny>>) -> PyResult<PyObject> {
        // Fast path: sum all elements of compact storage
        if over.is_none() {
            if let Some(compact) = self.as_compact() {
                let result = compact.sum_all();
                return Ok(result.into_pyobject(py)?.into_any().unbind());
            }
        }
        let core = self.to_core();
        super::array_sum(&core, py, over)
    }

    fn __rshift__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let core = self.to_core();
        super::array_reduce(&core, py, rhs)
    }

    fn __matmul__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let core = self.to_core();
        super::array_reduce(&core, py, rhs)
    }

    #[getter]
    fn index_sets(&self, py: Python<'_>) -> PyResult<PyObject> {
        let sets = self
            .storage
            .index_sets_ref()
            .iter()
            .map(|set| set.clone_ref(py))
            .collect::<Vec<_>>();
        Ok(PyTuple::new(py, sets)?.into())
    }

    #[getter]
    fn shape(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(PyTuple::new(py, self.storage.shape().to_vec())?.into())
    }

    #[getter]
    fn values(&self) -> Vec<PyExpr> {
        self.get_values()
    }

    fn flatten(&self) -> Vec<PyExpr> {
        self.get_values()
    }

    fn __len__(&self) -> usize {
        self.storage.count()
    }

    #[pyo3(signature = (ufunc, method, *inputs, **kwargs))]
    fn __array_ufunc__(
        &self,
        py: Python<'_>,
        ufunc: &Bound<'_, PyAny>,
        method: &str,
        inputs: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<PyObject> {
        let core = self.to_core();
        super::array_ufunc(
            &core,
            py,
            |ob| ob.is_instance_of::<PyExprArray>(),
            ufunc,
            method,
            inputs,
            kwargs,
        )
    }

    fn __array_function__(
        &self,
        py: Python<'_>,
        func: &Bound<'_, PyAny>,
        _types: &Bound<'_, PyAny>,
        args: &Bound<'_, PyTuple>,
        kwargs: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let core = self.to_core();
        super::array_function(&core, py, func, _types, args, kwargs)
    }

    fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(tuple) = index.cast::<PyTuple>() {
            return self.getitem_tuple(py, tuple);
        }

        // Materialize for indexing
        let core = self.to_core();

        if let Ok(idx) = index.extract::<usize>() {
            return core
                .values
                .get(idx)
                .cloned()
                .ok_or_else(|| {
                    ArrayIndexError::new_err(format!(
                        "index {} out of range for array of size {}",
                        idx,
                        core.values.len()
                    ))
                })
                .and_then(|v| Ok(v.into_pyobject(py)?.into_any().unbind()));
        }

        if let Some(mask_indices) = maybe_boolean_mask_indices(py, index, core.values.len())? {
            let filtered_values = mask_indices
                .iter()
                .map(|&idx| core.values[idx].clone())
                .collect::<Vec<PyExpr>>();
            let n = filtered_values.len();
            let result = PyExprArray::new(Vec::new(), vec![n], filtered_values);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let selected = slice_indices(slice, core.values.len())?;
            let sliced_values = selected
                .iter()
                .map(|&idx| core.values[idx].clone())
                .collect::<Vec<PyExpr>>();
            let n = sliced_values.len();
            let result = PyExprArray::new(Vec::new(), vec![n], sliced_values);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        Err(ArrayIndexError::new_err(
            "index must be an integer, tuple, slice, or a boolean numpy array",
        ))
    }

    fn __repr__(&self) -> String {
        format!("ExprArray(shape={:?})", self.storage.shape())
    }
}
