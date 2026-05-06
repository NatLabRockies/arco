use crate::sparse_export_dict;
use crate::{PyModel, PyObject};
use arco_ops::model::model::SparseMatrixExport;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub(crate) fn get_columns(model: &PyModel, py: Python<'_>) -> PyResult<PyObject> {
    let dict = PyDict::new(py);

    for (var_id, coeffs) in model.inner.columns() {
        let coeff_list: Vec<(u32, f64)> = coeffs
            .iter()
            .map(|(cid, coeff)| (cid.inner(), *coeff))
            .collect();
        dict.set_item(var_id.inner(), coeff_list)?;
    }

    Ok(dict.unbind().into())
}

pub(crate) fn export_csc(model: &PyModel, py: Python<'_>) -> PyResult<PyObject> {
    let matrix = model.inner.export_csc();
    sparse_export_dict(py, matrix.shape, |dict| {
        dict.set_item("col_ptrs", matrix.col_ptrs)?;
        dict.set_item("row_indices", matrix.row_indices)?;
        dict.set_item("values", matrix.values)
    })
}

pub(crate) fn export_crs(model: &PyModel, py: Python<'_>) -> PyResult<PyObject> {
    let matrix = model.inner.export_crs();
    sparse_export_dict(py, matrix.shape, |dict| {
        dict.set_item("row_ptrs", matrix.row_ptrs)?;
        dict.set_item("col_indices", matrix.col_indices)?;
        dict.set_item("values", matrix.values)
    })
}

pub(crate) fn export_coo(model: &PyModel, py: Python<'_>) -> PyResult<PyObject> {
    let matrix = model.inner.export_coo();
    sparse_export_dict(py, matrix.shape, |dict| {
        dict.set_item("rows", matrix.rows)?;
        dict.set_item("cols", matrix.cols)?;
        dict.set_item("values", matrix.values)
    })
}

pub(crate) fn export_arrow() -> PyResult<PyObject> {
    Err(PyRuntimeError::new_err(
        "Arrow export is not enabled in this build",
    ))
}
