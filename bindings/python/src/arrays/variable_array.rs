use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::PyObject;
use crate::py_modules::bounds::BoundsSpec;
use crate::py_modules::errors::{ArrayDimensionError, ArrayIndexError};
use crate::py_modules::expr::PyExpr;
use crate::py_modules::index_set::PyIndexSet;
use crate::py_modules::variable::PyVariable;

use arco_model::VariableId;
use arco_model::expr::Expr;

use super::LinearArrayCore;
use super::indexing::{
    AxisIndex, maybe_boolean_mask_indices, resolve_axis_index, selected_flat_indices,
    slice_indices, sliced_2d_index_sets, sliced_and_index_sets,
};

use super::{
    BroadcastCompareOperand, CompactExprStorage, ComparisonSense, ExpressionTermCounts,
    PyConstraintArray, PyExprArray, SparseCompareOperand, SparseDiffSource, SparseExprNode,
    array_add, array_cumsum, array_diff, array_function, array_mul, array_neg, array_reduce,
    array_roll, array_rsub, array_sub, array_sum, array_truediv, array_ufunc,
    combine_sparse_expr_same_shape, compare_with_compact_fallback, diff_sparse_expr,
    expression_term_counts, multiply_sparse_variables_with_labeled_operand,
    multiply_sparse_variables_with_scalar, parse_sparse_axes, reduced_sparse_flat_index,
    roll_sparse_expr, set_solver_matrix_memory_estimate, try_broadcast_compare,
    try_extract_compact,
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
    variables: Vec<Option<PyVariable>>,
}

enum SparseBounds {
    Uniform(BoundsSpec),
    PerSlot(Vec<BoundsSpec>),
}

impl SparseBounds {
    fn at(&self, active_pos: usize) -> BoundsSpec {
        match self {
            SparseBounds::Uniform(bounds) => *bounds,
            SparseBounds::PerSlot(bounds) => {
                debug_assert!(
                    active_pos < bounds.len(),
                    "active_pos {active_pos} out of range for per-slot bounds (len {})",
                    bounds.len()
                );
                bounds[active_pos]
            }
        }
    }
}

/// Sparse storage for arrays with inactive dense slots.
struct SparseStorage {
    active_indices: Vec<usize>,
    var_ids: Vec<u32>,
    bounds: SparseBounds,
    name: Option<String>,
}

/// Internal storage enum for VariableArray.
enum VariableStorage {
    /// Compact: only metadata, no Vec<PyExpr> or Vec<PyVariable>.
    Compact(CompactStorage),
    /// Sparse: only active variable slots are stored.
    Sparse(SparseStorage),
    /// Full: stores all expressions and variables.
    Full(FullStorage),
}

/// A multi-dimensional array of decision variables.
/// Created by `Model.add_variables(axes=(T, G), bounds=...)`. Any operation produces ExprArray.
#[pyo3_macros::pyclass(name = "VariableArray")]
pub struct PyVariableArray {
    storage: VariableStorage,
    index_sets: Vec<Py<PyIndexSet>>,
    shape: Vec<usize>,
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
                variables: variables.into_iter().map(Some).collect(),
            }),
            index_sets,
            shape,
        })
    }

    pub(crate) fn new_sparse(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        values: Vec<PyExpr>,
        variables: Vec<Option<PyVariable>>,
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

    /// Create a VariableArray with sparse active-slot storage.
    pub fn new_active_sparse(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        active_indices: Vec<usize>,
        var_ids: Vec<u32>,
        bounds: BoundsSpec,
        name: Option<String>,
    ) -> Self {
        debug_assert_eq!(
            active_indices.len(),
            var_ids.len(),
            "active_indices and var_ids must have the same length"
        );
        Self {
            storage: VariableStorage::Sparse(SparseStorage {
                active_indices,
                var_ids,
                bounds: SparseBounds::Uniform(bounds),
                name,
            }),
            index_sets,
            shape,
        }
    }

    pub fn new_active_sparse_with_bounds(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        active_indices: Vec<usize>,
        var_ids: Vec<u32>,
        bounds: Vec<BoundsSpec>,
        name: Option<String>,
    ) -> Self {
        debug_assert_eq!(
            active_indices.len(),
            var_ids.len(),
            "active_indices and var_ids must have the same length"
        );
        debug_assert_eq!(
            active_indices.len(),
            bounds.len(),
            "active_indices and per-slot bounds must have the same length"
        );
        Self {
            storage: VariableStorage::Sparse(SparseStorage {
                active_indices,
                var_ids,
                bounds: SparseBounds::PerSlot(bounds),
                name,
            }),
            index_sets,
            shape,
        }
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
    pub fn to_core(&self) -> LinearArrayCore {
        match &self.storage {
            VariableStorage::Full(full) => full.core.clone_with_gil(),
            VariableStorage::Sparse(sparse) => {
                let mut values = vec![PyExpr::from_expr(Expr::from_constant(0.0)); self.len()];
                for (active_idx, var_id) in sparse.active_indices.iter().zip(sparse.var_ids.iter())
                {
                    values[*active_idx] = PyExpr::from_term(*var_id, 1.0);
                }
                LinearArrayCore::new(self.clone_index_sets(), self.shape.clone(), values)
            }
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
            VariableStorage::Sparse(_) => self.shape.iter().product(),
            VariableStorage::Full(f) => f.core.values.len(),
        }
    }

    fn active_len(&self) -> usize {
        match &self.storage {
            VariableStorage::Compact(c) => c.count,
            VariableStorage::Sparse(sparse) => sparse.active_indices.len(),
            VariableStorage::Full(f) => f
                .variables
                .iter()
                .filter(|variable| variable.is_some())
                .count(),
        }
    }

    fn storage_kind(&self) -> &'static str {
        match &self.storage {
            VariableStorage::Compact(_) => "compact",
            VariableStorage::Sparse(_) => "sparse",
            VariableStorage::Full(_) => "full",
        }
    }

    pub(crate) fn term_counts(&self) -> ExpressionTermCounts {
        match &self.storage {
            VariableStorage::Compact(compact) => {
                CompactExprStorage::from_variable_array(compact.start_var_id, compact.count)
                    .term_counts()
            }
            VariableStorage::Sparse(sparse) => ExpressionTermCounts {
                linear: sparse.active_indices.len(),
                quadratic: 0,
                cubic: 0,
            },
            VariableStorage::Full(full) => expression_term_counts(&full.core.values),
        }
    }

    /// Clone index_sets (requires GIL).
    fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
        Python::attach(|py| self.index_sets.iter().map(|s| s.clone_ref(py)).collect())
    }

    /// Create a 1D subarray from a list of flat indices.
    fn subarray_from_indices(&self, indices: &[usize]) -> PyResult<PyVariableArray> {
        let vals = indices
            .iter()
            .map(|&i| {
                self.expr_at(i)
                    .ok_or_else(|| ArrayIndexError::new_err(format!("flat index {i} out of range")))
            })
            .collect::<PyResult<Vec<_>>>()?;
        let vars = indices.iter().map(|&i| self.variable_at(i)).collect();
        let index_sets = Python::attach(|py| {
            if self.shape.len() == 1 && self.index_sets.len() == 1 {
                sliced_and_index_sets(
                    py,
                    &self.index_sets,
                    &self.shape,
                    &[AxisIndex::Range(indices.to_vec())],
                )
                .unwrap_or_default()
            } else {
                Vec::new()
            }
        });
        Ok(PyVariableArray::new_sparse(
            index_sets,
            vec![indices.len()],
            vals,
            vars,
        ))
    }

    /// Reconstruct a PyVariable for the given flat index.
    fn variable_at(&self, idx: usize) -> Option<PyVariable> {
        match &self.storage {
            VariableStorage::Full(full) => full.variables.get(idx).cloned().flatten(),
            VariableStorage::Sparse(sparse) => {
                sparse
                    .active_indices
                    .binary_search(&idx)
                    .ok()
                    .map(|active_pos| {
                        PyVariable::new(
                            sparse.var_ids[active_pos],
                            sparse.name.as_ref().map(|base| {
                                if self.len() == 1 {
                                    base.clone()
                                } else {
                                    format!("{base}[{idx}]")
                                }
                            }),
                            sparse.bounds.at(active_pos),
                        )
                    })
            }
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
            VariableStorage::Sparse(_) => {
                if idx >= self.len() {
                    return None;
                }
                Some(self.variable_at(idx).map_or_else(
                    || PyExpr::from_expr(Expr::from_constant(0.0)),
                    |v| v.to_expr(),
                ))
            }
            VariableStorage::Compact(compact) => {
                if idx >= compact.count {
                    return None;
                }
                Some(PyExpr::from_term(compact.var_id_at(idx), 1.0))
            }
        }
    }

    pub(crate) fn get_values(&self) -> Vec<PyExpr> {
        match &self.storage {
            VariableStorage::Full(full) => full.core.values.clone(),
            VariableStorage::Sparse(_) => self.to_core().values,
            VariableStorage::Compact(compact) => (0..compact.count)
                .map(|i| PyExpr::from_term(compact.var_id_at(i), 1.0))
                .collect(),
        }
    }

    pub(crate) fn get_variable_refs(&self) -> Vec<PyVariable> {
        match &self.storage {
            VariableStorage::Full(full) => full.variables.iter().flatten().cloned().collect(),
            VariableStorage::Sparse(sparse) => sparse
                .active_indices
                .iter()
                .enumerate()
                .map(|(active_pos, active_idx)| {
                    PyVariable::new(
                        sparse.var_ids[active_pos],
                        sparse.name.as_ref().map(|base| {
                            if self.len() == 1 {
                                base.clone()
                            } else {
                                format!("{base}[{active_idx}]")
                            }
                        }),
                        sparse.bounds.at(active_pos),
                    )
                })
                .collect(),
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

    pub fn get_variable_slots(&self) -> Vec<Option<PyVariable>> {
        match &self.storage {
            VariableStorage::Full(full) => full.variables.clone(),
            VariableStorage::Sparse(sparse) => {
                let mut slots = vec![None; self.len()];
                for (active_pos, active_idx) in sparse.active_indices.iter().enumerate() {
                    slots[*active_idx] = Some(PyVariable::new(
                        sparse.var_ids[active_pos],
                        sparse.name.as_ref().map(|base| {
                            if self.len() == 1 {
                                base.clone()
                            } else {
                                format!("{base}[{active_idx}]")
                            }
                        }),
                        sparse.bounds.at(active_pos),
                    ));
                }
                slots
            }
            VariableStorage::Compact(compact) => (0..compact.count)
                .map(|i| {
                    Some(PyVariable::new(
                        compact.var_id_at(i),
                        compact.var_name_at(i),
                        compact.bounds_spec,
                    ))
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
    pub fn as_compact_expr(&self) -> Option<CompactExprStorage> {
        match &self.storage {
            VariableStorage::Compact(c) => Some(CompactExprStorage::from_variable_array(
                c.start_var_id,
                c.count,
            )),
            VariableStorage::Sparse(_) => None,
            VariableStorage::Full(_) => None,
        }
    }

    pub(crate) fn sparse_expr_entries(&self) -> Option<(&[usize], Vec<PyExpr>)> {
        match &self.storage {
            VariableStorage::Sparse(sparse) => Some((
                sparse.active_indices.as_slice(),
                sparse
                    .var_ids
                    .iter()
                    .map(|var_id| PyExpr::from_term(*var_id, 1.0))
                    .collect::<Vec<_>>(),
            )),
            VariableStorage::Compact(_) | VariableStorage::Full(_) => None,
        }
    }

    pub(crate) fn sparse_var_entries(&self) -> Option<(&[usize], &[u32])> {
        match &self.storage {
            VariableStorage::Sparse(sparse) => {
                Some((sparse.active_indices.as_slice(), sparse.var_ids.as_slice()))
            }
            VariableStorage::Compact(_) | VariableStorage::Full(_) => None,
        }
    }

    pub(crate) fn sparse_expr_node(
        &self,
        handle: Py<PyVariableArray>,
    ) -> Option<std::sync::Arc<SparseExprNode>> {
        match &self.storage {
            VariableStorage::Sparse(sparse) => Some(SparseExprNode::variable(
                handle,
                self.shape.clone(),
                sparse.active_indices.clone(),
            )),
            VariableStorage::Compact(_) | VariableStorage::Full(_) => None,
        }
    }

    pub(crate) fn is_sparse(&self) -> bool {
        matches!(&self.storage, VariableStorage::Sparse(_))
    }

    pub(crate) fn variable_id_at_flat(&self, index: usize) -> Option<u32> {
        match &self.storage {
            VariableStorage::Compact(compact) => {
                (index < compact.count).then(|| compact.var_id_at(index))
            }
            VariableStorage::Sparse(sparse) => sparse
                .active_indices
                .binary_search(&index)
                .ok()
                .map(|position| sparse.var_ids[position]),
            VariableStorage::Full(full) => full
                .variables
                .get(index)
                .and_then(|variable| variable.as_ref().map(|variable| variable.var_id)),
        }
    }

    pub(crate) fn index_sets_ref(&self) -> &[Py<PyIndexSet>] {
        &self.index_sets
    }

    /// Fast-path sum for compact storage: build all linear terms directly
    /// without materializing PyExpr objects.
    fn sum_all_compact(start: u32, count: usize) -> PyExpr {
        let linear = (0..count)
            .map(|i| (VariableId::new(start + i as u32), 1.0))
            .collect();
        PyExpr::from_expr(Expr::from_linear(linear))
    }

    fn sum_all_sparse(sparse: &SparseStorage) -> PyExpr {
        let linear = sparse
            .var_ids
            .iter()
            .map(|var_id| (VariableId::new(*var_id), 1.0))
            .collect();
        PyExpr::from_expr(Expr::from_linear(linear))
    }

    pub(crate) fn sparse_reduction_core_for_axis(
        &self,
        axis: usize,
        output_index_sets: Vec<Py<PyIndexSet>>,
        output_shape: Vec<usize>,
    ) -> LinearArrayCore {
        let source_strides = arco_arrays::row_major_strides(&self.shape);
        let reduced_strides = arco_arrays::row_major_strides(&output_shape);
        let summed_axes = (0..self.shape.len()).map(|current| current == axis);
        let summed_axes = summed_axes.collect::<Vec<_>>();
        let out_len = output_shape.iter().product::<usize>().max(1);
        let mut values = vec![PyExpr::default(); out_len];

        if let VariableStorage::Sparse(sparse) = &self.storage {
            for (active_idx, var_id) in sparse.active_indices.iter().zip(sparse.var_ids.iter()) {
                let reduced_idx = reduced_sparse_flat_index(
                    *active_idx,
                    &self.shape,
                    &source_strides,
                    &reduced_strides,
                    &summed_axes,
                );
                values[reduced_idx].add_assign_owned(PyExpr::from_term(*var_id, 1.0));
            }
        } else {
            // This is an internal representation transition: callers only
            // create the deferred form from sparse storage.  Keep a safe
            // fallback for malformed internal state rather than exposing an
            // empty result or panicking.
            let source = self.to_core();
            for (source_idx, expr) in source.values.into_iter().enumerate() {
                let reduced_idx = reduced_sparse_flat_index(
                    source_idx,
                    &self.shape,
                    &source_strides,
                    &reduced_strides,
                    &summed_axes,
                );
                values[reduced_idx].add_assign_owned(expr);
            }
        }

        LinearArrayCore::new(output_index_sets, output_shape, values)
    }

    fn sum_sparse_over_axis(
        &self,
        py: Python<'_>,
        sparse: &SparseStorage,
        over: &Bound<'_, PyAny>,
        source: Py<PyVariableArray>,
    ) -> PyResult<PyObject> {
        let axes = parse_sparse_axes(&self.index_sets, py, over)?;
        let mut summed_axes = vec![false; self.shape.len()];
        for &axis in &axes {
            summed_axes[axis] = true;
        }

        let mut out_shape = Vec::new();
        let mut out_index_sets = Vec::new();
        for (axis, index_set) in self.index_sets.iter().enumerate() {
            if !summed_axes[axis] {
                out_shape.push(self.shape[axis]);
                out_index_sets.push(index_set.clone_ref(py));
            }
        }

        if axes.len() == 1 && !out_shape.is_empty() {
            let array = PyExprArray::from_deferred_variable_reduction(
                source,
                axes[0],
                out_index_sets,
                out_shape,
            );
            return Ok(array.into_pyobject(py)?.into_any().unbind());
        }

        let source_strides = arco_arrays::row_major_strides(&self.shape);
        let reduced_strides = arco_arrays::row_major_strides(&out_shape);
        let out_len = out_shape.iter().product::<usize>().max(1);
        let mut values = vec![PyExpr::default(); out_len];

        for (active_idx, var_id) in sparse.active_indices.iter().zip(sparse.var_ids.iter()) {
            let reduced_idx = reduced_sparse_flat_index(
                *active_idx,
                &self.shape,
                &source_strides,
                &reduced_strides,
                &summed_axes,
            );
            values[reduced_idx].add_assign_owned(PyExpr::from_term(*var_id, 1.0));
        }

        if out_shape.is_empty() {
            let expr = values.pop().unwrap_or_default();
            Ok(expr.into_pyobject(py)?.into_any().unbind())
        } else {
            let array = PyExprArray::new(out_index_sets, out_shape, values);
            Ok(array.into_pyobject(py)?.into_any().unbind())
        }
    }

    /// Collect linear terms directly for compact storage (for objective extraction).
    pub fn collect_linear_terms_fast(&self) -> Vec<(VariableId, f64)> {
        match &self.storage {
            VariableStorage::Compact(compact) => (0..compact.count)
                .map(|i| (VariableId::new(compact.var_id_at(i)), 1.0))
                .collect(),
            VariableStorage::Sparse(sparse) => sparse
                .var_ids
                .iter()
                .map(|var_id| (VariableId::new(*var_id), 1.0))
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
        left_handle: Py<PyVariableArray>,
        rhs: &Bound<'_, PyAny>,
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintArray> {
        if let VariableStorage::Sparse(_) = &self.storage {
            if let Ok(rhs_array) = rhs.extract::<Py<PyExprArray>>() {
                let rhs_ref = rhs_array.bind(rhs.py()).borrow();
                if rhs_ref.storage.shape() == self.shape {
                    if let Some(right_node) = rhs_ref.sparse_compare_node() {
                        if let Some(left_node) =
                            self.sparse_expr_node(left_handle.clone_ref(rhs.py()))
                        {
                            return Ok(PyConstraintArray::from_sparse_arithmetic_lazy_compare(
                                left_node,
                                right_node,
                                sense,
                                self.shape.clone(),
                                self.clone_index_sets(),
                            ));
                        }
                    }
                }
                if rhs_ref.storage.shape() == self.shape && rhs_ref.sparse_entries().is_some() {
                    return Ok(PyConstraintArray::from_sparse_lazy_compare(
                        SparseCompareOperand::Variable(left_handle),
                        SparseCompareOperand::Expr(rhs_array),
                        sense,
                        self.shape.clone(),
                        self.clone_index_sets(),
                    ));
                }
            }
        }

        if let Ok(rhs_array) = rhs.extract::<Py<PyExprArray>>() {
            let optimized = {
                let rhs_ref = rhs_array.bind(rhs.py()).borrow();
                try_broadcast_compare(
                    BroadcastCompareOperand::Variable(left_handle.clone_ref(rhs.py())),
                    &self.shape,
                    &self.index_sets,
                    BroadcastCompareOperand::Expr(rhs_array.clone_ref(rhs.py())),
                    rhs_ref.storage.shape(),
                    rhs_ref.storage.index_sets_ref(),
                    sense,
                )?
            };
            if let Some(array) = optimized {
                return Ok(array);
            }
        }
        if let Ok(rhs_array) = rhs.extract::<Py<PyVariableArray>>() {
            let optimized = {
                let rhs_ref = rhs_array.bind(rhs.py()).borrow();
                try_broadcast_compare(
                    BroadcastCompareOperand::Variable(left_handle.clone_ref(rhs.py())),
                    &self.shape,
                    &self.index_sets,
                    BroadcastCompareOperand::Variable(rhs_array.clone_ref(rhs.py())),
                    rhs_ref.get_shape(),
                    rhs_ref.index_sets_ref(),
                    sense,
                )?
            };
            if let Some(array) = optimized {
                return Ok(array);
            }
        }

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
        if tuple.len() > self.shape.len() {
            return Err(ArrayDimensionError::new_err(
                "tuple indexing cannot specify more dimensions than the array rank",
            ));
        }
        if self.shape.len() == 2 && tuple.len() == 2 {
            let nrows = self.shape[0];
            let ncols = self.shape[1];
            let idx0 = tuple.get_item(0)?;
            let idx1 = tuple.get_item(1)?;

            let rows = resolve_axis_index(&idx0, nrows)?;
            let cols = resolve_axis_index(&idx1, ncols)?;

            match (&rows, &cols) {
                (AxisIndex::Single(r), AxisIndex::Single(c)) => {
                    let flat_idx = r * ncols + c;
                    if let Some(var) = self.variable_at(flat_idx) {
                        return Ok(var.into_pyobject(py)?.into_any().unbind());
                    }
                    let expr = self.expr_at(flat_idx).ok_or_else(|| {
                        ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
                    })?;
                    return Ok(expr.into_pyobject(py)?.into_any().unbind());
                }
                (AxisIndex::Range(row_indices), AxisIndex::Range(col_indices))
                    if self.index_sets.len() == 2 =>
                {
                    let new_nrows = row_indices.len();
                    let new_ncols = col_indices.len();
                    let mut vals = Vec::with_capacity(new_nrows * new_ncols);
                    let mut vars = Vec::with_capacity(new_nrows * new_ncols);
                    for &r in row_indices {
                        for &c in col_indices {
                            let flat_idx = r * ncols + c;
                            vals.push(self.expr_at(flat_idx).ok_or_else(|| {
                                ArrayIndexError::new_err(format!(
                                    "flat index {} out of range",
                                    flat_idx
                                ))
                            })?);
                            vars.push(self.variable_at(flat_idx));
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
                    let result = PyVariableArray::new_sparse(
                        new_index_sets,
                        vec![new_nrows, new_ncols],
                        vals,
                        vars,
                    );
                    return Ok(result.into_pyobject(py)?.into_any().unbind());
                }
                _ => {}
            }
        }

        let mut selections = Vec::with_capacity(self.shape.len());
        for axis in 0..self.shape.len() {
            if axis < tuple.len() {
                selections.push(resolve_axis_index(
                    &tuple.get_item(axis)?,
                    self.shape[axis],
                )?);
            } else {
                selections.push(AxisIndex::Range((0..self.shape[axis]).collect()));
            }
        }

        let (flat_indices, out_shape) = selected_flat_indices(&self.shape, &selections);
        if out_shape.is_empty() {
            let flat_idx = *flat_indices.first().ok_or_else(|| {
                ArrayIndexError::new_err("scalar tuple indexing did not resolve any element")
            })?;
            if let Some(var) = self.variable_at(flat_idx) {
                return Ok(var.into_pyobject(py)?.into_any().unbind());
            }
            let expr = self.expr_at(flat_idx).ok_or_else(|| {
                ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
            })?;
            return Ok(expr.into_pyobject(py)?.into_any().unbind());
        }

        let vals = flat_indices
            .iter()
            .map(|&idx| {
                self.expr_at(idx).ok_or_else(|| {
                    ArrayIndexError::new_err(format!("flat index {} out of range", idx))
                })
            })
            .collect::<PyResult<Vec<_>>>()?;
        let vars = flat_indices
            .iter()
            .map(|&idx| self.variable_at(idx))
            .collect::<Vec<_>>();
        let new_index_sets = sliced_and_index_sets(py, &self.index_sets, &self.shape, &selections)?;
        let result = PyVariableArray::new_sparse(new_index_sets, out_shape, vals, vars);
        Ok(result.into_pyobject(py)?.into_any().unbind())
    }
}

// Explicit #[pyo3_macros::pymethods] with compact fast paths.
#[pyo3_macros::pymethods]
impl PyVariableArray {
    fn __add__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        let self_ref = slf.borrow();
        if let Some(left_node) = self_ref.sparse_expr_node(slf.clone().unbind()) {
            if let Ok(other_handle) = other.extract::<Py<PyVariableArray>>() {
                let other_ref = other_handle.bind(other.py()).borrow();
                if other_ref.shape == self_ref.shape {
                    if let Some(right_node) =
                        other_ref.sparse_expr_node(other_handle.clone_ref(other.py()))
                    {
                        if let Some(node) =
                            super::SparseExprNode::add(left_node.clone(), right_node, 1.0)
                        {
                            return Ok(PyExprArray::from_sparse_lazy(
                                self_ref.clone_index_sets(),
                                self_ref.shape.clone(),
                                node,
                            ));
                        }
                    }
                }
            }
            if let Ok(other_handle) = other.extract::<Py<PyExprArray>>() {
                let other_ref = other_handle.bind(other.py()).borrow();
                if other_ref.storage.shape() == self_ref.shape {
                    if let Some(right_node) = other_ref.storage.sparse_node() {
                        if let Some(node) = super::SparseExprNode::add(left_node, right_node, 1.0) {
                            return Ok(PyExprArray::from_sparse_lazy(
                                self_ref.clone_index_sets(),
                                self_ref.shape.clone(),
                                node,
                            ));
                        }
                    }
                }
            }
        }
        if let Some(self_compact) = self_ref.as_compact_expr() {
            if let Some(other_compact) = try_extract_compact(other) {
                if self_compact.count == other_compact.count {
                    return Ok(self_ref.wrap_compact_expr(self_compact.add_compact(&other_compact)));
                }
            }
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self_ref.wrap_compact_expr(self_compact.add_constant(value)));
            }
        }
        if let Some((left_indices, left_values)) = self_ref.sparse_expr_entries() {
            if let Ok(other_array) = other.extract::<PyRef<'_, PyVariableArray>>() {
                if other_array.shape == self_ref.shape {
                    if let Some((right_indices, right_values)) = other_array.sparse_expr_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            &self_ref.index_sets,
                            &self_ref.shape,
                            left_indices,
                            &left_values,
                            right_indices,
                            &right_values,
                            1.0,
                        ));
                    }
                }
            }
            if let Ok(other_array) = other.extract::<PyRef<'_, PyExprArray>>() {
                if other_array.storage.shape() == self_ref.shape {
                    if let Some((right_indices, right_values)) = other_array.sparse_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            &self_ref.index_sets,
                            &self_ref.shape,
                            left_indices,
                            &left_values,
                            right_indices,
                            right_values,
                            1.0,
                        ));
                    }
                }
            }
        }
        let core = self_ref.to_core();
        array_add(&core, other)
    }
    fn __radd__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        Self::__add__(slf, other)
    }
    fn __sub__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        let self_ref = slf.borrow();
        if let Some(left_node) = self_ref.sparse_expr_node(slf.clone().unbind()) {
            if let Ok(other_handle) = other.extract::<Py<PyVariableArray>>() {
                let other_ref = other_handle.bind(other.py()).borrow();
                if other_ref.shape == self_ref.shape {
                    if let Some(right_node) =
                        other_ref.sparse_expr_node(other_handle.clone_ref(other.py()))
                    {
                        if let Some(node) =
                            super::SparseExprNode::add(left_node.clone(), right_node, -1.0)
                        {
                            return Ok(PyExprArray::from_sparse_lazy(
                                self_ref.clone_index_sets(),
                                self_ref.shape.clone(),
                                node,
                            ));
                        }
                    }
                }
            }
            if let Ok(other_handle) = other.extract::<Py<PyExprArray>>() {
                let other_ref = other_handle.bind(other.py()).borrow();
                if other_ref.storage.shape() == self_ref.shape {
                    if let Some(right_node) = other_ref.storage.sparse_node() {
                        if let Some(node) = super::SparseExprNode::add(left_node, right_node, -1.0)
                        {
                            return Ok(PyExprArray::from_sparse_lazy(
                                self_ref.clone_index_sets(),
                                self_ref.shape.clone(),
                                node,
                            ));
                        }
                    }
                }
            }
        }
        if let Some(self_compact) = self_ref.as_compact_expr() {
            if let Some(other_compact) = try_extract_compact(other) {
                if self_compact.count == other_compact.count {
                    return Ok(self_ref.wrap_compact_expr(self_compact.sub_compact(&other_compact)));
                }
            }
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self_ref.wrap_compact_expr(self_compact.add_constant(-value)));
            }
        }
        if let Some((left_indices, left_values)) = self_ref.sparse_expr_entries() {
            if let Ok(other_array) = other.extract::<PyRef<'_, PyVariableArray>>() {
                if other_array.shape == self_ref.shape {
                    if let Some((right_indices, right_values)) = other_array.sparse_expr_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            &self_ref.index_sets,
                            &self_ref.shape,
                            left_indices,
                            &left_values,
                            right_indices,
                            &right_values,
                            -1.0,
                        ));
                    }
                }
            }
            if let Ok(other_array) = other.extract::<PyRef<'_, PyExprArray>>() {
                if other_array.storage.shape() == self_ref.shape {
                    if let Some((right_indices, right_values)) = other_array.sparse_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            &self_ref.index_sets,
                            &self_ref.shape,
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
        let core = self_ref.to_core();
        array_sub(&core, other)
    }
    fn __rsub__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        let self_ref = slf.borrow();
        if let Some(self_compact) = self_ref.as_compact_expr() {
            if let Ok(value) = other.extract::<f64>() {
                return Ok(self_ref.wrap_compact_expr(self_compact.scale(-1.0).add_constant(value)));
            }
            if let Some(other_compact) = try_extract_compact(other) {
                if other_compact.count == self_compact.count {
                    return Ok(self_ref.wrap_compact_expr(other_compact.sub_compact(&self_compact)));
                }
            }
        }
        if let Some((right_indices, right_values)) = self_ref.sparse_expr_entries() {
            if let Ok(other_array) = other.extract::<PyRef<'_, PyVariableArray>>() {
                if other_array.shape == self_ref.shape {
                    if let Some((left_indices, left_values)) = other_array.sparse_expr_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            &self_ref.index_sets,
                            &self_ref.shape,
                            left_indices,
                            &left_values,
                            right_indices,
                            &right_values,
                            -1.0,
                        ));
                    }
                }
            }
            if let Ok(other_array) = other.extract::<PyRef<'_, PyExprArray>>() {
                if other_array.storage.shape() == self_ref.shape {
                    if let Some((left_indices, left_values)) = other_array.sparse_entries() {
                        return Ok(combine_sparse_expr_same_shape(
                            &self_ref.index_sets,
                            &self_ref.shape,
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
        let core = self_ref.to_core();
        array_rsub(&core, other)
    }
    fn __mul__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        let self_ref = slf.borrow();
        if let Some(node) = self_ref.sparse_expr_node(slf.clone().unbind()) {
            if let Ok(scalar) = other.extract::<f64>() {
                if let Some(node) = super::SparseExprNode::scale(node, scalar) {
                    return Ok(PyExprArray::from_sparse_lazy(
                        self_ref.clone_index_sets(),
                        self_ref.shape.clone(),
                        node,
                    ));
                }
            }
        }
        if let Some(self_compact) = self_ref.as_compact_expr() {
            if let Ok(scalar) = other.extract::<f64>() {
                return Ok(self_ref.wrap_compact_expr(self_compact.scale(scalar)));
            }
        }
        if let VariableStorage::Sparse(sparse) = &self_ref.storage {
            if let Ok(scalar) = other.extract::<f64>() {
                return Ok(multiply_sparse_variables_with_scalar(
                    &self_ref.index_sets,
                    &self_ref.shape,
                    &sparse.active_indices,
                    &sparse.var_ids,
                    scalar,
                ));
            }
            if let Some(result) = multiply_sparse_variables_with_labeled_operand(
                &self_ref.index_sets,
                &sparse.active_indices,
                &sparse.var_ids,
                other.py(),
                other,
            )? {
                return Ok(result);
            }
        }
        let core = self_ref.to_core();
        array_mul(&core, other)
    }
    fn __rmul__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
        Self::__mul__(slf, other)
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
    fn __ge__(slf: &Bound<'_, Self>, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
        slf.borrow()
            .compare(slf.clone().unbind(), rhs, ComparisonSense::GreaterEqual)
    }
    fn __le__(slf: &Bound<'_, Self>, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
        slf.borrow()
            .compare(slf.clone().unbind(), rhs, ComparisonSense::LessEqual)
    }
    fn __eq__(slf: &Bound<'_, Self>, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
        slf.borrow()
            .compare(slf.clone().unbind(), rhs, ComparisonSense::Equal)
    }
    #[pyo3(signature = (*, over=None))]
    fn sum(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        over: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyObject> {
        let self_ref = slf.borrow();
        // Fast path: sum all elements of compact storage without materializing
        if over.is_none() {
            if let VariableStorage::Compact(compact) = &self_ref.storage {
                let result = Self::sum_all_compact(compact.start_var_id, compact.count);
                return Ok(result.into_pyobject(py)?.into_any().unbind());
            }
            if let VariableStorage::Sparse(sparse) = &self_ref.storage {
                let result = Self::sum_all_sparse(sparse);
                return Ok(result.into_pyobject(py)?.into_any().unbind());
            }
        }
        if let (VariableStorage::Sparse(sparse), Some(over)) = (&self_ref.storage, over) {
            return self_ref.sum_sparse_over_axis(py, sparse, over, slf.clone().unbind());
        }
        let core = self_ref.to_core();
        array_sum(&core, py, over)
    }
    #[pyo3(signature = (*, over))]
    fn cumsum(&self, py: Python<'_>, over: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let core = self.to_core();
        array_cumsum(&core, py, over)
    }
    #[pyo3(signature = (*, over))]
    fn diff(slf: &Bound<'_, Self>, py: Python<'_>, over: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let self_ref = slf.borrow();
        if let VariableStorage::Sparse(sparse) = &self_ref.storage {
            let axes = parse_sparse_axes(&self_ref.index_sets, py, over)?;
            if axes.len() != 1 {
                return Err(ArrayDimensionError::new_err(
                    "np.diff requires exactly one IndexSet axis",
                ));
            }
            if let Some(node) = self_ref
                .sparse_expr_node(slf.clone().unbind())
                .and_then(|node| super::SparseExprNode::diff(node, axes[0]))
            {
                let axis = axes[0];
                let axis_size = self_ref.shape[axis];
                let selected = (1..axis_size).collect::<Vec<_>>();
                let mut out_shape = self_ref.shape.clone();
                out_shape[axis] = axis_size.saturating_sub(1);
                let mut out_index_sets = self_ref.clone_index_sets();
                out_index_sets[axis] =
                    super::slice_index_set(py, &self_ref.index_sets[axis], &selected)?;
                let result = PyExprArray::from_sparse_lazy(out_index_sets, out_shape, node);
                return Ok(result.into_pyobject(py)?.into_any().unbind());
            }
            return diff_sparse_expr(
                &self_ref.index_sets,
                &self_ref.shape,
                &sparse.active_indices,
                SparseDiffSource::VariableIds(&sparse.var_ids),
                py,
                over,
            );
        }
        let core = self_ref.to_core();
        array_diff(&core, py, over)
    }
    #[pyo3(signature = (*, shift, over))]
    fn roll(&self, py: Python<'_>, shift: isize, over: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let VariableStorage::Sparse(sparse) = &self.storage {
            let values = sparse
                .var_ids
                .iter()
                .map(|var_id| PyExpr::from_term(*var_id, 1.0))
                .collect::<Vec<_>>();
            return roll_sparse_expr(
                &self.index_sets,
                &self.shape,
                &sparse.active_indices,
                &values,
                py,
                shift,
                over,
            );
        }
        let core = self.to_core();
        array_roll(&core, py, shift, over)
    }
    fn __rshift__(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        rhs: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let self_ref = slf.borrow();
        if let VariableStorage::Sparse(sparse) = &self_ref.storage {
            return self_ref.sum_sparse_over_axis(py, sparse, rhs, slf.clone().unbind());
        }
        let core = self_ref.to_core();
        array_reduce(&core, py, rhs)
    }
    fn __matmul__(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        rhs: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let self_ref = slf.borrow();
        if let VariableStorage::Sparse(sparse) = &self_ref.storage {
            return self_ref.sum_sparse_over_axis(py, sparse, rhs, slf.clone().unbind());
        }
        let core = self_ref.to_core();
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
    fn dense_count(&self) -> usize {
        self.len()
    }

    #[getter]
    fn active_count(&self) -> usize {
        self.active_len()
    }

    fn memory_estimate(&self, py: Python<'_>) -> PyResult<PyObject> {
        let term_counts = self.term_counts();
        let dense_slots = self.len();
        let active_slots = self.active_len();
        let inactive_slots = dense_slots.saturating_sub(active_slots);
        let active_density = if dense_slots == 0 {
            0.0
        } else {
            active_slots as f64 / dense_slots as f64
        };
        let linear_term_bytes = std::mem::size_of::<(VariableId, f64)>();
        let estimated_dense_linear_term_bytes = dense_slots.saturating_mul(linear_term_bytes);
        let estimated_inactive_linear_term_bytes = inactive_slots.saturating_mul(linear_term_bytes);
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
            estimated_dense_linear_term_bytes,
        )?;
        estimate.set_item(
            "estimated_inactive_linear_term_bytes",
            estimated_inactive_linear_term_bytes,
        )?;
        set_solver_matrix_memory_estimate(&estimate, active_slots, term_counts.linear)?;
        Ok(estimate.into_any().unbind())
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

    fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let mut items = Vec::with_capacity(self.len());
        for idx in 0..self.len() {
            if let Some(variable) = self.variable_at(idx) {
                items.push(variable.into_pyobject(py)?.into_any().unbind());
            } else {
                let expr = self.expr_at(idx).ok_or_else(|| {
                    ArrayIndexError::new_err(format!(
                        "index {} out of range for array of size {}",
                        idx,
                        self.len()
                    ))
                })?;
                items.push(expr.into_pyobject(py)?.into_any().unbind());
            }
        }
        Ok(PyList::new(py, items)?.call_method0("__iter__")?.unbind())
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
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        func: &Bound<'_, PyAny>,
        _types: &Bound<'_, PyAny>,
        args: &Bound<'_, PyTuple>,
        kwargs: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let self_ref = slf.borrow();
        let func_name = func.getattr("__name__")?.extract::<String>()?;
        if func_name == "diff" {
            if let VariableStorage::Sparse(sparse) = &self_ref.storage {
                let axis = kwargs.cast::<PyDict>()?.get_item("axis")?.ok_or_else(|| {
                    ArrayDimensionError::new_err("np.diff requires axis=IndexSet")
                })?;
                let axes = parse_sparse_axes(&self_ref.index_sets, py, &axis)?;
                if axes.len() != 1 {
                    return Err(ArrayDimensionError::new_err(
                        "np.diff requires exactly one IndexSet axis",
                    ));
                }
                if let Some(node) = self_ref
                    .sparse_expr_node(slf.clone().unbind())
                    .and_then(|node| super::SparseExprNode::diff(node, axes[0]))
                {
                    let axis = axes[0];
                    let axis_size = self_ref.shape[axis];
                    let selected = (1..axis_size).collect::<Vec<_>>();
                    let mut out_shape = self_ref.shape.clone();
                    out_shape[axis] = axis_size.saturating_sub(1);
                    let mut out_index_sets = self_ref.clone_index_sets();
                    out_index_sets[axis] =
                        super::slice_index_set(py, &self_ref.index_sets[axis], &selected)?;
                    let result = PyExprArray::from_sparse_lazy(out_index_sets, out_shape, node);
                    return Ok(result.into_pyobject(py)?.into_any().unbind());
                }
                return diff_sparse_expr(
                    &self_ref.index_sets,
                    &self_ref.shape,
                    &sparse.active_indices,
                    SparseDiffSource::VariableIds(&sparse.var_ids),
                    py,
                    &axis,
                );
            }
        }
        if func_name == "roll" {
            if let VariableStorage::Sparse(sparse) = &self_ref.storage {
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
                let axes = parse_sparse_axes(&self_ref.index_sets, py, &axis)?;
                if axes.len() != 1 {
                    return Err(ArrayDimensionError::new_err(
                        "np.roll requires exactly one IndexSet axis",
                    ));
                }
                if let Some(node) = self_ref.sparse_expr_node(slf.clone().unbind()) {
                    if let Some(node) = super::SparseExprNode::roll(node, axes[0], shift) {
                        let result = PyExprArray::from_sparse_lazy(
                            self_ref.clone_index_sets(),
                            self_ref.shape.clone(),
                            node,
                        );
                        return Ok(result.into_pyobject(py)?.into_any().unbind());
                    }
                }
                let values = sparse
                    .var_ids
                    .iter()
                    .map(|var_id| PyExpr::from_term(*var_id, 1.0))
                    .collect::<Vec<_>>();
                return roll_sparse_expr(
                    &self_ref.index_sets,
                    &self_ref.shape,
                    &sparse.active_indices,
                    &values,
                    py,
                    shift,
                    &axis,
                );
            }
        }
        let core = self_ref.to_core();
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
            if idx >= self.len() {
                return Err(ArrayIndexError::new_err(format!(
                    "index {} out of range for array of size {}",
                    idx,
                    self.len()
                )));
            }
            if let Some(variable) = self.variable_at(idx) {
                return Ok(variable.into_pyobject(py)?.into_any().unbind());
            }
            let expr = self.expr_at(idx).ok_or_else(|| {
                ArrayIndexError::new_err(format!(
                    "index {} out of range for array of size {}",
                    idx,
                    self.len()
                ))
            })?;
            return Ok(expr.into_pyobject(py)?.into_any().unbind());
        }

        if let Some(mask_indices) = maybe_boolean_mask_indices(py, index, self.len())? {
            let result = self.subarray_from_indices(&mask_indices)?;
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let selected = slice_indices(slice, self.len())?;
            let result = self.subarray_from_indices(&selected)?;
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

#[cfg(test)]
mod tests {
    use crate::py_modules::arrays::{PyConstraintArray, PyExprArray, PyVariableArray};
    use crate::py_modules::bounds::BoundsSpec;
    use crate::py_modules::expr::PyExpr;
    use crate::py_modules::index_set::{IndexMember, PyIndexSet};
    use arco_model::Bounds;
    use pyo3::prelude::*;

    #[test]
    fn sparse_python_comparison_keeps_constraint_terms_lazy() -> PyResult<()> {
        Python::initialize();
        Python::attach(|py| {
            let axis = Py::new(
                py,
                PyIndexSet {
                    name: "axis".to_string(),
                    members: (0..4).map(IndexMember::Int).collect(),
                },
            )?;
            let bounds = BoundsSpec {
                bounds: Bounds::new(0.0, f64::INFINITY),
                is_integer: false,
                is_binary: false,
            };
            let left = Py::new(
                py,
                PyVariableArray::new_active_sparse(
                    vec![axis.clone_ref(py)],
                    vec![4],
                    vec![0, 2],
                    vec![0, 1],
                    bounds,
                    None,
                ),
            )?;
            let right = Py::new(
                py,
                PyExprArray::from_sparse(
                    vec![axis.clone_ref(py)],
                    vec![4],
                    vec![0, 2],
                    vec![PyExpr::from_term(10, 1.0), PyExpr::from_term(11, 1.0)],
                ),
            )?;

            let comparison = left
                .bind(py)
                .call_method1("__ge__", (right.bind(py),))?
                .extract::<PyRef<'_, PyConstraintArray>>()?;
            assert!(comparison.exprs().is_empty());
            Ok(())
        })
    }
}
