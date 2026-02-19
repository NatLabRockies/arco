use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyType};
use std::collections::HashSet;

type PyObject = Py<PyAny>;

pub(crate) fn is_pydantic_schema(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<bool> {
    let Ok(schema_type) = schema.cast::<PyType>() else {
        return Ok(false);
    };
    let pydantic = PyModule::import(py, "pydantic")?;
    let base_model_any = pydantic.getattr("BaseModel")?;
    let base_model = base_model_any.cast::<PyType>()?;
    schema_type.is_subclass(base_model)
}

pub(crate) fn is_dataclass_schema(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<bool> {
    let dataclasses = PyModule::import(py, "dataclasses")?;
    let is_dataclass = dataclasses.getattr("is_dataclass")?;
    is_dataclass.call1((schema,))?.extract::<bool>()
}

pub(crate) fn dataclass_fields(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<Py<PyDict>> {
    let dataclasses = PyModule::import(py, "dataclasses")?;
    let fields = dataclasses.getattr("fields")?.call1((schema,))?;
    let dict = PyDict::new(py);
    for item in fields.try_iter()? {
        let item = item?;
        let name = item.getattr("name")?;
        let field_type = item.getattr("type")?;
        dict.set_item(name, field_type)?;
    }
    Ok(dict.unbind())
}

pub(crate) fn compare_fields(
    fields_a: &Bound<'_, PyDict>,
    fields_b: &Bound<'_, PyDict>,
) -> PyResult<(bool, String)> {
    let keys_a: HashSet<String> = fields_a
        .keys()
        .iter()
        .filter_map(|key| key.extract::<String>().ok())
        .collect();
    let keys_b: HashSet<String> = fields_b
        .keys()
        .iter()
        .filter_map(|key| key.extract::<String>().ok())
        .collect();
    if keys_a != keys_b {
        let missing_a: HashSet<_> = keys_b.difference(&keys_a).cloned().collect();
        let missing_b: HashSet<_> = keys_a.difference(&keys_b).cloned().collect();
        let mut parts = Vec::new();
        if !missing_a.is_empty() {
            parts.push(format!("missing in first: {missing_a:?}"));
        }
        if !missing_b.is_empty() {
            parts.push(format!("missing in second: {missing_b:?}"));
        }
        return Ok((false, parts.join(", ")));
    }
    for key in keys_a {
        let value_a = fields_a.get_item(&key)?.unwrap();
        let value_b = fields_b.get_item(&key)?.unwrap();
        if !value_a.eq(&value_b)? {
            let value_a_str = value_a.str()?.to_str()?.to_string();
            let value_b_str = value_b.str()?.to_str()?.to_string();
            return Ok((
                false,
                format!("Type mismatch for field '{key}': {value_a_str} != {value_b_str}"),
            ));
        }
    }
    Ok((true, String::new()))
}

pub(crate) fn validate_data(
    py: Python<'_>,
    data: PyObject,
    schema: &Bound<'_, PyAny>,
    context: &str,
) -> PyResult<PyObject> {
    if data.is_none(py) {
        let msg = format!(
            "ARCO_BLOCK_502: {context} received None data for schema {}",
            schema.get_type().name()?
        );
        tracing::error!(
            component = "block",
            operation = "validate_data",
            status = "error",
            "{msg}"
        );
        return Err(PyValueError::new_err(msg));
    }
    if data.bind(py).is_instance(schema)? {
        return Ok(data);
    }
    if is_pydantic_schema(py, schema)? {
        match schema.call_method1("model_validate", (data.clone_ref(py),)) {
            Ok(validated) => return Ok(validated.unbind()),
            Err(err) => {
                let msg = format!("ARCO_BLOCK_502: {context} validation failed: {err}");
                tracing::error!(
                    component = "block",
                    operation = "validate_data",
                    status = "error",
                    "{msg}"
                );
                return Err(PyValueError::new_err(msg));
            }
        }
    }
    if is_dataclass_schema(py, schema)? {
        let data_any = data.bind(py);
        if data_any.is_instance_of::<PyDict>() {
            let dict = data_any.cast::<PyDict>()?;
            match schema.call((), Some(dict)) {
                Ok(instance) => return Ok(instance.unbind()),
                Err(err) => {
                    let msg =
                        format!("ARCO_BLOCK_502: {context} dataclass construction failed: {err}");
                    tracing::error!(
                        component = "block",
                        operation = "validate_data",
                        status = "error",
                        "{msg}"
                    );
                    return Err(PyValueError::new_err(msg));
                }
            }
        }
        let msg = format!(
            "ARCO_BLOCK_502: {context} dataclass requires dict, got {}",
            data_any.get_type().name()?
        );
        tracing::error!(
            component = "block",
            operation = "validate_data",
            status = "error",
            "{msg}"
        );
        return Err(PyValueError::new_err(msg));
    }
    let msg = format!(
        "ARCO_BLOCK_502: {context} unsupported schema type {}",
        schema.get_type().name()?
    );
    tracing::error!(
        component = "block",
        operation = "validate_data",
        status = "error",
        "{msg}"
    );
    Err(PyValueError::new_err(msg))
}

pub(crate) fn outputs_schema_dict(
    py: Python<'_>,
    schema: &Bound<'_, PyAny>,
) -> PyResult<Py<PyDict>> {
    if is_pydantic_schema(py, schema)? {
        let fields_any = schema.getattr("model_fields")?;
        let fields = fields_any.cast::<PyDict>()?;
        let dict = PyDict::new(py);
        for key in fields.keys() {
            dict.set_item(key, py.None())?;
        }
        return Ok(dict.unbind());
    }
    if is_dataclass_schema(py, schema)? {
        let dict = PyDict::new(py);
        let fields = dataclass_fields(py, schema)?;
        for key in fields.bind(py).keys() {
            dict.set_item(key, py.None())?;
        }
        return Ok(dict.unbind());
    }
    Ok(PyDict::new(py).unbind())
}

pub(crate) fn coerce_inputs<'py>(
    py: Python<'py>,
    block_name: &str,
    input_schemas: &Bound<'py, PyDict>,
    inputs: Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyDict>> {
    let coerced = PyDict::new(py);
    for (key, value) in inputs.iter() {
        let schema = input_schemas.get_item(&key)?;
        let coerced_value = coerce_schema(py, value.unbind(), schema, block_name, &key, "input")?;
        coerced.set_item(key, coerced_value)?;
    }
    Ok(coerced)
}

pub(crate) fn coerce_outputs<'py>(
    py: Python<'py>,
    block_name: &str,
    output_schemas: &Bound<'py, PyDict>,
    outputs: Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyDict>> {
    let coerced = PyDict::new(py);
    for (key, value) in outputs.iter() {
        let schema = output_schemas.get_item(&key)?;
        let coerced_value = coerce_schema(py, value.unbind(), schema, block_name, &key, "output")?;
        coerced.set_item(key, coerced_value)?;
    }
    Ok(coerced)
}

fn coerce_schema(
    py: Python<'_>,
    value: PyObject,
    schema: Option<Bound<'_, PyAny>>,
    block: &str,
    key: &Bound<'_, PyAny>,
    kind: &str,
) -> PyResult<PyObject> {
    let Some(schema) = schema else {
        return Ok(value);
    };
    if schema.is_none() {
        return Ok(value);
    }
    if value.bind(py).is_instance(&schema)? {
        return Ok(value);
    }
    if is_pydantic_schema(py, &schema)? {
        let validated = schema.call_method1("model_validate", (value.clone_ref(py),))?;
        return Ok(validated.unbind());
    }
    if is_dataclass_schema(py, &schema)? {
        if value.bind(py).is_instance_of::<PyDict>() {
            let dict = value.bind(py).cast::<PyDict>()?;
            let instance = schema.call((), Some(dict))?;
            return Ok(instance.unbind());
        }
        let msg = format!(
            "ARCO_BLOCK_502: {kind} '{}' for block '{block}' does not match schema",
            key.str()?.to_str()?
        );
        tracing::error!(
            component = "block",
            operation = "validate",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    let msg = format!(
        "ARCO_BLOCK_502: {kind} '{}' for block '{block}' does not match schema",
        key.str()?.to_str()?
    );
    tracing::error!(
        component = "block",
        operation = "validate",
        status = "error",
        "{msg}"
    );
    Err(PyRuntimeError::new_err(msg))
}
