use crate::sparse_export_dict;
use crate::{PyModel, PyObject};
use arco_model::model::SparseMatrixExport;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

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

pub(crate) fn write_lp(model: &PyModel, path: &Path) -> PyResult<()> {
    let path_display = path.display().to_string();
    let file = File::create(path).map_err(|error| {
        PyIOError::new_err(format!("failed to write LP file {path_display}: {error}"))
    })?;
    let mut writer = BufWriter::new(file);
    arco_format::write_model_view_lp(&model.inner, &mut writer).map_err(|error| {
        PyIOError::new_err(format!("failed to write LP file {path_display}: {error}"))
    })?;
    writer.flush().map_err(|error| {
        PyIOError::new_err(format!("failed to write LP file {path_display}: {error}"))
    })
}
