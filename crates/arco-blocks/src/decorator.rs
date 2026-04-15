use crate::PyObject;
use crate::schema::{dataclass_fields, is_dataclass_schema, is_pydantic_schema};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

pub(crate) const ARCO_BLOCK_MARKER_ATTR: &str = "__arco_block_marker__";
pub(crate) const ARCO_BLOCK_NAME_ATTR: &str = "__arco_block_name__";
pub(crate) const ARCO_BLOCK_INPUT_SCHEMA_ATTR: &str = "__arco_block_input_schema__";
pub(crate) const ARCO_BLOCK_INPUT_FIELDS_ATTR: &str = "__arco_block_input_fields__";
pub(crate) const ARCO_BLOCK_EXPECTS_CTX_ATTR: &str = "__arco_block_expects_ctx__";

#[pyclass]
struct FunctionBlockDecorator {
    name: Option<String>,
}

#[pymethods]
impl FunctionBlockDecorator {
    fn __call__(&self, py: Python<'_>, func: PyObject) -> PyResult<PyObject> {
        decorate_block_function(py, func.bind(py), self.name.as_deref())
    }
}

#[pyfunction]
#[pyo3(signature = (func=None, *, name=None))]
pub(crate) fn block(
    py: Python<'_>,
    func: Option<PyObject>,
    name: Option<String>,
) -> PyResult<PyObject> {
    if let Some(func) = func {
        return decorate_block_function(py, func.bind(py), name.as_deref());
    }
    Ok(Py::new(py, FunctionBlockDecorator { name })?.into_any())
}

fn decorate_block_function(
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    name_override: Option<&str>,
) -> PyResult<PyObject> {
    let (name, input_schema, input_fields, expects_ctx) =
        typed_block_meta_from_function(py, func, name_override)?;
    func.setattr(ARCO_BLOCK_MARKER_ATTR, true)?;
    func.setattr(ARCO_BLOCK_NAME_ATTR, name)?;
    func.setattr(ARCO_BLOCK_INPUT_SCHEMA_ATTR, input_schema)?;
    func.setattr(ARCO_BLOCK_INPUT_FIELDS_ATTR, input_fields.bind(py))?;
    func.setattr(ARCO_BLOCK_EXPECTS_CTX_ATTR, expects_ctx)?;
    Ok(func.clone().unbind())
}

fn typed_block_meta_from_function(
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    name_override: Option<&str>,
) -> PyResult<(String, PyObject, Py<PyDict>, bool)> {
    if !func.is_callable() {
        return Err(PyTypeError::new_err("block: expected a callable"));
    }
    let inspect = PyModule::import(py, "inspect")?;
    let signature = inspect.getattr("signature")?.call1((func,))?;
    let empty = inspect.getattr("_empty")?;
    let parameter = inspect.getattr("Parameter")?;
    let var_positional = parameter.getattr("VAR_POSITIONAL")?;
    let var_keyword = parameter.getattr("VAR_KEYWORD")?;
    let keyword_only = parameter.getattr("KEYWORD_ONLY")?;

    let mut params: Vec<PyObject> = Vec::new();
    let parameter_values = signature.getattr("parameters")?.call_method0("values")?;
    for param in parameter_values.try_iter()? {
        params.push(param?.unbind());
    }
    if params.len() != 2 && params.len() != 3 {
        return Err(PyTypeError::new_err(
            "block: expected signature (model, data) or (model, data, ctx)",
        ));
    }
    for param in &params {
        let kind = param.bind(py).getattr("kind")?;
        if kind.eq(&var_positional)? || kind.eq(&var_keyword)? {
            return Err(PyTypeError::new_err(
                "block: variadic *args/**kwargs are not supported",
            ));
        }
        if kind.eq(&keyword_only)? {
            return Err(PyTypeError::new_err(
                "block: keyword-only parameters are not supported",
            ));
        }
    }

    let input_schema = params[1].bind(py).getattr("annotation")?;
    if input_schema.is(&empty) {
        return Err(PyTypeError::new_err(
            "block: data parameter must include a schema annotation",
        ));
    }
    if !is_dataclass_schema(py, &input_schema)? && !is_pydantic_schema(py, &input_schema)? {
        return Err(PyTypeError::new_err(
            "block: data annotation must be a dataclass or pydantic BaseModel type",
        ));
    }

    let input_fields = if is_dataclass_schema(py, &input_schema)? {
        dataclass_fields(py, &input_schema)?
    } else {
        let out = PyDict::new(py);
        let model_fields_any = input_schema.getattr("model_fields")?;
        let model_fields = model_fields_any.cast::<PyDict>()?;
        for (name, field) in model_fields.iter() {
            out.set_item(name, field.getattr("annotation")?)?;
        }
        out.unbind()
    };

    let name = if let Some(name) = name_override {
        name.to_string()
    } else {
        func.getattr("__name__")?.extract::<String>()?
    };
    Ok((name, input_schema.unbind(), input_fields, params.len() == 3))
}
