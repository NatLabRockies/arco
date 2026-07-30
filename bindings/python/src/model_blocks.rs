use crate::py_modules::enums::PyLpAlgorithm;
use crate::py_modules::errors::{BlockArtifactError, BlockContractError, BlockResultError};
use crate::py_modules::serde_bridge;
use crate::{BlockPort, PyModel, PyObject, PySolveResult};
use arco_blocks::{DropPolicy, build_execution_levels, retention_for_policy};
use arco_model::{InspectOptions, ModelSnapshot};
use arco_solver::Solution;
use arco_solver::SolverStatus;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const ARCO_BLOCK_MARKER_ATTR: &str = "__arco_block_marker__";
const ARCO_BLOCK_NAME_ATTR: &str = "__arco_block_name__";
const ARCO_BLOCK_INPUT_SCHEMA_ATTR: &str = "__arco_block_input_schema__";
const ARCO_BLOCK_INPUT_FIELDS_ATTR: &str = "__arco_block_input_fields__";
const ARCO_BLOCK_EXPECTS_CTX_ATTR: &str = "__arco_block_expects_ctx__";

#[pyo3_macros::pyclass(name = "BlockPorts")]
pub struct PyBlockPorts {
    block_name: String,
    kind: String,
    keys: HashSet<String>,
}

#[pyo3_macros::pymethods]
impl PyBlockPorts {
    fn __getattr__(&self, key: &str) -> PyResult<BlockPort> {
        if !self.keys.contains(key) {
            return Err(BlockContractError::new_err(format!(
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
        self.keys()
    }

    fn keys(&self) -> Vec<String> {
        let mut keys = self.keys.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }
}

/// A handle returned by model.add_block() with typed `.in_` and `.out` accessors.
#[pyo3_macros::pyclass(name = "BlockHandle")]
pub struct PyBlockHandle {
    name: String,
    input_keys: HashSet<String>,
    output_keys: HashSet<String>,
}

#[pyo3_macros::pymethods]
impl PyBlockHandle {
    /// Get an input port reference for linking.
    fn input(&self, key: String) -> PyResult<BlockPort> {
        if !self.input_keys.contains(&key) {
            return Err(BlockContractError::new_err(format!(
                "Unknown input port '{}.{}'",
                self.name, key
            )));
        }
        Ok(BlockPort::new_input(self.name.clone(), key))
    }

    /// Get an output port reference for linking.
    fn output(&self, key: String) -> PyResult<BlockPort> {
        if !self.output_keys.contains(&key) {
            return Err(BlockContractError::new_err(format!(
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
#[pyo3_macros::pyclass(name = "BlockResults")]
pub struct PyBlockResults {
    /// Ordered mapping: block_name -> SolveResult
    results: Vec<(String, Py<PySolveResult>)>,
    artifacts: Vec<BlockRunArtifacts>,
}

#[derive(Clone)]
struct BlockRunArtifacts {
    name: String,
    model_snapshot: Value,
    solution_summary: Value,
}

impl PyBlockResults {
    fn report_rows(&self, py: Python<'_>) -> Vec<Value> {
        self.results
            .iter()
            .enumerate()
            .map(|(order, (name, result))| {
                let borrowed = result.borrow(py);
                let inner = borrowed.inner();
                json!({
                    "order": order,
                    "name": name,
                    "status": inner.status_string().to_uppercase(),
                    "objective_value": inner.objective_value,
                    "variable_count": inner.primal_values.len(),
                    "constraint_count": inner.constraint_duals.len(),
                })
            })
            .collect()
    }

    fn diagnostic_rows(&self) -> Vec<Value> {
        self.artifacts
            .iter()
            .enumerate()
            .map(|(order, artifacts)| {
                stage_diagnostics_artifact(
                    order,
                    &artifacts.name,
                    &artifacts.model_snapshot,
                    &artifacts.solution_summary,
                )
            })
            .collect()
    }

    fn artifact_manifest_rows(&self, policy: DropPolicy) -> Vec<Value> {
        let retention = retention_for_policy(policy);
        let mut artifacts = Vec::new();
        if retention.keep_diagnostics {
            artifacts.push("stage_diagnostics");
        }
        if retention.keep_model {
            artifacts.push("model_snapshot");
        }
        if retention.keep_solution {
            artifacts.push("solution_summary");
        }

        self.results
            .iter()
            .enumerate()
            .map(|(order, (name, _))| {
                json!({
                    "order": order,
                    "name": name,
                    "artifacts": artifacts,
                })
            })
            .collect()
    }

    fn write_artifact_rows(&self, directory: &Path, policy: DropPolicy) -> PyResult<Vec<Value>> {
        let retention = retention_for_policy(policy);
        fs::create_dir_all(directory).map_err(|err| {
            BlockArtifactError::new_err(format!(
                "failed to create block artifact directory '{}': {err}",
                directory.display()
            ))
        })?;

        let mut rows = Vec::new();
        for (order, artifacts) in self.artifacts.iter().enumerate() {
            let block_dir_name = format!(
                "{order:03}-{}",
                sanitize_artifact_path_part(&artifacts.name)
            );
            let block_dir = directory.join(&block_dir_name);
            fs::create_dir_all(&block_dir).map_err(|err| {
                BlockArtifactError::new_err(format!(
                    "failed to create block artifact directory '{}': {err}",
                    block_dir.display()
                ))
            })?;

            let mut files = Vec::new();
            if retention.keep_diagnostics {
                let relative_path = format!("{block_dir_name}/stage_diagnostics.json");
                let stage_diagnostics = stage_diagnostics_artifact(
                    order,
                    &artifacts.name,
                    &artifacts.model_snapshot,
                    &artifacts.solution_summary,
                );
                write_json_file(&directory.join(&relative_path), &stage_diagnostics)?;
                files.push(json!({
                    "artifact": "stage_diagnostics",
                    "path": relative_path,
                }));
            }
            if retention.keep_model {
                let relative_path = format!("{block_dir_name}/model_snapshot.json");
                write_json_file(&directory.join(&relative_path), &artifacts.model_snapshot)?;
                files.push(json!({
                    "artifact": "model_snapshot",
                    "path": relative_path,
                }));
            }
            if retention.keep_solution {
                let relative_path = format!("{block_dir_name}/solution_summary.json");
                write_json_file(&directory.join(&relative_path), &artifacts.solution_summary)?;
                files.push(json!({
                    "artifact": "solution_summary",
                    "path": relative_path,
                }));
            }

            rows.push(json!({
                "order": order,
                "name": artifacts.name,
                "files": files,
            }));
        }

        let manifest = json!({
            "policy": drop_policy_name(policy),
            "blocks": rows,
        });
        write_json_file(&directory.join("manifest.json"), &manifest)?;
        let Some(blocks) = manifest.get("blocks").and_then(Value::as_array) else {
            return Err(BlockArtifactError::new_err(
                "block artifact manifest must contain block rows",
            ));
        };
        Ok(blocks.clone())
    }
}

#[pyo3_macros::pymethods]
impl PyBlockResults {
    fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<Py<PySolveResult>> {
        self.results
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, result)| result.clone_ref(py))
            .ok_or_else(|| BlockResultError::new_err(format!("block result '{key}' not found")))
    }

    fn __len__(&self) -> usize {
        self.results.len()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.results.iter().any(|(name, _)| name == key)
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let keys = self.keys();
        Ok(PyList::new(py, keys)?.call_method0("__iter__")?.unbind())
    }

    fn keys(&self) -> Vec<String> {
        self.results.iter().map(|(name, _)| name.clone()).collect()
    }

    #[pyo3(signature = (key, default=None))]
    fn get(&self, py: Python<'_>, key: &str, default: Option<PyObject>) -> PyObject {
        self.results
            .iter()
            .find(|(name, _)| name == key)
            .map_or_else(
                || default.unwrap_or_else(|| py.None()),
                |(_, result)| result.clone_ref(py).into_any(),
            )
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

    fn statuses(&self, py: Python<'_>) -> PyResult<PyObject> {
        let statuses = PyDict::new(py);
        for (name, result) in &self.results {
            let status = result.borrow(py).inner().status_string().to_uppercase();
            statuses.set_item(name, status)?;
        }
        Ok(statuses.into_any().unbind())
    }

    fn report(&self, py: Python<'_>) -> PyResult<PyObject> {
        let rows = PyList::empty(py);
        for row_value in self.report_rows(py) {
            let Some(row_object) = row_value.as_object() else {
                return Err(BlockResultError::new_err(
                    "block report row must be an object",
                ));
            };
            let row = PyDict::new(py);
            for key in [
                "order",
                "name",
                "status",
                "objective_value",
                "variable_count",
                "constraint_count",
            ] {
                let Some(value) = row_object.get(key) else {
                    return Err(BlockResultError::new_err("block report row is incomplete"));
                };
                let py_value = serde_bridge::json_to_py(py, value)?;
                row.set_item(key, py_value.bind(py))?;
            }
            rows.append(row)?;
        }
        Ok(rows.into_any().unbind())
    }

    fn report_json(&self, py: Python<'_>) -> PyResult<String> {
        serde_json::to_string(&self.report_rows(py)).map_err(|err| {
            BlockResultError::new_err(format!("failed to encode block report as JSON: {err}"))
        })
    }

    fn diagnostics(&self, py: Python<'_>) -> PyResult<PyObject> {
        let rows = PyList::empty(py);
        for row_value in self.diagnostic_rows() {
            let py_value = serde_bridge::json_to_py(py, &row_value)?;
            rows.append(py_value.bind(py))?;
        }
        Ok(rows.into_any().unbind())
    }

    fn diagnostics_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.diagnostic_rows()).map_err(|err| {
            BlockResultError::new_err(format!("failed to encode block diagnostics as JSON: {err}"))
        })
    }

    #[pyo3(signature = (*, policy = "summary"))]
    fn artifact_manifest(&self, py: Python<'_>, policy: &str) -> PyResult<PyObject> {
        let rows = PyList::empty(py);
        for row_value in self.artifact_manifest_rows(parse_drop_policy(policy)?) {
            let Some(row_object) = row_value.as_object() else {
                return Err(BlockArtifactError::new_err(
                    "block artifact manifest row must be an object",
                ));
            };
            let row = PyDict::new(py);
            for key in ["order", "name", "artifacts"] {
                let Some(value) = row_object.get(key) else {
                    return Err(BlockArtifactError::new_err(
                        "block artifact manifest row is incomplete",
                    ));
                };
                let py_value = serde_bridge::json_to_py(py, value)?;
                row.set_item(key, py_value.bind(py))?;
            }
            rows.append(row)?;
        }
        Ok(rows.into_any().unbind())
    }

    #[pyo3(signature = (*, policy = "summary"))]
    fn artifact_manifest_json(&self, policy: &str) -> PyResult<String> {
        serde_json::to_string(&self.artifact_manifest_rows(parse_drop_policy(policy)?)).map_err(
            |err| {
                BlockArtifactError::new_err(format!(
                    "failed to encode block artifact manifest as JSON: {err}"
                ))
            },
        )
    }

    #[pyo3(signature = (directory, *, policy = "summary"))]
    fn write_artifacts(
        &self,
        py: Python<'_>,
        directory: PathBuf,
        policy: &str,
    ) -> PyResult<PyObject> {
        let rows = PyList::empty(py);
        for row_value in self.write_artifact_rows(&directory, parse_drop_policy(policy)?)? {
            let Some(row_object) = row_value.as_object() else {
                return Err(BlockArtifactError::new_err(
                    "block artifact writer row must be an object",
                ));
            };
            let row = PyDict::new(py);
            for key in ["order", "name", "files"] {
                let Some(value) = row_object.get(key) else {
                    return Err(BlockArtifactError::new_err(
                        "block artifact writer row is incomplete",
                    ));
                };
                let py_value = serde_bridge::json_to_py(py, value)?;
                row.set_item(key, py_value.bind(py))?;
            }
            rows.append(row)?;
        }
        Ok(rows.into_any().unbind())
    }

    fn __repr__(&self) -> String {
        let names: Vec<&str> = self.results.iter().map(|(n, _)| n.as_str()).collect();
        format!("BlockResults({})", names.join(", "))
    }
}

fn parse_drop_policy(policy: &str) -> PyResult<DropPolicy> {
    match policy {
        "model" => Ok(DropPolicy::KeepModel),
        "summary" => Ok(DropPolicy::KeepSummary),
        "none" => Ok(DropPolicy::DropAll),
        other => Err(BlockContractError::new_err(format!(
            "unknown block artifact policy `{other}`; expected 'model', 'summary', or 'none'"
        ))),
    }
}

fn drop_policy_name(policy: DropPolicy) -> &'static str {
    match policy {
        DropPolicy::KeepModel => "model",
        DropPolicy::KeepSummary => "summary",
        DropPolicy::DropAll => "none",
    }
}

fn sanitize_artifact_path_part(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "block".to_string()
    } else {
        sanitized
    }
}

fn write_json_file(path: &Path, value: &Value) -> PyResult<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| {
        BlockArtifactError::new_err(format!(
            "failed to encode block artifact '{}': {err}",
            path.display()
        ))
    })?;
    fs::write(path, bytes).map_err(|err| {
        BlockArtifactError::new_err(format!(
            "failed to write block artifact '{}': {err}",
            path.display()
        ))
    })
}

fn solution_summary_artifact(solution: &Solution) -> Value {
    json!({
        "status": solution.status_string().to_uppercase(),
        "objective_value": solution.objective_value,
        "solve_time_seconds": solution.solve_time_seconds,
        "variable_count": solution.primal_values.len(),
        "constraint_count": solution.constraint_duals.len(),
        "primal_values": solution.primal_values,
        "variable_duals": solution.variable_duals,
        "constraint_duals": solution.constraint_duals,
        "row_values": solution.row_values,
        "metadata": solution.metadata,
    })
}

fn stage_diagnostics_artifact(
    order: usize,
    name: &str,
    model_snapshot: &Value,
    solution_summary: &Value,
) -> Value {
    json!({
        "order": order,
        "name": name,
        "status": solution_summary.get("status").cloned().unwrap_or(Value::Null),
        "objective_value": solution_summary
            .get("objective_value")
            .cloned()
            .unwrap_or(Value::Null),
        "result": {
            "variable_count": solution_summary
                .get("variable_count")
                .cloned()
                .unwrap_or(Value::Null),
            "constraint_count": solution_summary
                .get("constraint_count")
                .cloned()
                .unwrap_or(Value::Null),
        },
        "model": model_snapshot
            .get("metadata")
            .cloned()
            .unwrap_or(Value::Null),
    })
}

fn model_snapshot_artifact(snapshot: ModelSnapshot) -> Value {
    json!({
        "metadata": {
            "variables": snapshot.metadata.variables,
            "constraints": snapshot.metadata.constraints,
            "coefficients": snapshot.metadata.coefficients,
            "memory": {
                "coefficient_value_bytes": snapshot.metadata.memory.coefficient_value_bytes,
                "coefficient_index_bytes": snapshot.metadata.memory.coefficient_index_bytes,
                "variable_column_pointer_bytes": snapshot.metadata.memory.variable_column_pointer_bytes,
                "sparse_matrix_bytes": snapshot.metadata.memory.sparse_matrix_bytes,
            },
        },
        "objective": snapshot.objective.map(|objective| {
            json!({
                "sense": objective.sense.map(|sense| match sense {
                    arco_model::Sense::Minimize => "MINIMIZE",
                    arco_model::Sense::Maximize => "MAXIMIZE",
                }),
                "name": objective.name,
                "term_count": objective.terms.len(),
            })
        }),
        "variables": snapshot.variables.into_iter().map(|variable| {
            json!({
                "id": variable.id.inner(),
                "name": variable.name,
                "is_integer": variable.is_integer,
                "is_active": variable.is_active,
            })
        }).collect::<Vec<_>>(),
        "constraints": snapshot.constraints.into_iter().map(|constraint| {
            json!({
                "id": constraint.id.inner(),
                "name": constraint.name,
                "nnz": constraint.nnz,
            })
        }).collect::<Vec<_>>(),
    })
}

/// Stored block definition for model.add_block()
pub(crate) struct BlockDef {
    name: String,
    build_adapter: PyObject,
    extract_adapter: PyObject,
    input_fields: Py<PyDict>,
    output_fields: Py<PyDict>,
    provided_inputs: Py<PyDict>,
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

#[pyo3_macros::pyclass]
struct TypedBlockBuilder {
    user_fn: PyObject,
    input_schema: PyObject,
    expects_ctx: bool,
}

#[pyo3_macros::pymethods]
impl TypedBlockBuilder {
    fn __call__(&self, py: Python<'_>, ctx: PyObject) -> PyResult<PyObject> {
        let inputs_obj = context_inputs(ctx.bind(py))?;
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
            return Err(BlockContractError::new_err(
                "block build function must return None",
            ));
        }
        Ok(model)
    }
}

#[pyo3_macros::pyclass]
struct TypedBlockExtractor {
    extract_fn: PyObject,
    input_schema: PyObject,
    output_schema: PyObject,
    expects_ctx: bool,
}

#[pyo3_macros::pymethods]
impl TypedBlockExtractor {
    fn __call__(&self, py: Python<'_>, solution: PyObject, ctx: PyObject) -> PyResult<PyObject> {
        let inputs = context_inputs(ctx.bind(py))?;
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

fn context_inputs(ctx: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    if let Ok(inputs) = ctx.getattr("inputs") {
        return Ok(inputs.unbind());
    }
    if let Ok(dict) = ctx.cast::<PyDict>() {
        if let Some(inputs) = dict.get_item("inputs")? {
            return Ok(inputs.unbind());
        }
    }
    Err(BlockContractError::new_err(
        "block context must expose inputs as an attribute or dict key",
    ))
}

#[pyo3_macros::pyclass]
struct TypedBlockDecorator {
    name: Option<String>,
}

#[pyo3_macros::pymethods]
impl TypedBlockDecorator {
    fn __call__(&self, py: Python<'_>, func: PyObject) -> PyResult<PyObject> {
        decorate_block_function(py, func.bind(py), self.name.as_deref())
    }
}

/// Stored link definition for model.link()
pub(crate) struct LinkDef {
    source: BlockPort,
    target: BlockPort,
}

impl PyModel {
    /// Execute composed block solve by delegating to BlockModel infrastructure.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn solve_composed(
        &mut self,
        py: Python<'_>,
        solver: Option<&Bound<'_, PyAny>>,
        log_to_console: Option<bool>,
        primal_start: Option<Vec<(u32, f64)>>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
        lp_algorithm: Option<PyLpAlgorithm>,
    ) -> PyResult<Py<PySolveResult>> {
        if primal_start.is_some_and(|values| !values.is_empty()) {
            return Err(
                crate::py_modules::errors::SolverInvalidSettingError::new_err(
                    "primal_start is not supported for composed models",
                ),
            );
        }

        let block_names = self
            .block_defs
            .iter()
            .map(|block| block.name.clone())
            .collect::<Vec<_>>();
        let links = self
            .link_defs
            .iter()
            .map(|link| {
                (
                    link.source.block_name.clone(),
                    link.target.block_name.clone(),
                )
            })
            .collect::<Vec<_>>();
        let execution_levels = build_execution_levels(&block_names, &links)
            .map_err(|error| BlockContractError::new_err(error.to_string()))?;

        let mut block_results: Vec<(String, Py<PySolveResult>)> = Vec::new();
        let mut block_artifacts: Vec<BlockRunArtifacts> = Vec::new();
        let block_outputs = PyDict::new(py);

        for level in execution_levels {
            for block_idx in level {
                let block_def = &self.block_defs[block_idx];
                let inputs = block_def.provided_inputs.bind(py).copy()?;

                for link in self
                    .link_defs
                    .iter()
                    .filter(|link| link.target.block_name == block_def.name)
                {
                    let source_outputs_any = block_outputs
                        .get_item(&link.source.block_name)?
                        .ok_or_else(|| {
                            BlockContractError::new_err(format!(
                                "link: source block '{}' output not available",
                                link.source.block_name
                            ))
                        })?;
                    let source_outputs = source_outputs_any.cast::<PyDict>()?;
                    let value = source_outputs.get_item(&link.source.key)?.ok_or_else(|| {
                        BlockContractError::new_err(format!(
                            "link: unknown source port '{}.{}'",
                            link.source.block_name, link.source.key
                        ))
                    })?;
                    inputs.set_item(&link.target.key, value)?;
                }

                let ctx = PyDict::new(py);
                ctx.set_item("inputs", &inputs)?;
                let model = block_def.build_adapter.bind(py).call1((ctx.clone(),))?;
                let model_snapshot = {
                    let model_ref = model.extract::<PyRef<'_, PyModel>>()?;
                    model_ref.inner.inspect(InspectOptions {
                        include_coefficients: false,
                        include_slacks: true,
                        variable_filter: None,
                        constraint_filter: None,
                    })
                };

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
                if let Some(algorithm) = lp_algorithm {
                    solve_kwargs.set_item("lp_algorithm", algorithm)?;
                }

                let solution_any = if solve_kwargs.is_empty() {
                    model.call_method0("solve")?
                } else {
                    model.call_method("solve", (), Some(&solve_kwargs))?
                };
                let solution: Py<PySolveResult> = solution_any.extract()?;

                let outputs_any = block_def
                    .extract_adapter
                    .bind(py)
                    .call1((solution.clone_ref(py), ctx.clone()))?;
                block_outputs.set_item(&block_def.name, outputs_any)?;
                let solution_summary = {
                    let borrowed = solution.borrow(py);
                    solution_summary_artifact(borrowed.inner())
                };
                block_artifacts.push(BlockRunArtifacts {
                    name: block_def.name.clone(),
                    model_snapshot: model_snapshot_artifact(model_snapshot),
                    solution_summary,
                });
                block_results.push((block_def.name.clone(), solution));
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
                artifacts: block_artifacts,
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
            crate::py_modules::solver::solve_failure_solution(aggregate_block_status(&statuses))
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
    use arco_solver::SolverStatus;

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
            return Err(BlockContractError::new_err(format!(
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
            return Err(BlockContractError::new_err(
                "link: source must be a block output port",
            ));
        }
        if target.kind != "input" {
            return Err(BlockContractError::new_err(
                "link: target must be a block input port",
            ));
        }

        let source_block = self
            .block_defs
            .iter()
            .find(|block| block.name == source.block_name)
            .ok_or_else(|| {
                BlockContractError::new_err(format!(
                    "link: unknown source block '{}'",
                    source.block_name
                ))
            })?;
        let target_block = self
            .block_defs
            .iter()
            .find(|block| block.name == target.block_name)
            .ok_or_else(|| {
                BlockContractError::new_err(format!(
                    "link: unknown target block '{}'",
                    target.block_name
                ))
            })?;

        let source_schema = source_block
            .output_fields
            .bind(py)
            .get_item(&source.key)?
            .ok_or_else(|| {
                BlockContractError::new_err(format!(
                    "link: unknown source port '{}.{}'",
                    source.block_name, source.key
                ))
            })?;
        let target_schema = target_block
            .input_fields
            .bind(py)
            .get_item(&target.key)?
            .ok_or_else(|| {
                BlockContractError::new_err(format!(
                    "link: unknown target port '{}.{}'",
                    target.block_name, target.key
                ))
            })?;
        if !source_schema.eq(&target_schema)? {
            return Err(BlockContractError::new_err(format!(
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

#[pyo3_macros::pyfunction]
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
        return Err(BlockContractError::new_err(
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

struct CallableSignature {
    signature: PyObject,
    empty: PyObject,
    var_positional: PyObject,
    var_keyword: PyObject,
    keyword_only: PyObject,
    params: Vec<PyObject>,
}

fn inspect_callable_signature(
    py: Python<'_>,
    callable: &Bound<'_, PyAny>,
) -> PyResult<CallableSignature> {
    let inspect = PyModule::import(py, "inspect")?;
    let signature = inspect.getattr("signature")?.call1((callable,))?;
    let empty = inspect.getattr("_empty")?;
    let parameter = inspect.getattr("Parameter")?;
    let var_positional = parameter.getattr("VAR_POSITIONAL")?;
    let var_keyword = parameter.getattr("VAR_KEYWORD")?;
    let keyword_only = parameter.getattr("KEYWORD_ONLY")?;

    let mut params = Vec::new();
    let parameter_values = signature.getattr("parameters")?.call_method0("values")?;
    for param in parameter_values.try_iter()? {
        params.push(param?.unbind());
    }

    Ok(CallableSignature {
        signature: signature.unbind(),
        empty: empty.unbind(),
        var_positional: var_positional.unbind(),
        var_keyword: var_keyword.unbind(),
        keyword_only: keyword_only.unbind(),
        params,
    })
}
fn typed_block_meta_from_function(
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    name_override: Option<&str>,
) -> PyResult<TypedBlockMeta> {
    if !func.is_callable() {
        return Err(BlockContractError::new_err("block: expected a callable"));
    }
    let inspected = inspect_callable_signature(py, func)?;
    let params = &inspected.params;
    if params.len() != 2 && params.len() != 3 {
        return Err(BlockContractError::new_err(
            "block: expected signature (model, data) or (model, data, ctx)",
        ));
    }
    for param in params {
        let kind = param.bind(py).getattr("kind")?;
        if kind.eq(inspected.var_positional.bind(py))? || kind.eq(inspected.var_keyword.bind(py))? {
            return Err(BlockContractError::new_err(
                "block: variadic *args/**kwargs are not supported",
            ));
        }
    }
    for param in params {
        let kind = param.bind(py).getattr("kind")?;
        if kind.eq(inspected.keyword_only.bind(py))? {
            return Err(BlockContractError::new_err(
                "block: keyword-only parameters are not supported",
            ));
        }
    }

    let input_schema = params[1].bind(py).getattr("annotation")?;
    if input_schema.is(inspected.empty.bind(py)) {
        return Err(BlockContractError::new_err(
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
        return Err(BlockContractError::new_err(format!(
            "add_block: extract for block '{block_name}' must be callable"
        )));
    }
    let inspected = inspect_callable_signature(py, extract)?;
    let params = &inspected.params;
    if params.len() != 2 && params.len() != 3 {
        return Err(BlockContractError::new_err(format!(
            "add_block: extract for block '{block_name}' must use (solution, data) or (solution, data, ctx)"
        )));
    }
    for param in params {
        let kind = param.bind(py).getattr("kind")?;
        if kind.eq(inspected.var_positional.bind(py))?
            || kind.eq(inspected.var_keyword.bind(py))?
            || kind.eq(inspected.keyword_only.bind(py))?
        {
            return Err(BlockContractError::new_err(format!(
                "add_block: extract for block '{block_name}' cannot use variadic or keyword-only parameters"
            )));
        }
    }

    let input_annotation = params[1].bind(py).getattr("annotation")?;
    if input_annotation.is(inspected.empty.bind(py)) {
        return Err(BlockContractError::new_err(format!(
            "add_block: extract for block '{block_name}' must annotate the data parameter"
        )));
    }
    if !input_annotation.eq(expected_input_schema)? {
        return Err(BlockContractError::new_err(format!(
            "add_block: extract data annotation must match block input schema for '{block_name}'"
        )));
    }

    let output_schema = inspected.signature.bind(py).getattr("return_annotation")?;
    if output_schema.is(inspected.empty.bind(py)) {
        return Err(BlockContractError::new_err(format!(
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
    Err(BlockContractError::new_err(format!(
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
    Err(BlockContractError::new_err(
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
        return Err(BlockContractError::new_err(format!(
            "{context} must be a dict or {} instance",
            schema.get_type().name()?
        )));
    }
    Err(BlockContractError::new_err(format!(
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
            return Err(BlockContractError::new_err(format!(
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
    Err(BlockContractError::new_err(format!(
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
