use crate::{PyModel, PyObject, PySolveResult};
use arco_blocks::BlockPort;
use arco_core::solver::SolverStatus;
use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};
use std::collections::HashSet;

const ARCO_BLOCK_MARKER_ATTR: &str = "__arco_block_marker__";
const ARCO_BLOCK_NAME_ATTR: &str = "__arco_block_name__";
const ARCO_BLOCK_INPUT_SCHEMA_ATTR: &str = "__arco_block_input_schema__";
const ARCO_BLOCK_INPUT_FIELDS_ATTR: &str = "__arco_block_input_fields__";
const ARCO_BLOCK_EXPECTS_CTX_ATTR: &str = "__arco_block_expects_ctx__";

#[pyclass(name = "BlockPorts")]
pub struct PyBlockPorts {
    block_name: String,
    kind: String,
    keys: HashSet<String>,
}

#[pymethods]
impl PyBlockPorts {
    fn __getattr__(&self, key: &str) -> PyResult<BlockPort> {
        if !self.keys.contains(key) {
            return Err(PyKeyError::new_err(format!(
                "Unknown {} port '{}.{}'",
                self.kind, self.block_name, key
            )));
        }
        match self.kind.as_str() {
            "input" => Ok(BlockPort::new_input(
                self.block_name.clone(),
                key.to_string(),
            )),
            _ => Ok(BlockPort::new_output(
                self.block_name.clone(),
                key.to_string(),
            )),
        }
    }

    fn __dir__(&self) -> Vec<String> {
        let mut keys = self.keys.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    fn keys(&self) -> Vec<String> {
        let mut keys = self.keys.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }
}

/// A handle returned by model.add_block() with typed `.in_` and `.out` accessors.
#[pyclass(name = "BlockHandle")]
pub struct PyBlockHandle {
    name: String,
    input_keys: HashSet<String>,
    output_keys: HashSet<String>,
}

#[pymethods]
impl PyBlockHandle {
    /// Get an input port reference for linking.
    fn input(&self, key: String) -> PyResult<BlockPort> {
        if !self.input_keys.contains(&key) {
            return Err(PyKeyError::new_err(format!(
                "Unknown input port '{}.{}'",
                self.name, key
            )));
        }
        Ok(BlockPort::new_input(self.name.clone(), key))
    }

    /// Get an output port reference for linking.
    fn output(&self, key: String) -> PyResult<BlockPort> {
        if !self.output_keys.contains(&key) {
            return Err(PyKeyError::new_err(format!(
                "Unknown output port '{}.{}'",
                self.name, key
            )));
        }
        Ok(BlockPort::new_output(self.name.clone(), key))
    }

    #[getter]
    fn name(&self) -> &str {
        &self.name
    }

    #[getter]
    fn in_(&self, py: Python<'_>) -> PyResult<Py<PyBlockPorts>> {
        Py::new(
            py,
            PyBlockPorts {
                block_name: self.name.clone(),
                kind: "input".to_string(),
                keys: self.input_keys.clone(),
            },
        )
    }

    #[getter]
    fn out(&self, py: Python<'_>) -> PyResult<Py<PyBlockPorts>> {
        Py::new(
            py,
            PyBlockPorts {
                block_name: self.name.clone(),
                kind: "output".to_string(),
                keys: self.output_keys.clone(),
            },
        )
    }

    fn __repr__(&self) -> String {
        format!("BlockHandle(name='{}')", self.name)
    }
}

/// Dict-like accessor for per-block results: `result.blocks["name"]`
#[pyclass(name = "BlockResults")]
pub struct PyBlockResults {
    /// Ordered mapping: block_name -> SolveResult
    results: Vec<(String, Py<PySolveResult>)>,
}

#[pymethods]
impl PyBlockResults {
    fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<Py<PySolveResult>> {
        self.results
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, result)| result.clone_ref(py))
            .ok_or_else(|| PyKeyError::new_err(key.to_string()))
    }

    fn __len__(&self) -> usize {
        self.results.len()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.results.iter().any(|(name, _)| name == key)
    }

    fn keys(&self) -> Vec<String> {
        self.results.iter().map(|(name, _)| name.clone()).collect()
    }

    fn values(&self, py: Python<'_>) -> Vec<Py<PySolveResult>> {
        self.results
            .iter()
            .map(|(_, result)| result.clone_ref(py))
            .collect()
    }

    fn items(&self, py: Python<'_>) -> Vec<(String, Py<PySolveResult>)> {
        self.results
            .iter()
            .map(|(name, result)| (name.clone(), result.clone_ref(py)))
            .collect()
    }

    fn __repr__(&self) -> String {
        let names: Vec<&str> = self.results.iter().map(|(n, _)| n.as_str()).collect();
        format!("BlockResults({})", names.join(", "))
    }
}

/// Stored block definition for model.add_block()
pub(crate) struct BlockDef {
    pub(crate) name: String,
    pub(crate) build_adapter: PyObject,
    pub(crate) extract_adapter: PyObject,
    pub(crate) input_fields: Py<PyDict>,
    pub(crate) output_fields: Py<PyDict>,
    pub(crate) provided_inputs: Py<PyDict>,
}

struct TypedBlockMeta {
    default_name: String,
    input_schema: PyObject,
    input_fields: Py<PyDict>,
    expects_ctx: bool,
}

struct TypedExtractMeta {
    output_schema: PyObject,
    output_fields: Py<PyDict>,
    expects_ctx: bool,
}

#[pyclass]
struct TypedBlockBuilder {
    user_fn: PyObject,
    input_schema: PyObject,
    expects_ctx: bool,
}

#[pymethods]
impl TypedBlockBuilder {
    fn __call__(&self, py: Python<'_>, ctx: PyObject) -> PyResult<PyObject> {
        let inputs_obj = ctx.bind(py).getattr("inputs")?.unbind();
        let data =
            coerce_to_schema_instance(py, inputs_obj, self.input_schema.bind(py), "Block input")?;
        let model = Py::new(py, PyModel::new(None, None)?)?.into_any();
        let result = if self.expects_ctx {
            self.user_fn.bind(py).call1((
                model.clone_ref(py),
                data.clone_ref(py),
                ctx.clone_ref(py),
            ))?
        } else {
            self.user_fn
                .bind(py)
                .call1((model.clone_ref(py), data.clone_ref(py)))?
        };
        if !result.is_none() {
            return Err(PyTypeError::new_err(
                "block build function must return None",
            ));
        }
        Ok(model)
    }
}

#[pyclass]
struct TypedBlockExtractor {
    extract_fn: PyObject,
    input_schema: PyObject,
    output_schema: PyObject,
    expects_ctx: bool,
}

#[pymethods]
impl TypedBlockExtractor {
    fn __call__(&self, py: Python<'_>, solution: PyObject, ctx: PyObject) -> PyResult<PyObject> {
        let inputs = ctx.bind(py).getattr("inputs")?.unbind();
        let data =
            coerce_to_schema_instance(py, inputs, self.input_schema.bind(py), "Block input")?;
        let outputs = if self.expects_ctx {
            self.extract_fn
                .bind(py)
                .call1((solution, data, ctx.clone_ref(py)))?
        } else {
            self.extract_fn.bind(py).call1((solution, data))?
        };
        schema_instance_to_dict(
            py,
            outputs.unbind(),
            self.output_schema.bind(py),
            "Block output",
        )
    }
}

#[pyclass]
struct TypedBlockDecorator {
    name: Option<String>,
}

#[pymethods]
impl TypedBlockDecorator {
    fn __call__(&self, py: Python<'_>, func: PyObject) -> PyResult<PyObject> {
        decorate_block_function(py, func.bind(py), self.name.as_deref())
    }
}

/// Stored link definition for model.link()
pub(crate) struct LinkDef {
    pub(crate) source: BlockPort,
    pub(crate) target: BlockPort,
}

impl PyModel {
    /// Execute composed block solve by delegating to BlockModel infrastructure.
    pub(crate) fn solve_composed(
        &mut self,
        py: Python<'_>,
        solver: Option<&Bound<'_, PyAny>>,
        log_to_console: Option<bool>,
        primal_start: Option<Vec<(u32, f64)>>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
    ) -> PyResult<Py<PySolveResult>> {
        if primal_start.is_some() {
            return Err(PyRuntimeError::new_err(
                "primal_start is not supported for composed models",
            ));
        }

        let blocks_module = py.import("arco.blocks")?;
        let block_model_class = blocks_module.getattr("BlockModel")?;

        // Create a BlockModel
        let kwargs = PyDict::new(py);
        kwargs.set_item("name", "Model")?;
        let block_model = block_model_class.call((), Some(&kwargs))?;

        // Add each block to the BlockModel
        for block_def in &self.block_defs {
            let add_kwargs = PyDict::new(py);
            add_kwargs.set_item("name", &block_def.name)?;
            add_kwargs.set_item("outputs", block_def.output_fields.bind(py))?;
            add_kwargs.set_item("extract", block_def.extract_adapter.bind(py))?;
            add_kwargs.set_item("inputs", block_def.provided_inputs.bind(py))?;
            add_kwargs.set_item("inputs_schema", block_def.input_fields.bind(py))?;

            block_model.call_method(
                "add_block",
                (block_def.build_adapter.bind(py),),
                Some(&add_kwargs),
            )?;
        }

        // Add links
        for link_def in &self.link_defs {
            block_model.call_method1("link", (link_def.source.clone(), link_def.target.clone()))?;
        }

        // Solve the block model
        let runs = if solver.is_some()
            || log_to_console.is_some()
            || time_limit.is_some()
            || mip_gap.is_some()
            || verbosity.is_some()
        {
            let solve_kwargs = PyDict::new(py);
            if let Some(solver) = solver {
                solve_kwargs.set_item("solver", solver)?;
            }
            if let Some(enabled) = log_to_console {
                solve_kwargs.set_item("log_to_console", enabled)?;
            }
            if let Some(limit) = time_limit {
                solve_kwargs.set_item("time_limit", limit)?;
            }
            if let Some(gap) = mip_gap {
                solve_kwargs.set_item("mip_gap", gap)?;
            }
            if let Some(level) = verbosity {
                solve_kwargs.set_item("verbosity", level)?;
            }
            block_model.call_method("solve", (), Some(&solve_kwargs))?
        } else {
            block_model.call_method0("solve")?
        };
        let runs_list = runs.cast::<PyList>()?;

        // Build per-block results
        let mut block_results = Vec::new();

        for run in runs_list.iter() {
            let name: String = run.getattr("name")?.extract()?;
            let solution_opt = run.getattr("solution")?;

            if solution_opt.is_none() {
                // Block was dropped — create a minimal error result
                let result = PySolveResult::new(crate::solve_failure_solution(
                    arco_core::solver::SolverStatus::Unknown,
                ));
                let py_result = Py::new(py, result)?;
                block_results.push((name, py_result));
            } else {
                // The solution is a PySolveResult from the sub-model's solve()
                let result: Py<PySolveResult> = solution_opt.extract()?;
                block_results.push((name, result));
            }
        }

        // Build the BlockResults container
        let block_results_obj: PyObject = Py::new(
            py,
            PyBlockResults {
                results: block_results
                    .iter()
                    .map(|(n, r)| (n.clone(), r.clone_ref(py)))
                    .collect(),
            },
        )?
        .into_any();

        // Derive a conservative top-level status from all block results.
        let primary_inner = if block_results.len() == 1 {
            let borrowed = block_results[0].1.borrow(py);
            borrowed.inner().clone()
        } else {
            let statuses = block_results
                .iter()
                .map(|(_, result)| result.borrow(py).inner().status)
                .collect::<Vec<_>>();
            crate::solve_failure_solution(aggregate_block_status(&statuses))
        };

        let result = PySolveResult::with_blocks(primary_inner, block_results_obj);
        let py_result = Py::new(py, result)?;

        self.last_solution = Some(py_result.clone_ref(py));
        Ok(py_result)
    }
}

fn aggregate_block_status(statuses: &[SolverStatus]) -> SolverStatus {
    if statuses.is_empty() {
        return SolverStatus::Unknown;
    }
    if statuses
        .iter()
        .all(|status| *status == SolverStatus::Optimal)
    {
        return SolverStatus::Optimal;
    }

    let has_infeasible = statuses.contains(&SolverStatus::Infeasible);
    let has_unbounded = statuses.contains(&SolverStatus::Unbounded);
    if has_infeasible && has_unbounded {
        return SolverStatus::Unknown;
    }
    if has_infeasible {
        return SolverStatus::Infeasible;
    }
    if has_unbounded {
        return SolverStatus::Unbounded;
    }
    if statuses.contains(&SolverStatus::Unknown) {
        return SolverStatus::Unknown;
    }

    let has_time_limit = statuses.contains(&SolverStatus::TimeLimit);
    let has_iteration_limit = statuses.contains(&SolverStatus::IterationLimit);
    if has_time_limit && has_iteration_limit {
        SolverStatus::Unknown
    } else if has_time_limit {
        SolverStatus::TimeLimit
    } else if has_iteration_limit {
        SolverStatus::IterationLimit
    } else {
        SolverStatus::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::aggregate_block_status;
    use arco_core::solver::SolverStatus;

    #[test]
    fn aggregate_returns_optimal_when_all_blocks_optimal() {
        let status = aggregate_block_status(&[SolverStatus::Optimal, SolverStatus::Optimal]);
        assert_eq!(status, SolverStatus::Optimal);
    }

    #[test]
    fn aggregate_prioritizes_infeasible_over_limits() {
        let status = aggregate_block_status(&[SolverStatus::Optimal, SolverStatus::Infeasible]);
        assert_eq!(status, SolverStatus::Infeasible);
    }

    #[test]
    fn aggregate_reports_unknown_on_conflicting_terminal_statuses() {
        let status = aggregate_block_status(&[SolverStatus::Infeasible, SolverStatus::Unbounded]);
        assert_eq!(status, SolverStatus::Unknown);
    }
}

impl PyModel {
    pub(crate) fn add_block_impl(
        &mut self,
        py: Python<'_>,
        block_fn: PyObject,
        name: Option<String>,
        data: Option<PyObject>,
        extract: PyObject,
    ) -> PyResult<PyBlockHandle> {
        let meta = typed_block_meta_from_decorated(py, block_fn.bind(py))?;
        let block_name = name.unwrap_or_else(|| meta.default_name.clone());
        let extract_meta = typed_extract_meta_from_function(
            py,
            extract.bind(py),
            meta.input_schema.bind(py),
            &block_name,
        )?;

        // Validate no duplicate block names
        if self.block_defs.iter().any(|b| b.name == block_name) {
            return Err(PyRuntimeError::new_err(format!(
                "add_block: block '{}' already exists",
                block_name
            )));
        }

        let provided_inputs = if let Some(data) = data {
            let typed_data =
                coerce_to_schema_instance(py, data, meta.input_schema.bind(py), "Block root data")?;
            let as_dict = schema_instance_to_dict(
                py,
                typed_data,
                meta.input_schema.bind(py),
                "Block root data",
            )?;
            as_dict.bind(py).cast::<PyDict>()?.clone().unbind()
        } else {
            PyDict::new(py).unbind()
        };

        let build_adapter = Py::new(
            py,
            TypedBlockBuilder {
                user_fn: block_fn,
                input_schema: meta.input_schema.clone_ref(py),
                expects_ctx: meta.expects_ctx,
            },
        )?
        .into_any();

        let extract_adapter = Py::new(
            py,
            TypedBlockExtractor {
                extract_fn: extract,
                input_schema: meta.input_schema.clone_ref(py),
                output_schema: extract_meta.output_schema.clone_ref(py),
                expects_ctx: extract_meta.expects_ctx,
            },
        )?
        .into_any();

        self.block_defs.push(BlockDef {
            name: block_name.clone(),
            build_adapter,
            extract_adapter,
            input_fields: meta.input_fields.clone_ref(py),
            output_fields: extract_meta.output_fields.clone_ref(py),
            provided_inputs,
        });

        Ok(PyBlockHandle {
            name: block_name,
            input_keys: collect_schema_keys(meta.input_fields.bind(py))?,
            output_keys: collect_schema_keys(extract_meta.output_fields.bind(py))?,
        })
    }

    pub(crate) fn link_impl(
        &mut self,
        py: Python<'_>,
        source: BlockPort,
        target: BlockPort,
    ) -> PyResult<()> {
        if source.kind != "output" {
            return Err(PyRuntimeError::new_err(
                "link: source must be a block output port",
            ));
        }
        if target.kind != "input" {
            return Err(PyRuntimeError::new_err(
                "link: target must be a block input port",
            ));
        }

        let source_block = self
            .block_defs
            .iter()
            .find(|block| block.name == source.block_name)
            .ok_or_else(|| {
                PyRuntimeError::new_err(format!(
                    "link: unknown source block '{}'",
                    source.block_name
                ))
            })?;
        let target_block = self
            .block_defs
            .iter()
            .find(|block| block.name == target.block_name)
            .ok_or_else(|| {
                PyRuntimeError::new_err(format!(
                    "link: unknown target block '{}'",
                    target.block_name
                ))
            })?;

        let source_schema = source_block
            .output_fields
            .bind(py)
            .get_item(&source.key)?
            .ok_or_else(|| {
                PyRuntimeError::new_err(format!(
                    "link: unknown source port '{}.{}'",
                    source.block_name, source.key
                ))
            })?;
        let target_schema = target_block
            .input_fields
            .bind(py)
            .get_item(&target.key)?
            .ok_or_else(|| {
                PyRuntimeError::new_err(format!(
                    "link: unknown target port '{}.{}'",
                    target.block_name, target.key
                ))
            })?;
        if !source_schema.eq(&target_schema)? {
            return Err(PyTypeError::new_err(format!(
                "link: type mismatch for '{}.{}' -> '{}.{}'",
                source.block_name, source.key, target.block_name, target.key
            )));
        }

        self.link_defs.push(LinkDef { source, target });
        Ok(())
    }

    pub(crate) fn has_blocks_impl(&self) -> bool {
        !self.block_defs.is_empty()
    }
}

#[pyfunction]
#[pyo3(signature = (func=None, *, name=None))]
pub(crate) fn typed_block(
    py: Python<'_>,
    func: Option<PyObject>,
    name: Option<String>,
) -> PyResult<PyObject> {
    if let Some(func) = func {
        return decorate_block_function(py, func.bind(py), name.as_deref());
    }
    Ok(Py::new(py, TypedBlockDecorator { name })?.into_any())
}

fn decorate_block_function(
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    name_override: Option<&str>,
) -> PyResult<PyObject> {
    let meta = typed_block_meta_from_function(py, func, name_override)?;
    func.setattr(ARCO_BLOCK_MARKER_ATTR, true)?;
    func.setattr(ARCO_BLOCK_NAME_ATTR, &meta.default_name)?;
    func.setattr(ARCO_BLOCK_INPUT_SCHEMA_ATTR, meta.input_schema.bind(py))?;
    func.setattr(ARCO_BLOCK_INPUT_FIELDS_ATTR, meta.input_fields.bind(py))?;
    func.setattr(ARCO_BLOCK_EXPECTS_CTX_ATTR, meta.expects_ctx)?;
    Ok(func.clone().unbind())
}

fn typed_block_meta_from_decorated(
    _py: Python<'_>,
    func: &Bound<'_, PyAny>,
) -> PyResult<TypedBlockMeta> {
    let marker = func
        .getattr(ARCO_BLOCK_MARKER_ATTR)
        .and_then(|value| value.extract::<bool>())
        .unwrap_or(false);
    if !marker {
        return Err(PyTypeError::new_err(
            "add_block: block function must be decorated with @arco.block",
        ));
    }
    let default_name = func.getattr(ARCO_BLOCK_NAME_ATTR)?.extract::<String>()?;
    let input_schema = func.getattr(ARCO_BLOCK_INPUT_SCHEMA_ATTR)?.unbind();
    let input_fields = func
        .getattr(ARCO_BLOCK_INPUT_FIELDS_ATTR)?
        .cast::<PyDict>()?
        .clone()
        .unbind();
    let expects_ctx = func
        .getattr(ARCO_BLOCK_EXPECTS_CTX_ATTR)?
        .extract::<bool>()?;

    Ok(TypedBlockMeta {
        default_name,
        input_schema,
        input_fields,
        expects_ctx,
    })
}

fn typed_block_meta_from_function(
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    name_override: Option<&str>,
) -> PyResult<TypedBlockMeta> {
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
    }
    for param in &params {
        let kind = param.bind(py).getattr("kind")?;
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
    validate_schema_type(py, &input_schema, "input")?;

    let default_name = if let Some(name) = name_override {
        name.to_string()
    } else {
        func.getattr("__name__")?.extract::<String>()?
    };

    let input_fields = schema_fields_dict(py, &input_schema)?;
    Ok(TypedBlockMeta {
        default_name,
        input_schema: input_schema.unbind(),
        input_fields,
        expects_ctx: params.len() == 3,
    })
}

fn typed_extract_meta_from_function(
    py: Python<'_>,
    extract: &Bound<'_, PyAny>,
    expected_input_schema: &Bound<'_, PyAny>,
    block_name: &str,
) -> PyResult<TypedExtractMeta> {
    if !extract.is_callable() {
        return Err(PyTypeError::new_err(format!(
            "add_block: extract for block '{block_name}' must be callable"
        )));
    }
    let inspect = PyModule::import(py, "inspect")?;
    let signature = inspect.getattr("signature")?.call1((extract,))?;
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
        return Err(PyTypeError::new_err(format!(
            "add_block: extract for block '{block_name}' must use (solution, data) or (solution, data, ctx)"
        )));
    }
    for param in &params {
        let kind = param.bind(py).getattr("kind")?;
        if kind.eq(&var_positional)? || kind.eq(&var_keyword)? || kind.eq(&keyword_only)? {
            return Err(PyTypeError::new_err(format!(
                "add_block: extract for block '{block_name}' cannot use variadic or keyword-only parameters"
            )));
        }
    }

    let input_annotation = params[1].bind(py).getattr("annotation")?;
    if input_annotation.is(&empty) {
        return Err(PyTypeError::new_err(format!(
            "add_block: extract for block '{block_name}' must annotate the data parameter"
        )));
    }
    if !input_annotation.eq(expected_input_schema)? {
        return Err(PyTypeError::new_err(format!(
            "add_block: extract data annotation must match block input schema for '{block_name}'"
        )));
    }

    let output_schema = signature.getattr("return_annotation")?;
    if output_schema.is(&empty) {
        return Err(PyTypeError::new_err(format!(
            "add_block: extract for block '{block_name}' must annotate its return type"
        )));
    }
    validate_schema_type(py, &output_schema, "output")?;

    let output_fields = schema_fields_dict(py, &output_schema)?;
    Ok(TypedExtractMeta {
        output_schema: output_schema.unbind(),
        output_fields,
        expects_ctx: params.len() == 3,
    })
}

fn validate_schema_type(py: Python<'_>, schema: &Bound<'_, PyAny>, role: &str) -> PyResult<()> {
    if is_dataclass_schema(py, schema)? || is_pydantic_schema(py, schema)? {
        return Ok(());
    }
    Err(PyTypeError::new_err(format!(
        "block: {role} schema must be a dataclass or pydantic BaseModel type"
    )))
}

fn is_dataclass_schema(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<bool> {
    if schema.cast::<PyType>().is_err() {
        return Ok(false);
    }
    let dataclasses = PyModule::import(py, "dataclasses")?;
    dataclasses
        .getattr("is_dataclass")?
        .call1((schema,))?
        .extract::<bool>()
}

fn is_pydantic_schema(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<bool> {
    let Ok(schema_type) = schema.cast::<PyType>() else {
        return Ok(false);
    };
    let Ok(pydantic) = PyModule::import(py, "pydantic") else {
        return Ok(false);
    };
    let base_model_any = pydantic.getattr("BaseModel")?;
    let base_model = base_model_any.cast::<PyType>()?;
    schema_type.is_subclass(base_model)
}

fn schema_fields_dict(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<Py<PyDict>> {
    if is_dataclass_schema(py, schema)? {
        let dataclasses = PyModule::import(py, "dataclasses")?;
        let fields = dataclasses.getattr("fields")?.call1((schema,))?;
        let out = PyDict::new(py);
        for field in fields.try_iter()? {
            let field = field?;
            out.set_item(field.getattr("name")?, field.getattr("type")?)?;
        }
        return Ok(out.unbind());
    }
    if is_pydantic_schema(py, schema)? {
        let out = PyDict::new(py);
        let fields_any = schema.getattr("model_fields")?;
        let fields = fields_any.cast::<PyDict>()?;
        for (name, field) in fields.iter() {
            out.set_item(name, field.getattr("annotation")?)?;
        }
        return Ok(out.unbind());
    }
    Err(PyTypeError::new_err(
        "Unsupported schema type while extracting fields",
    ))
}

fn coerce_to_schema_instance(
    py: Python<'_>,
    value: PyObject,
    schema: &Bound<'_, PyAny>,
    context: &str,
) -> PyResult<PyObject> {
    if value.bind(py).is_instance(schema)? {
        return Ok(value);
    }
    if is_pydantic_schema(py, schema)? {
        let validated = schema.call_method1("model_validate", (value.clone_ref(py),))?;
        return Ok(validated.unbind());
    }
    if is_dataclass_schema(py, schema)? {
        if value.bind(py).is_instance_of::<PyDict>() {
            let dict = value.bind(py).cast::<PyDict>()?;
            let instance = schema.call((), Some(dict))?;
            return Ok(instance.unbind());
        }
        return Err(PyValueError::new_err(format!(
            "{context} must be a dict or {} instance",
            schema.get_type().name()?
        )));
    }
    Err(PyTypeError::new_err(format!(
        "{context} has unsupported schema type"
    )))
}

fn schema_instance_to_dict(
    py: Python<'_>,
    value: PyObject,
    schema: &Bound<'_, PyAny>,
    context: &str,
) -> PyResult<PyObject> {
    if value.bind(py).is_instance_of::<PyDict>() {
        return Ok(value);
    }
    if is_pydantic_schema(py, schema)? {
        return Ok(value.bind(py).call_method0("model_dump")?.unbind());
    }
    if is_dataclass_schema(py, schema)? {
        if !value.bind(py).is_instance(schema)? {
            return Err(PyValueError::new_err(format!(
                "{context} must be a {} instance",
                schema.get_type().name()?
            )));
        }
        let fields = schema_fields_dict(py, schema)?;
        let out = PyDict::new(py);
        for (name, _) in fields.bind(py).iter() {
            let key = name.extract::<String>()?;
            out.set_item(&key, value.bind(py).getattr(&key)?)?;
        }
        return Ok(out.into_any().unbind());
    }
    Err(PyTypeError::new_err(format!(
        "{context} has unsupported schema type"
    )))
}

fn collect_schema_keys(fields: &Bound<'_, PyDict>) -> PyResult<HashSet<String>> {
    let mut keys = HashSet::new();
    for key in fields.keys().iter() {
        keys.insert(key.extract::<String>()?);
    }
    Ok(keys)
}
