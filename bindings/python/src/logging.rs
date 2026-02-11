//! Logging and diagnostics functions.

use arco_highs::highs_version;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::env;
use std::fs::{File, OpenOptions};
use std::io;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::PyObject;

fn open_log_file(path: &str) -> PyResult<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| PyRuntimeError::new_err(format!("Failed to open log file: {err}")))
}

fn map_init_err<E: std::fmt::Display>(err: E) -> PyErr {
    PyRuntimeError::new_err(format!("Failed to initialize logging: {err}"))
}

/// Enable structured logging for Arco.
///
/// When `level` is None, this reads `ARCO_TRACE` if set. If `ARCO_TRACE` is
/// unset, the default level is `off`. Returns True when logging is initialized,
/// False if a subscriber is already configured.
#[pyfunction]
#[pyo3(signature = (*, level=None))]
pub fn enable_logging(level: Option<String>) -> PyResult<bool> {
    if tracing::dispatcher::has_been_set() {
        return Ok(false);
    }

    let level_value = level
        .or_else(|| env::var("ARCO_TRACE").ok())
        .unwrap_or_else(|| "off".to_string());

    let filter = if level_value.eq_ignore_ascii_case("off") {
        EnvFilter::default().add_directive(LevelFilter::OFF.into())
    } else {
        EnvFilter::try_new(&level_value)
            .map_err(|err| PyRuntimeError::new_err(format!("Invalid log filter: {err}")))?
    };

    let format = env::var("ARCO_LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
    let log_file = env::var("ARCO_LOG_FILE").ok();
    let use_json = format.eq_ignore_ascii_case("json");

    if !use_json && !format.eq_ignore_ascii_case("pretty") {
        return Err(PyRuntimeError::new_err(
            "Invalid ARCO_LOG_FORMAT (expected 'json' or 'pretty')",
        ));
    }

    if use_json {
        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_writer(io::stderr)
            .json();
        let base = tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer);
        if let Some(path) = log_file {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(open_log_file(&path)?)
                .with_ansi(false)
                .json();
            base.with(file_layer).try_init().map_err(map_init_err)?;
        } else {
            base.try_init().map_err(map_init_err)?;
        }
    } else {
        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_writer(io::stderr)
            .pretty();
        let base = tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer);
        if let Some(path) = log_file {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(open_log_file(&path)?)
                .with_ansi(false)
                .pretty();
            base.with(file_layer).try_init().map_err(map_init_err)?;
        } else {
            base.try_init().map_err(map_init_err)?;
        }
    }

    Ok(true)
}

/// Return solver metadata for debugging and diagnostics.
#[pyfunction]
pub fn solver_info(py: Python<'_>) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("solver", "HiGHS")?;
    match highs_version() {
        Some(version) => dict.set_item("version", version)?,
        None => dict.set_item("version", py.None())?,
    }
    Ok(dict.unbind().into())
}

/// Register logging functions with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(enable_logging, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(solver_info, m)?)?;
    Ok(())
}
