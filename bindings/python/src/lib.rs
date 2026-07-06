use pyo3::prelude::*;

macro_rules! wrap_pyfunction {
    ($function:path, $py_or_module:expr) => {{
        use $function as wrapped_pyfunction;
        pyo3::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
            $py_or_module,
            &wrapped_pyfunction::_PYO3_DEF,
        )
    }};
}

#[path = "logging.rs"]
mod logging;
#[path = "solution_summary.rs"]
mod solution_summary;

pub(crate) type PyObject = Py<PyAny>;

/// The Arco Python module.
#[pyo3_macros::pymodule]
fn arco(m: &Bound<'_, PyModule>) -> PyResult<()> {
    arco_python_core::register(m)?;
    logging::register(m)?;
    solution_summary::register(m)
}
