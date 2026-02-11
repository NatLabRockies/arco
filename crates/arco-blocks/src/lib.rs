//! Block orchestration primitives for Arco (Python bindings).

mod dag;
mod error;
mod once_map;

use crate::dag::BlockDag;
use crate::error::BlockError;
use crate::once_map::OnceMap;
use arco_tools::{capture_rss_bytes, rss_delta};
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{
    PyAny, PyBytes, PyDict, PyFloat, PyList, PySequence, PySequenceMethods, PyString, PyType,
};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

type PyObject = Py<PyAny>;

/// Log a [`BlockError`] and convert it to a Python exception.
fn log_block_error(err: BlockError) -> PyErr {
    tracing::error!(
        component = "block",
        operation = "solve",
        status = "error",
        "{err}"
    );
    PyRuntimeError::new_err(err.to_string())
}

#[pyclass(name = "DropPolicy", eq, eq_int, from_py_object)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPolicy {
    #[pyo3(name = "DROP_ALL")]
    DropAll,
    #[pyo3(name = "KEEP_SUMMARY")]
    KeepSummary,
    #[pyo3(name = "KEEP_MODEL")]
    KeepModel,
}

#[pyclass(name = "BlockContext")]
pub struct BlockContext {
    inputs: Py<PyDict>,
    attachments: Py<PyDict>,
}

#[pymethods]
impl BlockContext {
    #[new]
    #[pyo3(signature = (*, inputs))]
    fn new(py: Python<'_>, inputs: &Bound<'_, PyAny>) -> PyResult<Self> {
        let inputs = inputs
            .cast::<PyDict>()
            .map_err(|_| PyTypeError::new_err("ARCO_BLOCK_502: inputs must be a dict"))?;
        Ok(Self {
            inputs: inputs.clone().unbind(),
            attachments: PyDict::new(py).unbind(),
        })
    }

    #[getter]
    fn inputs(&self, py: Python<'_>) -> Py<PyDict> {
        self.inputs.clone_ref(py)
    }

    #[getter]
    fn attachments(&self, py: Python<'_>) -> Py<PyDict> {
        self.attachments.clone_ref(py)
    }

    fn attach(&self, py: Python<'_>, key: String, value: PyObject) -> PyResult<()> {
        self.attachments.bind(py).set_item(key, value)
    }
}

enum TransformStep {
    Custom(Py<PyAny>),
    Scale(Py<PyAny>),
    Offset(Py<PyAny>),
    Shift(i64),
    Clip { lower: f64, upper: f64 },
    Select(Vec<usize>),
}

#[pyclass(name = "Transform")]
pub struct Transform {
    steps: Vec<TransformStep>,
}

#[pymethods]
impl Transform {
    #[new]
    #[pyo3(signature = (steps=None))]
    fn new(steps: Option<Vec<PyObject>>) -> Self {
        let steps = steps
            .unwrap_or_default()
            .into_iter()
            .map(TransformStep::Custom)
            .collect();
        Self { steps }
    }

    fn __or__(&self, py: Python<'_>, other: &Transform) -> Transform {
        let mut steps = clone_steps(py, &self.steps);
        steps.extend(clone_steps(py, &other.steps));
        Transform { steps }
    }

    fn apply(&self, py: Python<'_>, values: PyObject) -> PyResult<PyObject> {
        let mut current = values;
        for step in &self.steps {
            current = apply_step(py, step, current)?;
        }
        Ok(current)
    }

    #[staticmethod]
    fn identity() -> Self {
        Self { steps: Vec::new() }
    }

    #[staticmethod]
    fn scale(factor: PyObject) -> Self {
        Self {
            steps: vec![TransformStep::Scale(factor)],
        }
    }

    #[staticmethod]
    fn offset(delta: PyObject) -> Self {
        Self {
            steps: vec![TransformStep::Offset(delta)],
        }
    }

    #[staticmethod]
    fn shift(periods: i64) -> Self {
        Self {
            steps: vec![TransformStep::Shift(periods)],
        }
    }

    #[staticmethod]
    fn clip(lower: f64, upper: f64) -> Self {
        Self {
            steps: vec![TransformStep::Clip { lower, upper }],
        }
    }

    #[staticmethod]
    fn select(indices: Vec<usize>) -> Self {
        Self {
            steps: vec![TransformStep::Select(indices)],
        }
    }
    fn clone_with_py(&self, py: Python<'_>) -> Transform {
        Transform {
            steps: clone_steps(py, &self.steps),
        }
    }
}

fn clone_steps(py: Python<'_>, steps: &[TransformStep]) -> Vec<TransformStep> {
    steps
        .iter()
        .map(|step| match step {
            TransformStep::Custom(func) => TransformStep::Custom(func.clone_ref(py)),
            TransformStep::Scale(factor) => TransformStep::Scale(factor.clone_ref(py)),
            TransformStep::Offset(delta) => TransformStep::Offset(delta.clone_ref(py)),
            TransformStep::Shift(periods) => TransformStep::Shift(*periods),
            TransformStep::Clip { lower, upper } => TransformStep::Clip {
                lower: *lower,
                upper: *upper,
            },
            TransformStep::Select(indices) => TransformStep::Select(indices.clone()),
        })
        .collect()
}

#[pyclass(subclass, dict, name = "BlockSpec")]
pub struct BlockSpec {
    build_fn: Option<Py<PyAny>>,
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

#[pyclass(name = "BlockPort", from_py_object)]
#[derive(Clone)]
pub struct BlockPort {
    #[pyo3(get)]
    pub block_name: String,
    #[pyo3(get)]
    pub key: String,
    #[pyo3(get)]
    pub kind: String,
}

impl BlockPort {
    pub fn new_input(block_name: String, key: String) -> Self {
        Self {
            block_name,
            key,
            kind: "input".to_string(),
        }
    }

    pub fn new_output(block_name: String, key: String) -> Self {
        Self {
            block_name,
            key,
            kind: "output".to_string(),
        }
    }
}

#[pyclass(name = "BlockLink")]
pub struct BlockLink {
    #[pyo3(get)]
    source: BlockPort,
    #[pyo3(get)]
    target: BlockPort,
    transform: Transform,
}

#[pymethods]
impl BlockLink {
    #[getter]
    fn transform(&self, py: Python<'_>) -> Transform {
        self.transform.clone_with_py(py)
    }
}

#[pyclass(name = "BlockDiagnostics", from_py_object)]
#[derive(Clone)]
pub struct BlockDiagnostics {
    #[pyo3(get)]
    build_ms: f64,
    #[pyo3(get)]
    solve_ms: f64,
    #[pyo3(get)]
    rss_bytes: Option<u64>,
    #[pyo3(get)]
    rss_delta_bytes: Option<i64>,
}

#[pyclass(name = "BlockRun")]
pub struct BlockRun {
    #[pyo3(get)]
    name: String,
    model: Option<PyObject>,
    solution: Option<PyObject>,
    outputs: Py<PyDict>,
    attachments: Py<PyDict>,
    #[pyo3(get)]
    diagnostics: BlockDiagnostics,
}

#[pymethods]
impl BlockRun {
    #[getter]
    fn model(&self, py: Python<'_>) -> Option<PyObject> {
        self.model.as_ref().map(|model| model.clone_ref(py))
    }

    #[getter]
    fn solution(&self, py: Python<'_>) -> Option<PyObject> {
        self.solution
            .as_ref()
            .map(|solution| solution.clone_ref(py))
    }

    #[getter]
    fn outputs(&self, py: Python<'_>) -> Py<PyDict> {
        self.outputs.clone_ref(py)
    }

    #[getter]
    fn attachments(&self, py: Python<'_>) -> Py<PyDict> {
        self.attachments.clone_ref(py)
    }

    #[pyo3(
        signature = (*, include_coeffs=false, include_slacks=true, variable_ids=None, constraint_ids=None)
    )]
    fn inspect(
        &self,
        py: Python<'_>,
        include_coeffs: bool,
        include_slacks: bool,
        variable_ids: Option<Vec<u32>>,
        constraint_ids: Option<Vec<u32>>,
    ) -> PyResult<Option<PyObject>> {
        let Some(model) = &self.model else {
            tracing::warn!(
                component = "block",
                operation = "inspect",
                status = "warning",
                block = %self.name,
                "Cannot inspect block, model was dropped"
            );
            return Ok(None);
        };
        let kwargs = PyDict::new(py);
        kwargs.set_item("include_coeffs", include_coeffs)?;
        kwargs.set_item("include_slacks", include_slacks)?;
        kwargs.set_item("variable_ids", variable_ids)?;
        kwargs.set_item("constraint_ids", constraint_ids)?;
        let snapshot = model
            .bind(py)
            .call_method("inspect", (), Some(&kwargs))?
            .unbind();
        Ok(Some(snapshot))
    }
}

#[pyclass(name = "BuildResult")]
pub struct BuildResult {
    #[pyo3(get)]
    model: PyObject,
    #[pyo3(get)]
    outputs: PyObject,
    #[pyo3(get)]
    spec_name: String,
    #[pyo3(get)]
    spec_version: String,
}

#[pyclass(name = "Block")]
pub struct Block {
    build: PyObject,
    name: String,
    inputs: Py<PyDict>,
    outputs: Py<PyDict>,
    extract: Option<PyObject>,
    cache_scaffolding: bool,
    warm_start: bool,
    drop_policy: DropPolicy,
}

#[pymethods]
impl Block {
    #[new]
    #[pyo3(
        signature = (build, *, name, inputs=None, outputs=None, extract=None, cache_scaffolding=false, warm_start=false, drop_policy=DropPolicy::KeepSummary)
    )]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        build: PyObject,
        name: String,
        inputs: Option<&Bound<'_, PyDict>>,
        outputs: Option<&Bound<'_, PyDict>>,
        extract: Option<PyObject>,
        cache_scaffolding: bool,
        warm_start: bool,
        drop_policy: DropPolicy,
    ) -> Self {
        Self {
            build,
            name,
            inputs: inputs.map_or_else(|| PyDict::new(py).unbind(), |dict| dict.clone().unbind()),
            outputs: outputs.map_or_else(|| PyDict::new(py).unbind(), |dict| dict.clone().unbind()),
            extract,
            cache_scaffolding,
            warm_start,
            drop_policy,
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[getter]
    fn inputs(&self, py: Python<'_>) -> Py<PyDict> {
        self.inputs.clone_ref(py)
    }

    #[getter]
    fn outputs(&self, py: Python<'_>) -> Py<PyDict> {
        self.outputs.clone_ref(py)
    }

    #[getter]
    fn cache_scaffolding(&self) -> bool {
        self.cache_scaffolding
    }

    #[getter]
    fn warm_start(&self) -> bool {
        self.warm_start
    }

    #[getter]
    fn drop_policy(&self) -> DropPolicy {
        self.drop_policy
    }

    fn input(&self, key: String) -> BlockPort {
        BlockPort {
            block_name: self.name.clone(),
            key,
            kind: "input".to_string(),
        }
    }

    fn output(&self, key: String) -> BlockPort {
        BlockPort {
            block_name: self.name.clone(),
            key,
            kind: "output".to_string(),
        }
    }

    #[staticmethod]
    #[pyo3(
        signature = (spec, *, drop_policy=DropPolicy::KeepSummary, warm_start=false, allow_slacks=false, slack_penalty=1e6)
    )]
    fn from_spec(
        py: Python<'_>,
        spec: &Bound<'_, PyAny>,
        drop_policy: DropPolicy,
        warm_start: bool,
        allow_slacks: bool,
        slack_penalty: f64,
    ) -> PyResult<Block> {
        validate_spec(spec)?;
        if allow_slacks {
            let msg = "ARCO_BLOCK_502: allow_slacks is not yet implemented in Block.from_spec(). Inject slacks in your spec.build() method instead.";
            tracing::error!(
                component = "block",
                operation = "from_spec",
                status = "error",
                "{msg}"
            );
            return Err(PyRuntimeError::new_err(msg));
        }
        let data_schema = get_spec_attr(spec, "data_schema")?;
        let outputs_schema = get_spec_attr(spec, "outputs_schema")?;
        let name = get_spec_attr(spec, "name")?
            .extract::<String>()
            .map_err(|_| PyRuntimeError::new_err("ARCO_BLOCK_501: spec.name must be str"))?;
        let outputs_dict = outputs_schema_dict(py, &outputs_schema)?;
        let outputs_dict_bound = outputs_dict.bind(py);
        let spec_obj = spec.clone().unbind();
        let build = Py::new(
            py,
            SpecBuilder {
                spec: spec_obj.clone_ref(py),
                _slack_penalty: slack_penalty,
            },
        )?
        .into_any();
        let extract = Py::new(py, SpecExtractor { spec: spec_obj })?.into_any();
        let inputs_dict = PyDict::new(py);
        inputs_dict.set_item("data", data_schema.clone())?;
        let block = Block::new(
            py,
            build,
            name,
            Some(&inputs_dict),
            Some(outputs_dict_bound),
            Some(extract),
            false,
            warm_start,
            drop_policy,
        );
        Ok(block)
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
        ctx_ref.attach(
            py,
            "_spec_name".to_string(),
            get_spec_attr(spec, "name")?.unbind(),
        )?;
        let spec_version = spec
            .getattr("version")
            .ok()
            .and_then(|value| value.extract::<String>().ok())
            .unwrap_or_else(|| "0.0.0".to_string());
        ctx_ref.attach(
            py,
            "_spec_version".to_string(),
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
        ctx_ref.attach(py, "_outputs".to_string(), outputs_validated)?;
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

#[pyclass(name = "BlockModel")]
pub struct BlockModel {
    name: String,
    blocks: Vec<Py<Block>>,
    inputs: HashMap<String, Py<PyDict>>,
    links: Vec<BlockLink>,
}

#[pymethods]
impl BlockModel {
    #[new]
    #[pyo3(signature = (*, name=None))]
    fn new(name: Option<String>) -> Self {
        Self {
            name: name.unwrap_or_else(|| "BlockModel".to_string()),
            blocks: Vec::new(),
            inputs: HashMap::new(),
            links: Vec::new(),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[pyo3(
        signature = (block_or_build, *, name=None, inputs=None, inputs_schema=None, outputs=None, extract=None, cache_scaffolding=false, warm_start=false, drop_policy=DropPolicy::KeepSummary)
    )]
    #[allow(clippy::too_many_arguments)]
    fn add_block(
        &mut self,
        py: Python<'_>,
        block_or_build: PyObject,
        name: Option<String>,
        inputs: Option<&Bound<'_, PyAny>>,
        inputs_schema: Option<&Bound<'_, PyAny>>,
        outputs: Option<&Bound<'_, PyAny>>,
        extract: Option<PyObject>,
        cache_scaffolding: bool,
        warm_start: bool,
        drop_policy: DropPolicy,
    ) -> PyResult<Py<Block>> {
        let block = if block_or_build.bind(py).is_instance_of::<Block>() {
            block_or_build.extract::<Py<Block>>(py)?
        } else {
            let name = name
                .ok_or_else(|| PyRuntimeError::new_err("ARCO_BLOCK_501: Block name is required"))?;
            let inputs_schema = inputs_schema
                .map(|value| value.cast::<PyDict>())
                .transpose()?;
            let outputs_schema = outputs.map(|value| value.cast::<PyDict>()).transpose()?;
            let block = Block::new(
                py,
                block_or_build,
                name,
                inputs_schema,
                outputs_schema,
                extract,
                cache_scaffolding,
                warm_start,
                drop_policy,
            );
            Py::new(py, block)?
        };

        let block_name = block.borrow(py).name.clone();
        if self
            .blocks
            .iter()
            .any(|existing| existing.borrow(py).name == block_name)
        {
            let msg = format!("ARCO_BLOCK_501: Block '{block_name}' already exists in BlockModel");
            tracing::error!(
                component = "block",
                operation = "add_block",
                status = "error",
                "{msg}"
            );
            return Err(PyRuntimeError::new_err(msg));
        }

        if let Some(inputs) = inputs {
            let inputs = inputs.cast::<PyDict>().map_err(|_| {
                PyTypeError::new_err(format!(
                    "ARCO_BLOCK_502: Inputs for block '{block_name}' must be a dict"
                ))
            })?;
            self.inputs
                .insert(block_name.clone(), inputs.clone().unbind());
        }

        self.blocks.push(block.clone_ref(py));
        Ok(block)
    }

    #[pyo3(signature = (source, target, transform=None))]
    fn link(
        &mut self,
        py: Python<'_>,
        source: BlockPort,
        target: BlockPort,
        transform: Option<&Transform>,
    ) -> PyResult<()> {
        if source.kind != "output" || target.kind != "input" {
            let msg = "ARCO_BLOCK_502: Links must connect block outputs to inputs";
            tracing::error!(
                component = "block",
                operation = "link",
                status = "error",
                "{msg}"
            );
            return Err(PyRuntimeError::new_err(msg));
        }
        let transform = transform.map_or_else(Transform::identity, |value| value.clone_with_py(py));
        self.links.push(BlockLink {
            source,
            target,
            transform,
        });
        Ok(())
    }

    fn validate(&self, py: Python<'_>) -> PyResult<()> {
        let mut name_to_index = HashMap::new();
        for (idx, block) in self.blocks.iter().enumerate() {
            name_to_index.insert(block.borrow(py).name.clone(), idx);
        }
        let block_names: HashSet<String> = name_to_index.keys().cloned().collect();

        for link in &self.links {
            if !block_names.contains(&link.source.block_name) {
                let msg = format!(
                    "ARCO_BLOCK_501: Block '{}' not found in BlockModel",
                    link.source.block_name
                );
                tracing::error!(
                    component = "block",
                    operation = "validate",
                    status = "error",
                    "{msg}"
                );
                return Err(PyRuntimeError::new_err(msg));
            }
            if !block_names.contains(&link.target.block_name) {
                let msg = format!(
                    "ARCO_BLOCK_501: Block '{}' not found in BlockModel",
                    link.target.block_name
                );
                tracing::error!(
                    component = "block",
                    operation = "validate",
                    status = "error",
                    "{msg}"
                );
                return Err(PyRuntimeError::new_err(msg));
            }
            let source_block = &self.blocks[name_to_index[&link.source.block_name]];
            let target_block = &self.blocks[name_to_index[&link.target.block_name]];
            let source_outputs = source_block.borrow(py).outputs.clone_ref(py);
            let target_inputs = target_block.borrow(py).inputs.clone_ref(py);
            if !source_outputs.bind(py).contains(&link.source.key)? {
                let msg = format!(
                    "ARCO_BLOCK_502: Output '{}' not defined on block '{}'",
                    link.source.key, link.source.block_name
                );
                tracing::error!(
                    component = "block",
                    operation = "validate",
                    status = "error",
                    "{msg}"
                );
                return Err(PyRuntimeError::new_err(msg));
            }
            if !target_inputs.bind(py).contains(&link.target.key)? {
                let msg = format!(
                    "ARCO_BLOCK_502: Input '{}' not defined on block '{}'",
                    link.target.key, link.target.block_name
                );
                tracing::error!(
                    component = "block",
                    operation = "validate",
                    status = "error",
                    "{msg}"
                );
                return Err(PyRuntimeError::new_err(msg));
            }
            let source_schema = source_outputs.bind(py).get_item(&link.source.key)?;
            let target_schema = target_inputs.bind(py).get_item(&link.target.key)?;
            if let (Some(source_schema), Some(target_schema)) = (source_schema, target_schema) {
                if !source_schema.is_none()
                    && !target_schema.is_none()
                    && !source_schema.eq(target_schema)?
                {
                    let msg = format!(
                        "ARCO_BLOCK_502: Output schema of '{}.{}' incompatible with '{}.{}'",
                        link.source.block_name,
                        link.source.key,
                        link.target.block_name,
                        link.target.key
                    );
                    tracing::error!(
                        component = "block",
                        operation = "validate",
                        status = "error",
                        "{msg}"
                    );
                    return Err(PyRuntimeError::new_err(msg));
                }
            }
        }

        for block in &self.blocks {
            let name = block.borrow(py).name.clone();
            let provided = self
                .inputs
                .get(&name)
                .map(|inputs| {
                    inputs
                        .bind(py)
                        .keys()
                        .iter()
                        .filter_map(|key| key.extract::<String>().ok())
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();
            let linked: HashSet<String> = self
                .links
                .iter()
                .filter(|link| link.target.block_name == name)
                .map(|link| link.target.key.clone())
                .collect();
            for key in block.borrow(py).inputs.bind(py).keys().iter() {
                let key = key.extract::<String>().unwrap_or_default();
                if !provided.contains(&key) && !linked.contains(&key) {
                    let msg = format!(
                        "ARCO_BLOCK_502: Input '{}' not provided for block '{}'",
                        key, name
                    );
                    tracing::error!(
                        component = "block",
                        operation = "validate",
                        status = "error",
                        "{msg}"
                    );
                    return Err(PyRuntimeError::new_err(msg));
                }
            }
        }

        Ok(())
    }

    #[pyo3(signature = (*, solver=None, log_to_console=None, time_limit=None, mip_gap=None, verbosity=None))]
    fn solve(
        &self,
        py: Python<'_>,
        solver: Option<PyObject>,
        log_to_console: Option<bool>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
    ) -> PyResult<Vec<Py<BlockRun>>> {
        self.validate(py)?;

        // Build the block DAG for parallel execution
        let block_names: Vec<String> = self
            .blocks
            .iter()
            .map(|b| b.borrow(py).name.clone())
            .collect();

        let links: Vec<(String, String)> = self
            .links
            .iter()
            .map(|link| {
                (
                    link.source.block_name.clone(),
                    link.target.block_name.clone(),
                )
            })
            .collect();

        let dag = BlockDag::from_links(&block_names, &links).map_err(log_block_error)?;

        // Compute execution levels (validates acyclicity internally)
        let execution_levels = dag.execution_levels().map_err(log_block_error)?;

        tracing::info!(
            component = "block",
            operation = "solve",
            status = "success",
            num_levels = execution_levels.len(),
            num_blocks = block_names.len(),
            "Block DAG analysis: {} execution levels found",
            execution_levels.len()
        );

        let mut runs: Vec<Py<BlockRun>> = Vec::new();

        // Execute each level sequentially (blocks within a level can run in parallel)
        for (level_idx, level_blocks) in execution_levels.iter().enumerate() {
            tracing::debug!(
                component = "block",
                operation = "solve",
                status = "success",
                level = level_idx,
                num_blocks_in_level = level_blocks.len(),
                "Executing level {} with {} blocks",
                level_idx,
                level_blocks.len()
            );

            // For now, execute sequentially within each level (parallel support can be added)
            for &block_idx in level_blocks {
                let block = &self.blocks[block_idx];
                let block_ref = block.borrow(py);
                let rss_before = rss_bytes();

                let inputs = self
                    .inputs
                    .get(&block_ref.name)
                    .map(|dict| dict.bind(py).copy())
                    .transpose()?
                    .unwrap_or_else(|| PyDict::new(py));

                let resolved = resolve_links(py, &block_ref.name, &self.links, &runs)?;
                let resolved_dict = resolved.bind(py).cast::<PyDict>()?;
                for (key, value) in resolved_dict.iter() {
                    inputs.set_item(key, value)?;
                }
                let inputs = coerce_inputs(py, &block_ref, inputs)?;
                let context = Py::new(
                    py,
                    BlockContext {
                        inputs: inputs.unbind(),
                        attachments: PyDict::new(py).unbind(),
                    },
                )?;

                let build_start = Instant::now();
                let model = block_ref.build.bind(py).call1((context.clone_ref(py),))?;
                let build_ms = build_start.elapsed().as_secs_f64() * 1000.0;
                let rss_after_build = rss_bytes();
                let model_class = model_type(py)?;
                if !model.is_instance(model_class.as_any())? {
                    let msg = format!(
                        "ARCO_BLOCK_502: Block '{}' build must return arco.Model",
                        block_ref.name
                    );
                    tracing::error!(
                        component = "block",
                        operation = "build",
                        status = "error",
                        "{msg}"
                    );
                    return Err(PyRuntimeError::new_err(msg));
                }

                let warm_start = block_ref.warm_start && !runs.is_empty();
                let solve_start = Instant::now();
                let solution = if warm_start {
                    let previous = runs.last().and_then(|run| {
                        run.borrow(py)
                            .solution
                            .as_ref()
                            .map(|solution| solution.clone_ref(py))
                    });
                    let hints = previous
                        .and_then(|solution| solution.bind(py).getattr("primal_values").ok())
                        .and_then(|values| values.extract::<Vec<f64>>().ok())
                        .map(|values| {
                            values
                                .into_iter()
                                .enumerate()
                                .map(|(idx, val)| (idx as u32, val))
                                .collect::<Vec<_>>()
                        });
                    if let Some(hints) = hints {
                        let kwargs = PyDict::new(py);
                        if let Some(ref solver) = solver {
                            kwargs.set_item("solver", solver.clone_ref(py))?;
                        }
                        if let Some(enabled) = log_to_console {
                            kwargs.set_item("log_to_console", enabled)?;
                        }
                        if let Some(limit) = time_limit {
                            kwargs.set_item("time_limit", limit)?;
                        }
                        if let Some(gap) = mip_gap {
                            kwargs.set_item("mip_gap", gap)?;
                        }
                        if let Some(level) = verbosity {
                            kwargs.set_item("verbosity", level)?;
                        }
                        kwargs.set_item("primal_start", hints)?;
                        model.call_method("solve", (), Some(&kwargs))?
                    } else if solver.is_some()
                        || log_to_console.is_some()
                        || time_limit.is_some()
                        || mip_gap.is_some()
                        || verbosity.is_some()
                    {
                        let kwargs = PyDict::new(py);
                        if let Some(ref solver) = solver {
                            kwargs.set_item("solver", solver.clone_ref(py))?;
                        }
                        if let Some(enabled) = log_to_console {
                            kwargs.set_item("log_to_console", enabled)?;
                        }
                        if let Some(limit) = time_limit {
                            kwargs.set_item("time_limit", limit)?;
                        }
                        if let Some(gap) = mip_gap {
                            kwargs.set_item("mip_gap", gap)?;
                        }
                        if let Some(level) = verbosity {
                            kwargs.set_item("verbosity", level)?;
                        }
                        model.call_method("solve", (), Some(&kwargs))?
                    } else {
                        model.call_method0("solve")?
                    }
                } else if solver.is_some()
                    || log_to_console.is_some()
                    || time_limit.is_some()
                    || mip_gap.is_some()
                    || verbosity.is_some()
                {
                    let kwargs = PyDict::new(py);
                    if let Some(ref solver) = solver {
                        kwargs.set_item("solver", solver.clone_ref(py))?;
                    }
                    if let Some(enabled) = log_to_console {
                        kwargs.set_item("log_to_console", enabled)?;
                    }
                    if let Some(limit) = time_limit {
                        kwargs.set_item("time_limit", limit)?;
                    }
                    if let Some(gap) = mip_gap {
                        kwargs.set_item("mip_gap", gap)?;
                    }
                    if let Some(level) = verbosity {
                        kwargs.set_item("verbosity", level)?;
                    }
                    model.call_method("solve", (), Some(&kwargs))?
                } else {
                    model.call_method0("solve")?
                };
                let solve_ms = solve_start.elapsed().as_secs_f64() * 1000.0;
                let rss_after_solve = rss_bytes();

                let solution_obj = solution.unbind();
                let context_ref = context.borrow(py);
                let outputs =
                    extract_outputs(py, &block_ref, solution_obj.clone_ref(py), &context)?;
                let outputs = coerce_outputs(py, &block_ref, outputs)?;
                let attachments = context_ref.attachments.clone_ref(py);

                let rss_delta_total = match (rss_before, rss_after_solve) {
                    (Some(before), Some(after)) => Some(after as i64 - before as i64),
                    _ => None,
                };

                let diagnostics = BlockDiagnostics {
                    build_ms,
                    solve_ms,
                    rss_bytes: rss_after_solve,
                    rss_delta_bytes: rss_delta_total,
                };

                log_block_phase(
                    &block_ref.name,
                    "build",
                    build_ms,
                    rss_after_build,
                    rss_delta(rss_before, rss_after_build),
                    warm_start,
                );
                log_block_phase(
                    &block_ref.name,
                    "solve",
                    solve_ms,
                    rss_after_solve,
                    rss_delta(rss_after_build, rss_after_solve),
                    warm_start,
                );

                tracing::info!(
                    component = "block",
                    operation = "solve",
                    status = "success",
                    block = %block_ref.name,
                    phase = "solve",
                    cache_hit = false,
                    warm_start,
                    level = level_idx,
                    "Block solved"
                );

                let (model_to_keep, solution_to_keep) = match block_ref.drop_policy {
                    DropPolicy::KeepModel => {
                        (Some(model.unbind()), Some(solution_obj.clone_ref(py)))
                    }
                    DropPolicy::KeepSummary => (None, Some(solution_obj.clone_ref(py))),
                    DropPolicy::DropAll => (None, None),
                };

                let run = BlockRun {
                    name: block_ref.name.clone(),
                    model: model_to_keep,
                    solution: solution_to_keep,
                    outputs: outputs.unbind(),
                    attachments,
                    diagnostics,
                };
                let run_py = Py::new(py, run)?;

                runs.push(run_py);
            }
        }

        Ok(runs)
    }
}

const ARCO_BLOCK_MARKER_ATTR: &str = "__arco_block_marker__";
const ARCO_BLOCK_NAME_ATTR: &str = "__arco_block_name__";
const ARCO_BLOCK_INPUT_SCHEMA_ATTR: &str = "__arco_block_input_schema__";
const ARCO_BLOCK_INPUT_FIELDS_ATTR: &str = "__arco_block_input_fields__";
const ARCO_BLOCK_EXPECTS_CTX_ATTR: &str = "__arco_block_expects_ctx__";

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
fn block(py: Python<'_>, func: Option<PyObject>, name: Option<String>) -> PyResult<PyObject> {
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

#[pyfunction]
#[pyo3(signature = (*, name, data_schema, outputs_schema, build, version="0.0.0"))]
fn block_spec(
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
fn build_model_from_spec(
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
fn inspect_model(
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
fn schemas_compatible(
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
fn specs_are_swappable(
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

pub fn add_blocks_submodule(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let blocks = PyModule::new(py, "blocks")?;
    blocks.add_class::<Block>()?;
    blocks.add_class::<BlockContext>()?;
    blocks.add_class::<BlockDiagnostics>()?;
    blocks.add_class::<BlockLink>()?;
    blocks.add_class::<BlockModel>()?;
    blocks.add_class::<BlockPort>()?;
    blocks.add_class::<BlockRun>()?;
    blocks.add_class::<BlockSpec>()?;
    blocks.add_class::<BuildResult>()?;
    blocks.add_class::<DropPolicy>()?;
    blocks.add_class::<Transform>()?;
    blocks.add_function(wrap_pyfunction!(block, &blocks)?)?;
    blocks.add_function(wrap_pyfunction!(block_spec, &blocks)?)?;
    blocks.add_function(wrap_pyfunction!(build_model_from_spec, &blocks)?)?;
    blocks.add_function(wrap_pyfunction!(inspect_model, &blocks)?)?;
    blocks.add_function(wrap_pyfunction!(schemas_compatible, &blocks)?)?;
    blocks.add_function(wrap_pyfunction!(specs_are_swappable, &blocks)?)?;
    parent.add_submodule(&blocks)?;
    let sys = PyModule::import(py, "sys")?;
    let modules_any = sys.getattr("modules")?;
    let modules = modules_any.cast::<PyDict>()?;
    modules.set_item("arco.blocks", &blocks)?;
    parent.setattr("blocks", &blocks)?;
    Ok(())
}

fn apply_step(py: Python<'_>, step: &TransformStep, values: PyObject) -> PyResult<PyObject> {
    let value_any = values.bind(py);
    match step {
        TransformStep::Custom(func) => Ok(func.bind(py).call1((values,))?.unbind()),
        TransformStep::Scale(factor) => apply_binary(py, value_any, factor.bind(py), "__mul__"),
        TransformStep::Offset(delta) => apply_binary(py, value_any, delta.bind(py), "__add__"),
        TransformStep::Shift(periods) => apply_shift(py, value_any, *periods),
        TransformStep::Clip { lower, upper } => apply_clip(py, value_any, *lower, *upper),
        TransformStep::Select(indices) => apply_select(py, value_any, indices),
    }
}

fn apply_binary(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
    rhs: &Bound<'_, PyAny>,
    op: &str,
) -> PyResult<PyObject> {
    if is_sequence(values) {
        let seq = values.cast::<PySequence>()?;
        let rhs_seq = if is_sequence(rhs) {
            Some(rhs.cast::<PySequence>()?)
        } else {
            None
        };
        let len = seq.len()?;
        let mut results = Vec::new();
        if let Some(rhs_seq) = rhs_seq {
            let rhs_len = rhs_seq.len()?;
            let count = len.min(rhs_len);
            for idx in 0..count {
                let left = seq.get_item(idx)?;
                let right = rhs_seq.get_item(idx)?;
                let value = left.call_method1(op, (right,))?;
                results.push(value.unbind());
            }
        } else {
            for idx in 0..len {
                let left = seq.get_item(idx)?;
                let value = left.call_method1(op, (rhs,))?;
                results.push(value.unbind());
            }
        }
        return Ok(PyList::new(py, results)?.into_any().unbind());
    }
    Ok(values.call_method1(op, (rhs,))?.unbind())
}

fn apply_shift(py: Python<'_>, values: &Bound<'_, PyAny>, periods: i64) -> PyResult<PyObject> {
    if !is_sequence(values) {
        return Ok(values.clone().unbind());
    }
    let seq = values.cast::<PySequence>()?;
    let len = seq.len()?;
    let mut items = Vec::with_capacity(len);
    for idx in 0..len {
        items.push(seq.get_item(idx)?.unbind());
    }
    if periods == 0 {
        return Ok(PyList::new(py, items)?.into_any().unbind());
    }
    let fill = PyFloat::new(py, 0.0).into_any().unbind();
    if periods > 0 {
        let shift = periods as usize;
        let mut out = Vec::with_capacity(len + shift);
        for _ in 0..shift {
            out.push(fill.clone_ref(py));
        }
        let keep = len.saturating_sub(shift);
        out.extend(items.into_iter().take(keep));
        return Ok(PyList::new(py, out)?.into_any().unbind());
    }
    let shift = (-periods) as usize;
    let mut out: Vec<PyObject> = items.into_iter().skip(shift).collect();
    for _ in 0..shift {
        out.push(fill.clone_ref(py));
    }
    Ok(PyList::new(py, out)?.into_any().unbind())
}

fn apply_clip(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
    lower: f64,
    upper: f64,
) -> PyResult<PyObject> {
    if is_sequence(values) {
        let seq = values.cast::<PySequence>()?;
        let len = seq.len()?;
        let mut out = Vec::with_capacity(len);
        for idx in 0..len {
            let value = seq.get_item(idx)?;
            let number = value.extract::<f64>()?;
            let clipped = number.max(lower).min(upper);
            out.push(PyFloat::new(py, clipped).into_any().unbind());
        }
        return Ok(PyList::new(py, out)?.into_any().unbind());
    }
    let number = values.extract::<f64>()?;
    Ok(PyFloat::new(py, number.max(lower).min(upper))
        .into_any()
        .unbind())
}

fn apply_select(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
    indices: &[usize],
) -> PyResult<PyObject> {
    if !is_sequence(values) {
        return Ok(values.clone().unbind());
    }
    let seq = values.cast::<PySequence>()?;
    let mut out = Vec::with_capacity(indices.len());
    for idx in indices {
        out.push(seq.get_item(*idx)?.unbind());
    }
    Ok(PyList::new(py, out)?.into_any().unbind())
}

fn is_sequence(value: &Bound<'_, PyAny>) -> bool {
    if value.is_instance_of::<PyString>() {
        return false;
    }
    if value.is_instance_of::<PyBytes>() {
        return false;
    }
    value.cast::<PySequence>().is_ok()
}

fn rss_bytes() -> Option<u64> {
    capture_rss_bytes("block")
}

fn log_block_phase(
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

fn model_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
    let module = PyModule::import(py, "arco.arco")?;
    let model_any = module.getattr("Model")?;
    Ok(model_any.cast::<PyType>()?.clone())
}

fn create_model(py: Python<'_>) -> PyResult<PyObject> {
    let model_type = model_type(py)?;
    Ok(model_type.call0()?.unbind())
}

fn validate_spec(spec: &Bound<'_, PyAny>) -> PyResult<()> {
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

fn get_spec_attr<'py>(spec: &Bound<'py, PyAny>, name: &str) -> PyResult<Bound<'py, PyAny>> {
    spec.getattr(name)
}

fn is_pydantic_schema(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<bool> {
    let Ok(schema_type) = schema.cast::<PyType>() else {
        return Ok(false);
    };
    let pydantic = PyModule::import(py, "pydantic")?;
    let base_model_any = pydantic.getattr("BaseModel")?;
    let base_model = base_model_any.cast::<PyType>()?;
    schema_type.is_subclass(base_model)
}

fn is_dataclass_schema(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<bool> {
    let dataclasses = PyModule::import(py, "dataclasses")?;
    let is_dataclass = dataclasses.getattr("is_dataclass")?;
    is_dataclass.call1((schema,))?.extract::<bool>()
}

fn dataclass_fields(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<Py<PyDict>> {
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

fn compare_fields(
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

fn validate_data(
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

fn outputs_schema_dict(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<Py<PyDict>> {
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

fn resolve_links(
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
        let transformed = link.transform.apply(py, value.unbind())?;
        resolved.set_item(&link.target.key, transformed)?;
    }
    Ok(resolved.into_any().unbind())
}

fn extract_outputs<'py>(
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

fn coerce_inputs<'py>(
    py: Python<'py>,
    block: &Block,
    inputs: Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyDict>> {
    let coerced = PyDict::new(py);
    for (key, value) in inputs.iter() {
        let schema = block.inputs.bind(py).get_item(&key)?;
        let coerced_value = coerce_schema(py, value.unbind(), schema, &block.name, &key, "input")?;
        coerced.set_item(key, coerced_value)?;
    }
    Ok(coerced)
}

fn coerce_outputs<'py>(
    py: Python<'py>,
    block: &Block,
    outputs: Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyDict>> {
    let coerced = PyDict::new(py);
    for (key, value) in outputs.iter() {
        let schema = block.outputs.bind(py).get_item(&key)?;
        let coerced_value = coerce_schema(py, value.unbind(), schema, &block.name, &key, "output")?;
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

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_policy_enum_values() {
        // Verify the enum variants exist and can be compared
        assert_eq!(DropPolicy::DropAll, DropPolicy::DropAll);
        assert_eq!(DropPolicy::KeepSummary, DropPolicy::KeepSummary);
        assert_eq!(DropPolicy::KeepModel, DropPolicy::KeepModel);

        // Verify they are different
        assert_ne!(DropPolicy::DropAll, DropPolicy::KeepSummary);
        assert_ne!(DropPolicy::KeepSummary, DropPolicy::KeepModel);
        assert_ne!(DropPolicy::DropAll, DropPolicy::KeepModel);
    }

    #[test]
    fn test_drop_policy_debug() {
        // Verify Debug is implemented
        let policy = DropPolicy::DropAll;
        let debug_str = format!("{:?}", policy);
        assert!(debug_str.contains("DropAll"));
    }

    #[test]
    fn test_drop_policy_clone() {
        let policy = DropPolicy::KeepModel;
        let cloned = policy;
        assert_eq!(policy, cloned);
    }

    #[test]
    fn test_drop_policy_copy() {
        let policy = DropPolicy::KeepSummary;
        let copied: DropPolicy = policy; // Copy trait
        assert_eq!(policy, copied);
    }

    #[test]
    fn test_block_diagnostics_creation() {
        let diag = BlockDiagnostics {
            build_ms: 10.5,
            solve_ms: 100.0,
            rss_bytes: Some(1024 * 1024),
            rss_delta_bytes: Some(512 * 1024),
        };

        assert_eq!(diag.build_ms, 10.5);
        assert_eq!(diag.solve_ms, 100.0);
        assert_eq!(diag.rss_bytes, Some(1024 * 1024));
        assert_eq!(diag.rss_delta_bytes, Some(512 * 1024));
    }

    #[test]
    fn test_block_diagnostics_clone() {
        let diag = BlockDiagnostics {
            build_ms: 1.0,
            solve_ms: 3.0,
            rss_bytes: None,
            rss_delta_bytes: None,
        };

        let cloned = diag.clone();
        assert_eq!(diag.build_ms, cloned.build_ms);
        assert_eq!(diag.solve_ms, cloned.solve_ms);
    }

    #[test]
    fn test_block_port_clone() {
        let port = BlockPort {
            block_name: "test_block".to_string(),
            key: "output_key".to_string(),
            kind: "output".to_string(),
        };

        let cloned = port.clone();
        assert_eq!(port.block_name, cloned.block_name);
        assert_eq!(port.key, cloned.key);
        assert_eq!(port.kind, cloned.kind);
    }
}
