use crate::sparse_export_dict;
use crate::{PyModel, PyObject};
use arco_model::model::SparseMatrixExport;
use pyo3::prelude::*;

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
