use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyObject;
use crate::errors::{ArrayDimensionError, ArrayIndexError};
use crate::expr::PyExpr;
use crate::index_set::PyIndexSet;
use crate::variable::PyVariable;

use super::LinearArrayCore;
use super::indexing::{
    AxisIndex, maybe_boolean_mask_indices, resolve_axis_index, slice_indices, sliced_2d_index_sets,
};

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
                let var = self.variables.get(flat_idx).cloned().ok_or_else(|| {
                    ArrayIndexError::new_err(format!("flat index {} out of range", flat_idx))
                })?;
                Ok(var.into_pyobject(py)?.into_any().unbind())
            }
            (AxisIndex::Single(r), AxisIndex::Range(col_indices)) => {
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
                let new_index_sets = sliced_2d_index_sets(
                    py,
                    &self.core.index_sets,
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

super::impl_array_ops!(PyVariableArray, {
    #[getter]
    fn variables(&self) -> Vec<PyVariable> {
        self.variables.clone()
    }

    fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(tuple) = index.cast::<PyTuple>() {
            return self.getitem_tuple(py, tuple);
        }

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

        if let Some(mask_indices) = maybe_boolean_mask_indices(py, index, self.core.values.len())? {
            let filtered_values = mask_indices
                .iter()
                .map(|&idx| self.core.values[idx].clone())
                .collect::<Vec<PyExpr>>();
            let filtered_variables = mask_indices
                .iter()
                .map(|&idx| self.variables[idx].clone())
                .collect::<Vec<PyVariable>>();
            let n = filtered_values.len();
            let result =
                PyVariableArray::new(Vec::new(), vec![n], filtered_values, filtered_variables);
            return Ok(result.into_pyobject(py)?.into_any().unbind());
        }

        if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
            let selected = slice_indices(slice, self.core.values.len())?;
            let sliced_values = selected
                .iter()
                .map(|&idx| self.core.values[idx].clone())
                .collect::<Vec<PyExpr>>();
            let sliced_variables = selected
                .iter()
                .map(|&idx| self.variables[idx].clone())
                .collect::<Vec<PyVariable>>();
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
