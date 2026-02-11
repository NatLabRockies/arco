//! Python wrappers for variable, expression, and constraint arrays.

use arco_expr::{ComparisonSense, Expr};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyObject;
use crate::errors::{
    ArrayDimensionError, ArrayIndexError, ArrayShapeMismatchError, ArrayTypeError,
    ExprDivisionByZeroError,
};
use crate::expr::PyExpr;
use crate::index_set::PyIndexSet;
use crate::variable::PyVariable;

/// Resolved axis index: either a single index or a range of indices.
enum AxisIndex {
    Single(usize),
    Range(Vec<usize>),
}

/// Resolve a Python index (int or slice) to an AxisIndex for one dimension.
fn resolve_axis_index(index: &Bound<'_, PyAny>, dim_size: usize) -> PyResult<AxisIndex> {
    // Try integer index (supports negative indexing)
    if let Ok(idx) = index.extract::<isize>() {
        let resolved = if idx < 0 {
            (dim_size as isize + idx) as usize
        } else {
            idx as usize
        };
        if resolved >= dim_size {
            return Err(ArrayIndexError::new_err(format!(
                "index {} out of range for dimension of size {}",
                idx, dim_size
            )));
        }
        return Ok(AxisIndex::Single(resolved));
    }
    // Try slice
    if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
        let indices = slice.indices(dim_size as isize)?;
        let mut result = Vec::new();
        let mut i = indices.start;
        while (indices.step > 0 && i < indices.stop) || (indices.step < 0 && i > indices.stop) {
            result.push(i as usize);
            i += indices.step;
        }
        return Ok(AxisIndex::Range(result));
    }
    Err(ArrayTypeError::new_err(
        "tuple index components must be integers or slices",
    ))
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
                result[dst_idx] = result[dst_idx].add(values[src_idx].clone());
            }
        }
    }

    result
}

// ============================================================================
// LinearArrayCore: shared storage and logic for both VariableArray and ExprArray
// ============================================================================

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
        self.values
            .iter()
            .cloned()
            .fold(PyExpr::default(), |acc, v| acc.add(v))
    }
}

// ============================================================================
// Shared free functions operating on LinearArrayCore
// ============================================================================

/// Extract a LinearArrayCore from a PyAny that is either a VariableArray or ExprArray.
fn extract_array_core(other: &Bound<'_, PyAny>) -> PyResult<LinearArrayCore> {
    if let Ok(va) = other.extract::<PyRef<'_, PyVariableArray>>() {
        return Ok(va.core.clone_with_gil());
    }
    if let Ok(ea) = other.extract::<PyRef<'_, PyExprArray>>() {
        return Ok(ea.core.clone_with_gil());
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
        return core.compare_core(&other.core, sense);
    }
    if let Ok(other) = rhs.extract::<PyRef<'_, PyExprArray>>() {
        return core.compare_core(&other.core, sense);
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
        let acc = values
            .into_iter()
            .fold(PyExpr::default(), |acc, v| acc.add(v));
        Ok(acc.into_pyobject(py)?.into_any().unbind())
    } else {
        let arr = PyExprArray {
            core: LinearArrayCore::new(index_sets, shape, values),
        };
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

/// Element-wise addition of a core with a Python operand.
fn array_add(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(other_core) = extract_array_core(other) {
        let result = core.combine(&other_core, |left, right| left.add(right.clone()))?;
        return Ok(PyExprArray { core: result });
    }
    if let Ok(value) = other.extract::<f64>() {
        return Ok(PyExprArray {
            core: core.add_scalar(value),
        });
    }
    let values: Vec<f64> = other.extract()?;
    Ok(PyExprArray {
        core: core.add_vec(&values)?,
    })
}

/// Element-wise subtraction: core - other.
fn array_sub(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(other_core) = extract_array_core(other) {
        let result = core.combine(&other_core, |left, right| left.add(right.scale(-1.0)))?;
        return Ok(PyExprArray { core: result });
    }
    if let Ok(value) = other.extract::<f64>() {
        return Ok(PyExprArray {
            core: core.sub_scalar(value),
        });
    }
    let values: Vec<f64> = other.extract()?;
    Ok(PyExprArray {
        core: core.sub_vec(&values)?,
    })
}

/// Element-wise reverse subtraction: other - core.
fn array_rsub(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(other_core) = extract_array_core(other) {
        let result = other_core.combine(core, |left, right| left.add(right.scale(-1.0)))?;
        return Ok(PyExprArray { core: result });
    }
    if let Ok(value) = other.extract::<f64>() {
        return Ok(PyExprArray {
            core: core.rsub_scalar(value),
        });
    }
    let values: Vec<f64> = other.extract()?;
    Ok(PyExprArray {
        core: core.rsub_vec(&values)?,
    })
}

/// Element-wise multiplication of a core with a scalar or vector.
fn array_mul(core: &LinearArrayCore, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
    if let Ok(scalar) = other.extract::<f64>() {
        return Ok(PyExprArray {
            core: core.scale_all(scalar),
        });
    }
    let weights: Vec<f64> = other.extract()?;
    Ok(PyExprArray {
        core: core.mul_vec(&weights)?,
    })
}

/// Division of a core by a scalar.
fn array_truediv(core: &LinearArrayCore, other: f64) -> PyResult<PyExprArray> {
    if other == 0.0 {
        return Err(ExprDivisionByZeroError::new_err("division by zero"));
    }
    Ok(PyExprArray {
        core: core.scale_all(1.0 / other),
    })
}

/// Negate all elements in a core.
fn array_neg(core: &LinearArrayCore) -> PyExprArray {
    PyExprArray {
        core: core.scale_all(-1.0),
    }
}

/// Return the index_sets as a Python tuple.
fn array_index_sets(core: &LinearArrayCore, py: Python<'_>) -> PyResult<PyObject> {
    let sets = core
        .index_sets
        .iter()
        .map(|set| set.clone_ref(py))
        .collect::<Vec<_>>();
    Ok(PyTuple::new(py, sets)?.into())
}

/// Return the shape as a Python tuple.
fn array_shape(core: &LinearArrayCore, py: Python<'_>) -> PyResult<PyObject> {
    Ok(PyTuple::new(py, core.shape.clone())?.into())
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
            .map(|i| crate::index_set::IndexMember::Int(i as i64))
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
            expr_array_to_pyobject(
                PyExprArray {
                    core: core.mul_vec(&weights)?,
                },
                py,
            )
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

// ============================================================================
// Macro to generate shared #[pymethods] for both array types
// ============================================================================

macro_rules! impl_array_ops {
    ($ty:ty, { $($extra:tt)* }) => {
        #[pymethods]
        impl $ty {
            fn __add__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
                array_add(&self.core, other)
            }
            fn __radd__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
                array_add(&self.core, other)
            }
            fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
                array_sub(&self.core, other)
            }
            fn __rsub__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
                array_rsub(&self.core, other)
            }
            fn __mul__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
                array_mul(&self.core, other)
            }
            fn __rmul__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyExprArray> {
                array_mul(&self.core, other)
            }
            fn __truediv__(&self, other: f64) -> PyResult<PyExprArray> {
                array_truediv(&self.core, other)
            }
            fn __neg__(&self) -> PyExprArray {
                array_neg(&self.core)
            }
            fn __ge__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
                compare_array_rhs(&self.core, rhs, ComparisonSense::GreaterEqual)
            }
            fn __le__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
                compare_array_rhs(&self.core, rhs, ComparisonSense::LessEqual)
            }
            fn __eq__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<PyConstraintArray> {
                compare_array_rhs(&self.core, rhs, ComparisonSense::Equal)
            }
            #[pyo3(signature = (*, over=None))]
            fn sum(&self, py: Python<'_>, over: Option<&Bound<'_, PyAny>>) -> PyResult<PyObject> {
                array_sum(&self.core, py, over)
            }
            fn __rshift__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
                array_reduce(&self.core, py, rhs)
            }
            fn __matmul__(&self, py: Python<'_>, rhs: &Bound<'_, PyAny>) -> PyResult<PyObject> {
                array_reduce(&self.core, py, rhs)
            }
            #[getter]
            fn index_sets(&self, py: Python<'_>) -> PyResult<PyObject> {
                array_index_sets(&self.core, py)
            }
            #[getter]
            fn shape(&self, py: Python<'_>) -> PyResult<PyObject> {
                array_shape(&self.core, py)
            }
            #[getter]
            fn values(&self) -> Vec<PyExpr> {
                self.core.values.clone()
            }
            fn flatten(&self) -> Vec<PyExpr> {
                self.core.values.clone()
            }
            fn __len__(&self) -> usize {
                self.core.values.len()
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
                array_ufunc(
                    &self.core,
                    py,
                    |ob| ob.is_instance_of::<$ty>(),
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
                array_function(&self.core, py, func, _types, args, kwargs)
            }

            $($extra)*
        }
    };
}

// ============================================================================
// PyVariableArray: a grid of decision variables from add_variables()
// ============================================================================

/// A multi-dimensional array of decision variables.
/// This is ONLY created by Model.add_variables(). Any operation on it produces ExprArray.
#[pyclass(name = "VariableArray")]
pub struct PyVariableArray {
    pub(crate) core: LinearArrayCore,
    /// Variable objects for each element (parallel to core.values)
    variables: Vec<PyVariable>,
}

impl PyVariableArray {
    pub fn new(
        index_sets: Vec<Py<PyIndexSet>>,
        shape: Vec<usize>,
        values: Vec<PyExpr>,
        variables: Vec<PyVariable>,
    ) -> Self {
        Self {
            core: LinearArrayCore::new(index_sets, shape, values),
            variables,
        }
    }

    pub fn get_values(&self) -> &[PyExpr] {
        &self.core.values
    }

    pub fn get_variable_refs(&self) -> &[PyVariable] {
        &self.variables
    }

    pub fn get_shape(&self) -> &[usize] {
        &self.core.shape
    }

    /// Handle tuple-based multi-dimensional indexing for VariableArray.
    /// Returns Variable for single element, VariableArray for slices.
    fn getitem_tuple(&self, py: Python<'_>, tuple: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
        if self.core.shape.len() != 2 || tuple.len() != 2 {
            return Err(ArrayDimensionError::new_err(
                "tuple indexing requires a 2D array and exactly 2 indices",
            ));
        }
        let nrows = self.core.shape[0];
        let ncols = self.core.shape[1];
        let idx0 = tuple.get_item(0)?;
        let idx1 = tuple.get_item(1)?;

        let rows = resolve_axis_index(&idx0, nrows)?;
        let cols = resolve_axis_index(&idx1, ncols)?;

        match (&rows, &cols) {
            (AxisIndex::Single(r), AxisIndex::Single(c)) => {
                // x[i, j] -> single element (Variable)
                let flat_idx = r * ncols + c;
                let var = self.variables.get(flat_idx).cloned().ok_or_else(|| {
                    ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
                })?;
                Ok(var.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Single(r), AxisIndex::Range(col_indices)) => {
                // x[int, :] -> 1D sub-array (single row)
                let mut vals = Vec::with_capacity(col_indices.len());
                let mut vars = Vec::with_capacity(col_indices.len());
                for &c in col_indices {
                    let flat_idx = r * ncols + c;
                    vals.push(self.core.values[flat_idx].clone());
                    vars.push(self.variables[flat_idx].clone());
                }
                let n = vals.len();
                let new_index_sets =
                    if col_indices.len() == ncols && self.core.index_sets.len() == 2 {
                        vec![self.core.index_sets[1].clone_ref(py)]
                    } else {
                        Vec::new()
                    };
                let result = PyVariableArray::new(new_index_sets, vec![n], vals, vars);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Range(row_indices), AxisIndex::Single(c)) => {
                // x[:, int] -> 1D sub-array (single column)
                let mut vals = Vec::with_capacity(row_indices.len());
                let mut vars = Vec::with_capacity(row_indices.len());
                for &r in row_indices {
                    let flat_idx = r * ncols + c;
                    vals.push(self.core.values[flat_idx].clone());
                    vars.push(self.variables[flat_idx].clone());
                }
                let n = vals.len();
                let new_index_sets =
                    if row_indices.len() == nrows && self.core.index_sets.len() == 2 {
                        vec![self.core.index_sets[0].clone_ref(py)]
                    } else {
                        Vec::new()
                    };
                let result = PyVariableArray::new(new_index_sets, vec![n], vals, vars);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Range(row_indices), AxisIndex::Range(col_indices)) => {
                // x[slice, slice] -> 2D sub-array
                let new_nrows = row_indices.len();
                let new_ncols = col_indices.len();
                let mut vals = Vec::with_capacity(new_nrows * new_ncols);
                let mut vars = Vec::with_capacity(new_nrows * new_ncols);
                for &r in row_indices {
                    for &c in col_indices {
                        let flat_idx = r * ncols + c;
                        vals.push(self.core.values[flat_idx].clone());
                        vars.push(self.variables[flat_idx].clone());
                    }
                }
                let new_index_sets = if self.core.index_sets.len() == 2 {
                    vec![
                        if row_indices.len() == nrows {
                            self.core.index_sets[0].clone_ref(py)
                        } else {
                            Py::new(
                                py,
                                PyIndexSet {
                                    name: format!("_slice_{}", new_nrows),
                                    members: (0..new_nrows)
                                        .map(|i| crate::index_set::IndexMember::Int(i as i64))
                                        .collect(),
                                },
                            )?
                        },
                        if col_indices.len() == ncols {
                            self.core.index_sets[1].clone_ref(py)
                        } else {
                            Py::new(
                                py,
                                PyIndexSet {
                                    name: format!("_slice_{}", new_ncols),
                                    members: (0..new_ncols)
                                        .map(|i| crate::index_set::IndexMember::Int(i as i64))
                                        .collect(),
                                },
                            )?
                        },
                    ]
                } else {
                    Vec::new()
                };
                let result =
                    PyVariableArray::new(new_index_sets, vec![new_nrows, new_ncols], vals, vars);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
        }
    }
}

impl_array_ops!(PyVariableArray, {
    #[getter]
    fn variables(&self) -> Vec<PyVariable> {
        self.variables.clone()
    }

    fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        // Try tuple indexing for multi-dimensional access
        if let Ok(tuple) = index.cast::<PyTuple>() {
            return self.getitem_tuple(py, tuple);
        }

        // Try integer index
        if let Ok(idx) = index.extract::<usize>() {
            return self
                .variables
                .get(idx)
                .cloned()
                .ok_or_else(|| {
                    ArrayIndexError::new_err(format!(
                        "index {} out of range for array of size {}",
                        idx,
                        self.variables.len()
                    ))
                })
                .and_then(|v| Ok(v.into_pyobject(py)?.into_any().unbind()));
        }

        // Try boolean numpy array masking
        let np = py.import("numpy")?;
        let ndarray_type = np.getattr("ndarray")?;
        if index.is_instance(&ndarray_type)? {
            let dtype = index.getattr("dtype")?;
            let kind: String = dtype.getattr("kind")?.extract()?;
            if kind == "b" {
                let flat_mask: Vec<bool> = index.call_method0("flatten")?.extract()?;
                if flat_mask.len() != self.core.values.len() {
                    return Err(ArrayShapeMismatchError::new_err(format!(
                        "boolean mask length {} does not match array length {}",
                        flat_mask.len(),
                        self.core.values.len()
                    )));
                }
                let filtered_values: Vec<PyExpr> = self
                    .core
                    .values
                    .iter()
                    .zip(flat_mask.iter())
                    .filter(|(_, m)| **m)
                    .map(|(v, _)| v.clone())
                    .collect();
                let filtered_variables: Vec<PyVariable> = self
                    .variables
                    .iter()
                    .zip(flat_mask.iter())
                    .filter(|(_, m)| **m)
                    .map(|(v, _)| v.clone())
                    .collect();
                let n = filtered_values.len();
                let result =
                    PyVariableArray::new(Vec::new(), vec![n], filtered_values, filtered_variables);
                return Ok(result.into_pyobject(py)?.into_any().unbind());
            }
        }

        // Try slice
        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let len = self.core.values.len() as isize;
            let indices = slice.indices(len)?;
            let start = indices.start;
            let stop = indices.stop;
            let step = indices.step;

            let mut sliced_values = Vec::new();
            let mut sliced_variables = Vec::new();
            let mut idx = start;
            while (step > 0 && idx < stop) || (step < 0 && idx > stop) {
                let ui = idx as usize;
                sliced_values.push(self.core.values[ui].clone());
                sliced_variables.push(self.variables[ui].clone());
                idx += step;
            }
            let n = sliced_values.len();
            let result = PyVariableArray::new(Vec::new(), vec![n], sliced_values, sliced_variables);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        Err(ArrayIndexError::new_err(
            "index must be an integer, tuple, slice, or a boolean numpy array",
        ))
    }

    fn __repr__(&self) -> String {
        format!("VariableArray(shape={:?})", self.core.shape)
    }
});

// ============================================================================
// PyExprArray: a grid of linear expressions (result of operations on arrays)
// ============================================================================

/// A multi-dimensional array of linear expressions.
/// This is the result of any operation on VariableArray or ExprArray.
#[pyclass(name = "ExprArray")]
pub struct PyExprArray {
    pub(crate) core: LinearArrayCore,
}

impl PyExprArray {
    pub fn new(index_sets: Vec<Py<PyIndexSet>>, shape: Vec<usize>, values: Vec<PyExpr>) -> Self {
        Self {
            core: LinearArrayCore::new(index_sets, shape, values),
        }
    }

    /// Handle tuple-based multi-dimensional indexing for ExprArray.
    /// Returns Expr for single element, ExprArray for slices.
    fn getitem_tuple(&self, py: Python<'_>, tuple: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
        if self.core.shape.len() != 2 || tuple.len() != 2 {
            return Err(ArrayDimensionError::new_err(
                "tuple indexing requires a 2D array and exactly 2 indices",
            ));
        }
        let nrows = self.core.shape[0];
        let ncols = self.core.shape[1];
        let idx0 = tuple.get_item(0)?;
        let idx1 = tuple.get_item(1)?;

        let rows = resolve_axis_index(&idx0, nrows)?;
        let cols = resolve_axis_index(&idx1, ncols)?;

        match (&rows, &cols) {
            (AxisIndex::Single(r), AxisIndex::Single(c)) => {
                // x[i, j] -> single element (Expr)
                let flat_idx = r * ncols + c;
                let expr = self.core.values.get(flat_idx).cloned().ok_or_else(|| {
                    ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
                })?;
                Ok(expr.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Single(r), AxisIndex::Range(col_indices)) => {
                let mut vals = Vec::with_capacity(col_indices.len());
                for &c in col_indices {
                    let flat_idx = r * ncols + c;
                    vals.push(self.core.values[flat_idx].clone());
                }
                let n = vals.len();
                let new_index_sets =
                    if col_indices.len() == ncols && self.core.index_sets.len() == 2 {
                        vec![self.core.index_sets[1].clone_ref(py)]
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
                    vals.push(self.core.values[flat_idx].clone());
                }
                let n = vals.len();
                let new_index_sets =
                    if row_indices.len() == nrows && self.core.index_sets.len() == 2 {
                        vec![self.core.index_sets[0].clone_ref(py)]
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
                        vals.push(self.core.values[flat_idx].clone());
                    }
                }
                let new_index_sets = if self.core.index_sets.len() == 2 {
                    vec![
                        if row_indices.len() == nrows {
                            self.core.index_sets[0].clone_ref(py)
                        } else {
                            Py::new(
                                py,
                                PyIndexSet {
                                    name: format!("_slice_{}", new_nrows),
                                    members: (0..new_nrows)
                                        .map(|i| crate::index_set::IndexMember::Int(i as i64))
                                        .collect(),
                                },
                            )?
                        },
                        if col_indices.len() == ncols {
                            self.core.index_sets[1].clone_ref(py)
                        } else {
                            Py::new(
                                py,
                                PyIndexSet {
                                    name: format!("_slice_{}", new_ncols),
                                    members: (0..new_ncols)
                                        .map(|i| crate::index_set::IndexMember::Int(i as i64))
                                        .collect(),
                                },
                            )?
                        },
                    ]
                } else {
                    Vec::new()
                };
                let result = PyExprArray::new(new_index_sets, vec![new_nrows, new_ncols], vals);
                Ok(result.into_pyobject(py)?.into_any().unbind())
            }
        }
    }
}

impl_array_ops!(PyExprArray, {
    fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        // Try tuple indexing for multi-dimensional access
        if let Ok(tuple) = index.cast::<PyTuple>() {
            return self.getitem_tuple(py, tuple);
        }

        // Try integer index -> returns Expr
        if let Ok(idx) = index.extract::<usize>() {
            return self
                .core
                .values
                .get(idx)
                .cloned()
                .ok_or_else(|| {
                    ArrayIndexError::new_err(format!(
                        "index {} out of range for array of size {}",
                        idx,
                        self.core.values.len()
                    ))
                })
                .and_then(|v| Ok(v.into_pyobject(py)?.into_any().unbind()));
        }

        // Try boolean numpy array masking
        let np = py.import("numpy")?;
        let ndarray_type = np.getattr("ndarray")?;
        if index.is_instance(&ndarray_type)? {
            let dtype = index.getattr("dtype")?;
            let kind: String = dtype.getattr("kind")?.extract()?;
            if kind == "b" {
                let flat_mask: Vec<bool> = index.call_method0("flatten")?.extract()?;
                if flat_mask.len() != self.core.values.len() {
                    return Err(ArrayShapeMismatchError::new_err(format!(
                        "boolean mask length {} does not match array length {}",
                        flat_mask.len(),
                        self.core.values.len()
                    )));
                }
                let filtered_values: Vec<PyExpr> = self
                    .core
                    .values
                    .iter()
                    .zip(flat_mask.iter())
                    .filter(|(_, m)| **m)
                    .map(|(v, _)| v.clone())
                    .collect();
                let n = filtered_values.len();
                let result = PyExprArray::new(Vec::new(), vec![n], filtered_values);
                return Ok(result.into_pyobject(py)?.into_any().unbind());
            }
        }

        // Try slice -> returns ExprArray
        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let len = self.core.values.len() as isize;
            let indices = slice.indices(len)?;
            let start = indices.start;
            let stop = indices.stop;
            let step = indices.step;

            let mut sliced_values = Vec::new();
            let mut idx = start;
            while (step > 0 && idx < stop) || (step < 0 && idx > stop) {
                let ui = idx as usize;
                sliced_values.push(self.core.values[ui].clone());
                idx += step;
            }
            let n = sliced_values.len();
            let result = PyExprArray::new(Vec::new(), vec![n], sliced_values);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        Err(ArrayIndexError::new_err(
            "index must be an integer, tuple, slice, or a boolean numpy array",
        ))
    }

    fn __repr__(&self) -> String {
        format!("ExprArray(shape={:?})", self.core.shape)
    }
});

// ============================================================================
// PyConstraintArray: a grid of constraint expressions
// ============================================================================

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

// ============================================================================
// Numpy helper functions
// ============================================================================

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
        (w, va.core.clone_with_gil())
    } else if let Ok(ea) = b.extract::<PyRef<'_, PyExprArray>>() {
        let w: Vec<f64> = a.extract()?;
        (w, ea.core.clone_with_gil())
    } else if let Ok(va) = a.extract::<PyRef<'_, PyVariableArray>>() {
        let w: Vec<f64> = b.extract()?;
        (w, va.core.clone_with_gil())
    } else if let Ok(ea) = a.extract::<PyRef<'_, PyExprArray>>() {
        let w: Vec<f64> = b.extract()?;
        (w, ea.core.clone_with_gil())
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

    let mut acc = PyExpr::default();
    for (w, expr) in weights.iter().zip(core.values.iter()) {
        acc = acc.add(expr.scale(*w));
    }
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
            (a.clone(), va.core.clone_with_gil(), false)
        } else if let Ok(ea) = b.extract::<PyRef<'_, PyExprArray>>() {
            (a.clone(), ea.core.clone_with_gil(), false)
        } else if let Ok(va) = a.extract::<PyRef<'_, PyVariableArray>>() {
            (b.clone(), va.core.clone_with_gil(), true)
        } else if let Ok(ea) = a.extract::<PyRef<'_, PyExprArray>>() {
            (b.clone(), ea.core.clone_with_gil(), true)
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
            let mut acc = PyExpr::default();
            for (w, expr) in weights.iter().zip(core.values.iter()) {
                acc = acc.add(expr.scale(*w));
            }
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

            for row in 0..rows {
                let start = row * cols;
                let end = start + cols;
                let mut acc = PyExpr::default();
                for (w, expr) in weights[start..end].iter().zip(core.values.iter()) {
                    acc = acc.add(expr.scale(*w));
                }
                values.push(acc);
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
            all_values.extend(va.core.values.iter().cloned());
        } else if let Ok(ea) = item.extract::<PyRef<'_, PyExprArray>>() {
            has_arrays = true;
            all_values.extend(ea.core.values.iter().cloned());
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
