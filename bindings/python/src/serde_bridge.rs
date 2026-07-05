//! Minimal JSON -> Python conversion helpers used for metadata fields.

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use serde_json::{Number, Value};

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

pub fn py_to_json(value: &Bound<'_, PyAny>) -> PyResult<Value> {
    if value.is_none() {
        return Ok(Value::Null);
    }
    if value.is_instance_of::<PyBool>() {
        return Ok(Value::Bool(value.extract()?));
    }
    if value.is_instance_of::<PyInt>() {
        if let Ok(v) = value.extract::<i64>() {
            return Ok(Value::Number(Number::from(v)));
        }
        if let Ok(v) = value.extract::<u64>() {
            return Ok(Value::Number(Number::from(v)));
        }
        return Err(MetadataConversionError::new_err(
            "metadata integers must fit in signed or unsigned 64-bit range",
        ));
    }
    if value.is_instance_of::<PyFloat>() {
        let number = Number::from_f64(value.extract::<f64>()?)
            .ok_or_else(|| MetadataConversionError::new_err("metadata floats must be finite"))?;
        return Ok(Value::Number(number));
    }
    if value.is_instance_of::<PyString>() {
        return Ok(Value::String(value.extract()?));
    }
    if let Ok(dict) = value.cast::<PyDict>() {
        let mut items = serde_json::Map::with_capacity(dict.len());
        for (key, item) in dict.iter() {
            let key = key.extract::<String>().map_err(|_| {
                MetadataConversionError::new_err("metadata object keys must be strings")
            })?;
            items.insert(key, py_to_json(&item)?);
        }
        return Ok(Value::Object(items));
    }
    if let Ok(list) = value.cast::<PyList>() {
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(py_to_json(&item)?);
        }
        return Ok(Value::Array(items));
    }
    Err(MetadataConversionError::new_err(
        "metadata must be JSON-compatible",
    ))
}
