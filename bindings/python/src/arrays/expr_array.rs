use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyObject;
use crate::errors::{ArrayDimensionError, ArrayIndexError};
use crate::expr::PyExpr;
use crate::index_set::PyIndexSet;

use super::LinearArrayCore;
use super::indexing::{
    AxisIndex, maybe_boolean_mask_indices, resolve_axis_index, slice_indices, sliced_2d_index_sets,
};

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
                let new_index_sets = sliced_2d_index_sets(
                    py,
                    &self.core.index_sets,
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
}

super::impl_array_ops!(PyExprArray, {
    fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(tuple) = index.cast::<PyTuple>() {
            return self.getitem_tuple(py, tuple);
        }

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

        if let Some(mask_indices) = maybe_boolean_mask_indices(py, index, self.core.values.len())? {
            let filtered_values = mask_indices
                .iter()
                .map(|&idx| self.core.values[idx].clone())
                .collect::<Vec<PyExpr>>();
            let n = filtered_values.len();
            let result = PyExprArray::new(Vec::new(), vec![n], filtered_values);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let selected = slice_indices(slice, self.core.values.len())?;
            let sliced_values = selected
                .iter()
                .map(|&idx| self.core.values[idx].clone())
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
        format!("ExprArray(shape={:?})", self.core.shape)
    }
});
