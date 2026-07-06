use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::PyObject;
use crate::py_modules::errors::{ArrayDimensionError, ArrayIndexError, ExprDivisionByZeroError};
use crate::py_modules::expr::PyExpr;
use crate::py_modules::index_set::PyIndexSet;

use super::indexing::{
    AxisIndex, maybe_boolean_mask_indices, resolve_axis_index, selected_flat_indices,
    slice_indices, sliced_2d_index_sets, sliced_and_index_sets,
};
use crate::py_modules::arrays::{
    CompactExprStorage, ComparisonSense, ExprArrayStorage, ExpressionTermCounts, LinearArrayCore,
    PyConstraintArray, PyVariableArray, SparseExprStorage, array_cumsum, array_diff, array_roll,
    combine_sparse_expr_same_shape, compare_sparse_expr_same_shape, compare_with_compact_fallback,
    diff_sparse_expr, expression_term_counts, multiply_sparse_expr_with_labeled_operand,
    multiply_sparse_expr_with_scalar, roll_sparse_expr, set_solver_matrix_memory_estimate,
    sum_sparse_expr, try_extract_compact,
};

/// A multi-dimensional array of linear expressions.
#[pyo3_macros::pyclass(name = "ExprArray")]
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
    pub fn from_compact(
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

    pub fn from_sparse(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        active_indices: Vec<usize>,
        values: Vec<PyExpr>,
    ) -> Self {
        debug_assert_eq!(
            active_indices.len(),
            values.len(),
            "SparseExprStorage invariant: active_indices and values must have the same length"
        );
        Self {
            storage: ExprArrayStorage::Sparse {
                storage: SparseExprStorage {
                    active_indices,
                    values,
                },
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
    pub fn to_core(&self) -> LinearArrayCore {
        self.storage.to_core()
    }

    /// Get compact storage if available.
    pub fn as_compact(&self) -> Option<&CompactExprStorage> {
        self.storage.as_compact()
    }

    /// Shared comparison logic for __ge__, __le__, __eq__.
    fn compare(
        &self,
        rhs: &Bound<'_, PyAny>,
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintArray> {
        if let Some((left_indices, left_values)) = self.sparse_entries() {
            if let Ok(rhs_array) = rhs.extract::<PyRef<'_, PyExprArray>>() {
                if rhs_array.storage.shape() == self.storage.shape() {
                    if let Some((right_indices, right_values)) = rhs_array.sparse_entries() {
                        return Ok(compare_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            left_values,
                            right_indices,
                            right_values,
                            sense,
                        ));
                    }
                }
            }
            if let Ok(rhs_array) = rhs.extract::<PyRef<'_, PyVariableArray>>() {
                if rhs_array.get_shape() == self.storage.shape() {
                    if let Some((right_indices, right_values)) = rhs_array.sparse_expr_entries() {
                        return Ok(compare_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            left_values,
                            right_indices,
                            &right_values,
                            sense,
                        ));
                    }
                }
            }
        }
        compare_with_compact_fallback(
            self.as_compact(),
            self.storage.shape(),
            self.storage.index_sets_ref(),
            || self.to_core(),
            rhs,
            sense,
        )
    }

    fn getitem_tuple(&self, py: Python<'_>, tuple: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
        let shape = self.storage.shape();
        if tuple.len() > shape.len() {
            return Err(ArrayDimensionError::new_err(
                "tuple indexing cannot specify more dimensions than the array rank",
            ));
        }
        // Materialize for indexing (indexing is rare in the hot path)
        let core = self.to_core();
        if shape.len() == 2 && tuple.len() == 2 {
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
                    return Ok(expr.into_pyobject(py)?.into_any().unbind());
                }
                (AxisIndex::Range(row_indices), AxisIndex::Range(col_indices))
                    if core.index_sets.len() == 2 =>
                {
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
                    return Ok(result.into_pyobject(py)?.into_any().unbind());
                }
                _ => {}
            }
        }

        let mut selections = Vec::with_capacity(shape.len());
        for (axis, axis_len) in shape.iter().copied().enumerate() {
            if axis < tuple.len() {
                selections.push(resolve_axis_index(&tuple.get_item(axis)?, axis_len)?);
            } else {
                selections.push(AxisIndex::Range((0..axis_len).collect()));
            }
        }
        let (flat_indices, out_shape) = selected_flat_indices(shape, &selections);
        if out_shape.is_empty() {
            let flat_idx = *flat_indices.first().ok_or_else(|| {
                ArrayIndexError::new_err("scalar tuple indexing did not resolve any element")
            })?;
            let expr = core.values.get(flat_idx).cloned().ok_or_else(|| {
                ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
            })?;
            return Ok(expr.into_pyobject(py)?.into_any().unbind());
        }
        let vals = flat_indices
            .iter()
            .map(|&idx| core.values[idx].clone())
            .collect::<Vec<_>>();
        let new_index_sets = sliced_and_index_sets(py, &core.index_sets, shape, &selections)?;
        let result = PyExprArray::new(new_index_sets, out_shape, vals);
        Ok(result.into_pyobject(py)?.into_any().unbind())
    }

    /// Get values, materializing if needed.
    pub fn get_values(&self) -> Vec<PyExpr> {
        match &self.storage {
            ExprArrayStorage::Full(core) => core.values.clone(),
            ExprArrayStorage::Compact {
                storage,
                index_sets,
                shape,
            } => storage.to_core(index_sets, shape).values,
            ExprArrayStorage::Sparse {
                storage,
                index_sets,
                shape,
            } => storage.to_core(index_sets, shape).values,
        }
    }

    fn storage_kind(&self) -> &'static str {
        match &self.storage {
            ExprArrayStorage::Compact { .. } => "compact",
            ExprArrayStorage::Sparse { .. } => "sparse",
            ExprArrayStorage::Full(_) => "full",
        }
    }

    fn term_counts(&self) -> ExpressionTermCounts {
        match &self.storage {
            ExprArrayStorage::Compact { storage, .. } => storage.term_counts(),
            ExprArrayStorage::Sparse { storage, .. } => storage.term_counts(),
            ExprArrayStorage::Full(core) => expression_term_counts(&core.values),
        }
    }

    pub fn sparse_entries(&self) -> Option<(&[usize], &[PyExpr])> {
        self.storage
            .as_sparse()
            .map(|storage| (storage.active_indices.as_slice(), storage.values.as_slice()))
    }

    fn relabeled_axis(
        &self,
        py: Python<'_>,
        old_axis: &Bound<'_, PyIndexSet>,
        new_axis: &Bound<'_, PyIndexSet>,
    ) -> PyResult<Self> {
        let old_name = old_axis.borrow().name.clone();
        let new_len = new_axis.borrow().members.len();
        let index_sets = self.storage.index_sets_ref();
        let mut axis_idx = None;
        for (idx, index_set) in index_sets.iter().enumerate() {
            let stored = index_set.bind(py).borrow();
            if stored.name == old_name {
                if stored.members.len() != new_len {
                    return Err(ArrayDimensionError::new_err(format!(
                        "cannot relabel axis '{}' of length {} to '{}' of length {}",
                        stored.name,
                        stored.members.len(),
                        new_axis.borrow().name,
                        new_len
                    )));
                }
                axis_idx = Some(idx);
                break;
            }
        }
        let axis_idx = axis_idx.ok_or_else(|| {
            ArrayIndexError::new_err(format!(
                "IndexSet '{}' is not a dimension of this array",
                old_name
            ))
        })?;

        let mut new_index_sets = index_sets
            .iter()
            .map(|index_set| index_set.clone_ref(py))
            .collect::<Vec<_>>();
        new_index_sets[axis_idx] = new_axis.clone().unbind();

        match &self.storage {
            ExprArrayStorage::Full(core) => Ok(Self::new(
                new_index_sets,
                core.shape.clone(),
                core.values.clone(),
            )),
            ExprArrayStorage::Compact { storage, shape, .. } => Ok(Self::from_compact(
                storage.clone(),
                new_index_sets,
                shape.clone(),
            )),
            ExprArrayStorage::Sparse { storage, shape, .. } => Ok(Self::from_sparse(
                new_index_sets,
                shape.clone(),
                storage.active_indices.clone(),
                storage.values.clone(),
            )),
        }
    }
}

// Explicit #[pyo3_macros::pymethods] with compact fast paths (replaces impl_array_ops! macro usage).
#[pyo3_macros::pymethods]
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
        if let Some((left_indices, left_values)) = self.sparse_entries() {
            if let Ok(other_array) = other.extract::<PyRef<'_, PyExprArray>>() {
                if other_array.storage.shape() == self.storage.shape() {
                    if let Some((right_indices, right_values)) = other_array.sparse_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            left_values,
                            right_indices,
                            right_values,
                            1.0,
                        ));
                    }
                }
            }
            if let Ok(other_array) = other.extract::<PyRef<'_, PyVariableArray>>() {
                if other_array.get_shape() == self.storage.shape() {
                    if let Some((right_indices, right_values)) = other_array.sparse_expr_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            left_values,
                            right_indices,
                            &right_values,
                            1.0,
                        ));
                    }
                }
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
        if let Some((left_indices, left_values)) = self.sparse_entries() {
            if let Ok(other_array) = other.extract::<PyRef<'_, PyExprArray>>() {
                if other_array.storage.shape() == self.storage.shape() {
                    if let Some((right_indices, right_values)) = other_array.sparse_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            left_values,
                            right_indices,
                            right_values,
                            -1.0,
                        ));
                    }
                }
            }
            if let Ok(other_array) = other.extract::<PyRef<'_, PyVariableArray>>() {
                if other_array.get_shape() == self.storage.shape() {
                    if let Some((right_indices, right_values)) = other_array.sparse_expr_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            left_values,
                            right_indices,
                            &right_values,
                            -1.0,
                        ));
                    }
                }
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
        if let Some((right_indices, right_values)) = self.sparse_entries() {
            if let Ok(other_array) = other.extract::<PyRef<'_, PyExprArray>>() {
                if other_array.storage.shape() == self.storage.shape() {
                    if let Some((left_indices, left_values)) = other_array.sparse_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            left_values,
                            right_indices,
                            right_values,
                            -1.0,
                        ));
                    }
                }
            }
            if let Ok(other_array) = other.extract::<PyRef<'_, PyVariableArray>>() {
                if other_array.get_shape() == self.storage.shape() {
                    if let Some((left_indices, left_values)) = other_array.sparse_expr_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            self.storage.index_sets_ref(),
                            self.storage.shape(),
                            left_indices,
                            &left_values,
                            right_indices,
                            right_values,
                            -1.0,
                        ));
                    }
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
        if let ExprArrayStorage::Sparse {
            storage,
            index_sets,
            shape,
        } = &self.storage
        {
            if let Ok(scalar) = other.extract::<f64>() {
                return Ok(multiply_sparse_expr_with_scalar(
                    index_sets, shape, storage, scalar,
                ));
            }
            if let Some(result) =
                multiply_sparse_expr_with_labeled_operand(index_sets, storage, other.py(), other)?
            {
                return Ok(result);
            }
        }
        let core = self.to_core();
        super::array_mul(&core, other)
    }

    fn __rmul__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        self.__mul__(other)
    }

    fn relabel_axis(
        &self,
        py: Python<'_>,
        old_axis: &Bound<'_, PyIndexSet>,
        new_axis: &Bound<'_, PyIndexSet>,
    ) -> PyResult<PyExprArray> {
        self.relabeled_axis(py, old_axis, new_axis)
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

    fn __ge__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
        self.compare(rhs, ComparisonSense::GreaterEqual)
    }

    fn __le__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
        self.compare(rhs, ComparisonSense::LessEqual)
    }

    fn __eq__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
        self.compare(rhs, ComparisonSense::Equal)
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
        if let ExprArrayStorage::Sparse {
            storage,
            index_sets,
            shape,
        } = &self.storage
        {
            return sum_sparse_expr(index_sets, shape, storage, py, over);
        }
        let core = self.to_core();
        super::array_sum(&core, py, over)
    }
    #[pyo3(signature = (*, over))]
    fn cumsum(&self, py: Python<'_>, over: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let core = self.to_core();
        array_cumsum(&core, py, over)
    }
    #[pyo3(signature = (*, over))]
    fn diff(&self, py: Python<'_>, over: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let ExprArrayStorage::Sparse {
            storage,
            index_sets,
            shape,
        } = &self.storage
        {
            return diff_sparse_expr(
                index_sets,
                shape,
                &storage.active_indices,
                &storage.values,
                py,
                over,
            );
        }
        let core = self.to_core();
        array_diff(&core, py, over)
    }
    #[pyo3(signature = (*, shift, over))]
    fn roll(&self, py: Python<'_>, shift: isize, over: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let ExprArrayStorage::Sparse {
            storage,
            index_sets,
            shape,
        } = &self.storage
        {
            return roll_sparse_expr(
                index_sets,
                shape,
                &storage.active_indices,
                &storage.values,
                py,
                shift,
                over,
            );
        }
        let core = self.to_core();
        array_roll(&core, py, shift, over)
    }

    fn __rshift__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let ExprArrayStorage::Sparse {
            storage,
            index_sets,
            shape,
        } = &self.storage
        {
            return sum_sparse_expr(index_sets, shape, storage, py, Some(rhs));
        }
        let core = self.to_core();
        super::array_reduce(&core, py, rhs)
    }

    fn __matmul__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let ExprArrayStorage::Sparse {
            storage,
            index_sets,
            shape,
        } = &self.storage
        {
            return sum_sparse_expr(index_sets, shape, storage, py, Some(rhs));
        }
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

    fn memory_estimate(&self, py: Python<'_>) -> PyResult<PyObject> {
        let term_counts = self.term_counts();
        let dense_slots = self.storage.count();
        let active_slots = match &self.storage {
            ExprArrayStorage::Sparse { storage, .. } => storage.active_indices.len(),
            ExprArrayStorage::Full(_) | ExprArrayStorage::Compact { .. } => dense_slots,
        };
        let inactive_slots = dense_slots.saturating_sub(active_slots);
        let active_density = if dense_slots == 0 {
            0.0
        } else {
            active_slots as f64 / dense_slots as f64
        };
        let estimate = PyDict::new(py);
        estimate.set_item("storage", self.storage_kind())?;
        estimate.set_item("dense_slots", dense_slots)?;
        estimate.set_item("active_slots", active_slots)?;
        estimate.set_item("inactive_slots", inactive_slots)?;
        estimate.set_item("active_density", active_density)?;
        estimate.set_item("linear_terms", term_counts.linear)?;
        estimate.set_item("quadratic_terms", term_counts.quadratic)?;
        estimate.set_item("cubic_terms", term_counts.cubic)?;
        estimate.set_item("estimated_term_bytes", term_counts.estimated_term_bytes())?;
        estimate.set_item(
            "estimated_dense_linear_term_bytes",
            dense_slots.saturating_mul(std::mem::size_of::<(arco_model::VariableId, f64)>()),
        )?;
        estimate.set_item(
            "estimated_inactive_linear_term_bytes",
            inactive_slots.saturating_mul(std::mem::size_of::<(arco_model::VariableId, f64)>()),
        )?;
        set_solver_matrix_memory_estimate(&estimate, active_slots, term_counts.linear)?;
        Ok(estimate.into_any().unbind())
    }

    fn __len__(&self) -> usize {
        self.storage.count()
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(PyList::new(py, self.get_values())?
            .call_method0("__iter__")?
            .unbind())
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
        let func_name = func.getattr("__name__")?.extract::<String>()?;
        if func_name == "diff" {
            if let ExprArrayStorage::Sparse {
                storage,
                index_sets,
                shape,
            } = &self.storage
            {
                let axis = kwargs.cast::<PyDict>()?.get_item("axis")?.ok_or_else(|| {
                    ArrayDimensionError::new_err("np.diff requires axis=IndexSet")
                })?;
                return diff_sparse_expr(
                    index_sets,
                    shape,
                    &storage.active_indices,
                    &storage.values,
                    py,
                    &axis,
                );
            }
        }
        if func_name == "roll" {
            if let ExprArrayStorage::Sparse {
                storage,
                index_sets,
                shape,
            } = &self.storage
            {
                let shift = if args.len() > 1 {
                    args.get_item(1)?.extract::<isize>()?
                } else {
                    kwargs
                        .cast::<PyDict>()?
                        .get_item("shift")?
                        .ok_or_else(|| ArrayDimensionError::new_err("np.roll requires shift"))?
                        .extract::<isize>()?
                };
                let axis = kwargs.cast::<PyDict>()?.get_item("axis")?.ok_or_else(|| {
                    ArrayDimensionError::new_err("np.roll requires axis=IndexSet")
                })?;
                return roll_sparse_expr(
                    index_sets,
                    shape,
                    &storage.active_indices,
                    &storage.values,
                    py,
                    shift,
                    &axis,
                );
            }
        }
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
            let index_sets = if core.shape.len() == 1 && core.index_sets.len() == 1 {
                sliced_and_index_sets(
                    py,
                    &core.index_sets,
                    &core.shape,
                    &[AxisIndex::Range(mask_indices.clone())],
                )?
            } else {
                Vec::new()
            };
            let result = PyExprArray::new(index_sets, vec![n], filtered_values);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let selected = slice_indices(slice, core.values.len())?;
            let sliced_values = selected
                .iter()
                .map(|&idx| core.values[idx].clone())
                .collect::<Vec<PyExpr>>();
            let n = sliced_values.len();
            let index_sets = if core.shape.len() == 1 && core.index_sets.len() == 1 {
                sliced_and_index_sets(
                    py,
                    &core.index_sets,
                    &core.shape,
                    &[AxisIndex::Range(selected.clone())],
                )?
            } else {
                Vec::new()
            };
            let result = PyExprArray::new(index_sets, vec![n], sliced_values);
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
