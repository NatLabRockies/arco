//! Block orchestration primitives for Arco (Python bindings).

mod dag;
mod decorator;
mod error;
mod once_map;
mod resolve;
mod schema;
mod spec;
mod transform;
mod util;

use crate::dag::BlockDag;
use crate::decorator::block;
use crate::resolve::{
    block_spec, build_model_from_spec, extract_outputs, inspect_model, resolve_links,
    schemas_compatible, specs_are_swappable,
};
use crate::schema::{coerce_inputs, coerce_outputs, outputs_schema_dict};
use crate::spec::{
    BlockSpec, get_spec_attr, make_spec_builder, make_spec_extractor, validate_spec,
};
use crate::transform::Transform;
use crate::util::{log_block_error, log_block_phase, model_type, rss_bytes};
pub use arco_ops as ops;
pub use arco_ops::highs::{Solver, highs_version};
pub use arco_ops::solver::{Solution, SolutionView, SolverConfig, SolverError, SolverStatus};
pub use arco_ops::{expr, highs, model, solver, targets};
use arco_tools::rss_delta;
use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Convenience type alias for a Python object reference.
pub type PyObject = Py<PyAny>;

fn block_runtime_error(operation: &'static str, msg: impl Into<String>) -> PyErr {
    let msg = msg.into();
    tracing::error!(component = "block", operation, status = "error", "{msg}");
    PyRuntimeError::new_err(msg)
}

fn build_solver_kwargs<'py>(
    py: Python<'py>,
    solver: Option<&PyObject>,
    log_to_console: Option<bool>,
    time_limit: Option<f64>,
    mip_gap: Option<f64>,
    verbosity: Option<u32>,
    primal_start: Option<&[(u32, f64)]>,
) -> PyResult<Option<Bound<'py, PyDict>>> {
    let kwargs = PyDict::new(py);

    if let Some(solver) = solver {
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
    if let Some(hints) = primal_start {
        kwargs.set_item("primal_start", hints)?;
    }

    if kwargs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(kwargs))
    }
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
    pub(crate) inputs: Py<PyDict>,
    pub(crate) attachments: Py<PyDict>,
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
    pub(crate) source: BlockPort,
    #[pyo3(get)]
    pub(crate) target: BlockPort,
    pub(crate) transform: Transform,
}

#[pymethods]
impl BlockLink {
    #[getter]
    fn transform(&self, py: Python<'_>) -> Transform {
        self.transform.clone_with_py_internal(py)
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
    pub(crate) name: String,
    model: Option<PyObject>,
    solution: Option<PyObject>,
    pub(crate) outputs: Py<PyDict>,
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
    pub(crate) model: PyObject,
    #[pyo3(get)]
    pub(crate) outputs: PyObject,
    #[pyo3(get)]
    pub(crate) spec_name: String,
    #[pyo3(get)]
    pub(crate) spec_version: String,
}

#[pyclass(name = "Block")]
pub struct Block {
    pub(crate) build: PyObject,
    pub(crate) name: String,
    pub(crate) inputs: Py<PyDict>,
    pub(crate) outputs: Py<PyDict>,
    pub(crate) extract: Option<PyObject>,
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
            return Err(block_runtime_error(
                "from_spec",
                "ARCO_BLOCK_502: allow_slacks is not yet implemented in Block.from_spec(). Inject slacks in your spec.build() method instead.",
            ));
        }
        let data_schema = get_spec_attr(spec, "data_schema")?;
        let outputs_schema = get_spec_attr(spec, "outputs_schema")?;
        let name = get_spec_attr(spec, "name")?
            .extract::<String>()
            .map_err(|_| PyRuntimeError::new_err("ARCO_BLOCK_501: spec.name must be str"))?;
        let outputs_dict = outputs_schema_dict(py, &outputs_schema)?;
        let outputs_dict_bound = outputs_dict.bind(py);
        let spec_obj = spec.clone().unbind();
        let build = make_spec_builder(py, spec_obj.clone_ref(py), slack_penalty)?;
        let extract = make_spec_extractor(py, spec_obj)?;
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
            return Err(block_runtime_error("add_block", msg));
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
            return Err(block_runtime_error(
                "link",
                "ARCO_BLOCK_502: Links must connect block outputs to inputs",
            ));
        }
        let transform = transform.map_or_else(Transform::identity_internal, |value| {
            value.clone_with_py_internal(py)
        });
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
                return Err(block_runtime_error("validate", msg));
            }
            if !block_names.contains(&link.target.block_name) {
                let msg = format!(
                    "ARCO_BLOCK_501: Block '{}' not found in BlockModel",
                    link.target.block_name
                );
                return Err(block_runtime_error("validate", msg));
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
                return Err(block_runtime_error("validate", msg));
            }
            if !target_inputs.bind(py).contains(&link.target.key)? {
                let msg = format!(
                    "ARCO_BLOCK_502: Input '{}' not defined on block '{}'",
                    link.target.key, link.target.block_name
                );
                return Err(block_runtime_error("validate", msg));
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
                    return Err(block_runtime_error("validate", msg));
                }
            }
        }

        for block in &self.blocks {
            let name = block.borrow(py).name.clone();
            let key_type_error = |key_kind: &str| {
                let msg = format!("ARCO_BLOCK_502: {key_kind} for block '{name}' must be str");
                tracing::error!(
                    component = "block",
                    operation = "validate",
                    status = "error",
                    "{msg}"
                );
                PyRuntimeError::new_err(msg)
            };
            let provided = if let Some(inputs) = self.inputs.get(&name) {
                let mut provided = HashSet::new();
                for key in inputs.bind(py).keys().iter() {
                    let key = key
                        .extract::<String>()
                        .map_err(|_| key_type_error("Provided input key"))?;
                    provided.insert(key);
                }
                provided
            } else {
                HashSet::new()
            };
            let linked: HashSet<String> = self
                .links
                .iter()
                .filter(|link| link.target.block_name == name)
                .map(|link| link.target.key.clone())
                .collect();
            for key in block.borrow(py).inputs.bind(py).keys().iter() {
                let key = key
                    .extract::<String>()
                    .map_err(|_| key_type_error("Input schema key"))?;
                if !provided.contains(&key) && !linked.contains(&key) {
                    let msg = format!(
                        "ARCO_BLOCK_502: Input '{}' not provided for block '{}'",
                        key, name
                    );
                    return Err(block_runtime_error("validate", msg));
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

        // Execute levels in topological order; blocks within a level run sequentially.
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
                let inputs = coerce_inputs(py, &block_ref.name, block_ref.inputs.bind(py), inputs)?;
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
                    return Err(block_runtime_error("build", msg));
                }

                let warm_start = block_ref.warm_start && !runs.is_empty();
                let solve_start = Instant::now();
                let warm_start_hints = if warm_start {
                    let previous = runs.last().and_then(|run| {
                        run.borrow(py)
                            .solution
                            .as_ref()
                            .map(|solution| solution.clone_ref(py))
                    });
                    previous
                        .and_then(|solution| solution.bind(py).getattr("primal_values").ok())
                        .and_then(|values| values.extract::<Vec<f64>>().ok())
                        .map(|values| {
                            values
                                .into_iter()
                                .enumerate()
                                .map(|(idx, val)| (idx as u32, val))
                                .collect::<Vec<_>>()
                        })
                } else {
                    None
                };

                let solve_kwargs = build_solver_kwargs(
                    py,
                    solver.as_ref(),
                    log_to_console,
                    time_limit,
                    mip_gap,
                    verbosity,
                    warm_start_hints.as_deref(),
                )?;
                let solution = if let Some(kwargs) = solve_kwargs {
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
                let outputs =
                    coerce_outputs(py, &block_ref.name, block_ref.outputs.bind(py), outputs)?;
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

    #[test]
    fn test_build_solver_kwargs_returns_none_when_empty() {
        Python::initialize();
        Python::attach(|py| {
            let kwargs = build_solver_kwargs(py, None, None, None, None, None, None)
                .expect("building kwargs should not fail");
            assert!(kwargs.is_none());
        });
    }

    #[test]
    fn test_build_solver_kwargs_includes_config_and_primal_start() {
        Python::initialize();
        Python::attach(|py| {
            let hints = vec![(0_u32, 1.25_f64), (3_u32, -2.0_f64)];
            let kwargs = build_solver_kwargs(
                py,
                None,
                Some(true),
                Some(15.0),
                Some(0.001),
                Some(2),
                Some(&hints),
            )
            .expect("building kwargs should not fail")
            .expect("kwargs should be present");

            let get = |key: &str| {
                kwargs
                    .get_item(key)
                    .expect("lookup should not fail")
                    .expect("key should be present")
            };

            assert!(get("log_to_console").extract::<bool>().unwrap());
            assert_eq!(get("time_limit").extract::<f64>().unwrap(), 15.0);
            assert_eq!(get("mip_gap").extract::<f64>().unwrap(), 0.001);
            assert_eq!(get("verbosity").extract::<u32>().unwrap(), 2);
            assert_eq!(
                get("primal_start").extract::<Vec<(u32, f64)>>().unwrap(),
                hints
            );
        });
    }

    #[test]
    fn test_block_runtime_error_preserves_message() {
        let err = block_runtime_error("test", "ARCO_BLOCK_599: test runtime error");

        Python::initialize();
        Python::attach(|py| {
            assert!(err.is_instance_of::<PyRuntimeError>(py));
            assert_eq!(
                err.to_string(),
                "RuntimeError: ARCO_BLOCK_599: test runtime error"
            );
        });
    }

    #[test]
    fn test_validate_errors_when_provided_input_key_is_not_string() {
        Python::initialize();
        Python::attach(|py| {
            let block_inputs = PyDict::new(py);
            block_inputs.set_item("required", py.None()).unwrap();
            let block = Block::new(
                py,
                py.None(),
                "blk".to_string(),
                Some(&block_inputs),
                None,
                None,
                false,
                false,
                DropPolicy::KeepSummary,
            );
            let block = Py::new(py, block).unwrap();

            let provided_inputs = PyDict::new(py);
            provided_inputs.set_item(1_i32, py.None()).unwrap();

            let mut model = BlockModel::new(Some("model".to_string()));
            model.blocks.push(block);
            model
                .inputs
                .insert("blk".to_string(), provided_inputs.unbind());

            let err = model.validate(py).unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("ARCO_BLOCK_502"));
            assert!(msg.contains("Provided input key for block 'blk' must be str"));
        });
    }

    #[test]
    fn test_validate_errors_when_block_input_schema_key_is_not_string() {
        Python::initialize();
        Python::attach(|py| {
            let block_inputs = PyDict::new(py);
            block_inputs.set_item(7_i32, py.None()).unwrap();
            let block = Block::new(
                py,
                py.None(),
                "blk".to_string(),
                Some(&block_inputs),
                None,
                None,
                false,
                false,
                DropPolicy::KeepSummary,
            );
            let block = Py::new(py, block).unwrap();

            let mut model = BlockModel::new(Some("model".to_string()));
            model.blocks.push(block);

            let err = model.validate(py).unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("ARCO_BLOCK_502"));
            assert!(msg.contains("Input schema key for block 'blk' must be str"));
        });
    }
}
