use arco_arrays::BroadcastPlan;
use arco_model::Bounds;
use arco_model::expr::ComparisonSense;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::sync::Arc;

use crate::PyObject;
use crate::py_modules::constraint::PyConstraint;
use crate::py_modules::errors::{ArrayIndexError, ArrayTypeError};
use crate::py_modules::expr::PyExpr;
use crate::py_modules::index_set::PyIndexSet;

use super::CompactTerm;
use super::{LinearArrayCore, PyExprArray, PyVariableArray, SparseExprNode};

pub(crate) type SparseConstraintRows<'a> = (&'a [PyExpr], &'a [f64], &'a [usize], ComparisonSense);

pub enum SparseCompareOperand {
    Expr(Py<PyExprArray>),
    Variable(Py<PyVariableArray>),
}

pub enum BroadcastCompareOperand {
    Expr(Py<PyExprArray>),
    Variable(Py<PyVariableArray>),
}

#[derive(Clone, Copy)]
pub enum SparseCompareValue<'a> {
    Expr(&'a PyExpr),
    Variable(u32),
}

#[derive(Clone, Copy)]
pub enum SparseCompareView<'a> {
    Expr {
        indices: &'a [usize],
        values: &'a [PyExpr],
    },
    Variable {
        indices: &'a [usize],
        var_ids: &'a [u32],
    },
}

impl<'a> SparseCompareView<'a> {
    fn indices(self) -> &'a [usize] {
        match self {
            Self::Expr { indices, .. } | Self::Variable { indices, .. } => indices,
        }
    }

    fn value_at(self, position: usize) -> SparseCompareValue<'a> {
        match self {
            Self::Expr { values, .. } => SparseCompareValue::Expr(&values[position]),
            Self::Variable { var_ids, .. } => SparseCompareValue::Variable(var_ids[position]),
        }
    }
}

pub struct SparseCompareMerge<'a> {
    left: SparseCompareView<'a>,
    right: SparseCompareView<'a>,
    left_pos: usize,
    right_pos: usize,
}

impl<'a> SparseCompareMerge<'a> {
    pub fn new(left: SparseCompareView<'a>, right: SparseCompareView<'a>) -> Self {
        Self {
            left,
            right,
            left_pos: 0,
            right_pos: 0,
        }
    }
}

impl<'a> Iterator for SparseCompareMerge<'a> {
    type Item = (
        usize,
        Option<SparseCompareValue<'a>>,
        Option<SparseCompareValue<'a>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        let left_indices = self.left.indices();
        let right_indices = self.right.indices();
        if self.left_pos == left_indices.len() && self.right_pos == right_indices.len() {
            return None;
        }

        let left_idx = left_indices.get(self.left_pos).copied();
        let right_idx = right_indices.get(self.right_pos).copied();
        let current_idx = match (left_idx, right_idx) {
            (Some(left), Some(right)) => left.min(right),
            (Some(left), None) => left,
            (None, Some(right)) => right,
            (None, None) => return None,
        };

        let left_value = (left_idx == Some(current_idx)).then(|| {
            let value = self.left.value_at(self.left_pos);
            self.left_pos += 1;
            value
        });
        let right_value = (right_idx == Some(current_idx)).then(|| {
            let value = self.right.value_at(self.right_pos);
            self.right_pos += 1;
            value
        });
        Some((current_idx, left_value, right_value))
    }
}

impl SparseCompareValue<'_> {
    pub fn constant(self) -> f64 {
        match self {
            Self::Expr(expr) => expr.constant(),
            Self::Variable(_) => 0.0,
        }
    }

    pub fn num_terms(self) -> usize {
        match self {
            Self::Expr(expr) => expr.inner().num_terms(),
            Self::Variable(_) => 1,
        }
    }

    pub fn to_expr(self) -> PyExpr {
        match self {
            Self::Expr(expr) => expr.clone(),
            Self::Variable(var_id) => PyExpr::from_term(var_id, 1.0),
        }
    }
}

fn sparse_arithmetic_row_is_nonzero(
    py: Python<'_>,
    left: &SparseExprNode,
    right: &SparseExprNode,
    index: usize,
) -> bool {
    [left.value_at(py, index), right.value_at(py, index)]
        .into_iter()
        .flatten()
        .any(|value| value.inner().num_terms() != 0 || value.constant() != 0.0)
}

impl SparseCompareOperand {
    fn with_view<R>(
        &self,
        py: Python<'_>,
        visit: impl FnOnce(SparseCompareView<'_>) -> R,
    ) -> PyResult<R> {
        fn validate_indices(indices: &[usize], value_count: usize) -> PyResult<()> {
            if indices.len() != value_count {
                return Err(ArrayTypeError::new_err(
                    "sparse comparison source has mismatched indices and values",
                ));
            }
            if indices.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(ArrayTypeError::new_err(
                    "sparse comparison source indices must be strictly increasing",
                ));
            }
            Ok(())
        }

        match self {
            Self::Expr(array) => {
                let array = array.bind(py).borrow();
                let (indices, values) = array.sparse_entries().ok_or_else(|| {
                    ArrayTypeError::new_err("sparse comparison source is no longer sparse")
                })?;
                validate_indices(indices, values.len())?;
                Ok(visit(SparseCompareView::Expr { indices, values }))
            }
            Self::Variable(array) => {
                let array = array.bind(py).borrow();
                let (indices, var_ids) = array.sparse_var_entries().ok_or_else(|| {
                    ArrayTypeError::new_err("sparse comparison source is no longer sparse")
                })?;
                validate_indices(indices, var_ids.len())?;
                Ok(visit(SparseCompareView::Variable { indices, var_ids }))
            }
        }
    }

    pub fn with_views<R>(
        &self,
        other: &Self,
        py: Python<'_>,
        visit: impl FnOnce(SparseCompareView<'_>, SparseCompareView<'_>) -> R,
    ) -> PyResult<R> {
        self.with_view(py, |left| other.with_view(py, |right| visit(left, right)))?
    }
}

pub enum BroadcastCompareValue {
    Expr(PyExpr),
    Variable(u32),
}

pub enum BroadcastCompareView<'a> {
    Expr {
        array: &'a PyExprArray,
        plan: &'a BroadcastPlan,
    },
    Variable {
        array: &'a PyVariableArray,
        plan: &'a BroadcastPlan,
    },
}

impl BroadcastCompareView<'_> {
    pub fn value_at(&self, target_index: usize) -> Option<BroadcastCompareValue> {
        match self {
            Self::Expr { array, plan } => array
                .value_at_flat(plan.source_offset_for_target_flat(target_index))
                .map(BroadcastCompareValue::Expr),
            Self::Variable { array, plan } => array
                .variable_id_at_flat(plan.source_offset_for_target_flat(target_index))
                .map(BroadcastCompareValue::Variable),
        }
    }

    fn constant_at(&self, target_index: usize) -> f64 {
        match self {
            Self::Expr { array, plan } => {
                array.constant_at_flat(plan.source_offset_for_target_flat(target_index))
            }
            Self::Variable { .. } => 0.0,
        }
    }
}

impl BroadcastCompareOperand {
    pub fn is_sparse(&self, py: Python<'_>) -> bool {
        match self {
            Self::Expr(array) => array.bind(py).borrow().is_sparse(),
            Self::Variable(array) => array.bind(py).borrow().is_sparse(),
        }
    }

    fn with_view<R>(
        &self,
        py: Python<'_>,
        plan: &BroadcastPlan,
        visit: impl FnOnce(BroadcastCompareView<'_>) -> R,
    ) -> R {
        match self {
            Self::Expr(array) => {
                let array = array.bind(py).borrow();
                visit(BroadcastCompareView::Expr {
                    array: &array,
                    plan,
                })
            }
            Self::Variable(array) => {
                let array = array.bind(py).borrow();
                visit(BroadcastCompareView::Variable {
                    array: &array,
                    plan,
                })
            }
        }
    }

    pub fn with_views<R>(
        &self,
        other: &Self,
        py: Python<'_>,
        left_plan: &BroadcastPlan,
        right_plan: &BroadcastPlan,
        visit: impl FnOnce(BroadcastCompareView<'_>, BroadcastCompareView<'_>) -> R,
    ) -> R {
        self.with_view(py, left_plan, |left| {
            other.with_view(py, right_plan, |right| visit(left, right))
        })
    }
}

impl BroadcastCompareValue {
    pub fn into_expr(self) -> PyExpr {
        match self {
            Self::Expr(expr) => expr,
            Self::Variable(var_id) => PyExpr::from_term(var_id, 1.0),
        }
    }
}

/// Right-hand side for compact constraints.
#[derive(Clone, Debug)]
pub enum CompactRhs {
    /// All elements share the same rhs value.
    Scalar(f64),
    /// Per-element rhs values.
    Vec(Vec<f64>),
}

/// Compact representation for constraints from compact expressions.
/// Terms and rhs are already adjusted (constant subtracted from rhs).
#[derive(Clone, Debug)]
pub struct CompactConstraintStorage {
    pub terms: Vec<CompactTerm>,
    pub sense: ComparisonSense,
    pub rhs: CompactRhs,
    pub count: usize,
}

impl CompactConstraintStorage {
    /// Get the rhs as a Vec<f64> (expanding scalar if needed).
    pub(crate) fn rhs_vec(&self) -> Vec<f64> {
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
    /// Lazy sparse comparison retaining immutable source arrays until insertion.
    SparseLazyCompare {
        left: SparseCompareOperand,
        right: SparseCompareOperand,
        sense: ComparisonSense,
    },
    /// Lazy comparison of two sparse arithmetic expression plans.
    SparseArithmeticLazyCompare {
        left: Arc<SparseExprNode>,
        right: Arc<SparseExprNode>,
        sense: ComparisonSense,
    },
    /// Lazy lower-rank comparison retaining source arrays and broadcast plans.
    BroadcastLazyCompare {
        left: BroadcastCompareOperand,
        right: BroadcastCompareOperand,
        left_plan: BroadcastPlan,
        right_plan: BroadcastPlan,
        sense: ComparisonSense,
    },
    /// Compact: constraints from compact expression patterns (not yet inserted).
    Compact(CompactConstraintStorage),
}

/// A multi-dimensional array of constraint expressions.
#[pyo3_macros::pyclass(name = "ConstraintArray")]
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
    pub(crate) fn new(
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
        debug_assert_eq!(
            exprs.len(),
            rhs.len(),
            "SparseRows invariant: exprs and rhs must have the same length"
        );
        debug_assert_eq!(
            exprs.len(),
            active_indices.len(),
            "SparseRows invariant: exprs and active_indices must have the same length"
        );
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

    pub(crate) fn from_sparse_lazy_compare(
        left: SparseCompareOperand,
        right: SparseCompareOperand,
        sense: ComparisonSense,
        shape: Vec<usize>,
        index_sets: Vec<Py<PyIndexSet>>,
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::SparseLazyCompare { left, right, sense },
            shape,
            index_sets,
            first_constraint_id: None,
            name: None,
        }
    }

    pub(crate) fn from_sparse_arithmetic_lazy_compare(
        left: Arc<SparseExprNode>,
        right: Arc<SparseExprNode>,
        sense: ComparisonSense,
        shape: Vec<usize>,
        index_sets: Vec<Py<PyIndexSet>>,
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::SparseArithmeticLazyCompare { left, right, sense },
            shape,
            index_sets,
            first_constraint_id: None,
            name: None,
        }
    }

    pub(crate) fn from_broadcast_lazy_compare(
        left: BroadcastCompareOperand,
        right: BroadcastCompareOperand,
        sense: ComparisonSense,
        shape: Vec<usize>,
        index_sets: Vec<Py<PyIndexSet>>,
        left_plan: BroadcastPlan,
        right_plan: BroadcastPlan,
    ) -> Self {
        Self {
            storage: ConstraintArrayStorage::BroadcastLazyCompare {
                left,
                right,
                left_plan,
                right_plan,
                sense,
            },
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
            ConstraintArrayStorage::SparseLazyCompare { .. } => &[],
            ConstraintArrayStorage::SparseArithmeticLazyCompare { .. } => &[],
            ConstraintArrayStorage::BroadcastLazyCompare { .. } => &[],
            ConstraintArrayStorage::Compact(_) => &[], // Should not be called on compact
        }
    }

    pub fn get_sense(&self) -> ComparisonSense {
        match &self.storage {
            ConstraintArrayStorage::Full { sense, .. }
            | ConstraintArrayStorage::SparseRows { sense, .. } => *sense,
            ConstraintArrayStorage::LazyCompare { sense, .. } => *sense,
            ConstraintArrayStorage::SparseLazyCompare { sense, .. } => *sense,
            ConstraintArrayStorage::SparseArithmeticLazyCompare { sense, .. } => *sense,
            ConstraintArrayStorage::BroadcastLazyCompare { sense, .. } => *sense,
            ConstraintArrayStorage::Compact(c) => c.sense,
        }
    }

    fn sparse_compare_rhs(
        left: &SparseCompareOperand,
        right: &SparseCompareOperand,
    ) -> PyResult<Vec<f64>> {
        Python::attach(|py| {
            left.with_views(right, py, |left, right| {
                let mut rhs = Vec::new();
                for (_, left, right) in SparseCompareMerge::new(left, right) {
                    let constant = left.map_or(0.0, SparseCompareValue::constant)
                        - right.map_or(0.0, SparseCompareValue::constant);
                    let num_terms = left.map_or(0, SparseCompareValue::num_terms)
                        + right.map_or(0, SparseCompareValue::num_terms);
                    if num_terms != 0 || constant != 0.0 {
                        rhs.push(-constant);
                    }
                }
                rhs
            })
        })
    }

    fn broadcast_compare_rhs(
        left: &BroadcastCompareOperand,
        right: &BroadcastCompareOperand,
        left_plan: &BroadcastPlan,
        right_plan: &BroadcastPlan,
        total: usize,
    ) -> Vec<f64> {
        Python::attach(|py| {
            left.with_views(right, py, left_plan, right_plan, |left, right| {
                (0..total)
                    .map(|index| -(left.constant_at(index) - right.constant_at(index)))
                    .collect()
            })
        })
    }

    pub fn get_rhs(&self) -> PyResult<Vec<f64>> {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. }
            | ConstraintArrayStorage::SparseRows { rhs, .. } => Ok(rhs.clone()),
            ConstraintArrayStorage::LazyCompare { left, right, .. } => Ok(left
                .values
                .iter()
                .zip(right.values.iter())
                .map(|(left_expr, right_expr)| {
                    let diff = left_expr.inner().add(&right_expr.inner().scale(-1.0));
                    -PyExpr::from_expr(diff).constant()
                })
                .collect()),
            ConstraintArrayStorage::SparseLazyCompare { left, right, .. } => {
                Self::sparse_compare_rhs(left, right)
            }
            ConstraintArrayStorage::SparseArithmeticLazyCompare { left, right, .. } => {
                Ok(Python::attach(|py| {
                    super::merge_sorted_indices(left.active_indices(), right.active_indices())
                        .into_iter()
                        .filter(|&index| sparse_arithmetic_row_is_nonzero(py, left, right, index))
                        .map(|index| {
                            let left_constant = left
                                .value_at(py, index)
                                .map_or(0.0, |value| value.constant());
                            let right_constant = right
                                .value_at(py, index)
                                .map_or(0.0, |value| value.constant());
                            -(left_constant - right_constant)
                        })
                        .collect()
                }))
            }
            ConstraintArrayStorage::BroadcastLazyCompare {
                left,
                right,
                left_plan,
                right_plan,
                ..
            } => Ok(Self::broadcast_compare_rhs(
                left,
                right,
                left_plan,
                right_plan,
                self.len()?,
            )),
            ConstraintArrayStorage::Compact(c) => Ok(c.rhs_vec()),
        }
    }

    pub fn shape_ref(&self) -> &[usize] {
        &self.shape
    }

    pub fn clone_index_sets(&self) -> Vec<Py<PyIndexSet>> {
        Python::attach(|py| {
            self.index_sets
                .iter()
                .map(|set| set.clone_ref(py))
                .collect()
        })
    }

    /// Get compact storage if available.
    pub fn as_compact(&self) -> Option<&CompactConstraintStorage> {
        match &self.storage {
            ConstraintArrayStorage::Compact(c) => Some(c),
            ConstraintArrayStorage::Full { .. } | ConstraintArrayStorage::SparseRows { .. } => None,
            ConstraintArrayStorage::LazyCompare { .. }
            | ConstraintArrayStorage::SparseLazyCompare { .. }
            | ConstraintArrayStorage::SparseArithmeticLazyCompare { .. }
            | ConstraintArrayStorage::BroadcastLazyCompare { .. } => None,
        }
    }

    pub fn as_lazy_compare(&self) -> Option<(&LinearArrayCore, &LinearArrayCore, ComparisonSense)> {
        match &self.storage {
            ConstraintArrayStorage::LazyCompare { left, right, sense } => {
                Some((left, right, *sense))
            }
            ConstraintArrayStorage::Full { .. }
            | ConstraintArrayStorage::SparseRows { .. }
            | ConstraintArrayStorage::SparseLazyCompare { .. }
            | ConstraintArrayStorage::SparseArithmeticLazyCompare { .. }
            | ConstraintArrayStorage::BroadcastLazyCompare { .. }
            | ConstraintArrayStorage::Compact(_) => None,
        }
    }

    pub fn as_sparse_rows(&self) -> Option<SparseConstraintRows<'_>> {
        match &self.storage {
            ConstraintArrayStorage::SparseRows {
                exprs,
                sense,
                rhs,
                active_indices,
            } => Some((exprs, rhs, active_indices, *sense)),
            ConstraintArrayStorage::Full { .. }
            | ConstraintArrayStorage::LazyCompare { .. }
            | ConstraintArrayStorage::SparseLazyCompare { .. }
            | ConstraintArrayStorage::SparseArithmeticLazyCompare { .. }
            | ConstraintArrayStorage::BroadcastLazyCompare { .. }
            | ConstraintArrayStorage::Compact(_) => None,
        }
    }

    pub fn as_sparse_lazy_compare(
        &self,
    ) -> Option<(
        &SparseCompareOperand,
        &SparseCompareOperand,
        ComparisonSense,
    )> {
        match &self.storage {
            ConstraintArrayStorage::SparseLazyCompare { left, right, sense } => {
                Some((left, right, *sense))
            }
            ConstraintArrayStorage::Full { .. }
            | ConstraintArrayStorage::SparseRows { .. }
            | ConstraintArrayStorage::LazyCompare { .. }
            | ConstraintArrayStorage::SparseArithmeticLazyCompare { .. }
            | ConstraintArrayStorage::BroadcastLazyCompare { .. }
            | ConstraintArrayStorage::Compact(_) => None,
        }
    }

    pub fn as_sparse_arithmetic_lazy_compare(
        &self,
    ) -> Option<(&Arc<SparseExprNode>, &Arc<SparseExprNode>, ComparisonSense)> {
        match &self.storage {
            ConstraintArrayStorage::SparseArithmeticLazyCompare { left, right, sense } => {
                Some((left, right, *sense))
            }
            ConstraintArrayStorage::Full { .. }
            | ConstraintArrayStorage::SparseRows { .. }
            | ConstraintArrayStorage::LazyCompare { .. }
            | ConstraintArrayStorage::SparseLazyCompare { .. }
            | ConstraintArrayStorage::BroadcastLazyCompare { .. }
            | ConstraintArrayStorage::Compact(_) => None,
        }
    }

    pub fn as_broadcast_lazy_compare(
        &self,
    ) -> Option<(
        &BroadcastCompareOperand,
        &BroadcastCompareOperand,
        &BroadcastPlan,
        &BroadcastPlan,
        ComparisonSense,
    )> {
        match &self.storage {
            ConstraintArrayStorage::BroadcastLazyCompare {
                left,
                right,
                left_plan,
                right_plan,
                sense,
            } => Some((left, right, left_plan, right_plan, *sense)),
            ConstraintArrayStorage::Full { .. }
            | ConstraintArrayStorage::SparseRows { .. }
            | ConstraintArrayStorage::LazyCompare { .. }
            | ConstraintArrayStorage::SparseLazyCompare { .. }
            | ConstraintArrayStorage::SparseArithmeticLazyCompare { .. }
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

    pub fn from_batch_shaped(
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

    fn len(&self) -> PyResult<usize> {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. }
            | ConstraintArrayStorage::SparseRows { rhs, .. } => Ok(rhs.len()),
            ConstraintArrayStorage::LazyCompare { left, .. } => Ok(left.values.len()),
            ConstraintArrayStorage::SparseLazyCompare { left, right, .. } => Python::attach(|py| {
                left.with_views(right, py, |left, right| {
                    SparseCompareMerge::new(left, right)
                        .filter(|(_, left, right)| {
                            let num_terms = left.map_or(0, SparseCompareValue::num_terms)
                                + right.map_or(0, SparseCompareValue::num_terms);
                            let constant = left.map_or(0.0, SparseCompareValue::constant)
                                - right.map_or(0.0, SparseCompareValue::constant);
                            num_terms != 0 || constant != 0.0
                        })
                        .count()
                })
            }),
            ConstraintArrayStorage::SparseArithmeticLazyCompare { left, right, .. } => {
                Python::attach(|py| {
                    let mut count = 0;
                    for row_idx in
                        super::merge_sorted_indices(left.active_indices(), right.active_indices())
                    {
                        if !sparse_arithmetic_row_is_nonzero(py, left, right, row_idx) {
                            continue;
                        }
                        count += 1;
                    }
                    Ok(count)
                })
            }
            ConstraintArrayStorage::BroadcastLazyCompare { .. } => Ok(self.shape.iter().product()),
            ConstraintArrayStorage::Compact(c) => Ok(c.count),
        }
    }

    /// Get the rhs value at a specific index without allocating.
    fn rhs_at(&self, index: usize) -> PyResult<f64> {
        match &self.storage {
            ConstraintArrayStorage::Full { rhs, .. }
            | ConstraintArrayStorage::SparseRows { rhs, .. } => {
                rhs.get(index).copied().ok_or_else(|| {
                    ArrayIndexError::new_err(format!("constraint index {index} out of range"))
                })
            }
            ConstraintArrayStorage::LazyCompare { left, right, .. } => {
                let left_expr = left.values.get(index).ok_or_else(|| {
                    ArrayIndexError::new_err(format!("constraint index {index} out of range"))
                })?;
                let right_expr = right.values.get(index).ok_or_else(|| {
                    ArrayIndexError::new_err(format!("constraint index {index} out of range"))
                })?;
                let diff = left_expr.inner().add(&right_expr.inner().scale(-1.0));
                Ok(-PyExpr::from_expr(diff).constant())
            }
            ConstraintArrayStorage::SparseLazyCompare { left, right, .. } => {
                Python::attach(|py| {
                    left.with_views(right, py, |left, right| {
                        let mut nonzero_index = 0usize;
                        for (_, left, right) in SparseCompareMerge::new(left, right) {
                            let num_terms = left.map_or(0, SparseCompareValue::num_terms)
                                + right.map_or(0, SparseCompareValue::num_terms);
                            let constant = left.map_or(0.0, SparseCompareValue::constant)
                                - right.map_or(0.0, SparseCompareValue::constant);
                            if num_terms != 0 || constant != 0.0 {
                                if nonzero_index == index {
                                    return Ok(-constant);
                                }
                                nonzero_index += 1;
                            }
                        }
                        Err(ArrayIndexError::new_err(format!(
                            "constraint index {index} out of range"
                        )))
                    })
                })?
            }
            ConstraintArrayStorage::SparseArithmeticLazyCompare { left, right, .. } => {
                Python::attach(|py| {
                    let mut nonzero_index = 0;
                    for row_idx in
                        super::merge_sorted_indices(left.active_indices(), right.active_indices())
                    {
                        if !sparse_arithmetic_row_is_nonzero(py, left, right, row_idx) {
                            continue;
                        }
                        if nonzero_index == index {
                            let left_constant = left
                                .value_at(py, row_idx)
                                .map_or(0.0, |value| value.constant());
                            let right_constant = right
                                .value_at(py, row_idx)
                                .map_or(0.0, |value| value.constant());
                            return Ok(-(left_constant - right_constant));
                        }
                        nonzero_index += 1;
                    }
                    Err(ArrayIndexError::new_err(format!(
                        "constraint index {index} out of range"
                    )))
                })
            }
            ConstraintArrayStorage::BroadcastLazyCompare {
                left,
                right,
                left_plan,
                right_plan,
                ..
            } => {
                if index >= self.shape.iter().product() {
                    return Err(ArrayIndexError::new_err(format!(
                        "constraint index {index} out of range"
                    )));
                }
                Ok(Python::attach(|py| {
                    left.with_views(right, py, left_plan, right_plan, |left, right| {
                        -(left.constant_at(index) - right.constant_at(index))
                    })
                }))
            }
            ConstraintArrayStorage::Compact(c) => match &c.rhs {
                CompactRhs::Scalar(v) => Ok(*v),
                CompactRhs::Vec(v) => v.get(index).copied().ok_or_else(|| {
                    ArrayIndexError::new_err(format!("constraint index {index} out of range"))
                }),
            },
        }
    }

    fn constraint_at(&self, index: usize) -> PyResult<PyConstraint> {
        let first_id = self.first_constraint_id.ok_or_else(|| {
            ArrayTypeError::new_err(
                "this ConstraintArray has not been added to a model yet and is not subscriptable",
            )
        })?;
        let rhs_val = self.rhs_at(index)?;
        let length = self.len()?;
        let bounds = match self.get_sense() {
            ComparisonSense::LessEqual => Bounds::new(f64::NEG_INFINITY, rhs_val),
            ComparisonSense::GreaterEqual => Bounds::new(rhs_val, f64::INFINITY),
            ComparisonSense::Equal => Bounds::new(rhs_val, rhs_val),
        };
        let name = self.name.as_ref().map(|base| {
            if length == 1 {
                base.clone()
            } else {
                format!("{base}[{index}]")
            }
        });
        Ok(PyConstraint::new(first_id + index as u32, name, bounds))
    }
}

#[pyo3_macros::pymethods]
impl PyConstraintArray {
    #[getter]
    fn sense(&self) -> String {
        self.get_sense().as_str().to_string()
    }

    #[getter]
    fn rhs(&self) -> PyResult<Vec<f64>> {
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

    fn __len__(&self) -> PyResult<usize> {
        self.len()
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let constraints = (0..self.len()?)
            .map(|index| self.constraint_at(index))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyList::new(py, constraints)?
            .call_method0("__iter__")?
            .unbind())
    }

    fn __getitem__(&self, index: usize) -> PyResult<PyConstraint> {
        let length = self.len()?;
        if index >= length {
            return Err(ArrayIndexError::new_err(format!(
                "index {} out of range for ConstraintArray of size {}",
                index, length
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
