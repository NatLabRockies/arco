//! Minimal JSON <-> Python conversion helpers used for metadata fields.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use serde_json::{Map, Number, Value};

use crate::PyObject;

pub fn py_any_to_json(value: &Bound<'_, PyAny>) -> PyResult<Value> {
    if value.is_none() {
        return Ok(Value::Null);
    }
    if let Ok(v) = value.extract::<bool>() {
        return Ok(Value::Bool(v));
    }
    if let Ok(v) = value.extract::<i64>() {
        return Ok(Value::Number(Number::from(v)));
    }
    if let Ok(v) = value.extract::<u64>() {
        return Ok(Value::Number(Number::from(v)));
    }
    if let Ok(v) = value.extract::<f64>() {
        let number = Number::from_f64(v).ok_or_else(|| {
            PyValueError::new_err("metadata float values must be finite JSON numbers")
        })?;
        return Ok(Value::Number(number));
    }
    if let Ok(v) = value.extract::<String>() {
        return Ok(Value::String(v));
    }
    if let Ok(dict) = value.cast::<PyDict>() {
        let mut object = Map::with_capacity(dict.len());
        for (k, v) in dict.iter() {
            let key = k.extract::<String>().map_err(|_| {
                PyTypeError::new_err("metadata dict keys must be strings for JSON conversion")
            })?;
            object.insert(key, py_any_to_json(&v)?);
        }
        return Ok(Value::Object(object));
    }
    if let Ok(list) = value.cast::<PyList>() {
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(py_any_to_json(&item)?);
        }
        return Ok(Value::Array(items));
    }
    if let Ok(tuple) = value.cast::<PyTuple>() {
        let mut items = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            items.push(py_any_to_json(&item)?);
        }
        return Ok(Value::Array(items));
    }
    Err(PyTypeError::new_err(
        "metadata must contain JSON-compatible values",
    ))
}

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
            Err(PyValueError::new_err("invalid JSON number value"))
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
