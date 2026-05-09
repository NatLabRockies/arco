use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyObject;
use crate::py_modules::bounds::BoundsSpec;
use crate::py_modules::errors::{ArrayDimensionError, ArrayIndexError};
use crate::py_modules::expr::PyExpr;
use crate::py_modules::index_set::PyIndexSet;
use crate::py_modules::variable::PyVariable;

use arco_ops::expression::{Expr, VariableId};

use super::LinearArrayCore;
use super::indexing::{
    AxisIndex, maybe_boolean_mask_indices, resolve_axis_index, slice_indices, sliced_2d_index_sets,
};
use super::{
    CompactExprStorage, ComparisonSense, PyConstraintArray, PyExprArray, array_add, array_function,
    array_mul, array_neg, array_reduce, array_rsub, array_sub, array_sum, array_truediv,
    array_ufunc, compare_with_compact_fallback, try_extract_compact,
};

/// Compact metadata for a contiguous block of variables with scalar bounds.
struct CompactStorage {
    start_var_id: u32,
    count: usize,
    bounds_spec: BoundsSpec,
    name: Option<String>,
}

impl CompactStorage {
    #[inline]
    fn var_id_at(&self, idx: usize) -> u32 {
        self.start_var_id + idx as u32
    }

    fn var_name_at(&self, idx: usize) -> Option<String> {
        self.name.as_ref().map(|base| {
            if self.count == 1 {
                base.clone()
            } else {
                format!("{base}[{idx}]")
            }
        })
    }
}

/// Full storage when we have per-element data (e.g. array bounds or sliced arrays).
struct FullStorage {
    core: LinearArrayCore,
    variables: Vec<PyVariable>,
}

/// Internal storage enum for VariableArray.
enum VariableStorage {
    /// Compact: only metadata, no Vec<PyExpr> or Vec<PyVariable>.
    Compact(CompactStorage),
    /// Full: stores all expressions and variables.
    Full(FullStorage),
}

/// A multi-dimensional array of decision variables.
/// Created by `Model.add_variables(T, G, bounds=...)`. Any operation on it produces ExprArray.
#[pyclass(name = "VariableArray")]
pub struct PyVariableArray {
    storage: VariableStorage,
    pub(crate) index_sets: Vec<Py<PyIndexSet>>,
    pub(crate) shape: Vec<usize>,
}

impl PyVariableArray {
    /// Create a VariableArray with full storage (per-element bounds or sliced arrays).
    pub fn new(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        values: Vec<PyExpr>,
        variables: Vec<PyVariable>,
    ) -> Self {
        Python::attach(|py| Self {
            storage: VariableStorage::Full(FullStorage {
                core: LinearArrayCore::new(
                    index_sets.iter().map(|s| s.clone_ref(py)).collect(),
                    shape.clone(),
                    values,
                ),
                variables,
            }),
            index_sets,
            shape,
        })
    }

    /// Create a VariableArray with compact storage (scalar bounds, contiguous var IDs).
    pub fn new_compact(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        start_var_id: u32,
        count: usize,
        bounds_spec: BoundsSpec,
        name: Option<String>,
    ) -> Self {
        Self {
            storage: VariableStorage::Compact(CompactStorage {
                start_var_id,
                count,
                bounds_spec,
                name,
            }),
            index_sets,
            shape,
        }
    }

    /// Materialize the full LinearArrayCore on demand.
    pub(crate) fn to_core(&self) -> LinearArrayCore {
        match &self.storage {
            VariableStorage::Full(full) => full.core.clone_with_gil(),
            VariableStorage::Compact(compact) => {
                let values = (0..compact.count)
                    .map(|i| PyExpr::from_term(compact.var_id_at(i), 1.0))
                    .collect();
                LinearArrayCore::new(self.clone_index_sets(), self.shape.clone(), values)
            }
        }
    }

    /// Get the total number of elements.
    fn len(&self) -> usize {
        match &self.storage {
            VariableStorage::Compact(c) => c.count,
            VariableStorage::Full(f) => f.core.values.len(),
        }
    }

    /// Clone index_sets (requires GIL).
    fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
        Python::attach(|py| self.index_sets.iter().map(|s| s.clone_ref(py)).collect())
    }

    /// Create a 1D subarray from a list of flat indices.
    fn subarray_from_indices(&self, indices: &[usize]) -> PyVariableArray {
        let vals = indices.iter().map(|&i| self.expr_at(i).unwrap()).collect();
        let vars = indices
            .iter()
            .map(|&i| self.variable_at(i).unwrap())
            .collect();
        PyVariableArray::new(Vec::new(), vec![indices.len()], vals, vars)
    }

    /// Reconstruct a PyVariable for the given flat index.
    fn variable_at(&self, idx: usize) -> Option<PyVariable> {
        match &self.storage {
            VariableStorage::Full(full) => full.variables.get(idx).cloned(),
            VariableStorage::Compact(compact) => {
                if idx >= compact.count {
                    return None;
                }
                Some(PyVariable::new(
                    compact.var_id_at(idx),
                    compact.var_name_at(idx),
                    compact.bounds_spec,
                ))
            }
        }
    }

    /// Reconstruct a PyExpr for the given flat index.
    fn expr_at(&self, idx: usize) -> Option<PyExpr> {
        match &self.storage {
            VariableStorage::Full(full) => full.core.values.get(idx).cloned(),
            VariableStorage::Compact(compact) => {
                if idx >= compact.count {
                    return None;
                }
                Some(PyExpr::from_term(compact.var_id_at(idx), 1.0))
            }
        }
    }

    pub fn get_values(&self) -> Vec<PyExpr> {
        match &self.storage {
            VariableStorage::Full(full) => full.core.values.clone(),
            VariableStorage::Compact(compact) => (0..compact.count)
                .map(|i| PyExpr::from_term(compact.var_id_at(i), 1.0))
                .collect(),
        }
    }

    pub fn get_variable_refs(&self) -> Vec<PyVariable> {
        match &self.storage {
            VariableStorage::Full(full) => full.variables.clone(),
            VariableStorage::Compact(compact) => (0..compact.count)
                .map(|i| {
                    PyVariable::new(
                        compact.var_id_at(i),
                        compact.var_name_at(i),
                        compact.bounds_spec,
                    )
                })
                .collect(),
        }
    }

    pub fn get_shape(&self) -> &[usize] {
        &self.shape
    }

    /// Wrap a compact expression result using this array's index sets and shape.
    fn wrap_compact_expr(&self, compact: CompactExprStorage) -> PyExprArray {
        PyExprArray::from_compact(compact, self.clone_index_sets(), self.shape.clone())
    }

    /// Return compact expression storage if this array uses compact storage.
    pub(crate) fn as_compact_expr(&self) -> Option<CompactExprStorage> {
        match &self.storage {
            VariableStorage::Compact(c) => Some(CompactExprStorage::from_variable_array(
                c.start_var_id,
                c.count,
            )),
            VariableStorage::Full(_) => None,
        }
    }

    /// Fast-path sum for compact storage: build all linear terms directly
    /// without materializing PyExpr objects.
    fn sum_all_compact(start: u32, count: usize) -> PyExpr {
        let linear = (0..count)
            .map(|i| (VariableId::new(start + i as u32), 1.0))
            .collect();
        PyExpr::from_expr(Expr::from_linear(linear))
    }

    /// Collect linear terms directly for compact storage (for objective extraction).
    pub fn collect_linear_terms_fast(&self) -> Vec<(VariableId, f64)> {
        match &self.storage {
            VariableStorage::Compact(compact) => (0..compact.count)
                .map(|i| (VariableId::new(compact.var_id_at(i)), 1.0))
                .collect(),
            VariableStorage::Full(full) => {
                let total: usize = full
                    .core
                    .values
                    .iter()
                    .map(|e| e.inner().linear_terms().len())
                    .sum();
                let mut terms = Vec::with_capacity(total);
                for expr in &full.core.values {
                    terms.extend_from_slice(expr.inner().linear_terms());
                }
                terms
            }
        }
    }

    /// Shared comparison logic for __ge__, __le__, __eq__.
    fn compare(
        &self,
        rhs: &Bound<'_, PyAny>,
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintArray> {
        let compact = self.as_compact_expr();
        compare_with_compact_fallback(
            compact.as_ref(),
            &self.shape,
            &self.index_sets,
            || self.to_core(),
            rhs,
            sense,
        )
    }

    fn getitem_tuple(&self, py: Python<'_>, tuple: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
        if self.shape.len() != 2 || tuple.len() != 2 {
            return Err(ArrayDimensionError::new_err(
                "tuple indexing requires a 2D array and exactly 2 indices",
            ));
        }
        let nrows = self.shape[0];
        let ncols = self.shape[1];
        let idx0 = tuple.get_item(0)?;
        let idx1 = tuple.get_item(1)?;

        let rows = resolve_axis_index(&idx0, nrows)?;
        let cols = resolve_axis_index(&idx1, ncols)?;

        match (&rows, &cols) {
            (AxisIndex::Single(r), AxisIndex::Single(c)) => {
                let flat_idx = r * ncols + c;
                let var = self.variable_at(flat_idx).ok_or_else(|| {
                    ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
                })?;
                Ok(var.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Single(r), AxisIndex::Range(col_indices)) => {
                let mut vals = Vec::with_capacity(col_indices.len());
                let mut vars = Vec::with_capacity(col_indices.len());
                for &c in col_indices {
                    let flat_idx = r * ncols + c;
                    vals.push(self.expr_at(flat_idx).unwrap());
                    vars.push(self.variable_at(flat_idx).unwrap());
                }
                let n = vals.len();
                let new_index_sets = if col_indices.len() == ncols && self.index_sets.len() == 2 {
                    vec![self.index_sets[1].clone_ref(py)]
                } else {
                    Vec::new()
                };
                let result = PyVariableArray::new(new_index_sets, vec![n], vals, vars);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Range(row_indices), AxisIndex::Single(c)) => {
                let mut vals = Vec::with_capacity(row_indices.len());
                let mut vars = Vec::with_capacity(row_indices.len());
                for &r in row_indices {
                    let flat_idx = r * ncols + c;
                    vals.push(self.expr_at(flat_idx).unwrap());
                    vars.push(self.variable_at(flat_idx).unwrap());
                }
                let n = vals.len();
                let new_index_sets = if row_indices.len() == nrows && self.index_sets.len() == 2 {
                    vec![self.index_sets[0].clone_ref(py)]
                } else {
                    Vec::new()
                };
                let result = PyVariableArray::new(new_index_sets, vec![n], vals, vars);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Range(row_indices), AxisIndex::Range(col_indices)) => {
                let new_nrows = row_indices.len();
                let new_ncols = col_indices.len();
                let mut vals = Vec::with_capacity(new_nrows * new_ncols);
                let mut vars = Vec::with_capacity(new_nrows * new_ncols);
                for &r in row_indices {
                    for &c in col_indices {
                        let flat_idx = r * ncols + c;
                        vals.push(self.expr_at(flat_idx).unwrap());
                        vars.push(self.variable_at(flat_idx).unwrap());
                    }
                }
                let new_index_sets = sliced_2d_index_sets(
                    py,
                    &self.index_sets,
                    nrows,
                    ncols,
                    row_indices,
                    col_indices,
                )?;
                let result =
                    PyVariableArray::new(new_index_sets, vec![new_nrows, new_ncols], vals, vars);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
        }
    }
}

// Explicit #[pymethods] with compact fast paths.
#[pymethods]
impl PyVariableArray {
    fn __add__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact_expr() {
            if let Some(other_compact) = try_extract_compact(other) {
                if self_compact.count == other_compact.count {
                    return Ok(self.wrap_compact_expr(self_compact.add_compact(&other_compact)));
                }
            }
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self.wrap_compact_expr(self_compact.add_constant(value)));
            }
        }
        let core = self.to_core();
        array_add(&core, other)
    }
    fn __radd__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        self.__add__(other)
    }
    fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact_expr() {
            if let Some(other_compact) = try_extract_compact(other) {
                if self_compact.count == other_compact.count {
                    return Ok(self.wrap_compact_expr(self_compact.sub_compact(&other_compact)));
                }
            }
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self.wrap_compact_expr(self_compact.add_constant(-value)));
            }
        }
        let core = self.to_core();
        array_sub(&core, other)
    }
    fn __rsub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact_expr() {
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self.wrap_compact_expr(self_compact.scale(-1.0).add_constant(value)));
            }
            if let Some(other_compact) = try_extract_compact(other) {
                if other_compact.count == self_compact.count {
                    return Ok(self.wrap_compact_expr(other_compact.sub_compact(&self_compact)));
                }
            }
        }
        let core = self.to_core();
        array_rsub(&core, other)
    }
    fn __mul__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        if let Some(self_compact) = self.as_compact_expr() {
            if let Ok(scalar) = other.extract::<f64>() {
                return Ok(self.wrap_compact_expr(self_compact.scale(scalar)));
            }
        }
        let core = self.to_core();
        array_mul(&core, other)
    }
    fn __rmul__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        self.__mul__(other)
    }
    fn __truediv__(&self, other: f64) -> PyResult<PyExprArray> {
        if other == 0.0 {
            return Err(crate::py_modules::errors::ExprDivisionByZeroError::new_err(
                "division by zero",
            ));
        }
        if let Some(self_compact) = self.as_compact_expr() {
            return Ok(self.wrap_compact_expr(self_compact.scale(1.0 / other)));
        }
        let core = self.to_core();
        array_truediv(&core, other)
    }
    fn __neg__(&self) -> PyExprArray {
        if let Some(self_compact) = self.as_compact_expr() {
            return self.wrap_compact_expr(self_compact.scale(-1.0));
        }
        let core = self.to_core();
        array_neg(&core)
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
        // Fast path: sum all elements of compact storage without materializing
        if over.is_none() {
            if let VariableStorage::Compact(compact) = &self.storage {
                let result = Self::sum_all_compact(compact.start_var_id, compact.count);
                return Ok(result.into_pyobject(py)?.into_any().unbind());
            }
        }
        let core = self.to_core();
        array_sum(&core, py, over)
    }
    fn __rshift__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let core = self.to_core();
        array_reduce(&core, py, rhs)
    }
    fn __matmul__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let core = self.to_core();
        array_reduce(&core, py, rhs)
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
    #[getter]
    fn shape(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(PyTuple::new(py, self.shape.clone())?.into())
    }
    #[getter]
    fn values(&self) -> Vec<PyExpr> {
        self.get_values()
    }
    fn flatten(&self) -> Vec<PyExpr> {
        self.get_values()
    }
    fn __len__(&self) -> usize {
        self.len()
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
        array_ufunc(
            &core,
            py,
            |ob| ob.is_instance_of::<PyVariableArray>(),
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
        array_function(&core, py, func, _types, args, kwargs)
    }

    #[getter]
    fn variables(&self) -> Vec<PyVariable> {
        self.get_variable_refs()
    }

    fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(tuple) = index.cast::<PyTuple>() {
            return self.getitem_tuple(py, tuple);
        }

        if let Ok(idx) = index.extract::<usize>() {
            return self
                .variable_at(idx)
                .ok_or_else(|| {
                    ArrayIndexError::new_err(format!(
                        "index {} out of range for array of size {}",
                        idx,
                        self.len()
                    ))
                })
                .and_then(|v| Ok(v.into_pyobject(py)?.into_any().unbind()));
        }

        if let Some(mask_indices) = maybe_boolean_mask_indices(py, index, self.len())? {
            let result = self.subarray_from_indices(&mask_indices);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let selected = slice_indices(slice, self.len())?;
            let result = self.subarray_from_indices(&selected);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        Err(ArrayIndexError::new_err(
            "index must be an integer, tuple, slice, or a boolean numpy array",
        ))
    }

    fn __repr__(&self) -> String {
        format!("VariableArray(shape={:?})", self.shape)
    }
}
