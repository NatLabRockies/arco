use crate::once_map::OnceMap;
use crate::schema::{
    compare_fields, dataclass_fields, is_dataclass_schema, is_pydantic_schema, validate_data,
};
use crate::spec::{get_spec_attr, validate_spec, BlockSpec};
use crate::util::create_model;
use crate::{Block, BlockContext, BlockLink, BlockRun, BuildResult};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

type PyObject = Py<PyAny>;

#[pyfunction]
#[pyo3(signature = (*, name, data_schema, outputs_schema, build, version="0.0.0"))]
pub(crate) fn block_spec(
    py: Python<'_>,
    name: String,
    data_schema: PyObject,
    outputs_schema: PyObject,
    build: PyObject,
    version: &str,
) -> PyResult<Py<BlockSpec>> {
    let spec = BlockSpec {
        build_fn: Some(build),
    };
    let spec = Py::new(py, spec)?;
    let spec_ref = spec.bind(py);
    spec_ref.setattr("name", name)?;
    spec_ref.setattr("data_schema", data_schema)?;
    spec_ref.setattr("outputs_schema", outputs_schema)?;
    spec_ref.setattr("version", version)?;
    Ok(spec)
}

#[pyfunction]
#[pyo3(signature = (*, spec, data, allow_slacks=false, slack_penalty=1e6))]
pub(crate) fn build_model_from_spec(
    py: Python<'_>,
    spec: &Bound<'_, PyAny>,
    data: PyObject,
    allow_slacks: bool,
    slack_penalty: f64,
) -> PyResult<BuildResult> {
    let _ = slack_penalty;
    if allow_slacks {
        let msg = "ARCO_BLOCK_502: allow_slacks is not yet implemented. Inject slacks in your spec.build() method instead.";
        tracing::error!(
            component = "block",
            operation = "build_model_from_spec",
            status = "error",
            "{msg}"
        );
        return Err(PyRuntimeError::new_err(msg));
    }
    validate_spec(spec)?;
    let data_schema = get_spec_attr(spec, "data_schema")?;
    let data_validated = validate_data(py, data, &data_schema, "build_model_from_spec")?;
    let inputs = PyDict::new(py);
    inputs.set_item("data", data_validated.clone_ref(py))?;
    let ctx = Py::new(
        py,
        BlockContext {
            inputs: inputs.unbind(),
            attachments: PyDict::new(py).unbind(),
        },
    )?;
    let model = create_model(py)?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("data", data_validated.clone_ref(py))?;
    kwargs.set_item("ctx", ctx.clone_ref(py))?;
    let outputs = spec.call_method("build", (model.clone_ref(py),), Some(&kwargs))?;
    let outputs_schema = get_spec_attr(spec, "outputs_schema")?;
    let outputs_validated = validate_data(
        py,
        outputs.unbind(),
        &outputs_schema,
        "build_model_from_spec output",
    )?;
    let spec_name = get_spec_attr(spec, "name")?.extract::<String>()?;
    let spec_version = get_spec_attr(spec, "version")
        .ok()
        .and_then(|value| value.extract::<String>().ok())
        .unwrap_or_else(|| "0.0.0".to_string());
    Ok(BuildResult {
        model,
        outputs: outputs_validated,
        spec_name,
        spec_version,
    })
}

#[pyfunction]
#[pyo3(signature = (*, model, constraints=None, variables=None, include_coeffs=false, include_slacks=true))]
pub(crate) fn inspect_model(
    py: Python<'_>,
    model: PyObject,
    constraints: Option<Vec<u32>>,
    variables: Option<Vec<u32>>,
    include_coeffs: bool,
    include_slacks: bool,
) -> PyResult<PyObject> {
    let kwargs = PyDict::new(py);
    kwargs.set_item("constraint_ids", constraints)?;
    kwargs.set_item("variable_ids", variables)?;
    kwargs.set_item("include_coeffs", include_coeffs)?;
    kwargs.set_item("include_slacks", include_slacks)?;
    Ok(model
        .bind(py)
        .call_method("inspect", (), Some(&kwargs))?
        .unbind())
}

#[pyfunction]
pub(crate) fn schemas_compatible(
    py: Python<'_>,
    schema_a: &Bound<'_, PyAny>,
    schema_b: &Bound<'_, PyAny>,
) -> PyResult<(bool, String)> {
    if schema_a.is(schema_b) {
        return Ok((true, String::new()));
    }
    let result = if is_pydantic_schema(py, schema_a)? && is_pydantic_schema(py, schema_b)? {
        let fields_any_a = schema_a.getattr("model_fields")?;
        let fields_a = fields_any_a.cast::<PyDict>()?;
        let fields_any_b = schema_b.getattr("model_fields")?;
        let fields_b = fields_any_b.cast::<PyDict>()?;
        compare_fields(fields_a, fields_b)?
    } else if is_dataclass_schema(py, schema_a)? && is_dataclass_schema(py, schema_b)? {
        let fields_a = dataclass_fields(py, schema_a)?;
        let fields_b = dataclass_fields(py, schema_b)?;
        compare_fields(fields_a.bind(py), fields_b.bind(py))?
    } else {
        let type_a = schema_a.get_type().name()?.to_str()?.to_string();
        let type_b = schema_b.get_type().name()?.to_str()?.to_string();
        if type_a != type_b {
            return Ok((false, format!("Schema types differ: {type_a} vs {type_b}")));
        }
        return Ok((
            false,
            format!("Incompatible schema types: {type_a} vs {type_b}"),
        ));
    };
    Ok(result)
}

#[pyfunction]
pub(crate) fn specs_are_swappable(
    py: Python<'_>,
    spec_a: &Bound<'_, PyAny>,
    spec_b: &Bound<'_, PyAny>,
) -> PyResult<(bool, String)> {
    let name_a = get_spec_attr(spec_a, "name")?.extract::<String>()?;
    let name_b = get_spec_attr(spec_b, "name")?.extract::<String>()?;
    if name_a != name_b {
        return Ok((false, format!("Names differ: '{name_a}' != '{name_b}'")));
    }
    let data_schema_a = get_spec_attr(spec_a, "data_schema")?;
    let data_schema_b = get_spec_attr(spec_b, "data_schema")?;
    let (data_compat, data_diff) = schemas_compatible(py, &data_schema_a, &data_schema_b)?;
    if !data_compat {
        return Ok((false, format!("Data schemas incompatible: {data_diff}")));
    }
    let outputs_schema_a = get_spec_attr(spec_a, "outputs_schema")?;
    let outputs_schema_b = get_spec_attr(spec_b, "outputs_schema")?;
    let (outputs_compat, outputs_diff) =
        schemas_compatible(py, &outputs_schema_a, &outputs_schema_b)?;
    if !outputs_compat {
        return Ok((
            false,
            format!("Output schemas incompatible: {outputs_diff}"),
        ));
    }
    Ok((true, String::new()))
}

pub(crate) fn resolve_links(
    py: Python<'_>,
    block_name: &str,
    links: &[BlockLink],
    runs: &[Py<BlockRun>],
) -> PyResult<PyObject> {
    let resolved = PyDict::new(py);
    let source_index_cache: OnceMap<String, usize> = OnceMap::default();

    for link in links {
        if link.target.block_name != block_name {
            continue;
        }
        let source_name = link.source.block_name.as_str();
        let source_index = if let Some(index) = source_index_cache.get(source_name) {
            index
        } else {
            let index = runs
                .iter()
                .position(|run| run.borrow(py).name == source_name)
                .ok_or_else(|| {
                    PyRuntimeError::new_err(format!(
                        "ARCO_BLOCK_501: Block '{}' not found in run list",
                        source_name
                    ))
                })?;
            if source_index_cache.register(link.source.block_name.clone()) {
                let _ = source_index_cache.done(link.source.block_name.clone(), index);
            }
            index
        };
        let value = runs[source_index]
            .borrow(py)
            .outputs
            .bind(py)
            .get_item(&link.source.key)?
            .ok_or_else(|| {
                PyRuntimeError::new_err(format!(
                    "ARCO_BLOCK_502: Output '{}' not available from block '{}'",
                    link.source.key, link.source.block_name
                ))
            })?;
        let transformed = link.transform.apply_internal(py, value.unbind())?;
        resolved.set_item(&link.target.key, transformed)?;
    }
    Ok(resolved.into_any().unbind())
}

pub(crate) fn extract_outputs<'py>(
    py: Python<'py>,
    block: &Block,
    solution: PyObject,
    context: &Py<BlockContext>,
) -> PyResult<Bound<'py, PyDict>> {
    let outputs = if let Some(extract) = &block.extract {
        let ctx_obj = context.clone_ref(py).into_any();
        let result = extract.bind(py).call1((solution.clone_ref(py), ctx_obj))?;
        result
            .cast::<PyDict>()
            .map_err(|_| {
                PyRuntimeError::new_err(format!(
                    "ARCO_BLOCK_502: Block '{}' extract must return dict",
                    block.name
                ))
            })?
            .clone()
    } else {
        PyDict::new(py)
    };
    for key in outputs.keys() {
        if block.outputs.bind(py).get_item(&key)?.is_none() {
            let key_name = key.str()?;
            let key_str = key_name.to_str()?;
            let msg = format!(
                "ARCO_BLOCK_502: Output '{key_str}' not declared on block '{}'",
                block.name
            );
            tracing::error!(
                component = "block",
                operation = "extract",
                status = "error",
                "{msg}"
            );
            return Err(PyRuntimeError::new_err(msg));
        }
    }
    Ok(outputs)
}
