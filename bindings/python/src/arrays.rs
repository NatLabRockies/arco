//! Python wrappers for variable, expression, and constraint arrays.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use arco_arrays::{AxisSpec, BroadcastPlan, LabeledShape};
use arco_model::VariableId;
use arco_model::expr::{ComparisonSense, Expr};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyTuple;

use crate::PyObject;
use crate::py_modules::errors::{
    ArrayDimensionError, ArrayIndexError, ArrayShapeMismatchError, ArrayTypeError,
    ExprDivisionByZeroError,
};
use crate::py_modules::expr::PyExpr;
use crate::py_modules::index_set::{IndexMember, PyIndexSet};

#[path = "arrays/constraint_array.rs"]
mod constraint_array;
#[path = "arrays/expr_array.rs"]
mod expr_array;
#[path = "arrays/indexing.rs"]
mod indexing;
#[path = "arrays/variable_array.rs"]
mod variable_array;

pub use constraint_array::PyConstraintArray;
pub use expr_array::PyExprArray;
pub use variable_array::PyVariableArray;

/// A sparse expression computation retained until a consumer asks for a row.
///
/// The node owns only the sparse index stream and its source handles.  This
/// keeps arithmetic on sparse arrays from allocating one `PyExpr` per active
/// row before a model insertion (or another operation that needs materialized
/// expressions).
pub struct SparseExprNode {
    shape: Arc<[usize]>,
    active_indices: Arc<[usize]>,
    depth: u8,
    kind: SparseExprNodeKind,
}

const MAX_SPARSE_EXPR_DEPTH: u8 = 32;

enum SparseExprNodeKind {
    Variable(Py<PyVariableArray>),
    Values {
        active_indices: Arc<[usize]>,
        values: Arc<[PyExpr]>,
    },
    Roll {
        source: Arc<SparseExprNode>,
        axis: usize,
        shift: isize,
        strides: Arc<[usize]>,
    },
    Diff {
        source: Arc<SparseExprNode>,
        source_stride: usize,
        source_block: usize,
        output_block: usize,
    },
    Scale {
        source: Arc<SparseExprNode>,
        factor: f64,
    },
    Add {
        left: Arc<SparseExprNode>,
        right: Arc<SparseExprNode>,
        right_scale: f64,
    },
}

impl SparseExprNode {
    pub(crate) fn variable(
        handle: Py<PyVariableArray>,
        shape: Vec<usize>,
        active_indices: Vec<usize>,
    ) -> Arc<Self> {
        Arc::new(Self {
            shape: shape.into(),
            active_indices: active_indices.into(),
            depth: 1,
            kind: SparseExprNodeKind::Variable(handle),
        })
    }

    pub(crate) fn values(
        shape: Vec<usize>,
        active_indices: Vec<usize>,
        values: Vec<PyExpr>,
    ) -> Option<Arc<Self>> {
        if active_indices.len() != values.len()
            || active_indices
                .windows(2)
                .any(|window| window[0] >= window[1])
        {
            return None;
        }
        let active_indices: Arc<[usize]> = active_indices.into();
        let values: Arc<[PyExpr]> = values.into();
        Some(Arc::new(Self {
            shape: shape.into(),
            active_indices: active_indices.clone(),
            depth: 1,
            kind: SparseExprNodeKind::Values {
                active_indices,
                values,
            },
        }))
    }

    pub(crate) fn roll(source: Arc<Self>, axis: usize, shift: isize) -> Option<Arc<Self>> {
        if source.depth >= MAX_SPARSE_EXPR_DEPTH {
            return None;
        }
        let shape = source.shape.clone();
        let strides = arco_arrays::row_major_strides(&shape);
        let axis_size = shape[axis];
        let shift = if axis_size == 0 {
            0
        } else {
            shift.rem_euclid(axis_size as isize)
        };
        let mut active_indices = source
            .active_indices
            .iter()
            .copied()
            .map(|index| sparse_rolled_flat_index(index, &shape, &strides, axis, shift))
            .collect::<Vec<_>>();
        active_indices.sort_unstable();
        Some(Arc::new(Self {
            shape,
            active_indices: active_indices.into(),
            depth: source.depth + 1,
            kind: SparseExprNodeKind::Roll {
                source,
                axis,
                shift,
                strides: strides.into(),
            },
        }))
    }

    pub(crate) fn scale(source: Arc<Self>, factor: f64) -> Option<Arc<Self>> {
        if source.depth >= MAX_SPARSE_EXPR_DEPTH {
            return None;
        }
        let shape = source.shape.clone();
        let active_indices = source.active_indices.clone();
        Some(Arc::new(Self {
            shape,
            active_indices,
            depth: source.depth + 1,
            kind: SparseExprNodeKind::Scale { source, factor },
        }))
    }

    pub(crate) fn diff(source: Arc<Self>, axis: usize) -> Option<Arc<Self>> {
        if source.depth >= MAX_SPARSE_EXPR_DEPTH {
            return None;
        }
        let source_shape = &source.shape;
        let axis_size = source_shape[axis];
        let source_stride = arco_arrays::row_major_strides(source_shape)[axis];
        let source_block = source_stride.checked_mul(axis_size)?;
        let output_block = source_stride.checked_mul(axis_size.saturating_sub(1))?;
        let mut shape = source_shape.to_vec();
        shape[axis] = axis_size.saturating_sub(1);
        let mut active_indices = Vec::new();
        for &index in source.active_indices() {
            let outer = index / source_block;
            let output_base = outer * output_block;
            let coordinate = (index / source_stride) % axis_size;
            if coordinate < axis_size.saturating_sub(1) {
                active_indices.push(output_base + index % source_block);
            }
            if coordinate > 0 {
                active_indices.push(output_base + index % source_block - source_stride);
            }
        }
        active_indices.sort_unstable();
        active_indices.dedup();
        Some(Arc::new(Self {
            shape: shape.into(),
            active_indices: active_indices.into(),
            depth: source.depth + 1,
            kind: SparseExprNodeKind::Diff {
                source,
                source_stride,
                source_block,
                output_block,
            },
        }))
    }

    pub(crate) fn add(left: Arc<Self>, right: Arc<Self>, right_scale: f64) -> Option<Arc<Self>> {
        let depth = left.depth.max(right.depth);
        if depth >= MAX_SPARSE_EXPR_DEPTH {
            return None;
        }
        let shape = left.shape.clone();
        let active_indices = merge_sorted_indices(&left.active_indices, &right.active_indices);
        Some(Arc::new(Self {
            shape,
            active_indices: active_indices.into(),
            depth: depth + 1,
            kind: SparseExprNodeKind::Add {
                left,
                right,
                right_scale,
            },
        }))
    }

    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    pub fn active_indices(&self) -> &[usize] {
        &self.active_indices
    }

    pub fn value_at(&self, py: Python<'_>, index: usize) -> Option<PyExpr> {
        match &self.kind {
            SparseExprNodeKind::Variable(handle) => handle
                .bind(py)
                .borrow()
                .variable_id_at_flat(index)
                .map(|var_id| PyExpr::from_term(var_id, 1.0)),
            SparseExprNodeKind::Values {
                active_indices,
                values,
            } => active_indices
                .binary_search(&index)
                .ok()
                .map(|position| values[position].clone()),
            SparseExprNodeKind::Roll {
                source,
                axis,
                shift,
                strides,
            } => {
                let source_index =
                    sparse_rolled_flat_index(index, &self.shape, strides, *axis, -*shift);
                source.value_at(py, source_index)
            }
            SparseExprNodeKind::Scale { source, factor } => {
                source.value_at(py, index).map(|expr| expr.scale(*factor))
            }
            SparseExprNodeKind::Diff {
                source,
                source_stride,
                source_block,
                output_block,
            } => {
                if *output_block == 0 {
                    return None;
                }
                let outer = index / output_block;
                let source_index = outer * source_block + index % output_block;
                let negative = source.value_at(py, source_index);
                let positive = source.value_at(py, source_index + source_stride);
                if negative.is_none() && positive.is_none() {
                    return None;
                }
                let mut value = PyExpr::default();
                if let Some(positive) = positive {
                    value.add_assign(&positive);
                }
                if let Some(negative) = negative {
                    value.add_assign_owned(negative.scale(-1.0));
                }
                (value.inner().num_terms() != 0 || value.constant() != 0.0).then_some(value)
            }
            SparseExprNodeKind::Add {
                left,
                right,
                right_scale,
            } => {
                let left_value = left.value_at(py, index);
                let right_value = right.value_at(py, index);
                if left_value.is_none() && right_value.is_none() {
                    return None;
                }
                let mut value = PyExpr::default();
                if let Some(left_value) = left_value {
                    value.add_assign(&left_value);
                }
                if let Some(right_value) = right_value {
                    value.add_assign_owned(right_value.scale(*right_scale));
                }
                (value.inner().num_terms() != 0 || value.constant() != 0.0).then_some(value)
            }
        }
    }
}

pub fn merge_sorted_indices(left: &[usize], right: &[usize]) -> Vec<usize> {
    let mut merged = Vec::with_capacity(left.len() + right.len());
    let (mut left_pos, mut right_pos) = (0, 0);
    while left_pos < left.len() || right_pos < right.len() {
        match (left.get(left_pos), right.get(right_pos)) {
            (Some(&left_index), Some(&right_index)) => {
                if left_index <= right_index {
                    merged.push(left_index);
                    left_pos += 1;
                    if left_index == right_index {
                        right_pos += 1;
                    }
                } else {
                    merged.push(right_index);
                    right_pos += 1;
                }
            }
            (Some(&left_index), None) => {
                merged.push(left_index);
                left_pos += 1;
            }
            (None, Some(&right_index)) => {
                merged.push(right_index);
                right_pos += 1;
            }
            (None, None) => break,
        }
    }
    merged
}

// Re-export compact types for use in lib.rs
pub use constraint_array::{
    BroadcastCompareOperand, BroadcastCompareValue, SparseCompareMerge, SparseCompareOperand,
    SparseCompareValue,
};
pub use constraint_array::{CompactConstraintStorage, CompactRhs};

type LabeledOperand = (Vec<Py<PyIndexSet>>, Vec<f64>);

#[derive(Clone, Copy, Debug, Default)]
pub struct ExpressionTermCounts {
    pub(crate) linear: usize,
    pub(crate) quadratic: usize,
    pub(crate) cubic: usize,
}

impl ExpressionTermCounts {
    pub(crate) fn estimated_term_bytes(self) -> usize {
        self.linear * std::mem::size_of::<(VariableId, f64)>()
            + self.quadratic * std::mem::size_of::<(VariableId, VariableId, f64)>()
            + self.cubic * std::mem::size_of::<(VariableId, VariableId, VariableId, f64)>()
    }
}

pub(crate) fn expression_term_counts(values: &[PyExpr]) -> ExpressionTermCounts {
    values
        .iter()
        .fold(ExpressionTermCounts::default(), |mut counts, expr| {
            counts.linear += expr.inner().linear_terms().len();
            counts.quadratic += expr.inner().quadratic_terms().len();
            counts.cubic += expr.inner().cubic_terms().len();
            counts
        })
}

pub(crate) fn set_solver_matrix_memory_estimate(
    estimate: &Bound<'_, PyDict>,
    variable_instances: usize,
    coefficient_instances: usize,
) -> PyResult<()> {
    let memory = arco_model::SnapshotMemoryEstimate::for_sparse_matrix(
        variable_instances,
        coefficient_instances,
    );
    estimate.set_item(
        "estimated_solver_coefficient_value_bytes",
        memory.coefficient_value_bytes,
    )?;
    estimate.set_item(
        "estimated_solver_coefficient_index_bytes",
        memory.coefficient_index_bytes,
    )?;
    estimate.set_item(
        "estimated_solver_variable_column_pointer_bytes",
        memory.variable_column_pointer_bytes,
    )?;
    estimate.set_item(
        "estimated_solver_sparse_matrix_bytes",
        memory.sparse_matrix_bytes,
    )?;
    Ok(())
}

/// Sum values along a specific axis in a flat row-major array.
///
/// For an array with shape [d0, d1, ..., dn], summing over axis `k` produces
/// a result with shape [d0, ..., d(k-1), d(k+1), ..., dn].
fn sum_over_axis(values: &[PyExpr], shape: &[usize], axis: usize) -> Vec<PyExpr> {
    let ndim = shape.len();
    let axis_size = shape[axis];

    // Product of dimensions before the axis
    let outer: usize = shape[..axis].iter().product();
    // Product of dimensions after the axis
    let inner: usize = shape[axis + 1..ndim].iter().product();

    let result_len = outer * inner;
    let mut result: Vec<PyExpr> = vec![PyExpr::default(); result_len];

    for o in 0..outer {
        for a in 0..axis_size {
            for i in 0..inner {
                let src_idx = o * axis_size * inner + a * inner + i;
                let dst_idx = o * inner + i;
                result[dst_idx].add_assign(&values[src_idx]);
            }
        }
    }

    result
}

fn axis_spec_from_bound(index_set: &Bound<'_, PyIndexSet>) -> AxisSpec {
    let borrowed = index_set.borrow();
    AxisSpec::new(borrowed.name.clone(), borrowed.members.len())
}

pub fn labeled_shape_from_index_sets(index_sets: &[Py<PyIndexSet>]) -> PyResult<LabeledShape> {
    Python::attach(|py| {
        let axes = index_sets
            .iter()
            .map(|index_set| axis_spec_from_bound(index_set.bind(py)))
            .collect();
        LabeledShape::new(axes).map_err(|err| ArrayDimensionError::new_err(err.to_string()))
    })
}

pub fn labeled_shape_from_axes_attr(
    obj: &Bound<'_, PyAny>,
    label: &str,
) -> PyResult<Option<LabeledShape>> {
    let Ok(axes_obj) = obj.getattr("axes") else {
        return Ok(None);
    };
    let axes_tuple = axes_obj.cast::<PyTuple>().map_err(|_| {
        ArrayTypeError::new_err(format!(
            "labeled {label} must expose axes as a tuple of IndexSet"
        ))
    })?;
    let axes = axes_tuple
        .iter()
        .map(|axis| {
            let axis = axis.cast::<PyIndexSet>().map_err(|_| {
                ArrayTypeError::new_err(format!("labeled {label} must expose IndexSet axes"))
            })?;
            Ok(axis_spec_from_bound(axis))
        })
        .collect::<PyResult<Vec<_>>>()?;
    LabeledShape::new(axes)
        .map(Some)
        .map_err(|err| ArrayDimensionError::new_err(err.to_string()))
}

fn extract_labeled_operand(
    py: Python<'_>,
    other: &Bound<'_, PyAny>,
) -> PyResult<Option<LabeledOperand>> {
    let Ok(axes_obj) = other.getattr("axes") else {
        return Ok(None);
    };
    let values_obj = other
        .getattr("values")
        .map_err(|_| ArrayTypeError::new_err("labeled operands must expose a values attribute"))?;
    let axes_tuple = axes_obj.cast::<PyTuple>().map_err(|_| {
        ArrayTypeError::new_err("labeled operands must expose axes as a tuple of IndexSet")
    })?;

    let mut index_sets = Vec::with_capacity(axes_tuple.len());
    for axis in axes_tuple.iter() {
        let axis = axis.cast::<PyIndexSet>().map_err(|_| {
            ArrayTypeError::new_err("labeled operands must expose axes as IndexSet values")
        })?;
        index_sets.push(axis.clone().unbind());
    }

    let np = py.import("numpy")?;
    let flat = np
        .call_method1("asarray", (&values_obj,))?
        .call_method0("flatten")?;
    let values = flat.extract::<Vec<f64>>()?;
    Ok(Some((index_sets, values)))
}

fn extract_labeled_numeric_values(
    py: Python<'_>,
    other: &Bound<'_, PyAny>,
    target_index_sets: &[Py<PyIndexSet>],
    target_len: usize,
) -> PyResult<Option<Vec<f64>>> {
    let Some((source_index_sets, values)) = extract_labeled_operand(py, other)? else {
        return Ok(None);
    };
    let source_shape = labeled_shape_from_index_sets(&source_index_sets)?;
    let target_shape = labeled_shape_from_index_sets(target_index_sets)?;
    let plan = BroadcastPlan::new(source_shape.clone(), target_shape)
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;

    let aligned = plan
        .broadcast_dense(&values)
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;
    if aligned.len() != target_len {
        return Err(ArrayShapeMismatchError::new_err(format!(
            "broadcasted labeled operand has length {} but target length is {}",
            aligned.len(),
            target_len
        )));
    }
    Ok(Some(aligned))
}

fn multiply_with_labeled_union(
    core: &LinearArrayCore,
    py: Python<'_>,
    other: &Bound<'_, PyAny>,
) -> PyResult<Option<LinearArrayCore>> {
    let Some((source_index_sets, source_values)) = extract_labeled_operand(py, other)? else {
        return Ok(None);
    };

    let mut union_index_sets = source_index_sets
        .iter()
        .map(|index_set| index_set.clone_ref(py))
        .collect::<Vec<_>>();
    let union_names = source_index_sets
        .iter()
        .map(|index_set| index_set.bind(py).borrow().name.clone())
        .collect::<BTreeSet<_>>();
    for index_set in &core.index_sets {
        let name = index_set.bind(py).borrow().name.clone();
        if !union_names.contains(&name) {
            union_index_sets.push(index_set.clone_ref(py));
        }
    }

    let target_shape = labeled_shape_from_index_sets(&union_index_sets)?;
    let core_shape = labeled_shape_from_index_sets(&core.index_sets)?;
    let source_shape = labeled_shape_from_index_sets(&source_index_sets)?;
    let expanded_exprs = BroadcastPlan::new(core_shape, target_shape.clone())
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?
        .broadcast_dense(&core.values)
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;
    let weights = BroadcastPlan::new(source_shape, target_shape.clone())
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?
        .broadcast_dense(&source_values)
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;

    let values = expanded_exprs
        .into_iter()
        .zip(weights)
        .map(|(expr, weight)| expr.scale(weight))
        .collect();
    Ok(Some(LinearArrayCore::new(
        union_index_sets,
        target_shape.shape(),
        values,
    )))
}

pub(super) fn multiply_sparse_variables_with_scalar(
    index_sets: &[Py<PyIndexSet>],
    shape: &[usize],
    active_indices: &[usize],
    var_ids: &[u32],
    factor: f64,
) -> PyExprArray {
    let values = var_ids
        .iter()
        .map(|var_id| PyExpr::from_term(*var_id, 1.0).scale(factor))
        .collect();
    PyExprArray::from_sparse(
        Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
        shape.to_vec(),
        active_indices.to_vec(),
        values,
    )
}

pub(super) fn multiply_sparse_variables_with_labeled_operand(
    index_sets: &[Py<PyIndexSet>],
    active_indices: &[usize],
    var_ids: &[u32],
    py: Python<'_>,
    other: &Bound<'_, PyAny>,
) -> PyResult<Option<PyExprArray>> {
    let Some((source_index_sets, source_values)) = extract_labeled_operand(py, other)? else {
        return Ok(None);
    };

    let mut union_index_sets = source_index_sets
        .iter()
        .map(|index_set| index_set.clone_ref(py))
        .collect::<Vec<_>>();
    let union_names = source_index_sets
        .iter()
        .map(|index_set| index_set.bind(py).borrow().name.clone())
        .collect::<BTreeSet<_>>();
    for index_set in index_sets {
        let name = index_set.bind(py).borrow().name.clone();
        if !union_names.contains(&name) {
            union_index_sets.push(index_set.clone_ref(py));
        }
    }

    let target_shape = labeled_shape_from_index_sets(&union_index_sets)?;
    let variable_shape = labeled_shape_from_index_sets(index_sets)?;
    let source_shape = labeled_shape_from_index_sets(&source_index_sets)?;
    let variable_plan = BroadcastPlan::new(variable_shape.clone(), target_shape.clone())
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;
    let source_plan = BroadcastPlan::new(source_shape, target_shape.clone())
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;

    let variable_total = variable_shape.total_len();
    let mut active_lookup = vec![None; variable_total];
    for (active_pos, active_idx) in active_indices.iter().copied().enumerate() {
        if active_idx >= variable_total {
            return Err(ArrayIndexError::new_err(format!(
                "active index {active_idx} out of range for sparse array of size {variable_total}"
            )));
        }
        active_lookup[active_idx] = Some(active_pos);
    }

    let mut out_indices = Vec::new();
    let mut out_values = Vec::new();
    for target_flat in 0..target_shape.total_len() {
        let variable_flat = variable_plan.source_offset_for_target_flat(target_flat);
        let Some(active_pos) = active_lookup[variable_flat] else {
            continue;
        };
        let source_flat = source_plan.source_offset_for_target_flat(target_flat);
        let weight = source_values[source_flat];
        if weight == 0.0 {
            continue;
        }
        out_indices.push(target_flat);
        out_values.push(PyExpr::from_term(var_ids[active_pos], 1.0).scale(weight));
    }

    Ok(Some(PyExprArray::from_sparse(
        union_index_sets,
        target_shape.shape(),
        out_indices,
        out_values,
    )))
}

pub(super) fn multiply_sparse_expr_with_scalar(
    index_sets: &[Py<PyIndexSet>],
    shape: &[usize],
    sparse: &SparseExprStorage,
    factor: f64,
) -> PyExprArray {
    let (active_indices, source_values) = sparse.materialized_entries();
    let values = source_values
        .into_iter()
        .map(|expr| expr.scale(factor))
        .collect();
    PyExprArray::from_sparse(
        Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
        shape.to_vec(),
        active_indices,
        values,
    )
}

pub(super) fn multiply_sparse_expr_with_labeled_operand(
    index_sets: &[Py<PyIndexSet>],
    sparse: &SparseExprStorage,
    py: Python<'_>,
    other: &Bound<'_, PyAny>,
) -> PyResult<Option<PyExprArray>> {
    let Some(sparse_values) = sparse.values() else {
        let (active_indices, values) = sparse.materialized_entries();
        let eager = SparseExprStorage::Eager {
            active_indices,
            values,
        };
        return multiply_sparse_expr_with_labeled_operand(index_sets, &eager, py, other);
    };
    let Some((source_index_sets, source_values)) = extract_labeled_operand(py, other)? else {
        return Ok(None);
    };

    let mut union_index_sets = source_index_sets
        .iter()
        .map(|index_set| index_set.clone_ref(py))
        .collect::<Vec<_>>();
    let union_names = source_index_sets
        .iter()
        .map(|index_set| index_set.bind(py).borrow().name.clone())
        .collect::<BTreeSet<_>>();
    for index_set in index_sets {
        let name = index_set.bind(py).borrow().name.clone();
        if !union_names.contains(&name) {
            union_index_sets.push(index_set.clone_ref(py));
        }
    }

    let target_shape = labeled_shape_from_index_sets(&union_index_sets)?;
    let expr_shape = labeled_shape_from_index_sets(index_sets)?;
    let source_shape = labeled_shape_from_index_sets(&source_index_sets)?;
    let expr_plan = BroadcastPlan::new(expr_shape.clone(), target_shape.clone())
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;
    let source_plan = BroadcastPlan::new(source_shape, target_shape.clone())
        .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;

    let expr_total = expr_shape.total_len();
    let mut active_lookup = vec![None; expr_total];
    for (active_pos, active_idx) in sparse.active_indices().iter().copied().enumerate() {
        if active_idx >= expr_total {
            return Err(ArrayIndexError::new_err(format!(
                "active index {active_idx} out of range for sparse expression array of size {expr_total}"
            )));
        }
        active_lookup[active_idx] = Some(active_pos);
    }

    let mut out_indices = Vec::new();
    let mut out_values = Vec::new();
    for target_flat in 0..target_shape.total_len() {
        let expr_flat = expr_plan.source_offset_for_target_flat(target_flat);
        let Some(active_pos) = active_lookup[expr_flat] else {
            continue;
        };
        let source_flat = source_plan.source_offset_for_target_flat(target_flat);
        let weight = source_values[source_flat];
        if weight == 0.0 {
            continue;
        }
        out_indices.push(target_flat);
        out_values.push(sparse_values[active_pos].scale(weight));
    }

    Ok(Some(PyExprArray::from_sparse(
        union_index_sets,
        target_shape.shape(),
        out_indices,
        out_values,
    )))
}

fn find_sparse_axis(
    index_sets: &[Py<PyIndexSet>],
    py: Python<'_>,
    index_set: &Bound<'_, PyIndexSet>,
) -> PyResult<usize> {
    let target_ptr = index_set.as_ptr();
    for (idx, stored) in index_sets.iter().enumerate() {
        if stored.as_ptr() == target_ptr {
            return Ok(idx);
        }
    }

    let target_name = &index_set.borrow().name;
    for (idx, stored) in index_sets.iter().enumerate() {
        let stored_ref = stored.bind(py).borrow();
        if &stored_ref.name == target_name {
            return Ok(idx);
        }
    }

    Err(ArrayIndexError::new_err(format!(
        "IndexSet '{}' is not a dimension of this array",
        index_set.borrow().name
    )))
}

pub(super) fn parse_sparse_axes(
    index_sets: &[Py<PyIndexSet>],
    py: Python<'_>,
    selection: &Bound<'_, PyAny>,
) -> PyResult<Vec<usize>> {
    let mut axes = Vec::new();
    if let Ok(single) = selection.cast::<PyIndexSet>() {
        axes.push(find_sparse_axis(index_sets, py, single)?);
    } else {
        let items: Vec<Bound<'_, PyAny>> = selection.try_iter()?.collect::<PyResult<Vec<_>>>()?;
        for item in &items {
            let index_set = item.cast::<PyIndexSet>().map_err(|_| {
                ArrayTypeError::new_err("axis/over must be an IndexSet or tuple of IndexSets")
            })?;
            axes.push(find_sparse_axis(index_sets, py, index_set)?);
        }
    }

    let mut seen = Vec::with_capacity(axes.len());
    for axis in &axes {
        if seen.contains(axis) {
            return Err(ArrayDimensionError::new_err(
                "axis/over cannot contain duplicate IndexSet dimensions",
            ));
        }
        seen.push(*axis);
    }
    Ok(axes)
}

pub(super) fn reduced_sparse_flat_index(
    flat_idx: usize,
    shape: &[usize],
    source_strides: &[usize],
    reduced_strides: &[usize],
    summed_axes: &[bool],
) -> usize {
    let mut remainder = flat_idx;
    let mut reduced_axis = 0usize;
    let mut reduced_idx = 0usize;

    for axis in 0..shape.len() {
        let coordinate = remainder / source_strides[axis];
        remainder %= source_strides[axis];
        if !summed_axes[axis] {
            reduced_idx += coordinate * reduced_strides[reduced_axis];
            reduced_axis += 1;
        }
    }

    reduced_idx
}

pub(super) fn sum_sparse_expr(
    index_sets: &[Py<PyIndexSet>],
    shape: &[usize],
    sparse: &SparseExprStorage,
    py: Python<'_>,
    over: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyObject> {
    let Some(sparse_values) = sparse.values() else {
        let core = sparse.to_core(index_sets, shape);
        return array_sum(&core, py, over);
    };
    let Some(over) = over else {
        let mut acc = PyExpr::default();
        for expr in sparse_values {
            acc.add_assign(expr);
        }
        return Ok(acc.into_pyobject(py)?.into_any().unbind());
    };

    let axes = parse_sparse_axes(index_sets, py, over)?;
    let mut summed_axes = vec![false; shape.len()];
    for axis in axes {
        summed_axes[axis] = true;
    }

    let mut out_shape = Vec::new();
    let mut out_index_sets = Vec::new();
    for (axis, index_set) in index_sets.iter().enumerate() {
        if !summed_axes[axis] {
            out_shape.push(shape[axis]);
            out_index_sets.push(index_set.clone_ref(py));
        }
    }

    let source_strides = arco_arrays::row_major_strides(shape);
    let reduced_strides = arco_arrays::row_major_strides(&out_shape);
    let out_len = out_shape.iter().product::<usize>().max(1);
    let mut values = vec![PyExpr::default(); out_len];

    for (active_idx, expr) in sparse.active_indices().iter().zip(sparse_values.iter()) {
        let reduced_idx = reduced_sparse_flat_index(
            *active_idx,
            shape,
            &source_strides,
            &reduced_strides,
            &summed_axes,
        );
        values[reduced_idx].add_assign(expr);
    }

    if out_shape.is_empty() {
        let expr = values.pop().unwrap_or_default();
        Ok(expr.into_pyobject(py)?.into_any().unbind())
    } else {
        let array = PyExprArray::new(out_index_sets, out_shape, values);
        Ok(array.into_pyobject(py)?.into_any().unbind())
    }
}

#[derive(Clone, Copy)]
pub(super) enum SparseDiffSource<'a> {
    Expressions(&'a [PyExpr]),
    VariableIds(&'a [u32]),
}

impl SparseDiffSource<'_> {
    fn len(self) -> usize {
        match self {
            Self::Expressions(values) => values.len(),
            Self::VariableIds(var_ids) => var_ids.len(),
        }
    }

    fn is_zero(self, source_pos: usize) -> bool {
        match self {
            Self::Expressions(values) => {
                let value = &values[source_pos];
                value.inner().num_terms() == 0 && value.constant() == 0.0
            }
            Self::VariableIds(_) => false,
        }
    }

    fn scaled(self, source_pos: usize, factor: f64) -> PyExpr {
        match self {
            Self::Expressions(values) => values[source_pos].scale(factor),
            Self::VariableIds(var_ids) => PyExpr::from_term(var_ids[source_pos], factor),
        }
    }
}

struct SparseDiffMapping {
    axis_size: usize,
    source_stride: usize,
    source_block: usize,
}

impl SparseDiffMapping {
    fn new(shape: &[usize], axis: usize) -> PyResult<Self> {
        let source_stride = shape[axis + 1..]
            .iter()
            .try_fold(1usize, |total, dimension| total.checked_mul(*dimension))
            .ok_or_else(|| ArrayDimensionError::new_err("sparse diff shape is too large"))?;
        let source_block = source_stride
            .checked_mul(shape[axis])
            .ok_or_else(|| ArrayDimensionError::new_err("sparse diff shape is too large"))?;
        Ok(Self {
            axis_size: shape[axis],
            source_block,
            source_stride,
        })
    }

    fn next(
        &self,
        active_indices: &[usize],
        source_pos: &mut usize,
        positive: bool,
    ) -> Option<(usize, usize)> {
        while let Some(active_idx) = active_indices.get(*source_pos).copied() {
            let current_source_pos = *source_pos;
            *source_pos += 1;
            if self.source_stride == 0 {
                return None;
            }
            let axis_coordinate = (active_idx / self.source_stride) % self.axis_size;
            let valid = if positive {
                axis_coordinate > 0
            } else {
                axis_coordinate < self.axis_size.saturating_sub(1)
            };
            if !valid {
                continue;
            }
            let outer = active_idx / self.source_block;
            let output_flat = active_idx
                - outer * self.source_stride
                - if positive { self.source_stride } else { 0 };
            return Some((output_flat, current_source_pos));
        }
        None
    }
}

struct SparseDiffCursor<'a> {
    active_indices: &'a [usize],
    source_pos: usize,
    positive: bool,
    current: Option<(usize, usize)>,
}

impl<'a> SparseDiffCursor<'a> {
    fn new(active_indices: &'a [usize], positive: bool) -> Self {
        Self {
            active_indices,
            source_pos: 0,
            positive,
            current: None,
        }
    }

    fn advance(&mut self, mapping: &SparseDiffMapping) {
        self.current = mapping.next(self.active_indices, &mut self.source_pos, self.positive);
    }

    fn source_pos_at(&self, output_idx: usize) -> Option<usize> {
        self.current
            .filter(|(current_idx, _)| *current_idx == output_idx)
            .map(|(_, source_pos)| source_pos)
    }
}

enum SparseDiffContribution {
    Positive(usize),
    Negative(usize),
    Both { positive: usize, negative: usize },
}

fn merge_sparse_diff_rows(
    mapping: &SparseDiffMapping,
    active_indices: &[usize],
    source: SparseDiffSource<'_>,
    mut visit: impl FnMut(usize, SparseDiffContribution),
) {
    let mut positive = SparseDiffCursor::new(active_indices, true);
    let mut negative = SparseDiffCursor::new(active_indices, false);
    positive.advance(mapping);
    negative.advance(mapping);

    while positive.current.is_some() || negative.current.is_some() {
        let current_idx = match (positive.current, negative.current) {
            (Some((positive_idx, _)), Some((negative_idx, _))) => positive_idx.min(negative_idx),
            (Some((positive_idx, _)), None) => positive_idx,
            (None, Some((negative_idx, _))) => negative_idx,
            (None, None) => return,
        };
        let positive_pos = positive.source_pos_at(current_idx);
        let negative_pos = negative.source_pos_at(current_idx);
        let contribution = match (
            positive_pos.filter(|source_pos| !source.is_zero(*source_pos)),
            negative_pos.filter(|source_pos| !source.is_zero(*source_pos)),
        ) {
            (Some(positive), Some(negative)) => {
                Some(SparseDiffContribution::Both { positive, negative })
            }
            (Some(positive), None) => Some(SparseDiffContribution::Positive(positive)),
            (None, Some(negative)) => Some(SparseDiffContribution::Negative(negative)),
            (None, None) => None,
        };
        if let Some(contribution) = contribution {
            visit(current_idx, contribution);
        }
        if positive_pos.is_some() {
            positive.advance(mapping);
        }
        if negative_pos.is_some() {
            negative.advance(mapping);
        }
    }
}

pub(super) fn diff_sparse_expr(
    index_sets: &[Py<PyIndexSet>],
    shape: &[usize],
    active_indices: &[usize],
    source: SparseDiffSource<'_>,
    py: Python<'_>,
    over: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let axes = parse_sparse_axes(index_sets, py, over)?;
    if axes.len() != 1 {
        return Err(ArrayDimensionError::new_err(
            "np.diff requires exactly one IndexSet axis",
        ));
    }
    let axis = axes[0];
    let axis_size = shape[axis];

    if active_indices.len() != source.len() {
        return Err(ArrayDimensionError::new_err(format!(
            "sparse diff source length {} does not match active index length {}",
            source.len(),
            active_indices.len()
        )));
    }
    let source_total = shape
        .iter()
        .try_fold(1usize, |total, dimension| total.checked_mul(*dimension))
        .ok_or_else(|| ArrayDimensionError::new_err("sparse diff shape is too large"))?;
    for (position, &active_idx) in active_indices.iter().enumerate() {
        if active_idx >= source_total {
            return Err(ArrayIndexError::new_err(format!(
                "sparse diff active index {active_idx} at position {position} exceeds source size {source_total}"
            )));
        }
        if position > 0 && active_indices[position - 1] >= active_idx {
            return Err(ArrayDimensionError::new_err(
                "sparse diff active indices must be strictly increasing",
            ));
        }
    }

    let mut out_shape = shape.to_vec();
    out_shape[axis] = axis_size.saturating_sub(1);
    let mut out_index_sets = Python::attach(|py| {
        index_sets
            .iter()
            .map(|index_set| index_set.clone_ref(py))
            .collect::<Vec<_>>()
    });
    let selected = (1..axis_size).collect::<Vec<_>>();
    out_index_sets[axis] = slice_index_set(py, &index_sets[axis], &selected)?;

    if axis_size <= 1 {
        return Ok(
            PyExprArray::from_sparse(out_index_sets, out_shape, Vec::new(), Vec::new())
                .into_pyobject(py)?
                .into_any()
                .unbind(),
        );
    }

    let mapping = SparseDiffMapping::new(shape, axis)?;
    let mut output_count = 0;
    merge_sparse_diff_rows(&mapping, active_indices, source, |_, _| {
        output_count += 1;
    });
    let mut out_indices = Vec::with_capacity(output_count);
    let mut out_values = Vec::with_capacity(output_count);
    merge_sparse_diff_rows(
        &mapping,
        active_indices,
        source,
        |current_idx, contribution| {
            let value = match contribution {
                SparseDiffContribution::Positive(source_pos) => source.scaled(source_pos, 1.0),
                SparseDiffContribution::Negative(source_pos) => source.scaled(source_pos, -1.0),
                SparseDiffContribution::Both { positive, negative } => {
                    let mut value = source.scaled(positive, 1.0);
                    value.add_assign_owned(source.scaled(negative, -1.0));
                    value
                }
            };
            out_indices.push(current_idx);
            out_values.push(value);
        },
    );

    Ok(
        PyExprArray::from_sparse(out_index_sets, out_shape, out_indices, out_values)
            .into_pyobject(py)?
            .into_any()
            .unbind(),
    )
}

fn sparse_rolled_flat_index(
    flat_idx: usize,
    shape: &[usize],
    strides: &[usize],
    roll_axis: usize,
    shift: isize,
) -> usize {
    let axis_size = shape[roll_axis];
    let mut remainder = flat_idx;
    let mut output_flat = 0usize;

    for (axis, stride) in strides.iter().copied().enumerate().take(shape.len()) {
        let mut coordinate = remainder / stride;
        remainder %= stride;
        if axis == roll_axis && axis_size > 0 {
            coordinate = (coordinate as isize + shift).rem_euclid(axis_size as isize) as usize;
        }
        output_flat += coordinate * stride;
    }

    output_flat
}

pub(super) fn roll_sparse_expr(
    index_sets: &[Py<PyIndexSet>],
    shape: &[usize],
    active_indices: &[usize],
    values: &[PyExpr],
    py: Python<'_>,
    shift: isize,
    over: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let axes = parse_sparse_axes(index_sets, py, over)?;
    if axes.len() != 1 {
        return Err(ArrayDimensionError::new_err(
            "np.roll requires exactly one IndexSet axis",
        ));
    }

    if shape[axes[0]] == 0 {
        return Ok(PyExprArray::from_sparse(
            Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
            shape.to_vec(),
            Vec::new(),
            Vec::new(),
        )
        .into_pyobject(py)?
        .into_any()
        .unbind());
    }

    let strides = arco_arrays::row_major_strides(shape);
    let mut rolled = active_indices
        .iter()
        .copied()
        .zip(values.iter().cloned())
        .map(|(active_idx, expr)| {
            (
                sparse_rolled_flat_index(active_idx, shape, &strides, axes[0], shift),
                expr,
            )
        })
        .collect::<Vec<_>>();
    rolled.sort_unstable_by_key(|(idx, _)| *idx);

    let (out_indices, out_values): (Vec<_>, Vec<_>) = rolled.into_iter().unzip();
    Ok(PyExprArray::from_sparse(
        Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
        shape.to_vec(),
        out_indices,
        out_values,
    )
    .into_pyobject(py)?
    .into_any()
    .unbind())
}

pub(super) fn combine_sparse_expr_same_shape(
    index_sets: &[Py<PyIndexSet>],
    shape: &[usize],
    left_indices: &[usize],
    left_values: &[PyExpr],
    right_indices: &[usize],
    right_values: &[PyExpr],
    right_scale: f64,
) -> PyExprArray {
    let mut left_pos = 0usize;
    let mut right_pos = 0usize;
    let mut out_indices = Vec::with_capacity(left_indices.len().max(right_indices.len()));
    let mut out_values = Vec::with_capacity(out_indices.capacity());

    while left_pos < left_indices.len() || right_pos < right_indices.len() {
        let left_idx = left_indices.get(left_pos).copied();
        let right_idx = right_indices.get(right_pos).copied();
        let current_idx = match (left_idx, right_idx) {
            (Some(left), Some(right)) => left.min(right),
            (Some(left), None) => left,
            (None, Some(right)) => right,
            (None, None) => break,
        };

        let mut expr = PyExpr::default();
        if left_idx == Some(current_idx) {
            expr.add_assign(&left_values[left_pos]);
            left_pos += 1;
        }
        if right_idx == Some(current_idx) {
            expr.add_assign_owned(right_values[right_pos].scale(right_scale));
            right_pos += 1;
        }

        if expr.inner().num_terms() == 0 && expr.constant() == 0.0 {
            continue;
        }
        out_indices.push(current_idx);
        out_values.push(expr);
    }

    PyExprArray::from_sparse(
        Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
        shape.to_vec(),
        out_indices,
        out_values,
    )
}

fn parse_named_axes(
    core: &LinearArrayCore,
    py: Python<'_>,
    selection: &Bound<'_, PyAny>,
) -> PyResult<Vec<usize>> {
    let mut selected_axes = Vec::new();

    if let Ok(single) = selection.cast::<PyIndexSet>() {
        selected_axes.push(core.find_axis(py, single)?);
    } else {
        let items: Vec<Bound<'_, PyAny>> = selection.try_iter()?.collect::<PyResult<Vec<_>>>()?;
        for item in &items {
            let index_set = item.cast::<PyIndexSet>().map_err(|_| {
                ArrayTypeError::new_err("axis/over must be an IndexSet or tuple of IndexSets")
            })?;
            selected_axes.push(core.find_axis(py, index_set)?);
        }
    }

    let core_shape = labeled_shape_from_index_sets(&core.index_sets)?;
    let selected_specs = selected_axes
        .iter()
        .map(|axis_idx| axis_spec_from_bound(core.index_sets[*axis_idx].bind(py)))
        .collect::<Vec<_>>();
    let mut axes_to_sum = core_shape
        .axis_indices(selected_specs.iter())
        .map_err(|err| ArrayDimensionError::new_err(err.to_string()))?;
    axes_to_sum.sort_unstable();
    axes_to_sum.reverse();
    Ok(axes_to_sum)
}

/// Shared storage for indexed linear expression arrays.
/// Both VariableArray and ExprArray compose this internally.
pub struct LinearArrayCore {
    pub index_sets: Vec<Py<PyIndexSet>>,
    pub shape: Vec<usize>,
    pub values: Vec<PyExpr>,
}

impl LinearArrayCore {
    pub(crate) fn new(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        values: Vec<PyExpr>,
    ) -> Self {
        Self {
            index_sets,
            shape,
            values,
        }
    }

    /// Clone the core, requires GIL for cloning Py<T>.
    pub(crate) fn clone_with_gil(&self) -> Self {
        Python::attach(|py| Self {
            index_sets: self.index_sets.iter().map(|s| s.clone_ref(py)).collect(),
            shape: self.shape.clone(),
            values: self.values.clone(),
        })
    }

    pub(crate) fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
        Python::attach(|py| {
            self.index_sets
                .iter()
                .map(|set| set.clone_ref(py))
                .collect()
        })
    }

    fn assert_same_shape(&self, other: &LinearArrayCore) -> PyResult<()> {
        if self.shape != other.shape {
            return Err(ArrayShapeMismatchError::new_err("array shapes must match"));
        }
        Ok(())
    }

    fn broadcast_to_target(
        &self,
        target_index_sets: &[Py<PyIndexSet>],
    ) -> PyResult<LinearArrayCore> {
        let source_shape = labeled_shape_from_index_sets(&self.index_sets)?;
        let target_shape = labeled_shape_from_index_sets(target_index_sets)?;
        let plan = BroadcastPlan::new(source_shape, target_shape.clone())
            .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;
        let values = plan
            .broadcast_dense(&self.values)
            .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?;
        Ok(LinearArrayCore::new(
            Python::attach(|py| {
                target_index_sets
                    .iter()
                    .map(|set| set.clone_ref(py))
                    .collect()
            }),
            target_shape.shape(),
            values,
        ))
    }

    fn combine(
        &self,
        other: &LinearArrayCore,
        combine: fn(&PyExpr, &PyExpr) -> PyExpr,
    ) -> PyResult<LinearArrayCore> {
        let aligned_other = other.broadcast_to_target(&self.index_sets)?;
        self.assert_same_shape(&aligned_other)?;
        let values = self
            .values
            .iter()
            .zip(aligned_other.values.iter())
            .map(|(left, right)| combine(left, right))
            .collect();
        Ok(LinearArrayCore::new(
            self.clone_index_sets(),
            self.shape.clone(),
            values,
        ))
    }

    fn scale_all(&self, factor: f64) -> LinearArrayCore {
        let values = self.values.iter().map(|expr| expr.scale(factor)).collect();
        LinearArrayCore::new(self.clone_index_sets(), self.shape.clone(), values)
    }

    fn add_scalar(&self, value: f64) -> LinearArrayCore {
        let values = self
            .values
            .iter()
            .map(|expr| expr.add_constant(value))
            .collect();
        LinearArrayCore::new(self.clone_index_sets(), self.shape.clone(), values)
    }

    fn sub_scalar(&self, value: f64) -> LinearArrayCore {
        self.add_scalar(-value)
    }

    fn rsub_scalar(&self, value: f64) -> LinearArrayCore {
        let values = self
            .values
            .iter()
            .map(|expr| expr.scale(-1.0).add_constant(value))
            .collect();
        LinearArrayCore::new(self.clone_index_sets(), self.shape.clone(), values)
    }

    fn add_vec(&self, rhs: &[f64]) -> PyResult<LinearArrayCore> {
        if rhs.len() != self.values.len() {
            return Err(ArrayShapeMismatchError::new_err(format!(
                "element-wise add length mismatch ({} vs {})",
                rhs.len(),
                self.values.len()
            )));
        }
        let values = self
            .values
            .iter()
            .zip(rhs.iter())
            .map(|(expr, v)| expr.add_constant(*v))
            .collect();
        Ok(LinearArrayCore::new(
            self.clone_index_sets(),
            self.shape.clone(),
            values,
        ))
    }

    fn sub_vec(&self, rhs: &[f64]) -> PyResult<LinearArrayCore> {
        if rhs.len() != self.values.len() {
            return Err(ArrayShapeMismatchError::new_err(format!(
                "element-wise sub length mismatch ({} vs {})",
                rhs.len(),
                self.values.len()
            )));
        }
        let values = self
            .values
            .iter()
            .zip(rhs.iter())
            .map(|(expr, v)| expr.add_constant(-*v))
            .collect();
        Ok(LinearArrayCore::new(
            self.clone_index_sets(),
            self.shape.clone(),
            values,
        ))
    }

    fn rsub_vec(&self, rhs: &[f64]) -> PyResult<LinearArrayCore> {
        if rhs.len() != self.values.len() {
            return Err(ArrayShapeMismatchError::new_err(format!(
                "element-wise rsub length mismatch ({} vs {})",
                rhs.len(),
                self.values.len()
            )));
        }
        let values = self
            .values
            .iter()
            .zip(rhs.iter())
            .map(|(expr, v)| expr.scale(-1.0).add_constant(*v))
            .collect();
        Ok(LinearArrayCore::new(
            self.clone_index_sets(),
            self.shape.clone(),
            values,
        ))
    }

    fn mul_vec(&self, weights: &[f64]) -> PyResult<LinearArrayCore> {
        if weights.len() != self.values.len() {
            return Err(ArrayShapeMismatchError::new_err(format!(
                "element-wise multiply length mismatch ({} vs {})",
                weights.len(),
                self.values.len()
            )));
        }
        let values = self
            .values
            .iter()
            .zip(weights.iter())
            .map(|(expr, w)| expr.scale(*w))
            .collect();
        Ok(LinearArrayCore::new(
            self.clone_index_sets(),
            self.shape.clone(),
            values,
        ))
    }

    fn compare_core(
        &self,
        other: &LinearArrayCore,
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintArray> {
        let (left, right) = if let Ok(aligned_other) = other.broadcast_to_target(&self.index_sets) {
            (self.clone_with_gil(), aligned_other)
        } else if let Ok(aligned_self) = self.broadcast_to_target(&other.index_sets) {
            (aligned_self, other.clone_with_gil())
        } else {
            return Err(ArrayShapeMismatchError::new_err("array shapes must match"));
        };
        left.assert_same_shape(&right)?;
        Ok(PyConstraintArray::from_lazy_compare(left, right, sense))
    }

    pub fn compare_scalar(&self, rhs: f64, sense: ComparisonSense) -> PyConstraintArray {
        let mut exprs = Vec::with_capacity(self.values.len());
        let mut rhs_values = Vec::with_capacity(self.values.len());
        for expr in &self.values {
            exprs.push(expr.without_constant());
            rhs_values.push(rhs - expr.constant());
        }
        PyConstraintArray::new(
            exprs,
            sense,
            rhs_values,
            self.shape.clone(),
            self.clone_index_sets(),
        )
    }

    pub fn compare_index_set(
        &self,
        rhs: &PyIndexSet,
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintArray> {
        if self.shape.is_empty() {
            return Err(ArrayDimensionError::new_err(
                "index set comparisons require array shape",
            ));
        }
        if rhs.members.len() != self.shape[0] {
            return Err(ArrayDimensionError::new_err(
                "index set size must match leading dimension",
            ));
        }
        let inner = self.shape.iter().skip(1).product::<usize>().max(1);
        let mut rhs_values = Vec::with_capacity(self.values.len());
        for member in &rhs.members {
            let value = member.as_f64().ok_or_else(|| {
                ArrayTypeError::new_err("index set members must be numeric for comparisons")
            })?;
            for _ in 0..inner {
                rhs_values.push(value);
            }
        }
        if rhs_values.len() != self.values.len() {
            return Err(ArrayShapeMismatchError::new_err(
                "broadcasted RHS does not match array size",
            ));
        }
        let mut exprs = Vec::with_capacity(self.values.len());
        for (expr, rhs) in self.values.iter().zip(rhs_values.iter_mut()) {
            *rhs -= expr.constant();
            exprs.push(expr.without_constant());
        }
        Ok(PyConstraintArray::new(
            exprs,
            sense,
            rhs_values,
            self.shape.clone(),
            self.clone_index_sets(),
        ))
    }

    fn compare_vec(
        &self,
        rhs_values: &[f64],
        sense: ComparisonSense,
    ) -> PyResult<PyConstraintArray> {
        if rhs_values.len() != self.values.len() {
            return Err(ArrayShapeMismatchError::new_err(format!(
                "RHS vector length {} does not match array length {}",
                rhs_values.len(),
                self.values.len()
            )));
        }
        let mut exprs = Vec::with_capacity(self.values.len());
        let mut rhs_out = Vec::with_capacity(self.values.len());
        for (expr, &rhs) in self.values.iter().zip(rhs_values.iter()) {
            exprs.push(expr.without_constant());
            rhs_out.push(rhs - expr.constant());
        }
        Ok(PyConstraintArray::new(
            exprs,
            sense,
            rhs_out,
            self.shape.clone(),
            self.clone_index_sets(),
        ))
    }

    /// Find the axis index for an IndexSet by matching Python object identity or name+size.
    fn find_axis(&self, py: Python<'_>, index_set: &Bound<'_, PyIndexSet>) -> PyResult<usize> {
        let target_ptr = index_set.as_ptr();
        // First try identity match (same Python object)
        for (i, stored) in self.index_sets.iter().enumerate() {
            if stored.as_ptr() == target_ptr {
                return Ok(i);
            }
        }
        // Fallback: match by axis label so narrowed slices still accept the
        // original IndexSet in axis=... calls.
        let target_name = &index_set.borrow().name;
        for (i, stored) in self.index_sets.iter().enumerate() {
            let stored_ref = stored.bind(py).borrow();
            if &stored_ref.name == target_name {
                return Ok(i);
            }
        }
        Err(ArrayIndexError::new_err(format!(
            "IndexSet '{}' is not a dimension of this array",
            index_set.borrow().name
        )))
    }

    /// Sum all elements to a scalar Expr.
    fn sum_all(&self) -> PyExpr {
        let mut total_linear = 0usize;
        let mut total_quadratic = 0usize;
        let mut total_cubic = 0usize;
        for v in &self.values {
            total_linear += v.inner().linear_terms().len();
            total_quadratic += v.inner().quadratic_terms().len();
            total_cubic += v.inner().cubic_terms().len();
        }
        let mut acc = Expr::new_empty();
        acc.reserve(total_linear, total_quadratic, total_cubic);
        for v in &self.values {
            acc.add_assign(v.inner());
        }
        PyExpr::from_expr(acc)
    }
}

/// A template for one term per element. Element `i`'s variable ID = `start_var_id + i`.
#[derive(Clone, Debug)]
pub struct CompactTerm {
    pub(crate) start_var_id: u32,
    pub(crate) coefficient: f64,
}

/// Compact expression storage: represents N elements with O(terms_per_element) memory.
/// For element `i`: `expr_i = constant + sum(term.coeff * var(term.start_var_id + i))`
#[derive(Clone, Debug)]
pub struct CompactExprStorage {
    pub(crate) terms: Vec<CompactTerm>,
    pub(crate) constant: f64,
    pub(crate) count: usize,
}

/// Sparse expression storage: inactive dense slots are implicit zeros.
#[derive(Clone)]
pub(crate) enum SparseExprStorage {
    Eager {
        active_indices: Vec<usize>,
        values: Vec<PyExpr>,
    },
    Lazy(Arc<SparseExprNode>),
}

impl SparseExprStorage {
    pub(crate) fn lazy(node: Arc<SparseExprNode>) -> Self {
        Self::Lazy(node)
    }

    pub(crate) fn active_indices(&self) -> &[usize] {
        match self {
            Self::Eager { active_indices, .. } => active_indices,
            Self::Lazy(node) => node.active_indices(),
        }
    }

    pub(crate) fn values(&self) -> Option<&[PyExpr]> {
        match self {
            Self::Eager { values, .. } => Some(values),
            Self::Lazy(_) => None,
        }
    }

    pub(crate) fn materialized_entries(&self) -> (Vec<usize>, Vec<PyExpr>) {
        match self {
            Self::Eager {
                active_indices,
                values,
            } => (active_indices.clone(), values.clone()),
            Self::Lazy(node) => Python::attach(|py| {
                let mut active_indices = Vec::with_capacity(node.active_indices().len());
                let mut values = Vec::with_capacity(node.active_indices().len());
                for &index in node.active_indices() {
                    if let Some(value) = node.value_at(py, index) {
                        active_indices.push(index);
                        values.push(value);
                    }
                }
                (active_indices, values)
            }),
        }
    }

    pub(crate) fn is_lazy(&self) -> bool {
        matches!(self, Self::Lazy(_))
    }

    pub(crate) fn node(&self) -> Option<Arc<SparseExprNode>> {
        match self {
            Self::Eager { .. } => None,
            Self::Lazy(node) => Some(node.clone()),
        }
    }

    pub(crate) fn active_count(&self) -> usize {
        match self {
            Self::Eager { active_indices, .. } => active_indices.len(),
            Self::Lazy(node) => Python::attach(|py| {
                node.active_indices()
                    .iter()
                    .filter(|&&index| node.value_at(py, index).is_some())
                    .count()
            }),
        }
    }

    pub(crate) fn value_at_flat(&self, py: Python<'_>, index: usize) -> Option<PyExpr> {
        match self {
            Self::Lazy(node) => node.value_at(py, index),
            Self::Eager {
                active_indices,
                values,
            } => active_indices
                .binary_search(&index)
                .ok()
                .map(|position| values[position].clone()),
        }
    }

    pub(crate) fn term_counts(&self) -> ExpressionTermCounts {
        match self {
            Self::Lazy(node) => Python::attach(|py| {
                node.active_indices().iter().fold(
                    ExpressionTermCounts::default(),
                    |mut counts, index| {
                        if let Some(expr) = node.value_at(py, *index) {
                            counts.linear += expr.inner().linear_terms().len();
                            counts.quadratic += expr.inner().quadratic_terms().len();
                            counts.cubic += expr.inner().cubic_terms().len();
                        }
                        counts
                    },
                )
            }),
            Self::Eager { values, .. } => expression_term_counts(values),
        }
    }

    pub(crate) fn to_core(
        &self,
        index_sets: &[Py<PyIndexSet>],
        shape: &[usize],
    ) -> LinearArrayCore {
        let total = shape.iter().product();
        let mut values = vec![PyExpr::default(); total];
        Python::attach(|py| {
            if let Self::Lazy(node) = self {
                for active_idx in node.active_indices() {
                    if let Some(expr) = node.value_at(py, *active_idx) {
                        values[*active_idx] = expr;
                    }
                }
            } else if let Self::Eager {
                active_indices,
                values: eager_values,
            } = self
            {
                for (active_idx, expr) in active_indices.iter().zip(eager_values.iter()) {
                    values[*active_idx] = expr.clone();
                }
            }
            LinearArrayCore::new(
                index_sets.iter().map(|set| set.clone_ref(py)).collect(),
                shape.to_vec(),
                values,
            )
        })
    }
}

impl CompactExprStorage {
    /// Create from a single variable array (coefficient 1.0, constant 0.0).
    pub(crate) fn from_variable_array(start_var_id: u32, count: usize) -> Self {
        Self {
            terms: vec![CompactTerm {
                start_var_id,
                coefficient: 1.0,
            }],
            constant: 0.0,
            count,
        }
    }

    /// Scale all coefficients and the constant.
    pub(crate) fn scale(&self, factor: f64) -> Self {
        Self {
            terms: self
                .terms
                .iter()
                .map(|t| CompactTerm {
                    start_var_id: t.start_var_id,
                    coefficient: t.coefficient * factor,
                })
                .collect(),
            constant: self.constant * factor,
            count: self.count,
        }
    }

    /// Add another compact storage, merging duplicate start_var_ids.
    pub(crate) fn add_compact(&self, other: &CompactExprStorage) -> Self {
        debug_assert_eq!(self.count, other.count);
        let mut terms = self.terms.clone();
        for other_term in &other.terms {
            if let Some(existing) = terms
                .iter_mut()
                .find(|t| t.start_var_id == other_term.start_var_id)
            {
                existing.coefficient += other_term.coefficient;
            } else {
                terms.push(other_term.clone());
            }
        }
        terms.retain(|t| t.coefficient != 0.0);
        Self {
            terms,
            constant: self.constant + other.constant,
            count: self.count,
        }
    }

    /// Subtract another compact storage.
    pub(crate) fn sub_compact(&self, other: &CompactExprStorage) -> Self {
        self.add_compact(&other.scale(-1.0))
    }

    /// Add a constant offset.
    pub(crate) fn add_constant(&self, value: f64) -> Self {
        Self {
            terms: self.terms.clone(),
            constant: self.constant + value,
            count: self.count,
        }
    }

    /// Materialize to LinearArrayCore (fallback).
    pub(crate) fn to_core(
        &self,
        index_sets: &[Py<PyIndexSet>],
        shape: &[usize],
    ) -> LinearArrayCore {
        let values = (0..self.count)
            .map(|i| {
                let terms: Vec<(VariableId, f64)> = self
                    .terms
                    .iter()
                    .map(|t| (VariableId::new(t.start_var_id + i as u32), t.coefficient))
                    .collect();
                PyExpr::from_expr(Expr::new(terms, self.constant))
            })
            .collect();
        Python::attach(|py| {
            LinearArrayCore::new(
                index_sets.iter().map(|s| s.clone_ref(py)).collect(),
                shape.to_vec(),
                values,
            )
        })
    }

    /// Collect linear terms for objective extraction.
    pub fn collect_linear_terms(&self) -> Vec<(VariableId, f64)> {
        let total = self.terms.len() * self.count;
        let mut result = Vec::with_capacity(total);
        for term in &self.terms {
            for i in 0..self.count {
                result.push((
                    VariableId::new(term.start_var_id + i as u32),
                    term.coefficient,
                ));
            }
        }
        result
    }

    /// Count expression terms without materializing each element.
    pub(crate) fn term_counts(&self) -> ExpressionTermCounts {
        ExpressionTermCounts {
            linear: self.terms.len() * self.count,
            quadratic: 0,
            cubic: 0,
        }
    }

    /// Sum all elements to a single PyExpr.
    pub(crate) fn sum_all(&self) -> PyExpr {
        let linear = self.collect_linear_terms();
        let total_constant = self.constant * self.count as f64;
        PyExpr::from_expr(Expr::new(linear, total_constant))
    }
}

/// Dual-storage for ExprArray: full materialized or compact pattern.
pub(crate) enum ExprArrayStorage {
    Full(LinearArrayCore),
    Compact {
        storage: CompactExprStorage,
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
    },
    Sparse {
        storage: SparseExprStorage,
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
    },
}

impl ExprArrayStorage {
    /// Materialize to LinearArrayCore.
    pub(crate) fn to_core(&self) -> LinearArrayCore {
        match self {
            ExprArrayStorage::Full(core) => core.clone_with_gil(),
            ExprArrayStorage::Compact {
                storage,
                index_sets,
                shape,
            } => storage.to_core(index_sets, shape),
            ExprArrayStorage::Sparse {
                storage,
                index_sets,
                shape,
            } => storage.to_core(index_sets, shape),
        }
    }

    pub(crate) fn count(&self) -> usize {
        match self {
            ExprArrayStorage::Full(core) => core.values.len(),
            ExprArrayStorage::Compact { storage, .. } => storage.count,
            ExprArrayStorage::Sparse { shape, .. } => shape.iter().product(),
        }
    }

    pub(crate) fn shape(&self) -> &[usize] {
        match self {
            ExprArrayStorage::Full(core) => &core.shape,
            ExprArrayStorage::Compact { shape, .. } => shape,
            ExprArrayStorage::Sparse { shape, .. } => shape,
        }
    }

    pub(crate) fn index_sets_ref(&self) -> &[Py<PyIndexSet>] {
        match self {
            ExprArrayStorage::Full(core) => &core.index_sets,
            ExprArrayStorage::Compact { index_sets, .. } => index_sets,
            ExprArrayStorage::Sparse { index_sets, .. } => index_sets,
        }
    }

    pub(crate) fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
        Python::attach(|py| {
            self.index_sets_ref()
                .iter()
                .map(|s| s.clone_ref(py))
                .collect()
        })
    }

    /// Get the compact storage if available.
    pub(crate) fn as_compact(&self) -> Option<&CompactExprStorage> {
        match self {
            ExprArrayStorage::Compact { storage, .. } => Some(storage),
            ExprArrayStorage::Full(_) | ExprArrayStorage::Sparse { .. } => None,
        }
    }

    pub(crate) fn as_sparse(&self) -> Option<&SparseExprStorage> {
        match self {
            ExprArrayStorage::Sparse { storage, .. } => Some(storage),
            ExprArrayStorage::Full(_) | ExprArrayStorage::Compact { .. } => None,
        }
    }

    pub(crate) fn sparse_node(&self) -> Option<Arc<SparseExprNode>> {
        self.as_sparse().and_then(SparseExprStorage::node)
    }
}

/// Try to extract a CompactExprStorage from a PyAny operand.
pub(crate) fn try_extract_compact(other: &Bound<'_, PyAny>) -> Option<CompactExprStorage> {
    if let Ok(va) = other.extract::<PyRef<'_, PyVariableArray>>() {
        return va.as_compact_expr();
    }
    if let Ok(ea) = other.extract::<PyRef<'_, PyExprArray>>() {
        return ea.as_compact().cloned();
    }
    None
}

/// Try to create a CompactConstraintStorage from a compact expression and a Python RHS.
/// Returns None if the RHS type cannot be handled compactly (e.g., another array).
pub fn try_make_compact_constraint(
    compact_expr: &CompactExprStorage,
    rhs: &Bound<'_, PyAny>,
    sense: ComparisonSense,
) -> Option<constraint_array::CompactConstraintStorage> {
    use constraint_array::{CompactConstraintStorage, CompactRhs};

    if let Ok(scalar) = rhs.extract::<f64>() {
        return Some(CompactConstraintStorage {
            terms: compact_expr.terms.clone(),
            sense,
            rhs: CompactRhs::Scalar(scalar - compact_expr.constant),
            count: compact_expr.count,
        });
    }
    if let Ok(index_set) = rhs.extract::<PyRef<'_, PyIndexSet>>() {
        let members_len = index_set.members.len();
        if members_len == 0 || compact_expr.count % members_len != 0 {
            return None; // shape mismatch, fall back
        }
        let inner = compact_expr.count / members_len;
        let mut rhs_values = Vec::with_capacity(compact_expr.count);
        for member in &index_set.members {
            let value = match member.as_f64() {
                Some(v) => v - compact_expr.constant,
                None => return None, // non-numeric, fall back
            };
            for _ in 0..inner {
                rhs_values.push(value);
            }
        }
        return Some(CompactConstraintStorage {
            terms: compact_expr.terms.clone(),
            sense,
            rhs: CompactRhs::Vec(rhs_values),
            count: compact_expr.count,
        });
    }
    if let Ok(rhs_values) = rhs.extract::<Vec<f64>>() {
        if rhs_values.len() != compact_expr.count {
            return None; // length mismatch, fall back
        }
        let adjusted: Vec<f64> = rhs_values
            .iter()
            .map(|v| v - compact_expr.constant)
            .collect();
        return Some(CompactConstraintStorage {
            terms: compact_expr.terms.clone(),
            sense,
            rhs: CompactRhs::Vec(adjusted),
            count: compact_expr.count,
        });
    }
    // Can't handle compactly (e.g., another array)
    None
}

/// Compare with compact fast path, falling back to full materialized comparison.
///
/// Used by both `PyVariableArray` and `PyExprArray` comparison operators (`__ge__`, `__le__`, `__eq__`).
pub(crate) fn compare_with_compact_fallback(
    compact: Option<&CompactExprStorage>,
    shape: &[usize],
    index_sets: &[Py<PyIndexSet>],
    core_fn: impl FnOnce() -> LinearArrayCore,
    rhs: &Bound<'_, PyAny>,
    sense: ComparisonSense,
) -> PyResult<PyConstraintArray> {
    if let Some(compact_expr) = compact {
        if let Some(compact_con) = try_make_compact_constraint(compact_expr, rhs, sense) {
            return Ok(PyConstraintArray::from_compact(
                compact_con,
                shape.to_vec(),
                Python::attach(|py| index_sets.iter().map(|s| s.clone_ref(py)).collect()),
            ));
        }
    }
    compare_array_rhs(&core_fn(), rhs, sense)
}

/// Retain sparse operands for a broadcast comparison and evaluate rows only at
/// constraint insertion.  Shape-equal comparisons stay on their existing
/// sparse path; this handles the lower-rank broadcast case where materializing
/// either operand would allocate one expression per target row.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_broadcast_compare(
    left: constraint_array::BroadcastCompareOperand,
    left_shape: &[usize],
    left_index_sets: &[Py<PyIndexSet>],
    right: constraint_array::BroadcastCompareOperand,
    right_shape: &[usize],
    right_index_sets: &[Py<PyIndexSet>],
    sense: ComparisonSense,
) -> PyResult<Option<PyConstraintArray>> {
    if left_shape == right_shape {
        return Ok(None);
    }

    let left_labeled = labeled_shape_from_index_sets(left_index_sets)?;
    let right_labeled = labeled_shape_from_index_sets(right_index_sets)?;
    let (target_shape, target_index_sets, left_plan, right_plan) = if let Ok(right_plan) =
        BroadcastPlan::new(right_labeled.clone(), left_labeled.clone())
    {
        let left_plan = BroadcastPlan::new(left_labeled.clone(), left_labeled)
            .map_err(|err| ArrayDimensionError::new_err(err.to_string()))?;
        (left_shape.to_vec(), left_index_sets, left_plan, right_plan)
    } else if let Ok(left_plan) = BroadcastPlan::new(left_labeled.clone(), right_labeled.clone()) {
        let right_plan = BroadcastPlan::new(right_labeled.clone(), right_labeled)
            .map_err(|err| ArrayDimensionError::new_err(err.to_string()))?;
        (
            right_shape.to_vec(),
            right_index_sets,
            left_plan,
            right_plan,
        )
    } else {
        return Ok(None);
    };

    let sparse = Python::attach(|py| left.is_sparse(py) || right.is_sparse(py));
    if !sparse {
        return Ok(None);
    }

    Ok(Some(PyConstraintArray::from_broadcast_lazy_compare(
        left,
        right,
        sense,
        target_shape,
        Python::attach(|py| {
            target_index_sets
                .iter()
                .map(|index_set| index_set.clone_ref(py))
                .collect()
        }),
        left_plan,
        right_plan,
    )))
}

/// Extract a LinearArrayCore from a PyAny that is either a VariableArray or ExprArray.
fn extract_array_core(other: &Bound<'_, PyAny>) -> PyResult<LinearArrayCore> {
    if let Ok(va) = other.extract::<PyRef<'_, PyVariableArray>>() {
        return Ok(va.to_core());
    }
    if let Ok(ea) = other.extract::<PyRef<'_, PyExprArray>>() {
        return Ok(ea.to_core());
    }
    Err(ArrayTypeError::new_err(
        "expected VariableArray or ExprArray",
    ))
}

/// Compare a LinearArrayCore with a Python RHS, returning a ConstraintArray.
fn compare_array_rhs(
    core: &LinearArrayCore,
    rhs: &Bound<'_, PyAny>,
    sense: ComparisonSense,
) -> PyResult<PyConstraintArray> {
    if let Ok(other) = rhs.extract::<PyRef<'_, PyVariableArray>>() {
        let other_core = other.to_core();
        return core.compare_core(&other_core, sense);
    }
    if let Ok(other) = rhs.extract::<PyRef<'_, PyExprArray>>() {
        let other_core = other.to_core();
        return core.compare_core(&other_core, sense);
    }
    if let Ok(index_set) = rhs.extract::<PyRef<'_, PyIndexSet>>() {
        return core.compare_index_set(&index_set, sense);
    }
    if let Ok(rhs) = rhs.extract::<f64>() {
        return Ok(core.compare_scalar(rhs, sense));
    }
    if let Some(rhs_values) =
        extract_labeled_numeric_values(rhs.py(), rhs, &core.index_sets, core.values.len())?
    {
        return core.compare_vec(&rhs_values, sense);
    }
    if let Ok(rhs_values) = rhs.extract::<Vec<f64>>() {
        return core.compare_vec(&rhs_values, sense);
    }
    Err(ArrayTypeError::new_err(
        "comparison RHS must be a float, list of floats, VariableArray, ExprArray, labeled param, or IndexSet",
    ))
}

/// If shape is empty, fold values to a scalar; otherwise wrap in ExprArray.
fn reduce_or_wrap(
    values: Vec<PyExpr>,
    shape: Vec<usize>,
    index_sets: Vec<Py<PyIndexSet>>,
    py: Python<'_>,
) -> PyResult<PyObject> {
    if shape.is_empty() {
        let mut acc = PyExpr::default();
        for v in values {
            acc.add_assign_owned(v);
        }
        Ok(acc.into_pyobject(py)?.into_any().unbind())
    } else {
        let arr = PyExprArray::new(index_sets, shape, values);
        Ok(arr.into_pyobject(py)?.into_any().unbind())
    }
}

/// Sum elements of a core, optionally over one or more index sets.
fn array_sum(
    core: &LinearArrayCore,
    py: Python<'_>,
    over: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyObject> {
    let Some(over) = over else {
        return Ok(core.sum_all().into_pyobject(py)?.into_any().unbind());
    };

    let axes_to_sum = parse_named_axes(core, py, over)?;

    let mut current_values = core.values.clone();
    let mut current_shape = core.shape.clone();
    let mut current_index_sets = core.clone_index_sets();

    for axis in axes_to_sum {
        let new_values = sum_over_axis(&current_values, &current_shape, axis);
        current_shape.remove(axis);
        current_index_sets.remove(axis);
        current_values = new_values;
    }

    reduce_or_wrap(current_values, current_shape, current_index_sets, py)
}

/// Reduction operator: sum over the axis matching the given IndexSet.
fn array_reduce(
    core: &LinearArrayCore,
    py: Python<'_>,
    rhs: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    array_sum(core, py, Some(rhs))
}

/// Wrap a LinearArrayCore into a full-storage PyExprArray.
fn wrap_core(core: LinearArrayCore) -> PyExprArray {
    PyExprArray {
        storage: ExprArrayStorage::Full(core),
    }
}

/// Element-wise addition of a core with a Python operand.
fn array_add(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(other_core) = extract_array_core(other) {
        let result = core.combine(&other_core, |left, right| left.add(right.clone()))?;
        return Ok(wrap_core(result));
    }
    if let Some(values) =
        extract_labeled_numeric_values(other.py(), other, &core.index_sets, core.values.len())?
    {
        return Ok(wrap_core(core.add_vec(&values)?));
    }
    if let Ok(value) = other.extract::<f64>() {
        return Ok(wrap_core(core.add_scalar(value)));
    }
    let values: Vec<f64> = other.extract()?;
    Ok(wrap_core(core.add_vec(&values)?))
}

/// Element-wise subtraction: core - other.
fn array_sub(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(other_core) = extract_array_core(other) {
        let result = core.combine(&other_core, |left, right| left.add(right.scale(-1.0)))?;
        return Ok(wrap_core(result));
    }
    if let Some(values) =
        extract_labeled_numeric_values(other.py(), other, &core.index_sets, core.values.len())?
    {
        return Ok(wrap_core(core.sub_vec(&values)?));
    }
    if let Ok(value) = other.extract::<f64>() {
        return Ok(wrap_core(core.sub_scalar(value)));
    }
    let values: Vec<f64> = other.extract()?;
    Ok(wrap_core(core.sub_vec(&values)?))
}

/// Element-wise reverse subtraction: other - core.
fn array_rsub(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(other_core) = extract_array_core(other) {
        let result = other_core.combine(core, |left, right| left.add(right.scale(-1.0)))?;
        return Ok(wrap_core(result));
    }
    if let Some(values) =
        extract_labeled_numeric_values(other.py(), other, &core.index_sets, core.values.len())?
    {
        return Ok(wrap_core(core.rsub_vec(&values)?));
    }
    if let Ok(value) = other.extract::<f64>() {
        return Ok(wrap_core(core.rsub_scalar(value)));
    }
    let values: Vec<f64> = other.extract()?;
    Ok(wrap_core(core.rsub_vec(&values)?))
}

/// Element-wise multiplication of a core with a scalar or vector.
fn array_mul(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(scalar) = other.extract::<f64>() {
        return Ok(wrap_core(core.scale_all(scalar)));
    }
    if let Some(result) = multiply_with_labeled_union(core, other.py(), other)? {
        return Ok(wrap_core(result));
    }
    if let Some(weights) =
        extract_labeled_numeric_values(other.py(), other, &core.index_sets, core.values.len())?
    {
        return Ok(wrap_core(core.mul_vec(&weights)?));
    }
    let weights: Vec<f64> = other.extract()?;
    Ok(wrap_core(core.mul_vec(&weights)?))
}

/// Division of a core by a scalar.
fn array_truediv(core: &LinearArrayCore, other: f64) -> PyResult<PyExprArray> {
    if other == 0.0 {
        return Err(ExprDivisionByZeroError::new_err("division by zero"));
    }
    Ok(wrap_core(core.scale_all(1.0 / other)))
}

/// Negate all elements in a core.
fn array_neg(core: &LinearArrayCore) -> PyExprArray {
    wrap_core(core.scale_all(-1.0))
}

/// np.diag(array, k): extract the k-th diagonal from a 2D array core.
fn numpy_diag(py: Python<'_>, core: &LinearArrayCore, k: i64) -> PyResult<PyObject> {
    if core.shape.len() != 2 {
        return Err(ArrayDimensionError::new_err("np.diag requires a 2D array"));
    }
    let nrows = core.shape[0];
    let ncols = core.shape[1];

    let (start_row, start_col) = if k >= 0 {
        (0usize, k as usize)
    } else {
        ((-k) as usize, 0usize)
    };

    let diag_len = {
        let max_row = nrows.saturating_sub(start_row);
        let max_col = ncols.saturating_sub(start_col);
        max_row.min(max_col)
    };

    if diag_len == 0 {
        return Err(ArrayIndexError::new_err(
            "diagonal offset k is out of range",
        ));
    }

    let mut diag_values = Vec::with_capacity(diag_len);

    for i in 0..diag_len {
        let row = start_row + i;
        let col = start_col + i;
        let flat_idx = row * ncols + col;
        diag_values.push(core.values[flat_idx].clone());
    }

    let diag_index_set = PyIndexSet {
        name: format!("diag_{}", k),
        members: (0..diag_len)
            .map(|i| crate::py_modules::index_set::IndexMember::Int(i as i64))
            .collect(),
    };
    let diag_index_set_py = Py::new(py, diag_index_set)?;

    let result = PyExprArray::new(vec![diag_index_set_py], vec![diag_len], diag_values);
    Ok(result.into_pyobject(py)?.into_any().unbind())
}

/// np.fliplr(array): flip a 2D array core left-to-right.
fn numpy_fliplr(py: Python<'_>, core: &LinearArrayCore) -> PyResult<PyObject> {
    if core.shape.len() != 2 {
        return Err(ArrayDimensionError::new_err(
            "np.fliplr requires a 2D array",
        ));
    }
    let nrows = core.shape[0];
    let ncols = core.shape[1];

    let mut new_values = Vec::with_capacity(core.values.len());

    for row in 0..nrows {
        for col in (0..ncols).rev() {
            let flat_idx = row * ncols + col;
            new_values.push(core.values[flat_idx].clone());
        }
    }

    let result = PyExprArray::new(core.clone_index_sets(), core.shape.clone(), new_values);
    Ok(result.into_pyobject(py)?.into_any().unbind())
}

/// Convert a PyExprArray to a PyObject.
fn expr_array_to_pyobject(arr: PyExprArray, py: Python<'_>) -> PyResult<PyObject> {
    Ok(arr.into_pyobject(py)?.into_any().unbind())
}

fn slice_index_set(
    py: Python<'_>,
    index_set: &Py<PyIndexSet>,
    selected: &[usize],
) -> PyResult<Py<PyIndexSet>> {
    let borrowed = index_set.bind(py).borrow();
    let members = selected
        .iter()
        .map(|idx| borrowed.members[*idx].clone())
        .collect::<Vec<_>>();
    Py::new(
        py,
        PyIndexSet {
            name: borrowed.name.clone(),
            members,
        },
    )
}

fn cumsum_over_axis(values: &[PyExpr], shape: &[usize], axis: usize) -> Vec<PyExpr> {
    let ndim = shape.len();
    let axis_size = shape[axis];
    let outer: usize = shape[..axis].iter().product();
    let inner: usize = shape[axis + 1..ndim].iter().product();
    let mut out = vec![PyExpr::default(); values.len()];

    for o in 0..outer {
        for i in 0..inner {
            let mut acc = PyExpr::default();
            for a in 0..axis_size {
                let idx = o * axis_size * inner + a * inner + i;
                acc.add_assign(&values[idx]);
                out[idx] = acc.clone();
            }
        }
    }

    out
}

fn diff_over_axis(
    values: &[PyExpr],
    shape: &[usize],
    axis: usize,
) -> (Vec<PyExpr>, Vec<usize>, Vec<usize>) {
    let ndim = shape.len();
    let axis_size = shape[axis];
    let outer: usize = shape[..axis].iter().product();
    let inner: usize = shape[axis + 1..ndim].iter().product();
    let mut out = Vec::with_capacity(outer * axis_size.saturating_sub(1) * inner);

    for o in 0..outer {
        for a in 1..axis_size {
            for i in 0..inner {
                let current_idx = o * axis_size * inner + a * inner + i;
                let prev_idx = current_idx - inner;
                out.push(values[current_idx].add(values[prev_idx].scale(-1.0)));
            }
        }
    }

    let mut new_shape = shape.to_vec();
    new_shape[axis] = axis_size.saturating_sub(1);
    let selected = (1..axis_size).collect();
    (out, new_shape, selected)
}

fn roll_over_axis(values: &[PyExpr], shape: &[usize], axis: usize, shift: isize) -> Vec<PyExpr> {
    let ndim = shape.len();
    let axis_size = shape[axis];
    let outer: usize = shape[..axis].iter().product();
    let inner: usize = shape[axis + 1..ndim].iter().product();
    let mut out = vec![PyExpr::default(); values.len()];
    let normalized = shift.rem_euclid(axis_size as isize) as usize;

    for o in 0..outer {
        for a in 0..axis_size {
            let shifted = (a + normalized) % axis_size;
            for i in 0..inner {
                let src_idx = o * axis_size * inner + a * inner + i;
                let dst_idx = o * axis_size * inner + shifted * inner + i;
                out[dst_idx] = values[src_idx].clone();
            }
        }
    }

    out
}

fn extract_axis_kwarg<'py>(
    kwargs: &'py Bound<'py, PyAny>,
    name: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    if kwargs.is_none() {
        return Ok(None);
    }
    let dict = kwargs.cast::<PyDict>()?;
    dict.get_item(name)
}

fn reject_unsupported_numpy_kwargs(
    function_name: &str,
    kwargs: &Bound<'_, PyAny>,
    supported: &[&str],
) -> PyResult<()> {
    if kwargs.is_none() {
        return Ok(());
    }
    let dict = kwargs.cast::<PyDict>()?;
    for (key, value) in dict.iter() {
        let key = key.extract::<String>()?;
        if supported.contains(&key.as_str()) || value.is_none() {
            continue;
        }
        return Err(ArrayTypeError::new_err(format!(
            "np.{function_name} with Arco arrays supports only {}; unsupported keyword '{key}'",
            supported.join(", ")
        )));
    }
    Ok(())
}

fn numpy_cumsum(
    py: Python<'_>,
    core: &LinearArrayCore,
    kwargs: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let axis = extract_axis_kwarg(kwargs, "axis")?
        .ok_or_else(|| ArrayDimensionError::new_err("np.cumsum requires axis=IndexSet"))?;
    let axes = parse_named_axes(core, py, &axis)?;
    if axes.len() != 1 {
        return Err(ArrayDimensionError::new_err(
            "np.cumsum requires exactly one IndexSet axis",
        ));
    }
    let result = PyExprArray::new(
        core.clone_index_sets(),
        core.shape.clone(),
        cumsum_over_axis(&core.values, &core.shape, axes[0]),
    );
    expr_array_to_pyobject(result, py)
}

fn numpy_diff(
    py: Python<'_>,
    core: &LinearArrayCore,
    kwargs: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let axis = extract_axis_kwarg(kwargs, "axis")?
        .ok_or_else(|| ArrayDimensionError::new_err("np.diff requires axis=IndexSet"))?;
    let axes = parse_named_axes(core, py, &axis)?;
    if axes.len() != 1 {
        return Err(ArrayDimensionError::new_err(
            "np.diff requires exactly one IndexSet axis",
        ));
    }
    let axis = axes[0];
    let (values, shape, selected) = diff_over_axis(&core.values, &core.shape, axis);
    let mut index_sets = core.clone_index_sets();
    index_sets[axis] = slice_index_set(py, &core.index_sets[axis], &selected)?;
    expr_array_to_pyobject(PyExprArray::new(index_sets, shape, values), py)
}

fn numpy_roll(
    py: Python<'_>,
    core: &LinearArrayCore,
    args: &Bound<'_, PyTuple>,
    kwargs: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let shift = if args.len() > 1 {
        args.get_item(1)?.extract::<isize>()?
    } else {
        extract_axis_kwarg(kwargs, "shift")?
            .ok_or_else(|| ArrayDimensionError::new_err("np.roll requires shift"))?
            .extract::<isize>()?
    };
    let axis = extract_axis_kwarg(kwargs, "axis")?
        .ok_or_else(|| ArrayDimensionError::new_err("np.roll requires axis=IndexSet"))?;
    let axes = parse_named_axes(core, py, &axis)?;
    if axes.len() != 1 {
        return Err(ArrayDimensionError::new_err(
            "np.roll requires exactly one IndexSet axis",
        ));
    }
    let result = PyExprArray::new(
        core.clone_index_sets(),
        core.shape.clone(),
        roll_over_axis(&core.values, &core.shape, axes[0], shift),
    );
    expr_array_to_pyobject(result, py)
}

pub(super) fn array_cumsum(
    core: &LinearArrayCore,
    py: Python<'_>,
    over: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let kwargs = PyDict::new(py);
    kwargs.set_item("axis", over)?;
    numpy_cumsum(py, core, kwargs.as_any())
}

pub(super) fn array_diff(
    core: &LinearArrayCore,
    py: Python<'_>,
    over: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let kwargs = PyDict::new(py);
    kwargs.set_item("axis", over)?;
    numpy_diff(py, core, kwargs.as_any())
}

pub(super) fn array_roll(
    core: &LinearArrayCore,
    py: Python<'_>,
    shift: isize,
    over: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let kwargs = PyDict::new(py);
    kwargs.set_item("axis", over)?;
    kwargs.set_item("shift", shift)?;
    let args = PyTuple::new(py, [shift])?;
    numpy_roll(py, core, &args, kwargs.as_any())
}

/// Handle numpy ufuncs on a LinearArrayCore.
fn array_ufunc(
    core: &LinearArrayCore,
    py: Python<'_>,
    is_self: impl Fn(&Bound<'_, PyAny>) -> bool,
    ufunc: &Bound<'_, PyAny>,
    method: &str,
    inputs: &Bound<'_, PyTuple>,
    _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<PyObject> {
    if method != "__call__" || inputs.len() != 2 {
        return Ok(py.NotImplemented().into_pyobject(py)?.unbind());
    }

    let ufunc_name = ufunc.getattr("__name__")?.extract::<String>()?;
    let a = inputs.get_item(0)?;
    let b = inputs.get_item(1)?;
    let other = if is_self(&a) { &b } else { &a };

    match ufunc_name.as_str() {
        "multiply" => {
            let weights = if let Some(values) =
                extract_labeled_numeric_values(py, other, &core.index_sets, core.values.len())?
            {
                values
            } else {
                let np = py.import("numpy")?;
                let flat = np
                    .call_method1("asarray", (other,))?
                    .call_method0("flatten")?;
                flat.extract()?
            };
            expr_array_to_pyobject(wrap_core(core.mul_vec(&weights)?), py)
        }
        "add" => expr_array_to_pyobject(array_add(core, other)?, py),
        "subtract" => {
            if is_self(&a) {
                expr_array_to_pyobject(array_sub(core, &b)?, py)
            } else {
                expr_array_to_pyobject(array_rsub(core, &a)?, py)
            }
        }
        "matmul" => numpy_matmul(py, inputs),
        _ => Ok(py.NotImplemented().into_pyobject(py)?.unbind()),
    }
}

/// Handle numpy __array_function__ protocol on a LinearArrayCore.
fn array_function(
    core: &LinearArrayCore,
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    _types: &Bound<'_, PyAny>,
    args: &Bound<'_, PyTuple>,
    kwargs: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let func_name = func.getattr("__name__")?.extract::<String>()?;
    let axis = extract_axis_kwarg(kwargs, "axis")?;

    match func_name.as_str() {
        "sum" => {
            reject_unsupported_numpy_kwargs("sum", kwargs, &["axis"])?;
            array_sum(core, py, axis.as_ref())
        }
        "cumsum" => numpy_cumsum(py, core, kwargs),
        "diff" => numpy_diff(py, core, kwargs),
        "roll" => numpy_roll(py, core, args, kwargs),
        "einsum" => numpy_einsum(py, args, kwargs),
        "dot" => numpy_dot(py, args),
        "matmul" => numpy_matmul(py, args),
        "diag" => {
            let k: i64 = if args.len() > 1 {
                args.get_item(1)?.extract()?
            } else if !kwargs.is_none() {
                let kw = kwargs.cast::<pyo3::types::PyDict>()?;
                kw.get_item("k")?
                    .map(|v| v.extract())
                    .transpose()?
                    .unwrap_or(0)
            } else {
                0
            };
            numpy_diag(py, core, k)
        }
        "fliplr" => numpy_fliplr(py, core),
        "concatenate" => numpy_concatenate(py, core, args, kwargs),
        _ => Ok(py.NotImplemented().into_pyobject(py)?.unbind()),
    }
}

/// Compute `sum(weights[i] * exprs[i])` into a single PyExpr.
///
/// Both slices must have the same length (caller is responsible for validation).
fn weighted_sum(weights: &[f64], exprs: &[PyExpr]) -> PyExpr {
    let mut acc = PyExpr::default();
    for (w, expr) in weights.iter().zip(exprs.iter()) {
        acc.add_assign_owned(expr.scale(*w));
    }
    acc
}

/// np.dot(a, b): weighted sum of 1D arrays (one ndarray, one VariableArray/ExprArray).
fn numpy_dot(py: Python<'_>, args: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
    if args.len() != 2 {
        return Err(ArrayDimensionError::new_err(
            "np.dot requires exactly 2 arguments",
        ));
    }
    let a = args.get_item(0)?;
    let b = args.get_item(1)?;

    // Determine which argument has the linear array core
    let (weights, core) = if let Ok(va) = b.extract::<PyRef<'_, PyVariableArray>>() {
        let w: Vec<f64> = a.extract()?;
        (w, va.to_core())
    } else if let Ok(ea) = b.extract::<PyRef<'_, PyExprArray>>() {
        let w: Vec<f64> = a.extract()?;
        (w, ea.to_core())
    } else if let Ok(va) = a.extract::<PyRef<'_, PyVariableArray>>() {
        let w: Vec<f64> = b.extract()?;
        (w, va.to_core())
    } else if let Ok(ea) = a.extract::<PyRef<'_, PyExprArray>>() {
        let w: Vec<f64> = b.extract()?;
        (w, ea.to_core())
    } else {
        return Err(ArrayTypeError::new_err(
            "np.dot requires one VariableArray/ExprArray and one array-like",
        ));
    };

    if core.shape.len() != 1 {
        return Err(ArrayDimensionError::new_err(
            "np.dot only supports 1D arrays",
        ));
    }
    if weights.len() != core.values.len() {
        return Err(ArrayShapeMismatchError::new_err(format!(
            "np.dot array lengths must match ({} vs {})",
            weights.len(),
            core.values.len()
        )));
    }

    let acc = weighted_sum(&weights, &core.values);
    Ok(acc.into_pyobject(py)?.into_any().unbind())
}

/// np.matmul(a, b): matrix-vector multiplication.
fn numpy_matmul(py: Python<'_>, args: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
    if args.len() != 2 {
        return Err(ArrayDimensionError::new_err(
            "np.matmul requires exactly 2 arguments",
        ));
    }

    let a = args.get_item(0)?;
    let b = args.get_item(1)?;

    // Extract the core and determine order
    let (ndarray_arg, core, variable_array_on_left) =
        if let Ok(va) = b.extract::<PyRef<'_, PyVariableArray>>() {
            (a.clone(), va.to_core(), false)
        } else if let Ok(ea) = b.extract::<PyRef<'_, PyExprArray>>() {
            (a.clone(), ea.to_core(), false)
        } else if let Ok(va) = a.extract::<PyRef<'_, PyVariableArray>>() {
            (b.clone(), va.to_core(), true)
        } else if let Ok(ea) = a.extract::<PyRef<'_, PyExprArray>>() {
            (b.clone(), ea.to_core(), true)
        } else {
            return Err(ArrayTypeError::new_err(
                "np.matmul requires one VariableArray/ExprArray and one array-like",
            ));
        };

    if core.shape.len() != 1 {
        return Err(ArrayDimensionError::new_err(
            "np.matmul currently supports only 1D VariableArray/ExprArray",
        ));
    }

    let np = py.import("numpy")?;
    let ndarray = np.call_method1("asarray", (&ndarray_arg,))?;
    let ndim: usize = ndarray.getattr("ndim")?.extract()?;
    let n = core.values.len();

    match ndim {
        1 => {
            let weights: Vec<f64> = ndarray.extract()?;
            if weights.len() != n {
                return Err(ArrayShapeMismatchError::new_err(format!(
                    "np.matmul 1D length mismatch ({} vs {})",
                    weights.len(),
                    n
                )));
            }
            let acc = weighted_sum(&weights, &core.values);
            Ok(acc.into_pyobject(py)?.into_any().unbind())
        }
        2 => {
            if variable_array_on_left {
                return Err(ArrayTypeError::new_err(
                    "VariableArray/ExprArray @ 2D ndarray is not supported; use ndarray @ array",
                ));
            }

            let shape: Vec<usize> = ndarray.getattr("shape")?.extract()?;
            let rows = shape[0];
            let cols = shape[1];
            if cols != n {
                return Err(ArrayShapeMismatchError::new_err(format!(
                    "matrix columns {} must match array length {}",
                    cols, n
                )));
            }

            let flat = ndarray.call_method0("flatten")?;
            let weights: Vec<f64> = flat.extract()?;
            let mut values = Vec::with_capacity(rows);

            for row_weights in weights.chunks(cols) {
                values.push(weighted_sum(row_weights, &core.values));
            }

            let result = PyExprArray::new(Vec::new(), vec![rows], values);
            Ok(result.into_pyobject(py)?.into_any().unbind())
        }
        _ => Err(ArrayDimensionError::new_err(
            "np.matmul supports only 1D or 2D array-like inputs",
        )),
    }
}

/// np.concatenate(arrays): concatenate a sequence of arrays and/or scalar arrays.
fn numpy_concatenate(
    py: Python<'_>,
    core: &LinearArrayCore,
    args: &Bound<'_, PyTuple>,
    kwargs: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    if args.is_empty() {
        return Err(ArrayDimensionError::new_err(
            "np.concatenate requires at least one argument",
        ));
    }
    let seq = args.get_item(0)?;
    let items: Vec<Bound<'_, PyAny>> = seq.try_iter()?.collect::<PyResult<Vec<_>>>()?;
    let axis = extract_axis_kwarg(kwargs, "axis")?;
    let axis = if let Some(axis) = axis {
        let axes = parse_named_axes(core, py, &axis)?;
        if axes.len() != 1 {
            return Err(ArrayDimensionError::new_err(
                "np.concatenate requires exactly one IndexSet axis",
            ));
        }
        axes[0]
    } else {
        0
    };

    let np = py.import("numpy")?;
    let mut arrays = Vec::new();
    let mut has_array_operand = false;
    for item in &items {
        if let Ok(va) = item.extract::<PyRef<'_, PyVariableArray>>() {
            has_array_operand = true;
            arrays.push(va.to_core());
        } else if let Ok(ea) = item.extract::<PyRef<'_, PyExprArray>>() {
            has_array_operand = true;
            arrays.push(ea.to_core());
        } else {
            let ndarray = np.call_method1("asarray", (item,))?;
            let shape: Vec<usize> = ndarray.getattr("shape")?.extract()?;
            let flat = ndarray.call_method0("flatten")?;
            let floats: Vec<f64> = flat.extract()?;
            let values = floats
                .into_iter()
                .map(|value| PyExpr::from_expr(Expr::from_constant(value)))
                .collect();
            arrays.push(LinearArrayCore::new(Vec::new(), shape, values));
        }
    }

    if !has_array_operand {
        return Err(ArrayTypeError::new_err(
            "np.concatenate requires at least one VariableArray or ExprArray operand",
        ));
    }

    let first = arrays
        .first()
        .ok_or_else(|| ArrayTypeError::new_err("np.concatenate requires at least one array"))?;
    let reference = arrays
        .iter()
        .find(|array| !array.index_sets.is_empty())
        .unwrap_or(first);
    let rank = reference.shape.len();
    for array in &arrays {
        if array.shape.len() != rank {
            return Err(ArrayDimensionError::new_err(
                "np.concatenate requires arrays with matching rank",
            ));
        }
        for dim in 0..rank {
            if dim != axis && array.shape[dim] != reference.shape[dim] {
                return Err(ArrayShapeMismatchError::new_err(
                    "np.concatenate requires matching non-concatenated dimensions",
                ));
            }
        }
    }

    let before: usize = reference.shape[..axis].iter().product();
    let after: usize = reference.shape[axis + 1..].iter().product();
    let axis_total: usize = arrays.iter().map(|array| array.shape[axis]).sum();
    let mut values = Vec::with_capacity(before * axis_total * after);

    for outer in 0..before {
        for array in &arrays {
            let axis_len = array.shape[axis];
            for idx in 0..axis_len {
                for inner in 0..after {
                    let flat = outer * axis_len * after + idx * after + inner;
                    values.push(array.values[flat].clone());
                }
            }
        }
    }

    let mut shape = reference.shape.clone();
    shape[axis] = axis_total;
    let mut index_sets = reference.clone_index_sets();
    let axis_members = arrays.iter().try_fold(Vec::new(), |mut acc, array| {
        if array.index_sets.is_empty() {
            for idx in 0..array.shape[axis] {
                acc.push(crate::py_modules::index_set::IndexMember::Int(idx as i64));
            }
        } else {
            let borrowed = array.index_sets[axis].bind(py).borrow();
            for member in &borrowed.members {
                acc.push(member.clone());
            }
        }
        Ok::<_, PyErr>(acc)
    })?;
    index_sets[axis] = Py::new(
        py,
        PyIndexSet {
            name: reference.index_sets[axis].bind(py).borrow().name.clone(),
            members: axis_members,
        },
    )?;
    let result = PyExprArray::new(index_sets, shape, values);
    Ok(result.into_pyobject(py)?.into_any().unbind())
}

fn parse_einsum_subscripts(
    subscripts: &str,
    operand_count: usize,
) -> PyResult<(Vec<Vec<char>>, Vec<char>)> {
    if subscripts.contains("...") {
        return Err(ArrayTypeError::new_err(
            "np.einsum with Arco arrays does not support ellipsis",
        ));
    }
    let (lhs, rhs) = subscripts.split_once("->").ok_or_else(|| {
        ArrayTypeError::new_err("np.einsum with Arco arrays requires explicit output subscripts")
    })?;
    let inputs = lhs
        .split(',')
        .map(|part| part.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if inputs.len() != operand_count {
        return Err(ArrayDimensionError::new_err(format!(
            "np.einsum expected {} operands but received {}",
            inputs.len(),
            operand_count
        )));
    }
    let output = rhs.chars().collect::<Vec<_>>();

    for term in inputs.iter().chain(std::iter::once(&output)) {
        let mut seen = BTreeSet::new();
        for label in term {
            if !label.is_ascii_alphabetic() {
                return Err(ArrayTypeError::new_err(
                    "np.einsum with Arco arrays only supports alphabetic subscripts",
                ));
            }
            if !seen.insert(*label) {
                return Err(ArrayTypeError::new_err(
                    "np.einsum with Arco arrays does not support repeated subscripts in one operand",
                ));
            }
        }
    }

    Ok((inputs, output))
}

fn einsum_shape_from_operand(py: Python<'_>, operand: &Bound<'_, PyAny>) -> PyResult<Vec<usize>> {
    if let Ok(va) = operand.extract::<PyRef<'_, PyVariableArray>>() {
        return Ok(va.get_shape().to_vec());
    }
    if let Ok(ea) = operand.extract::<PyRef<'_, PyExprArray>>() {
        return Ok(ea.storage.shape().to_vec());
    }

    let np = py.import("numpy")?;
    let array = np.call_method1("asarray", (operand,))?;
    array.getattr("shape")?.extract()
}

fn numpy_einsum(
    py: Python<'_>,
    args: &Bound<'_, PyTuple>,
    kwargs: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    if args.len() < 2 {
        return Err(ArrayDimensionError::new_err(
            "np.einsum requires a subscript string and at least one operand",
        ));
    }

    let subscripts = args.get_item(0)?.extract::<String>()?;
    let operands = args.iter().skip(1).collect::<Vec<Bound<'_, PyAny>>>();
    let (input_terms, output_term) = parse_einsum_subscripts(&subscripts, operands.len())?;
    let kwargs_dict = if kwargs.is_none() {
        None
    } else {
        Some(kwargs.cast::<PyDict>()?)
    };

    let linear_operands = operands
        .iter()
        .enumerate()
        .filter_map(|(idx, operand)| extract_array_core(operand).ok().map(|core| (idx, core)))
        .collect::<Vec<_>>();
    if linear_operands.len() > 1 {
        return Err(ArrayTypeError::new_err(
            "np.einsum with Arco arrays supports exactly one VariableArray or ExprArray operand",
        ));
    }
    if linear_operands.is_empty() {
        let np = py.import("numpy")?;
        let dense_args = operands
            .iter()
            .map(|operand| np.call_method1("asarray", (operand,)))
            .collect::<PyResult<Vec<_>>>()?;
        let mut call_args = Vec::with_capacity(dense_args.len() + 1);
        call_args.push(subscripts.into_pyobject(py)?.into_any().unbind());
        for operand in dense_args {
            call_args.push(operand.into_any().unbind());
        }
        return np
            .getattr("einsum")?
            .call(PyTuple::new(py, call_args)?, kwargs_dict)
            .map(|value| value.unbind());
    }

    let (expr_index, core) = &linear_operands[0];
    let expr_index = *expr_index;
    let expr_spec = &input_terms[expr_index];
    if expr_spec.len() != core.shape.len() {
        return Err(ArrayDimensionError::new_err(format!(
            "np.einsum subscript rank {} does not match Arco array rank {}",
            expr_spec.len(),
            core.shape.len()
        )));
    }

    let mut label_sizes = BTreeMap::<char, usize>::new();
    for (term, operand) in input_terms.iter().zip(operands.iter()) {
        let shape = einsum_shape_from_operand(py, operand)?;
        if shape.len() != term.len() {
            return Err(ArrayDimensionError::new_err(format!(
                "np.einsum operand rank {} does not match subscript rank {}",
                shape.len(),
                term.len()
            )));
        }
        for (label, size) in term.iter().zip(shape.iter()) {
            if let Some(existing) = label_sizes.insert(*label, *size) {
                if existing != *size {
                    return Err(ArrayShapeMismatchError::new_err(format!(
                        "np.einsum label '{}' has incompatible dimensions {} and {}",
                        label, existing, size
                    )));
                }
            }
        }
    }

    let numeric_specs = input_terms
        .iter()
        .enumerate()
        .filter_map(|(idx, term)| (idx != expr_index).then_some(term))
        .map(|term| term.iter().collect::<String>())
        .collect::<Vec<_>>();
    let coeff_labels = {
        let mut labels = output_term.clone();
        for label in expr_spec {
            if !labels.contains(label) {
                labels.push(*label);
            }
        }
        labels
    };
    for label in &output_term {
        if !label_sizes.contains_key(label) {
            return Err(ArrayDimensionError::new_err(format!(
                "np.einsum output label '{}' does not appear in any input term",
                label
            )));
        }
    }

    let coeff_shape = coeff_labels
        .iter()
        .map(|label| label_sizes[label])
        .collect::<Vec<_>>();
    let coeff_source_labels = coeff_labels
        .iter()
        .copied()
        .filter(|label| {
            numeric_specs
                .iter()
                .any(|spec| spec.chars().any(|candidate| candidate == *label))
        })
        .collect::<Vec<_>>();

    let coeffs = if numeric_specs.is_empty() {
        vec![1.0; coeff_shape.iter().product::<usize>().max(1)]
    } else {
        let np = py.import("numpy")?;
        let numeric_inputs = operands
            .iter()
            .enumerate()
            .filter_map(|(idx, operand)| (idx != expr_index).then_some(operand))
            .map(|operand| np.call_method1("asarray", (operand,)))
            .collect::<PyResult<Vec<_>>>()?;
        let coeff_subscripts = format!(
            "{}->{}",
            numeric_specs.join(","),
            coeff_source_labels.iter().collect::<String>()
        );
        let mut call_args = Vec::with_capacity(numeric_inputs.len() + 1);
        call_args.push(coeff_subscripts.into_pyobject(py)?.into_any().unbind());
        for operand in numeric_inputs {
            call_args.push(operand.into_any().unbind());
        }
        let result = np
            .getattr("einsum")?
            .call(PyTuple::new(py, call_args)?, kwargs_dict)?;
        let array = np.call_method1("asarray", (result,))?;
        let flat = array.call_method0("flatten")?;
        let dense = flat.extract::<Vec<f64>>()?;
        if coeff_source_labels == coeff_labels {
            dense
        } else {
            let source_shape = LabeledShape::new(
                coeff_source_labels
                    .iter()
                    .map(|label| AxisSpec::new(label.to_string(), label_sizes[label]))
                    .collect(),
            )
            .map_err(|err| ArrayDimensionError::new_err(err.to_string()))?;
            let target_shape = LabeledShape::new(
                coeff_labels
                    .iter()
                    .map(|label| AxisSpec::new(label.to_string(), label_sizes[label]))
                    .collect(),
            )
            .map_err(|err| ArrayDimensionError::new_err(err.to_string()))?;
            BroadcastPlan::new(source_shape, target_shape)
                .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?
                .broadcast_dense(&dense)
                .map_err(|err| ArrayShapeMismatchError::new_err(err.to_string()))?
        }
    };

    let coeff_strides = arco_arrays::row_major_strides(&coeff_shape);
    let expr_strides = arco_arrays::row_major_strides(&core.shape);
    let output_shape = output_term
        .iter()
        .map(|label| label_sizes[label])
        .collect::<Vec<_>>();
    let output_strides = arco_arrays::row_major_strides(&output_shape);
    let output_len = output_shape.iter().product::<usize>().max(1);
    let mut output_values = vec![PyExpr::default(); output_len];

    let label_positions = coeff_labels
        .iter()
        .enumerate()
        .map(|(idx, label)| (*label, idx))
        .collect::<BTreeMap<_, _>>();

    for (flat_idx, coefficient) in coeffs.iter().copied().enumerate() {
        if coefficient == 0.0 {
            continue;
        }
        let mut remainder = flat_idx;
        let mut coords = vec![0usize; coeff_shape.len()];
        for (idx, stride) in coeff_strides.iter().enumerate() {
            coords[idx] = if *stride == 0 { 0 } else { remainder / stride };
            remainder %= stride;
        }

        let expr_flat = expr_spec
            .iter()
            .enumerate()
            .map(|(axis, label)| {
                label_positions
                    .get(label)
                    .map(|position| coords[*position] * expr_strides[axis])
                    .ok_or_else(|| {
                        ArrayDimensionError::new_err(format!(
                            "np.einsum label '{}' is missing from coefficient shape",
                            label
                        ))
                    })
            })
            .collect::<PyResult<Vec<_>>>()?
            .into_iter()
            .sum::<usize>();

        let output_flat = if output_shape.is_empty() {
            0
        } else {
            output_term
                .iter()
                .enumerate()
                .map(|(axis, label)| {
                    label_positions
                        .get(label)
                        .map(|position| coords[*position] * output_strides[axis])
                        .ok_or_else(|| {
                            ArrayDimensionError::new_err(format!(
                                "np.einsum output label '{}' is missing from coefficient shape",
                                label
                            ))
                        })
                })
                .collect::<PyResult<Vec<_>>>()?
                .into_iter()
                .sum::<usize>()
        };
        let expr = core.values.get(expr_flat).ok_or_else(|| {
            ArrayIndexError::new_err(format!(
                "np.einsum expression flat index {} out of range for array of size {}",
                expr_flat,
                core.values.len()
            ))
        })?;
        let output_len = output_values.len();
        let output = output_values.get_mut(output_flat).ok_or_else(|| {
            ArrayIndexError::new_err(format!(
                "np.einsum output flat index {} out of range for array of size {}",
                output_flat, output_len
            ))
        })?;
        output.add_assign_owned(expr.scale(coefficient));
    }

    if output_shape.is_empty() {
        let Some(output) = output_values.pop() else {
            return Err(ArrayDimensionError::new_err(
                "np.einsum scalar result did not produce an expression",
            ));
        };
        return Ok(output.into_pyobject(py)?.into_any().unbind());
    }

    let mut index_sets = Vec::with_capacity(output_term.len());
    for label in &output_term {
        if let Some(expr_axis) = expr_spec.iter().position(|candidate| candidate == label) {
            index_sets.push(core.index_sets[expr_axis].clone_ref(py));
            continue;
        }
        let size = label_sizes[label];
        index_sets.push(Py::new(
            py,
            PyIndexSet {
                name: label.to_string(),
                members: (0..size).map(|idx| IndexMember::Int(idx as i64)).collect(),
            },
        )?);
    }

    let result = PyExprArray::new(index_sets, output_shape, output_values);
    Ok(result.into_pyobject(py)?.into_any().unbind())
}

/// Register array classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVariableArray>()?;
    m.add_class::<PyExprArray>()?;
    m.add_class::<PyConstraintArray>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ExprArrayStorage, SparseDiffSource, SparseExprStorage, diff_sparse_expr};
    use crate::py_modules::index_set::{IndexMember, PyIndexSet};
    use pyo3::prelude::*;

    fn run_sparse_diff(
        py: Python<'_>,
        active_indices: &[usize],
        variable_ids: &[u32],
    ) -> PyResult<Py<PyAny>> {
        let axis = Py::new(
            py,
            PyIndexSet {
                name: "axis".to_string(),
                members: (0..4).map(IndexMember::Int).collect(),
            },
        )?;
        let index_sets = vec![axis.clone_ref(py)];
        diff_sparse_expr(
            &index_sets,
            &[4],
            active_indices,
            SparseDiffSource::VariableIds(variable_ids),
            py,
            axis.bind(py).as_any(),
        )
    }

    #[test]
    fn sparse_diff_output_storage_uses_exact_capacity() -> PyResult<()> {
        Python::initialize();
        Python::attach(|py| {
            let result = run_sparse_diff(py, &[1, 2, 3], &[10, 11, 12])?;
            let result = result.bind(py).extract::<PyRef<'_, super::PyExprArray>>()?;
            let ExprArrayStorage::Sparse { storage, .. } = &result.storage else {
                panic!("sparse diff should return sparse expression storage");
            };
            let SparseExprStorage::Eager {
                active_indices,
                values,
            } = storage
            else {
                panic!("sparse diff should return eager expression storage");
            };

            assert_eq!(active_indices.len(), 3);
            assert_eq!(values.len(), 3);
            assert_eq!(active_indices.capacity(), 3);
            assert_eq!(values.capacity(), 3);
            Ok(())
        })
    }

    #[test]
    fn sparse_diff_rejects_malformed_active_indices() -> PyResult<()> {
        Python::initialize();
        Python::attach(|py| {
            assert!(run_sparse_diff(py, &[1, 2, 3], &[10, 11]).is_err());
            assert!(run_sparse_diff(py, &[2, 1], &[10, 11]).is_err());
            assert!(run_sparse_diff(py, &[1, 1], &[10, 11]).is_err());
            assert!(run_sparse_diff(py, &[4], &[10]).is_err());
            Ok(())
        })
    }
}
