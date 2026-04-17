use crate::BlockContext;
use crate::PyObject;
use crate::schema::{is_dataclass_schema, is_pydantic_schema, validate_data};
use crate::util::create_model;
use pyo3::exceptions::{PyAttributeError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyString};

#[pyclass(subclass, dict, name = "BlockSpec")]
pub struct BlockSpec {
    pub(crate) build_fn: Option<Py<PyAny>>,
}

#[pymethods]
impl BlockSpec {
    #[new]
    fn new() -> Self {
        Self { build_fn: None }
    }

    #[pyo3(signature = (model, *, data, ctx))]
    fn build(
        &self,
        py: Python<'_>,
        model: PyObject,
        data: PyObject,
        ctx: PyObject,
    ) -> PyResult<PyObject> {
        let Some(build_fn) = &self.build_fn else {
            return Err(PyRuntimeError::new_err(
                "ARCO_BLOCK_502: build() is not implemented",
            ));
        };
        let kwargs = PyDict::new(py);
        kwargs.set_item("data", data.clone_ref(py))?;
        kwargs.set_item("ctx", ctx.clone_ref(py))?;
        let result = build_fn
            .bind(py)
            .call((model.clone_ref(py),), Some(&kwargs))?;
        Ok(result.unbind())
    }
}

pub(crate) fn validate_spec(spec: &Bound<'_, PyAny>) -> PyResult<()> {
    if spec.get_type().name()?.to_str()? == "BlockSpec" {
        let msg = "ARCO_BLOCK_502: BlockSpec is abstract";
        tracing::error!(
            component = "block",
            operation = "from_spec",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    if !spec.hasattr("name")? || spec.getattr("name")?.is_none() {
        let msg = "ARCO_BLOCK_501: BlockSpec must have a non-empty 'name' attribute";
        tracing::error!(
            component = "block",
            operation = "from_spec",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    if !spec.hasattr("data_schema")? {
        let msg = "ARCO_BLOCK_501: BlockSpec must have a 'data_schema' attribute";
        tracing::error!(
            component = "block",
            operation = "from_spec",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    if !spec.hasattr("outputs_schema")? {
        let msg = "ARCO_BLOCK_501: BlockSpec must have an 'outputs_schema' attribute";
        tracing::error!(
            component = "block",
            operation = "from_spec",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    if !spec.hasattr("build")? {
        let msg = "ARCO_BLOCK_501: BlockSpec must have a callable 'build' method";
        tracing::error!(
            component = "block",
            operation = "from_spec",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    if !spec.getattr("build")?.is_callable() {
        let msg = "ARCO_BLOCK_501: BlockSpec must have a callable 'build' method";
        tracing::error!(
            component = "block",
            operation = "from_spec",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    Ok(())
}

pub(crate) fn get_spec_attr<'py>(
    spec: &Bound<'py, PyAny>,
    name: &str,
) -> PyResult<Bound<'py, PyAny>> {
    spec.getattr(name)
}

pub(crate) fn spec_version_or_default(spec: &Bound<'_, PyAny>) -> PyResult<String> {
    match spec.getattr("version") {
        Ok(value) => value.extract::<String>().map_err(|_| {
            let msg = "ARCO_BLOCK_502: BlockSpec 'version' attribute must be str";
            tracing::error!(
                component = "block",
                operation = "spec_version",
                status = "error",
                "{msg}"
            );
            PyRuntimeError::new_err(msg)
        }),
        Err(error) => {
            if error.is_instance_of::<PyAttributeError>(spec.py()) {
                Ok("0.0.0".to_string())
            } else {
                Err(error)
            }
        }
    }
}

#[pyclass]
struct SpecBuilder {
    spec: Py<PyAny>,
    _slack_penalty: f64,
}

#[pymethods]
impl SpecBuilder {
    fn __call__(&self, py: Python<'_>, ctx: PyObject) -> PyResult<PyObject> {
        let ctx_ref: PyRef<'_, BlockContext> = ctx.bind(py).extract()?;
        let data_raw = ctx_ref.inputs.bind(py).get_item("data")?;
        let data_raw = data_raw.ok_or_else(|| {
            PyRuntimeError::new_err("ARCO_BLOCK_502: Missing data input for spec build")
        })?;
        let spec = self.spec.bind(py);
        let data_schema = get_spec_attr(spec, "data_schema")?;
        let data_validated = validate_data(py, data_raw.unbind(), &data_schema, "Block input")?;
        ctx_ref
            .attachments
            .bind(py)
            .set_item("_spec_name", get_spec_attr(spec, "name")?.unbind())?;
        let spec_version = spec_version_or_default(spec)?;
        ctx_ref.attachments.bind(py).set_item(
            "_spec_version",
            PyString::new(py, &spec_version).into_any().unbind(),
        )?;
        let model = create_model(py)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("data", data_validated.clone_ref(py))?;
        kwargs.set_item("ctx", ctx.clone_ref(py))?;
        let outputs = spec.call_method("build", (model.clone_ref(py),), Some(&kwargs))?;
        let outputs_schema = get_spec_attr(spec, "outputs_schema")?;
        let outputs_validated =
            validate_data(py, outputs.unbind(), &outputs_schema, "Block output")?;
        ctx_ref
            .attachments
            .bind(py)
            .set_item("_outputs", outputs_validated)?;
        Ok(model)
    }
}

#[pyclass]
struct SpecExtractor {
    spec: Py<PyAny>,
}

#[pymethods]
impl SpecExtractor {
    fn __call__(
        &self,
        py: Python<'_>,
        _solution: PyObject,
        ctx: &BlockContext,
    ) -> PyResult<PyObject> {
        let outputs = ctx.attachments.bind(py).get_item("_outputs")?;
        let Some(outputs) = outputs else {
            return Ok(PyDict::new(py).into_any().unbind());
        };
        if outputs.is_instance_of::<PyDict>() {
            return Ok(outputs.unbind());
        }
        let spec = self.spec.bind(py);
        if is_pydantic_schema(py, &get_spec_attr(spec, "outputs_schema")?)? {
            return Ok(outputs.call_method0("model_dump")?.unbind());
        }
        if is_dataclass_schema(py, &get_spec_attr(spec, "outputs_schema")?)? {
            let dataclasses = PyModule::import(py, "dataclasses")?;
            let asdict = dataclasses.getattr("asdict")?;
            return Ok(asdict.call1((outputs,))?.unbind());
        }
        Ok(PyDict::new(py).into_any().unbind())
    }
}

pub(crate) fn make_spec_builder(
    py: Python<'_>,
    spec: Py<PyAny>,
    slack_penalty: f64,
) -> PyResult<PyObject> {
    Ok(Py::new(
        py,
        SpecBuilder {
            spec,
            _slack_penalty: slack_penalty,
        },
    )?
    .into_any())
}

pub(crate) fn make_spec_extractor(py: Python<'_>, spec: Py<PyAny>) -> PyResult<PyObject> {
    Ok(Py::new(py, SpecExtractor { spec })?.into_any())
}

#[cfg(test)]
mod tests {
    use crate::spec::spec_version_or_default;
    use pyo3::prelude::*;

    #[test]
    fn spec_version_defaults_to_zero_when_attribute_missing() {
        Python::initialize();
        Python::attach(|py| {
            let types = PyModule::import(py, "types").expect("types module should import");
            let namespace = types
                .getattr("SimpleNamespace")
                .expect("SimpleNamespace should exist")
                .call0()
                .expect("SimpleNamespace() should construct");

            let version = spec_version_or_default(&namespace)
                .expect("missing version should default to 0.0.0");
            assert_eq!(version, "0.0.0");
        });
    }

    #[test]
    fn spec_version_rejects_non_string_values() {
        Python::initialize();
        Python::attach(|py| {
            let types = PyModule::import(py, "types").expect("types module should import");
            let namespace = types
                .getattr("SimpleNamespace")
                .expect("SimpleNamespace should exist")
                .call0()
                .expect("SimpleNamespace() should construct");
            namespace
                .setattr("version", 7_i32)
                .expect("setting version should succeed");

            let error = spec_version_or_default(&namespace)
                .expect_err("non-string version should fail fast");
            assert!(
                error
                    .to_string()
                    .contains("BlockSpec 'version' attribute must be str"),
                "unexpected error: {error}"
            );
        });
    }
}
