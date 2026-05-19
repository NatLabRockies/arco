//! Minimal JSON -> Python conversion helpers used for metadata fields.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde_json::Value;

use crate::PyObject;
use crate::py_modules::errors::MetadataConversionError;

pub fn json_to_py(py: Python<'_>, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(v) => {
            let py_bool = (*v).into_pyobject(py)?;
            Ok(py_bool.to_owned().into_any().unbind())
        }
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                return Ok(i.into_pyobject(py)?.into_any().unbind());
            }
            if let Some(u) = v.as_u64() {
                return Ok(u.into_pyobject(py)?.into_any().unbind());
            }
            if let Some(f) = v.as_f64() {
                return Ok(f.into_pyobject(py)?.into_any().unbind());
            }
            Err(MetadataConversionError::new_err(
                "invalid JSON number value in metadata",
            ))
        }
        Value::String(v) => Ok(v.into_pyobject(py)?.into_any().unbind()),
        Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                let py_item = json_to_py(py, item)?;
                list.append(py_item.bind(py))?;
            }
            Ok(list.unbind().into_any())
        }
        Value::Object(items) => {
            let dict = PyDict::new(py);
            for (k, v) in items {
                let py_value = json_to_py(py, v)?;
                dict.set_item(k, py_value.bind(py))?;
            }
            Ok(dict.unbind().into_any())
        }
    }
}
