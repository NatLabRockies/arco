//! Python wrappers for variable, expression, and constraint arrays.

use arco_ops::expression::{ComparisonSense, Expr, VariableId};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyObject;
use crate::py_modules::errors::{
    ArrayDimensionError, ArrayIndexError, ArrayShapeMismatchError, ArrayTypeError,
    ExprDivisionByZeroError,
};
use crate::py_modules::expr::PyExpr;
use crate::py_modules::index_set::PyIndexSet;

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

// Re-export compact types for use in lib.rs
pub(crate) use constraint_array::CompactConstraintStorage;

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

/// Shared storage for indexed linear expression arrays.
/// Both VariableArray and ExprArray compose this internally.
pub(crate) struct LinearArrayCore {
    pub index_sets: Vec<Py<PyIndexSet>>,
    pub shape: Vec<usize>,
    pub values: Vec<PyExpr>,
}

impl LinearArrayCore {
    pub fn new(index_sets: Vec<Py<PyIndexSet>>, shape: Vec<usize>, values: Vec<PyExpr>) -> Self {
        Self {
            index_sets,
            shape,
            values,
        }
    }

    /// Clone the core, requires GIL for cloning Py<T>.
    pub fn clone_with_gil(&self) -> Self {
        Python::attach(|py| Self {
            index_sets: self.index_sets.iter().map(|s| s.clone_ref(py)).collect(),
            shape: self.shape.clone(),
            values: self.values.clone(),
        })
    }

    pub fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
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

    fn combine(
        &self,
        other: &LinearArrayCore,
        combine: fn(&PyExpr, &PyExpr) -> PyExpr,
    ) -> PyResult<LinearArrayCore> {
        self.assert_same_shape(other)?;
        let values = self
            .values
            .iter()
            .zip(other.values.iter())
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
        self.assert_same_shape(other)?;
        let mut exprs = Vec::with_capacity(self.values.len());
        let mut rhs = Vec::with_capacity(self.values.len());
        for (left, right) in self.values.iter().zip(other.values.iter()) {
            let diff = left.inner().add(&right.inner().scale(-1.0));
            let diff_expr = PyExpr::from_expr(diff);
            exprs.push(diff_expr.without_constant());
            rhs.push(-diff_expr.constant());
        }
        Ok(PyConstraintArray::new(
            exprs,
            sense,
            rhs,
            self.shape.clone(),
            self.clone_index_sets(),
        ))
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
        // Fallback: match by name and size
        let target_name = &index_set.borrow().name;
        let target_size = index_set.borrow().members.len();
        for (i, stored) in self.index_sets.iter().enumerate() {
            let stored_ref = stored.bind(py).borrow();
            if &stored_ref.name == target_name && stored_ref.members.len() == target_size {
                return Ok(i);
            }
        }
        Err(ArrayIndexError::new_err(format!(
            "IndexSet '{}' is not a dimension of this array",
            index_set.borrow().name
        )))
    }

    /// Perform sum over one axis, returning (new_values, new_shape, new_index_sets).
    fn sum_over_one(
        &self,
        py: Python<'_>,
        axis: usize,
    ) -> (Vec<PyExpr>, Vec<usize>, Vec<Py<PyIndexSet>>) {
        let new_values = sum_over_axis(&self.values, &self.shape, axis);
        let mut new_shape = self.shape.clone();
        new_shape.remove(axis);
        let new_index_sets: Vec<Py<PyIndexSet>> = self
            .index_sets
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != axis)
            .map(|(_, s)| s.clone_ref(py))
            .collect();
        (new_values, new_shape, new_index_sets)
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
pub(crate) struct CompactTerm {
    pub start_var_id: u32,
    pub coefficient: f64,
}

/// Compact expression storage: represents N elements with O(terms_per_element) memory.
/// For element `i`: `expr_i = constant + sum(term.coeff * var(term.start_var_id + i))`
#[derive(Clone, Debug)]
pub(crate) struct CompactExprStorage {
    pub terms: Vec<CompactTerm>,
    pub constant: f64,
    pub count: usize,
}

impl CompactExprStorage {
    /// Create from a single variable array (coefficient 1.0, constant 0.0).
    pub fn from_variable_array(start_var_id: u32, count: usize) -> Self {
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
    pub fn scale(&self, factor: f64) -> Self {
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
    pub fn add_compact(&self, other: &CompactExprStorage) -> Self {
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
    pub fn sub_compact(&self, other: &CompactExprStorage) -> Self {
        self.add_compact(&other.scale(-1.0))
    }

    /// Add a constant offset.
    pub fn add_constant(&self, value: f64) -> Self {
        Self {
            terms: self.terms.clone(),
            constant: self.constant + value,
            count: self.count,
        }
    }

    /// Materialize to LinearArrayCore (fallback).
    pub fn to_core(&self, index_sets: &[Py<PyIndexSet>], shape: &[usize]) -> LinearArrayCore {
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

    /// Sum all elements to a single PyExpr.
    pub fn sum_all(&self) -> PyExpr {
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
}

impl ExprArrayStorage {
    /// Materialize to LinearArrayCore.
    pub fn to_core(&self) -> LinearArrayCore {
        match self {
            ExprArrayStorage::Full(core) => core.clone_with_gil(),
            ExprArrayStorage::Compact {
                storage,
                index_sets,
                shape,
            } => storage.to_core(index_sets, shape),
        }
    }

    pub fn count(&self) -> usize {
        match self {
            ExprArrayStorage::Full(core) => core.values.len(),
            ExprArrayStorage::Compact { storage, .. } => storage.count,
        }
    }

    pub fn shape(&self) -> &[usize] {
        match self {
            ExprArrayStorage::Full(core) => &core.shape,
            ExprArrayStorage::Compact { shape, .. } => shape,
        }
    }

    pub fn index_sets_ref(&self) -> &[Py<PyIndexSet>] {
        match self {
            ExprArrayStorage::Full(core) => &core.index_sets,
            ExprArrayStorage::Compact { index_sets, .. } => index_sets,
        }
    }

    pub fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
        Python::attach(|py| {
            self.index_sets_ref()
                .iter()
                .map(|s| s.clone_ref(py))
                .collect()
        })
    }

    /// Get the compact storage if available.
    pub fn as_compact(&self) -> Option<&CompactExprStorage> {
        match self {
            ExprArrayStorage::Compact { storage, .. } => Some(storage),
            ExprArrayStorage::Full(_) => None,
        }
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
pub(crate) fn try_make_compact_constraint(
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
    if let Ok(rhs_values) = rhs.extract::<Vec<f64>>() {
        return core.compare_vec(&rhs_values, sense);
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "comparison RHS must be a float, list of floats, VariableArray, ExprArray, or IndexSet",
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

    let mut axes_to_sum: Vec<usize> = Vec::new();

    if let Ok(single) = over.cast::<PyIndexSet>() {
        axes_to_sum.push(core.find_axis(py, single)?);
    } else {
        let items: Vec<Bound<'_, PyAny>> = over.try_iter()?.collect::<PyResult<Vec<_>>>()?;
        for item in &items {
            let index_set = item.cast::<PyIndexSet>().map_err(|_| {
                ArrayTypeError::new_err("over= must be an IndexSet or tuple of IndexSets")
            })?;
            axes_to_sum.push(core.find_axis(py, index_set)?);
        }
    }

    axes_to_sum.sort_unstable();
    axes_to_sum.dedup();
    axes_to_sum.reverse();

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
    let index_set = rhs.cast::<PyIndexSet>().map_err(|_| {
        ArrayTypeError::new_err(">> / @ operator requires an IndexSet on the right-hand side")
    })?;
    let axis = core.find_axis(py, index_set)?;
    let (new_values, new_shape, new_index_sets) = core.sum_over_one(py, axis);
    reduce_or_wrap(new_values, new_shape, new_index_sets, py)
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
            let np = py.import("numpy")?;
            let flat = np
                .call_method1("asarray", (other,))?
                .call_method0("flatten")?;
            let weights: Vec<f64> = flat.extract()?;
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

    match func_name.as_str() {
        "sum" => array_sum(core, py, None),
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
        "concatenate" => numpy_concatenate(py, args),
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
fn numpy_concatenate(py: Python<'_>, args: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
    if args.is_empty() {
        return Err(ArrayDimensionError::new_err(
            "np.concatenate requires at least one argument",
        ));
    }
    let seq = args.get_item(0)?;
    let items: Vec<Bound<'_, PyAny>> = seq.try_iter()?.collect::<PyResult<Vec<_>>>()?;

    let mut all_values = Vec::new();
    let mut has_arrays = false;

    for item in &items {
        if let Ok(va) = item.extract::<PyRef<'_, PyVariableArray>>() {
            has_arrays = true;
            all_values.extend(va.get_values());
        } else if let Ok(ea) = item.extract::<PyRef<'_, PyExprArray>>() {
            has_arrays = true;
            all_values.extend(ea.get_values());
        } else {
            // Try to extract as a flat array of floats
            let np = py.import("numpy")?;
            let flat = np
                .call_method1("asarray", (item,))?
                .call_method0("flatten")?;
            let floats: Vec<f64> = flat.extract()?;
            for f in &floats {
                all_values.push(PyExpr::from_expr(Expr::from_constant(*f)));
            }
        }
    }

    if !has_arrays {
        return Err(ArrayTypeError::new_err(
            "np.concatenate requires at least one VariableArray or ExprArray",
        ));
    }

    let n = all_values.len();
    let result = PyExprArray::new(Vec::new(), vec![n], all_values);
    Ok(result.into_pyobject(py)?.into_any().unbind())
}

/// Register array classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVariableArray>()?;
    m.add_class::<PyExprArray>()?;
    m.add_class::<PyConstraintArray>()?;
    Ok(())
}
