use crate::error::BlockError;
use arco_tools::capture_rss_bytes;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};

type PyObject = Py<PyAny>;

pub(crate) fn log_block_error(err: BlockError) -> PyErr {
    tracing::error!(
        component = "block",
        operation = "solve",
        status = "error",
        "{err}"
    );
    PyRuntimeError::new_err(err.to_string())
}

pub(crate) fn rss_bytes() -> Option<u64> {
    capture_rss_bytes("block")
}

pub(crate) fn log_block_phase(
    block: &str,
    phase: &str,
    duration_ms: f64,
    rss_bytes: Option<u64>,
    rss_delta_bytes: Option<i64>,
    warm_start: bool,
) {
    tracing::info!(
        component = "block",
        operation = phase,
        status = "success",
        block,
        phase,
        cache_hit = false,
        warm_start,
        duration_ms,
        rss_bytes,
        rss_delta_bytes,
        "Block phase complete"
    );
}

pub(crate) fn model_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
    let module = PyModule::import(py, "arco.arco")?;
    let model_any = module.getattr("Model")?;
    Ok(model_any.cast::<PyType>()?.clone())
}

pub(crate) fn create_model(py: Python<'_>) -> PyResult<PyObject> {
    let model_type = model_type(py)?;
    Ok(model_type.call0()?.unbind())
}
