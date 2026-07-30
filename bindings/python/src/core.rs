// Python bindings for Arco optimization using PyO3.
//
// This module exposes Arco's model builder and solver to Python with zero-copy access
// to solution data through memoryview.

mod py_modules;

pub use crate::py_modules::solution::PySolveResult;

use crate::py_modules as pym;
use crate::py_modules::serde_bridge;
use arco_model::expr::ComparisonSense;
use arco_model::{
    Bounds, ConstraintId, InspectOptions, Model, Objective, PrettyPrintOptions, Sense, SlackBound,
    Variable, VariableId,
};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};

macro_rules! wrap_pyfunction {
    ($function:path, $py_or_module:expr) => {{
        use $function as wrapped_pyfunction;
        pyo3::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
            $py_or_module,
            &wrapped_pyfunction::_PYO3_DEF,
        )
    }};
}

pub(crate) type PyObject = Py<PyAny>;

#[pyo3_macros::pyfunction(name = "_diagnostic_codes")]
fn diagnostic_codes(py: Python<'_>) -> PyResult<PyObject> {
    let codes = PyDict::new(py);
    codes.set_item(
        "ALGEBRA_PARSE_ERROR",
        arco_diagnostics::codes::ALGEBRA_PARSE_ERROR,
    )?;
    codes.set_item("ARRAY_DIMENSION", arco_diagnostics::codes::ARRAY_DIMENSION)?;
    codes.set_item("ARRAY_INDEX", arco_diagnostics::codes::ARRAY_INDEX)?;
    codes.set_item("ARRAY_OVERFLOW", arco_diagnostics::codes::ARRAY_OVERFLOW)?;
    codes.set_item(
        "ARRAY_SHAPE_MISMATCH",
        arco_diagnostics::codes::ARRAY_SHAPE_MISMATCH,
    )?;
    codes.set_item("ARRAY_TYPE", arco_diagnostics::codes::ARRAY_TYPE)?;
    codes.set_item(
        "BLOCK_ARTIFACT_IO",
        arco_diagnostics::codes::BLOCK_ARTIFACT_IO,
    )?;
    codes.set_item("BLOCK_CONTRACT", arco_diagnostics::codes::BLOCK_CONTRACT)?;
    codes.set_item("BLOCK_RESULT", arco_diagnostics::codes::BLOCK_RESULT)?;
    codes.set_item("BOUNDS_INVALID", arco_diagnostics::codes::BOUNDS_INVALID)?;
    codes.set_item("CONFIG_IO", arco_diagnostics::codes::CONFIG_IO)?;
    codes.set_item(
        "CONFIG_MISSING_DIRECTORY",
        arco_diagnostics::codes::CONFIG_MISSING_DIRECTORY,
    )?;
    codes.set_item(
        "CONFIG_MISSING_PROJECT_DIRECTORY",
        arco_diagnostics::codes::CONFIG_MISSING_PROJECT_DIRECTORY,
    )?;
    codes.set_item(
        "CONFIG_SECRET_REFERENCE_REQUIRED",
        arco_diagnostics::codes::CONFIG_SECRET_REFERENCE_REQUIRED,
    )?;
    codes.set_item(
        "CONFIG_SELECTION",
        arco_diagnostics::codes::CONFIG_SELECTION,
    )?;
    codes.set_item("CONFIG_TOML", arco_diagnostics::codes::CONFIG_TOML)?;
    codes.set_item("COMPILE_CSV", arco_diagnostics::codes::COMPILE_CSV)?;
    codes.set_item(
        "COMPILE_EMPTY_TUPLE_REDUCTION",
        arco_diagnostics::codes::COMPILE_EMPTY_TUPLE_REDUCTION,
    )?;
    codes.set_item(
        "COMPILE_INVALID_CONSTRAINT_FILTER",
        arco_diagnostics::codes::COMPILE_INVALID_CONSTRAINT_FILTER,
    )?;
    codes.set_item(
        "COMPILE_INVALID_FORMULATION",
        arco_diagnostics::codes::COMPILE_INVALID_FORMULATION,
    )?;
    codes.set_item(
        "COMPILE_INVALID_NUMBER",
        arco_diagnostics::codes::COMPILE_INVALID_NUMBER,
    )?;
    codes.set_item(
        "COMPILE_MISSING_ASSET",
        arco_diagnostics::codes::COMPILE_MISSING_ASSET,
    )?;
    codes.set_item(
        "COMPILE_MISSING_COLUMN",
        arco_diagnostics::codes::COMPILE_MISSING_COLUMN,
    )?;
    codes.set_item(
        "COMPILE_MISSING_DATA",
        arco_diagnostics::codes::COMPILE_MISSING_DATA,
    )?;
    codes.set_item(
        "COMPILE_MISSING_DATA_POINT",
        arco_diagnostics::codes::COMPILE_MISSING_DATA_POINT,
    )?;
    codes.set_item(
        "COMPILE_MISSING_DECLARATION",
        arco_diagnostics::codes::COMPILE_MISSING_DECLARATION,
    )?;
    codes.set_item(
        "COMPILE_MISSING_PARAMETER",
        arco_diagnostics::codes::COMPILE_MISSING_PARAMETER,
    )?;
    codes.set_item(
        "COMPILE_MISSING_SCENARIO",
        arco_diagnostics::codes::COMPILE_MISSING_SCENARIO,
    )?;
    codes.set_item(
        "CONSTRAINT_BOUNDS_MISSING",
        arco_diagnostics::codes::CONSTRAINT_BOUNDS_MISSING,
    )?;
    codes.set_item(
        "CONSTRAINT_INVALID_BOUNDS",
        arco_diagnostics::codes::CONSTRAINT_INVALID_BOUNDS,
    )?;
    codes.set_item(
        "CONSTRAINT_INVALID_ID",
        arco_diagnostics::codes::CONSTRAINT_INVALID_ID,
    )?;
    codes.set_item(
        "CONSTRAINT_NOT_FOUND",
        arco_diagnostics::codes::CONSTRAINT_NOT_FOUND,
    )?;
    codes.set_item(
        "CONSTRAINT_SENSE",
        arco_diagnostics::codes::CONSTRAINT_SENSE,
    )?;
    codes.set_item("CONSTRAINT_TYPE", arco_diagnostics::codes::CONSTRAINT_TYPE)?;
    codes.set_item("CSC_CONTIGUITY", arco_diagnostics::codes::CSC_CONTIGUITY)?;
    codes.set_item("CSC_DIMENSION", arco_diagnostics::codes::CSC_DIMENSION)?;
    codes.set_item("CSC_DTYPE", arco_diagnostics::codes::CSC_DTYPE)?;
    codes.set_item(
        "CSC_INVALID_DATA",
        arco_diagnostics::codes::CSC_INVALID_DATA,
    )?;
    codes.set_item(
        "CSC_NEGATIVE_INDEX",
        arco_diagnostics::codes::CSC_NEGATIVE_INDEX,
    )?;
    codes.set_item(
        "DEPENDENCY_MISSING",
        arco_diagnostics::codes::DEPENDENCY_MISSING,
    )?;
    codes.set_item(
        "DRIVER_BACKEND_NOT_AVAILABLE",
        arco_diagnostics::codes::DRIVER_BACKEND_NOT_AVAILABLE,
    )?;
    codes.set_item(
        "DRIVER_INSPECT_FORMAT",
        arco_diagnostics::codes::DRIVER_INSPECT_FORMAT,
    )?;
    codes.set_item("DRIVER_JSON", arco_diagnostics::codes::DRIVER_JSON)?;
    codes.set_item(
        "EXPR_COEFFICIENT",
        arco_diagnostics::codes::EXPR_COEFFICIENT,
    )?;
    codes.set_item(
        "EXPR_CONSTANT_OFFSET",
        arco_diagnostics::codes::EXPR_CONSTANT_OFFSET,
    )?;
    codes.set_item(
        "EXPR_DIVISION_BY_ZERO",
        arco_diagnostics::codes::EXPR_DIVISION_BY_ZERO,
    )?;
    codes.set_item(
        "EXPR_NOT_SINGLE_VARIABLE",
        arco_diagnostics::codes::EXPR_NOT_SINGLE_VARIABLE,
    )?;
    codes.set_item("EXPR_TYPE", arco_diagnostics::codes::EXPR_TYPE)?;
    codes.set_item(
        "INDEX_SET_ARGUMENT",
        arco_diagnostics::codes::INDEX_SET_ARGUMENT,
    )?;
    codes.set_item("INDEX_SET_EMPTY", arco_diagnostics::codes::INDEX_SET_EMPTY)?;
    codes.set_item("INDEX_SET_INDEX", arco_diagnostics::codes::INDEX_SET_INDEX)?;
    codes.set_item("INDEX_SET_TYPE", arco_diagnostics::codes::INDEX_SET_TYPE)?;
    codes.set_item("LOGGING_CONFIG", arco_diagnostics::codes::LOGGING_CONFIG)?;
    codes.set_item("LOGGING_IO", arco_diagnostics::codes::LOGGING_IO)?;
    codes.set_item(
        "METADATA_CONVERSION",
        arco_diagnostics::codes::METADATA_CONVERSION,
    )?;
    codes.set_item(
        "MODEL_BINARY_BOUNDS",
        arco_diagnostics::codes::MODEL_BINARY_BOUNDS,
    )?;
    codes.set_item("MODEL_EMPTY", arco_diagnostics::codes::MODEL_EMPTY)?;
    codes.set_item(
        "OBJECTIVE_ALREADY_SET",
        arco_diagnostics::codes::OBJECTIVE_ALREADY_SET,
    )?;
    codes.set_item("OBJECTIVE_INDEX", arco_diagnostics::codes::OBJECTIVE_INDEX)?;
    codes.set_item(
        "OBJECTIVE_MISSING",
        arco_diagnostics::codes::OBJECTIVE_MISSING,
    )?;
    codes.set_item("SEMANTIC_CSV", arco_diagnostics::codes::SEMANTIC_CSV)?;
    codes.set_item(
        "SEMANTIC_AMBIGUOUS_TUPLE_SUBSET_INDEX",
        arco_diagnostics::codes::SEMANTIC_AMBIGUOUS_TUPLE_SUBSET_INDEX,
    )?;
    codes.set_item(
        "SEMANTIC_DUPLICATE_DATA_BINDING",
        arco_diagnostics::codes::SEMANTIC_DUPLICATE_DATA_BINDING,
    )?;
    codes.set_item(
        "SEMANTIC_DUPLICATE_DECLARATION",
        arco_diagnostics::codes::SEMANTIC_DUPLICATE_DECLARATION,
    )?;
    codes.set_item(
        "SEMANTIC_DUPLICATE_MODEL_DECLARATION",
        arco_diagnostics::codes::SEMANTIC_DUPLICATE_MODEL_DECLARATION,
    )?;
    codes.set_item(
        "SEMANTIC_DUPLICATE_TUPLE_ROWS",
        arco_diagnostics::codes::SEMANTIC_DUPLICATE_TUPLE_ROWS,
    )?;
    codes.set_item(
        "SEMANTIC_EXPRESSION_CYCLE",
        arco_diagnostics::codes::SEMANTIC_EXPRESSION_CYCLE,
    )?;
    codes.set_item(
        "SEMANTIC_MISSING_CELL",
        arco_diagnostics::codes::SEMANTIC_MISSING_CELL,
    )?;
    codes.set_item(
        "SEMANTIC_MISSING_COLUMN",
        arco_diagnostics::codes::SEMANTIC_MISSING_COLUMN,
    )?;
    codes.set_item(
        "SEMANTIC_MISSING_DECLARATION",
        arco_diagnostics::codes::SEMANTIC_MISSING_DECLARATION,
    )?;
    codes.set_item(
        "SEMANTIC_MISSING_INITIAL_BOUNDARY",
        arco_diagnostics::codes::SEMANTIC_MISSING_INITIAL_BOUNDARY,
    )?;
    codes.set_item(
        "SEMANTIC_MISSING_MODEL",
        arco_diagnostics::codes::SEMANTIC_MISSING_MODEL,
    )?;
    codes.set_item(
        "SEMANTIC_MISSING_MODEL_USE",
        arco_diagnostics::codes::SEMANTIC_MISSING_MODEL_USE,
    )?;
    codes.set_item(
        "SEMANTIC_MISSING_SCENARIO",
        arco_diagnostics::codes::SEMANTIC_MISSING_SCENARIO,
    )?;
    codes.set_item(
        "SEMANTIC_SCENARIO_COUNT",
        arco_diagnostics::codes::SEMANTIC_SCENARIO_COUNT,
    )?;
    codes.set_item(
        "SEMANTIC_TUPLE_SET_SCHEMA_MISMATCH",
        arco_diagnostics::codes::SEMANTIC_TUPLE_SET_SCHEMA_MISMATCH,
    )?;
    codes.set_item(
        "SEMANTIC_TUPLE_SUBSET_DOMAIN_MISMATCH",
        arco_diagnostics::codes::SEMANTIC_TUPLE_SUBSET_DOMAIN_MISMATCH,
    )?;
    codes.set_item(
        "SEMANTIC_UNKNOWN_SCENARIO_DATA_BINDING",
        arco_diagnostics::codes::SEMANTIC_UNKNOWN_SCENARIO_DATA_BINDING,
    )?;
    codes.set_item(
        "SEMANTIC_UNRESOLVED_FILTER_IDENTIFIER",
        arco_diagnostics::codes::SEMANTIC_UNRESOLVED_FILTER_IDENTIFIER,
    )?;
    codes.set_item(
        "SEMANTIC_UNRESOLVED_RULE_SET_FILTER_IDENTIFIER",
        arco_diagnostics::codes::SEMANTIC_UNRESOLVED_RULE_SET_FILTER_IDENTIFIER,
    )?;
    codes.set_item("SLACK_BOUND", arco_diagnostics::codes::SLACK_BOUND)?;
    codes.set_item(
        "SLACK_INVALID_PENALTY",
        arco_diagnostics::codes::SLACK_INVALID_PENALTY,
    )?;
    codes.set_item(
        "SLACK_VALUE_UNAVAILABLE",
        arco_diagnostics::codes::SLACK_VALUE_UNAVAILABLE,
    )?;
    codes.set_item(
        "SOLVER_INFEASIBLE",
        arco_diagnostics::codes::SOLVER_INFEASIBLE,
    )?;
    codes.set_item("SOLVER_INDEX", arco_diagnostics::codes::SOLVER_INDEX)?;
    codes.set_item("SOLVER_INTERNAL", arco_diagnostics::codes::SOLVER_INTERNAL)?;
    codes.set_item(
        "SOLVER_ITERATION_LIMIT",
        arco_diagnostics::codes::SOLVER_ITERATION_LIMIT,
    )?;
    codes.set_item(
        "SOLVER_INVALID_SETTING",
        arco_diagnostics::codes::SOLVER_INVALID_SETTING,
    )?;
    codes.set_item(
        "SOLVER_MODEL_SIZE_LIMIT",
        arco_diagnostics::codes::SOLVER_MODEL_SIZE_LIMIT,
    )?;
    codes.set_item(
        "SOLVER_NOT_AVAILABLE",
        arco_diagnostics::codes::SOLVER_NOT_AVAILABLE,
    )?;
    codes.set_item(
        "SOLVER_TIME_LIMIT",
        arco_diagnostics::codes::SOLVER_TIME_LIMIT,
    )?;
    codes.set_item("SOLVER_TYPE", arco_diagnostics::codes::SOLVER_TYPE)?;
    codes.set_item(
        "SOLVER_UNBOUNDED",
        arco_diagnostics::codes::SOLVER_UNBOUNDED,
    )?;
    codes.set_item(
        "SOURCE_INVALID_ALGEBRA",
        arco_diagnostics::codes::SOURCE_INVALID_ALGEBRA,
    )?;
    codes.set_item(
        "SOURCE_INVALID_INCLUDE",
        arco_diagnostics::codes::SOURCE_INVALID_INCLUDE,
    )?;
    codes.set_item(
        "SOURCE_INVALID_VALUE",
        arco_diagnostics::codes::SOURCE_INVALID_VALUE,
    )?;
    codes.set_item("SOURCE_IO", arco_diagnostics::codes::SOURCE_IO)?;
    codes.set_item("SOURCE_KDL", arco_diagnostics::codes::SOURCE_KDL)?;
    codes.set_item(
        "SOURCE_MISSING_ARGUMENT",
        arco_diagnostics::codes::SOURCE_MISSING_ARGUMENT,
    )?;
    codes.set_item(
        "SOURCE_MISSING_NODE",
        arco_diagnostics::codes::SOURCE_MISSING_NODE,
    )?;
    codes.set_item(
        "SOURCE_MISSING_PROPERTY",
        arco_diagnostics::codes::SOURCE_MISSING_PROPERTY,
    )?;
    codes.set_item(
        "SOURCE_UNSUPPORTED_DECLARATION",
        arco_diagnostics::codes::SOURCE_UNSUPPORTED_DECLARATION,
    )?;
    codes.set_item(
        "VARIABLE_INVALID_ID",
        arco_diagnostics::codes::VARIABLE_INVALID_ID,
    )?;
    codes.set_item(
        "VARIABLE_INVALID_BOUNDS",
        arco_diagnostics::codes::VARIABLE_INVALID_BOUNDS,
    )?;
    codes.set_item(
        "VARIABLE_NOT_FOUND",
        arco_diagnostics::codes::VARIABLE_NOT_FOUND,
    )?;
    codes.set_item(
        "TARGET_EMPTY_VARIABLE_SET",
        arco_diagnostics::codes::TARGET_EMPTY_VARIABLE_SET,
    )?;
    Ok(codes.into_any().unbind())
}

#[pyo3_macros::pyclass(name = "BlockPort", from_py_object)]
#[derive(Clone)]
pub struct BlockPort {
    #[pyo3(get)]
    pub(crate) block_name: String,
    #[pyo3(get)]
    pub(crate) key: String,
    #[pyo3(get)]
    pub(crate) kind: String,
}

impl BlockPort {
    pub(crate) fn new_input(block_name: String, key: String) -> Self {
        Self {
            block_name,
            key,
            kind: "input".to_string(),
        }
    }

    pub(crate) fn new_output(block_name: String, key: String) -> Self {
        Self {
            block_name,
            key,
            kind: "output".to_string(),
        }
    }
}

fn sparse_export_dict<F>(py: Python<'_>, shape: (usize, usize), fill: F) -> PyResult<PyObject>
where
    F: FnOnce(&Bound<'_, PyDict>) -> PyResult<()>,
{
    let dict = PyDict::new(py);
    fill(&dict)?;
    dict.set_item("shape", (shape.0 as u32, shape.1 as u32))?;
    Ok(dict.unbind().into())
}

mod py_exports;
use py_exports::{
    BoundsSpec, PyBlockHandle, PyBlockPorts, PyBlockResults, PyBounds, PyComparisonSense,
    PyConstraint, PyConstraintArray, PyConstraintExpr, PyElasticHandle, PyExpr, PyExprArray,
    PyIndexSet, PyLpAlgorithm, PyModelSnapshot, PySense, PySimplifyLevel, PySlackVariable,
    PyVariable,
    PyVariableArray, SolverSettings,
};

/// Python wrapper for the Arco optimization model
#[pyo3_macros::pyclass(name = "Model")]
pub struct PyModel {
    inner: Model,
    solver_settings: SolverSettings,
    default_backend: String,
    last_solution: Option<Py<PySolveResult>>,
    /// Block definitions added via add_block()
    block_defs: Vec<pym::model_blocks::BlockDef>,
    /// Links between blocks
    link_defs: Vec<pym::model_blocks::LinkDef>,
    /// Compact metadata for arrays created via add_variables() for pretty-printing.
    array_print_specs: Vec<pym::model_pretty::ArrayPrintSpec>,
    /// Compact metadata for named constraint blocks.
    constraint_print_specs: Vec<pym::model_pretty::ConstraintPrintSpec>,
    /// Nonlinear constraints and objective registered via `add_nonlinear_constraint`
    /// and `minimize`/`maximize` with a `NonlinearExpr`. Only meaningful when the
    /// `ipopt` feature is enabled.
    #[cfg(feature = "ipopt")]
    pub(crate) nonlinear_state: pym::nonlinear_state::NonlinearState,
}

impl PyModel {
    fn from_parts(
        inner: Model,
        solver_settings: SolverSettings,
        default_backend: String,
    ) -> Self {
        Self {
            inner,
            solver_settings,
            default_backend,
            last_solution: None,
            block_defs: Vec::new(),
            link_defs: Vec::new(),
            array_print_specs: Vec::new(),
            constraint_print_specs: Vec::new(),
            #[cfg(feature = "ipopt")]
            nonlinear_state: pym::nonlinear_state::NonlinearState::default(),
        }
    }
}

#[pyo3_macros::pymethods]
impl PyModel {
    /// Create a new model
    #[new]
    #[pyo3(signature = (*, simplify_level=None, solver=None))]
    fn new(
        simplify_level: Option<PySimplifyLevel>,
        solver: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        pym::model_init::new_model(simplify_level, solver)
    }

    /// Build a model directly from CSC data.
    #[classmethod]
    #[pyo3(
        signature = (*, num_constraints, num_variables, col_ptrs, row_indices, values, var_lower, var_upper, con_lower, con_upper, is_integer, simplify_level=None)
    )]
    #[allow(clippy::too_many_arguments)]
    fn from_csc(
        _cls: &Bound<'_, PyType>,
        num_constraints: usize,
        num_variables: usize,
        col_ptrs: &Bound<'_, PyAny>,
        row_indices: &Bound<'_, PyAny>,
        values: &Bound<'_, PyAny>,
        var_lower: &Bound<'_, PyAny>,
        var_upper: &Bound<'_, PyAny>,
        con_lower: &Bound<'_, PyAny>,
        con_upper: &Bound<'_, PyAny>,
        is_integer: &Bound<'_, PyAny>,
        simplify_level: Option<PySimplifyLevel>,
    ) -> PyResult<Self> {
        pym::model_init::from_csc_model(
            num_constraints,
            num_variables,
            col_ptrs,
            row_indices,
            values,
            var_lower,
            var_upper,
            con_lower,
            con_upper,
            is_integer,
            simplify_level,
        )
    }

    /// Pre-allocate storage for additional variables and constraints.
    #[pyo3(signature = (*, num_variables=0, num_constraints=0))]
    fn reserve(&mut self, num_variables: usize, num_constraints: usize) {
        if num_variables > 0 {
            self.inner.reserve_variables(num_variables);
        }
        if num_constraints > 0 {
            self.inner.reserve_constraints(num_constraints);
        }
    }

    /// Add a variable to the model.
    ///
    /// # Arguments
    /// * `bounds` - Bounds or bound constant (e.g. NonNegativeFloat, Binary)
    /// * `is_integer` - Whether the variable is integer-constrained
    /// * `is_binary` - Whether the variable is binary
    /// * `name` - Optional name for the variable
    /// * `metadata` - Optional JSON-compatible metadata attached to the variable
    ///
    /// # Returns
    /// A Variable object
    #[pyo3(signature = (*, bounds, is_integer=false, is_binary=false, name=None, metadata=None))]
    fn add_variable(
        &mut self,
        bounds: BoundsSpec,
        is_integer: bool,
        is_binary: bool,
        name: Option<String>,
        metadata: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyVariable> {
        let metadata = metadata.map(serde_bridge::py_to_json).transpose()?;
        let effective_bounds = Self::effective_bounds(&bounds, is_integer, is_binary)?;

        let var = Variable {
            bounds: bounds.bounds,
            is_integer: effective_bounds.is_integer,
            is_active: true,
        };

        let var_id = self
            .inner
            .add_variable(var)
            .map_err(pym::errors::model_error_to_py)?;

        if let Some(ref n) = name {
            self.inner
                .set_variable_name(var_id, n.clone())
                .map_err(pym::errors::model_error_to_py)?;
        }

        if let Some(metadata) = metadata {
            self.inner
                .set_variable_metadata(var_id, metadata)
                .map_err(pym::errors::model_error_to_py)?;
        }

        Ok(PyVariable::new(var_id.inner(), name, effective_bounds))
    }

    /// Add a vector or grid of variables to the model.
    #[pyo3(signature = (*, axes, bounds, is_integer=false, is_binary=false, active=None, name=None))]
    #[allow(clippy::too_many_arguments)]
    fn add_variables(
        &mut self,
        py: Python<'_>,
        axes: &Bound<'_, PyAny>,
        bounds: &Bound<'_, PyAny>,
        is_integer: bool,
        is_binary: bool,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let index_sets = extract_index_sets(axes)?;

        if index_sets.is_empty() {
            return Err(pym::errors::IndexSetEmptyError::new_err(
                "axes must be non-empty",
            ));
        }

        let labeled_shape = pym::arrays::labeled_shape_from_index_sets(&index_sets)?;
        let shape = labeled_shape.shape();
        if shape.iter().any(|size| *size == 0) {
            return Err(pym::errors::IndexSetEmptyError::new_err(
                "index sets must be non-empty",
            ));
        }

        let total = shape.iter().try_fold(1usize, |acc, &size| {
            acc.checked_mul(size)
                .ok_or_else(|| pym::errors::ArrayOverflowError::new_err("array size overflow"))
        })?;

        // Try scalar bounds first (BoundsSpec), then per-element array bounds
        if let Ok(scalar_bounds) = bounds.extract::<BoundsSpec>() {
            return self.add_variables_scalar_bounds(
                py,
                index_sets,
                &shape,
                total,
                scalar_bounds,
                is_integer,
                is_binary,
                active,
                name,
            );
        }

        // Try per-element array bounds: Bounds object with numpy array lo/hi
        self.add_variables_array_bounds(
            py, index_sets, &shape, total, bounds, is_integer, is_binary, active, name,
        )
    }

    /// Deactivate a variable without removing its column.
    #[pyo3(signature = (*, var_id))]
    fn deactivate_variable(&mut self, var_id: u32) -> PyResult<()> {
        self.inner
            .deactivate_variable(VariableId::new(var_id))
            .map_err(pym::errors::model_error_to_py)
    }

    /// Activate a previously deactivated variable.
    #[pyo3(signature = (*, var_id))]
    fn activate_variable(&mut self, var_id: u32) -> PyResult<()> {
        self.inner
            .activate_variable(VariableId::new(var_id))
            .map_err(pym::errors::model_error_to_py)
    }

    /// Check whether a variable is active.
    #[pyo3(signature = (*, var_id))]
    fn is_variable_active(&self, var_id: u32) -> PyResult<bool> {
        self.inner
            .is_variable_active(VariableId::new(var_id))
            .map_err(pym::errors::model_error_to_py)
    }

    /// Add a constraint to the model.
    #[pyo3(signature = (expr, *, bounds=None, name=None))]
    fn add_constraint(
        &mut self,
        expr: &Bound<'_, PyAny>,
        bounds: Option<PyBounds>,
        name: Option<String>,
    ) -> PyResult<PyConstraint> {
        let (expr, constraint_bounds) =
            if let Ok(constraint_expr) = expr.extract::<PyConstraintExpr>() {
                let inner = constraint_expr.inner().clone();
                let expr = inner.expr().clone();
                let bounds = bounds.map_or_else(
                    || bounds_from_sense(inner.sense(), inner.rhs()),
                    |value| value.inner,
                );
                (expr, bounds)
            } else if let Ok(linear_expr) = expr
                .extract::<PyRef<'_, PyVariable>>()
                .map(|v| v.to_expr())
                .or_else(|_| expr.extract::<PyExpr>())
            {
                let bounds = bounds.ok_or_else(|| {
                    pym::errors::ConstraintBoundsMissingError::new_err(
                        "bounds are required when expression has no comparison",
                    )
                })?;
                let (expr, offset) = linear_expr.into_parts();
                let bounds = Bounds::new(bounds.inner.lower - offset, bounds.inner.upper - offset);
                (expr, bounds)
            } else {
                return Err(pym::errors::ConstraintTypeError::new_err(
                    "expected an Expr, Variable, or ConstraintExpr",
                ));
            };

        let constraint_id = self
            .inner
            .add_expr_constraint(expr, constraint_bounds)
            .map_err(pym::errors::model_error_to_py)?;
        self.set_constraint_name_if_provided(constraint_id, name.clone())?;
        Ok(PyConstraint::new(
            constraint_id.inner(),
            name,
            constraint_bounds,
        ))
    }

    /// Add a batch of constraints to the model.
    ///
    /// Returns a `ConstraintArray` representing the added constraints.
    /// Uses compact insertion when possible (zero per-element allocation),
    /// falling back to a batch path for materialized expressions.
    #[pyo3(signature = (expr, *, sense=PyComparisonSense::GreaterEqual, rhs=None, active=None, name=None))]
    #[allow(clippy::too_many_arguments)]
    fn add_constraints(
        &mut self,
        expr: &Bound<'_, PyAny>,
        sense: PyComparisonSense,
        rhs: Option<&Bound<'_, PyAny>>,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        // Branch 1: ConstraintArray input
        if let Ok(array) = expr.extract::<PyRef<'_, PyConstraintArray>>() {
            if rhs.is_some() || sense != PyComparisonSense::GreaterEqual {
                return Err(pym::errors::ConstraintSenseError::new_err(
                    "sense/rhs are not supported for comparison arrays",
                ));
            }

            // Fast path: compact constraint storage
            if let Some(compact) = array.as_compact() {
                return self.add_constraints_compact_shaped_internal(
                    compact,
                    active,
                    name,
                    array.shape_ref(),
                    &array.clone_index_sets(),
                );
            }
            if let Some((left, right, lazy_sense)) = array.as_lazy_compare() {
                return self.add_constraints_lazy_compare_shaped_internal(
                    left,
                    right,
                    lazy_sense,
                    active,
                    name,
                    array.shape_ref(),
                    &array.clone_index_sets(),
                );
            }
            if let Some((exprs, sparse_rhs, row_indices, sparse_sense)) = array.as_sparse_rows() {
                return self.add_constraints_sparse_rows_internal(
                    exprs,
                    sparse_sense,
                    sparse_rhs,
                    row_indices,
                    active,
                    name,
                    array.shape_ref(),
                    &array.clone_index_sets(),
                );
            }

            // Full path
            return self.add_constraints_shaped_internal(
                array.exprs().to_vec(),
                array.get_sense(),
                array.get_rhs(),
                active,
                name,
                array.shape_ref(),
                &array.clone_index_sets(),
            );
        }

        // Branch 2: VariableArray or ExprArray input
        let sense: ComparisonSense = sense.into();
        let rhs_obj = rhs.ok_or_else(|| {
            pym::errors::ConstraintBoundsMissingError::new_err(
                "rhs is required for add_constraints",
            )
        })?;

        if let Ok(array) = expr.extract::<PyRef<'_, PyVariableArray>>() {
            let compact = array.as_compact_expr();
            return self.add_constraints_from_array(
                compact,
                || array.to_core(),
                rhs_obj,
                sense,
                active,
                name,
            );
        }

        if let Ok(array) = expr.extract::<PyRef<'_, PyExprArray>>() {
            let compact = array.as_compact().cloned();
            return self.add_constraints_from_array(
                compact,
                || array.to_core(),
                rhs_obj,
                sense,
                active,
                name,
            );
        }

        Err(pym::errors::ConstraintTypeError::new_err(
            "expected ConstraintArray, VariableArray, or ExprArray",
        ))
    }

    /// Attach slack variables to a constraint bound, returning a SlackVariable.
    #[pyo3(signature = (constraint, *, bound, penalty, name=None))]
    fn add_slack(
        slf: &Bound<'_, Self>,
        constraint: &Bound<'_, PyAny>,
        bound: String,
        penalty: f64,
        name: Option<String>,
    ) -> PyResult<PySlackVariable> {
        let parsed_bound = parse_slack_bound(&bound)?;
        let py_constraint = extract_constraint(constraint)?;
        let constraint_id = {
            let constraint_ref = py_constraint.bind(constraint.py()).borrow();
            ConstraintId::new(constraint_ref.constraint_id)
        };
        let handle = slf
            .borrow_mut()
            .inner
            .add_slack(constraint_id, parsed_bound, penalty, name.clone())
            .map_err(pym::errors::model_error_to_py)?;
        let model_obj: PyObject = slf.clone().unbind().into_any();

        Ok(PySlackVariable::new(
            py_constraint,
            handle.bound.as_str().to_string(),
            handle.penalty,
            handle.name.clone(),
            handle.var_ids,
            model_obj,
        ))
    }

    /// Attach slack variables to multiple constraints, returning a list of SlackVariables.
    #[pyo3(signature = (constraints, *, bound, penalty, name=None))]
    fn add_slacks(
        slf: &Bound<'_, Self>,
        constraints: &Bound<'_, PyAny>,
        bound: String,
        penalty: &Bound<'_, PyAny>,
        name: Option<String>,
    ) -> PyResult<Vec<PySlackVariable>> {
        let parsed_bound = parse_slack_bound(&bound)?;
        let py = constraints.py();
        let model_obj: PyObject = slf.clone().unbind().into_any();

        // Extract constraints as a list
        let constraint_list: Vec<Bound<'_, PyAny>> =
            constraints.try_iter()?.collect::<PyResult<Vec<_>>>()?;

        // Extract penalty: either a single float or a numpy array
        let penalties: Vec<f64> = if let Ok(single) = penalty.extract::<f64>() {
            vec![single; constraint_list.len()]
        } else {
            // Try as numpy array or iterable of floats
            let arr: Vec<f64> = penalty.extract()?;
            if arr.len() != constraint_list.len() {
                return Err(pym::errors::SlackInvalidPenaltyError::new_err(format!(
                    "penalty array length {} does not match constraints length {}",
                    arr.len(),
                    constraint_list.len()
                )));
            }
            arr
        };

        let mut results = Vec::with_capacity(constraint_list.len());
        for (con_any, pen) in constraint_list.iter().zip(penalties.iter()) {
            let py_constraint = extract_constraint(con_any)?;
            let constraint_id = {
                let constraint_ref = py_constraint.bind(py).borrow();
                ConstraintId::new(constraint_ref.constraint_id)
            };
            let handle = slf
                .borrow_mut()
                .inner
                .add_slack(constraint_id, parsed_bound, *pen, name.clone())
                .map_err(pym::errors::model_error_to_py)?;

            results.push(PySlackVariable::new(
                py_constraint,
                handle.bound.as_str().to_string(),
                handle.penalty,
                handle.name.clone(),
                handle.var_ids,
                model_obj.clone_ref(py),
            ));
        }

        Ok(results)
    }

    /// Attach asymmetric slack penalties to a constraint.
    #[pyo3(signature = (constraint, *, upper_penalty=None, lower_penalty=None, name=None))]
    fn make_elastic(
        &mut self,
        constraint: &Bound<'_, PyAny>,
        upper_penalty: Option<f64>,
        lower_penalty: Option<f64>,
        name: Option<String>,
    ) -> PyResult<PyElasticHandle> {
        let constraint_id = extract_constraint_id(constraint)?;
        let handle = self
            .inner
            .make_elastic(constraint_id, upper_penalty, lower_penalty, name)
            .map_err(pym::errors::model_error_to_py)?;
        Ok(PyElasticHandle::from_handle(handle))
    }

    /// Expert: set a coefficient by solver-order matrix indices.
    ///
    /// # Arguments
    /// * `var_idx` - Index of the variable (column)
    /// * `constraint_idx` - Index of the constraint (row)
    /// * `coeff` - The coefficient value
    #[pyo3(signature = (*, var_idx, constraint_idx, coeff))]
    fn set_coefficient(&mut self, var_idx: u32, constraint_idx: u32, coeff: f64) -> PyResult<()> {
        let var_id = VariableId::new(var_idx);
        let constraint_id = ConstraintId::new(constraint_idx);

        self.inner
            .set_coefficient(var_id, constraint_id, coeff)
            .map_err(pym::errors::model_error_to_py)
    }

    /// Expert: set a variable name by solver-order variable index.
    #[pyo3(signature = (*, index, name))]
    fn set_variable_name(&mut self, index: u32, name: String) -> PyResult<()> {
        self.inner
            .set_variable_name(VariableId::new(index), name)
            .map_err(pym::errors::model_error_to_py)
    }

    /// Expert: set a constraint name by solver-order constraint index.
    #[pyo3(signature = (*, index, name))]
    fn set_constraint_name(&mut self, index: u32, name: String) -> PyResult<()> {
        self.inner
            .set_constraint_name(ConstraintId::new(index), name)
            .map_err(pym::errors::model_error_to_py)
    }

    /// Expert: set the objective from solver-order variable indices.
    ///
    /// # Arguments
    /// * `sense` - The optimization sense (Minimize or Maximize)
    /// * `terms` - List of (variable_index, coefficient) tuples
    #[pyo3(signature = (*, sense, terms, name=None))]
    fn set_objective(
        &mut self,
        sense: PySense,
        terms: Vec<(u32, f64)>,
        name: Option<String>,
    ) -> PyResult<()> {
        let objective_terms: Vec<(VariableId, f64)> = terms
            .into_iter()
            .map(|(idx, coeff)| (VariableId::new(idx), coeff))
            .collect();

        let objective = Objective {
            sense: Some(sense.into()),
            terms: objective_terms,
        };

        self.inner
            .set_objective(objective)
            .map_err(|error| match error {
                arco_model::ModelError::InvalidVariableId(var_id) => {
                    pym::errors::ObjectiveIndexError::new_err(format!(
                        "objective term references variable index {} outside 0..{}",
                        var_id.inner(),
                        self.inner.num_variables()
                    ))
                }
                other => pym::errors::model_error_to_py(other),
            })?;

        self.inner
            .set_objective_name(name)
            .map_err(pym::errors::model_error_to_py)?;
        Ok(())
    }

    /// Minimize a linear expression.
    #[pyo3(signature = (expr, *, name=None))]
    fn minimize(&mut self, expr: &Bound<'_, PyAny>, name: Option<String>) -> PyResult<()> {
        #[cfg(feature = "ipopt")]
        {
            if pym::model_nlp::try_set_nonlinear_objective(
                self,
                expr,
                Sense::Minimize,
                name.clone(),
            ) {
                return Ok(());
            }
        }
        self.set_objective_from_expr(expr, Sense::Minimize, name)
    }

    /// Maximize a linear expression.
    #[pyo3(signature = (expr, *, name=None))]
    fn maximize(&mut self, expr: &Bound<'_, PyAny>, name: Option<String>) -> PyResult<()> {
        #[cfg(feature = "ipopt")]
        {
            if pym::model_nlp::try_set_nonlinear_objective(
                self,
                expr,
                Sense::Maximize,
                name.clone(),
            ) {
                return Ok(());
            }
        }
        self.set_objective_from_expr(expr, Sense::Maximize, name)
    }

    /// Append linear expression terms to an existing objective.
    #[pyo3(signature = (expr))]
    fn add_objective_terms(&mut self, expr: &Bound<'_, PyAny>) -> PyResult<()> {
        self.add_objective_terms_from_expr(expr)
    }

    /// Add a nonlinear constraint to the model.
    ///
    /// Accepts a `NonlinearConstraintExpr` (the result of comparing a
    /// `NonlinearExpr`) or a linear `ConstraintExpr` (auto-promoted). Only
    /// honored when `solve(solver=arco.Ipopt(...))` is called.
    #[cfg(feature = "ipopt")]
    #[pyo3(signature = (expr, *, name=None))]
    fn add_nonlinear_constraint(
        &mut self,
        expr: &Bound<'_, PyAny>,
        name: Option<String>,
    ) -> PyResult<()> {
        pym::model_nlp::add_nonlinear_constraint(self, expr, name)
    }

    /// Get current expression simplification level.
    fn simplify_level(&self) -> PySimplifyLevel {
        self.inner.simplify_level().into()
    }

    /// Update the expression simplification level.
    #[pyo3(signature = (*, level))]
    fn set_expr_simplify(&mut self, level: PySimplifyLevel) -> PyResult<()> {
        self.inner
            .set_expr_simplify(level.into())
            .map_err(pym::errors::model_error_to_py)
    }

    /// Solve the model and return a solution.
    ///
    /// Set `log_to_console=True` to enable solver logs.
    /// Set `primal_start` to a list of (variable_id, value) tuples for warm-start hints.
    /// Optional solver controls include `time_limit`, `mip_gap`, and `verbosity`.
    /// Pass `solver=arco.Xpress()` to use FICO Xpress instead of the default HiGHS solver.
    #[pyo3(
        signature = (*, solver=None, log_to_console=None, primal_start=None, time_limit=None, mip_gap=None, verbosity=None, lp_algorithm=None)
    )]
    #[allow(clippy::too_many_arguments)]
    fn solve(
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
        if primal_start
            .as_ref()
            .is_some_and(|values| !values.is_empty())
        {
            return Err(pym::errors::SolverInvalidSettingError::new_err(
                "primal_start is not supported on this solve path",
            ));
        }

        // Composed model: delegate to block orchestration
        if !self.block_defs.is_empty() {
            return self.solve_composed(
                py,
                solver,
                log_to_console,
                primal_start,
                time_limit,
                mip_gap,
                verbosity,
                lp_algorithm,
            );
        }

        // Nonlinear model: dispatch to IPOPT path when an Ipopt solver is selected.
        #[cfg(feature = "ipopt")]
        {
            let backend = pym::solver::detect_default_backend(solver);
            if backend == "ipopt" || self.nonlinear_state.has_any() {
                let mut settings = if let Some(s) = solver {
                    pym::solver::extract_solver_settings(Some(s))?
                } else {
                    self.solver_settings.clone()
                };
                if let Some(log) = log_to_console {
                    settings.log_to_console = Some(log);
                }
                if let Some(v) = verbosity {
                    settings.verbosity = Some(v);
                }
                if let Some(algorithm) = lp_algorithm {
                    settings.set_lp_algorithm(algorithm);
                }
                pym::solver::validate_backend_settings("ipopt", &settings)?;
                let _ = time_limit;
                let _ = mip_gap;
                let solver = pym::solver::PySolver { settings };
                let py_result = pym::model_nlp::solve_with_ipopt(self, py, &solver)?;
                self.last_solution = Some(py_result.clone_ref(py));
                return Ok(py_result);
            }
        }

        let py_result = pym::model_solve::solve_model(
            self,
            py,
            solver,
            log_to_console,
            primal_start,
            time_limit,
            mip_gap,
            verbosity,
            lp_algorithm,
        )?;
        self.last_solution = Some(py_result.clone_ref(py));
        Ok(py_result)
    }

    /// Internal: access the last solve result for SlackVariable.value.
    #[getter]
    fn _last_solution(&self, py: Python<'_>) -> Option<Py<PySolveResult>> {
        self.last_solution.as_ref().map(|s| s.clone_ref(py))
    }

    /// Get the number of variables in the model
    #[getter]
    fn num_variables(&self) -> usize {
        self.inner.num_variables()
    }

    /// Get the number of constraints in the model
    #[getter]
    fn num_constraints(&self) -> usize {
        self.inner.num_constraints()
    }

    /// Get the number of non-zero coefficients in the model.
    #[getter]
    fn nnz(&self) -> usize {
        self.inner.num_coefficients()
    }

    /// Iterate all variables as Variable objects.
    #[getter]
    fn variables(&self) -> Vec<PyVariable> {
        let num = self.inner.num_variables();
        let mut result = Vec::with_capacity(num);
        for i in 0..num {
            let var_id = VariableId::new(i as u32);
            if let Ok(var) = self.inner.get_variable(var_id) {
                let name = self.reconstruct_variable_name(i as u32);
                result.push(PyVariable::from_model_variable(i as u32, name, &var));
            }
        }
        result
    }

    /// Iterate all constraints as Constraint objects.
    #[getter]
    fn constraints(&self) -> Vec<PyConstraint> {
        let num = self.inner.num_constraints();
        let mut result = Vec::with_capacity(num);
        for i in 0..num {
            let con_id = ConstraintId::new(i as u32);
            if let Ok(con) = self.inner.get_constraint(con_id) {
                let name = self.reconstruct_constraint_name(i as u32);
                result.push(PyConstraint::new(i as u32, name, con.bounds));
            }
        }
        result
    }

    /// Returns an iterator over all constraints in the model.
    fn list_constraints(slf: PyRef<'_, Self>) -> pym::iterators::PyConstraintIterator {
        let total = slf.inner.num_constraints();
        pym::iterators::PyConstraintIterator::new(slf.into(), total)
    }

    /// Returns an iterator over all variables in the model.
    fn list_variables(slf: PyRef<'_, Self>) -> pym::iterators::PyVariableIterator {
        let total = slf.inner.num_variables();
        pym::iterators::PyVariableIterator::new(slf.into(), total)
    }

    /// Returns a constraint by exact name match.
    ///
    /// Raises ConstraintNotFoundError if no constraint with the given name exists.
    #[pyo3(signature = (*, name))]
    fn get_constraint(&self, name: &str) -> PyResult<PyConstraint> {
        let con_id = self.find_constraint_by_name(name).ok_or_else(|| {
            pym::errors::ConstraintNotFoundError::new_err(format!(
                "constraint named '{name}' does not exist"
            ))
        })?;

        let con = self
            .inner
            .get_constraint(con_id)
            .map_err(pym::errors::model_error_to_py)?;

        Ok(PyConstraint::new(
            con_id.inner(),
            Some(name.to_string()),
            con.bounds,
        ))
    }

    /// Returns a variable by exact name match.
    ///
    /// Raises VariableNotFoundError if no variable with the given name exists.
    #[pyo3(signature = (*, name))]
    fn get_variable(&self, name: &str) -> PyResult<PyVariable> {
        let var_id = self.find_variable_by_name(name).ok_or_else(|| {
            pym::errors::VariableNotFoundError::new_err(format!(
                "variable named '{name}' does not exist"
            ))
        })?;
        let var = self
            .inner
            .get_variable(var_id)
            .map_err(pym::errors::model_error_to_py)?;
        Ok(PyVariable::from_model_variable(
            var_id.inner(),
            Some(name.to_string()),
            &var,
        ))
    }

    /// Returns metadata for a variable handle.
    #[pyo3(signature = (variable))]
    fn get_variable_metadata(
        &self,
        py: Python<'_>,
        variable: &PyVariable,
    ) -> PyResult<Option<PyObject>> {
        let var_id = VariableId::new(variable.var_id);
        self.inner
            .get_variable(var_id)
            .map_err(pym::errors::model_error_to_py)?;
        self.inner
            .get_variable_metadata(var_id)
            .map(|metadata| serde_bridge::json_to_py(py, metadata))
            .transpose()
    }

    fn __str__(&self) -> String {
        let adapter = pym::model_pretty::PythonPrettyAdapter { model: self };
        self.inner
            .format_ascii_with_adapter(&adapter, PrettyPrintOptions::preview())
    }

    /// Pretty-print the model in a human-readable algebraic form.
    fn pprint(&self, py: Python<'_>) -> PyResult<()> {
        let adapter = pym::model_pretty::PythonPrettyAdapter { model: self };
        let rendered = self
            .inner
            .format_ascii_with_adapter(&adapter, PrettyPrintOptions::full());
        let builtins = py.import("builtins")?;
        builtins.call_method1("print", (rendered,))?;
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!(
            "Model(variables={}, constraints={})",
            self.inner.num_variables(),
            self.inner.num_constraints()
        )
    }

    /// Export CSC matrix in a sparse-matrix compatible format.
    ///
    /// Returns dict with keys:
    /// - col_ptrs: list of column pointers (length = num_variables + 1)
    /// - row_indices: list of row indices
    /// - values: list of non-zero values
    /// - shape: tuple (num_constraints, num_variables)
    fn export_csc(&self, py: Python<'_>) -> PyResult<PyObject> {
        pym::model_inspect::export_csc(self, py)
    }

    /// Export CRS matrix in a sparse-matrix compatible format.
    ///
    /// Returns dict with keys:
    /// - row_ptrs: list of row pointers (length = num_constraints + 1)
    /// - col_indices: list of column indices
    /// - values: list of non-zero values
    /// - shape: tuple (num_constraints, num_variables)
    fn export_crs(&self, py: Python<'_>) -> PyResult<PyObject> {
        pym::model_inspect::export_crs(self, py)
    }

    /// Export COO matrix in a sparse-matrix compatible format.
    ///
    /// Returns dict with keys:
    /// - rows: list of row indices
    /// - cols: list of column indices
    /// - values: list of non-zero values
    /// - shape: tuple (num_constraints, num_variables)
    fn export_coo(&self, py: Python<'_>) -> PyResult<PyObject> {
        pym::model_inspect::export_coo(self, py)
    }

    /// Inspect the model structure and return a snapshot.
    #[pyo3(signature = (*, include_coeffs=false, include_slacks=true, variable_ids=None, constraint_ids=None))]
    fn inspect(
        &self,
        py: Python<'_>,
        include_coeffs: bool,
        include_slacks: bool,
        variable_ids: Option<Vec<u32>>,
        constraint_ids: Option<Vec<u32>>,
    ) -> PyResult<PyModelSnapshot> {
        let options = InspectOptions {
            include_coefficients: include_coeffs,
            include_slacks,
            variable_filter: variable_ids.map(|ids| ids.into_iter().map(VariableId::new).collect()),
            constraint_filter: constraint_ids
                .map(|ids| ids.into_iter().map(ConstraintId::new).collect()),
        };

        let mut snapshot = self.inner.inspect(options);
        for variable in &mut snapshot.variables {
            if variable.name.is_none() {
                variable.name = self.reconstruct_variable_name(variable.id.inner());
            }
        }
        for constraint in &mut snapshot.constraints {
            if constraint.name.is_none() {
                constraint.name = self.reconstruct_constraint_name(constraint.id.inner());
            }
        }
        PyModelSnapshot::from_snapshot(py, snapshot)
    }

    /// Return sparse matrix column-density diagnostics without exporting matrix arrays.
    #[pyo3(signature = (*, top_n=20, dense_threshold=100))]
    fn matrix_profile(
        &self,
        py: Python<'_>,
        top_n: usize,
        dense_threshold: usize,
    ) -> PyResult<PyObject> {
        let columns = self.inner.columns();
        let mut empty_columns = 0usize;
        let mut singleton_columns = 0usize;
        let mut two_entry_columns = 0usize;
        let mut nnz_le_5_columns = 0usize;
        let mut nnz_le_10_columns = 0usize;
        let mut nnz_le_50_columns = 0usize;
        let mut nnz_le_100_columns = 0usize;
        let mut nnz_le_500_columns = 0usize;
        let mut nnz_le_1000_columns = 0usize;
        let mut nnz_gt_1000_columns = 0usize;
        let mut dense_columns = 0usize;
        let mut max_column_nnz = 0usize;
        let mut min_nonzero_column_nnz: Option<usize> = None;
        let mut total_nnz = 0usize;
        let mut top_columns: Vec<(u32, usize)> = Vec::with_capacity(top_n);

        for (variable_id, column) in columns {
            let nnz = column.len();
            total_nnz = total_nnz.saturating_add(nnz);
            max_column_nnz = max_column_nnz.max(nnz);
            if nnz > 0 {
                min_nonzero_column_nnz =
                    Some(min_nonzero_column_nnz.map_or(nnz, |min| min.min(nnz)));
            }
            if nnz >= dense_threshold {
                dense_columns = dense_columns.saturating_add(1);
            }
            match nnz {
                0 => empty_columns = empty_columns.saturating_add(1),
                1 => singleton_columns = singleton_columns.saturating_add(1),
                2 => two_entry_columns = two_entry_columns.saturating_add(1),
                3..=5 => nnz_le_5_columns = nnz_le_5_columns.saturating_add(1),
                6..=10 => nnz_le_10_columns = nnz_le_10_columns.saturating_add(1),
                11..=50 => nnz_le_50_columns = nnz_le_50_columns.saturating_add(1),
                51..=100 => nnz_le_100_columns = nnz_le_100_columns.saturating_add(1),
                101..=500 => nnz_le_500_columns = nnz_le_500_columns.saturating_add(1),
                501..=1000 => nnz_le_1000_columns = nnz_le_1000_columns.saturating_add(1),
                _ => nnz_gt_1000_columns = nnz_gt_1000_columns.saturating_add(1),
            }

            if top_n > 0 {
                if top_columns.len() < top_n {
                    top_columns.push((variable_id.inner(), nnz));
                } else if let Some((min_idx, (_, min_nnz))) = top_columns
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, (_, candidate_nnz))| *candidate_nnz)
                {
                    if nnz > *min_nnz {
                        top_columns[min_idx] = (variable_id.inner(), nnz);
                    }
                }
            }
        }

        top_columns.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

        let profile = PyDict::new(py);
        let num_variables = self.inner.num_variables();
        let average_column_nnz = if num_variables == 0 {
            0.0
        } else {
            total_nnz as f64 / num_variables as f64
        };
        profile.set_item("num_variables", num_variables)?;
        profile.set_item("num_constraints", self.inner.num_constraints())?;
        profile.set_item("num_coefficients", total_nnz)?;
        profile.set_item("average_column_nnz", average_column_nnz)?;
        profile.set_item("max_column_nnz", max_column_nnz)?;
        profile.set_item("min_nonzero_column_nnz", min_nonzero_column_nnz)?;
        profile.set_item("dense_threshold", dense_threshold)?;
        profile.set_item("dense_columns", dense_columns)?;

        let buckets = PyDict::new(py);
        buckets.set_item("eq_0", empty_columns)?;
        buckets.set_item("eq_1", singleton_columns)?;
        buckets.set_item("eq_2", two_entry_columns)?;
        buckets.set_item("le_5", nnz_le_5_columns)?;
        buckets.set_item("le_10", nnz_le_10_columns)?;
        buckets.set_item("le_50", nnz_le_50_columns)?;
        buckets.set_item("le_100", nnz_le_100_columns)?;
        buckets.set_item("le_500", nnz_le_500_columns)?;
        buckets.set_item("le_1000", nnz_le_1000_columns)?;
        buckets.set_item("gt_1000", nnz_gt_1000_columns)?;
        profile.set_item("column_nnz_buckets", buckets)?;

        let top = PyList::empty(py);
        for (variable_id, nnz) in top_columns {
            let row = PyDict::new(py);
            row.set_item("variable_id", variable_id)?;
            row.set_item("name", self.reconstruct_variable_name(variable_id))?;
            row.set_item("nnz", nnz)?;
            top.append(row)?;
        }
        profile.set_item("top_columns", top)?;
        Ok(profile.unbind().into())
    }

    /// Add a typed block function to this model for composed optimization.
    ///
    /// Blocks must be decorated with `@arco.block` and use the function signature
    /// `(model, data)` or `(model, data, ctx)` where `data` is a dataclass or
    /// pydantic model annotation.
    ///
    /// The `extract` callable must use `(solution, data)` or
    /// `(solution, data, ctx)` and return a dataclass or pydantic model.
    ///
    /// Returns a BlockHandle with typed `.in_` and `.out` port accessors.
    #[pyo3(signature = (block_fn, *, name=None, data=None, extract))]
    fn add_block(
        &mut self,
        py: Python<'_>,
        block_fn: PyObject,
        name: Option<String>,
        data: Option<PyObject>,
        extract: PyObject,
    ) -> PyResult<PyBlockHandle> {
        self.add_block_impl(py, block_fn, name, data, extract)
    }

    /// Link a block output to a block input for composed models.
    #[pyo3(signature = (source, target))]
    fn link(&mut self, py: Python<'_>, source: BlockPort, target: BlockPort) -> PyResult<()> {
        self.link_impl(py, source, target)
    }

    /// Whether this model has blocks (is a composed model).
    #[getter]
    fn has_blocks(&self) -> bool {
        self.has_blocks_impl()
    }
}

/// Extract a `PyConstraint` from a Python object.
fn extract_constraint(ob: &Bound<'_, PyAny>) -> PyResult<Py<PyConstraint>> {
    ob.extract::<Py<PyConstraint>>()
        .map_err(|_| pym::errors::ConstraintTypeError::new_err("expected a Constraint"))
}
/// Extract a `ConstraintId` from a `PyConstraint`.
fn extract_constraint_id(ob: &Bound<'_, PyAny>) -> PyResult<ConstraintId> {
    let constraint = extract_constraint(ob)?;
    let constraint_ref = constraint.bind(ob.py()).borrow();
    Ok(ConstraintId::new(constraint_ref.constraint_id))
}

/// Extract a `Vec<Py<PyIndexSet>>` from the keyword-only `axes` argument.
fn extract_index_sets(axes: &Bound<'_, PyAny>) -> PyResult<Vec<Py<PyIndexSet>>> {
    axes.extract::<Vec<Py<PyIndexSet>>>().map_err(|_| {
        pym::errors::ArrayTypeError::new_err(
            "add_variables() expects axes=(...) with IndexSet values, \
             e.g. model.add_variables(axes=(T, G), bounds=...)",
        )
    })
}

/// Extract a `PyExpr` from a Python object that may be a `PyExpr`, `PyVariable`, or scalar.
fn extract_expr(ob: &Bound<'_, PyAny>) -> PyResult<PyExpr> {
    Ok(ob.extract::<pym::expr::ExprLike>()?.0)
}

/// Collect linear terms from a slice of PyExpr values into a single Vec.
fn collect_linear_terms(values: &[PyExpr]) -> Vec<(VariableId, f64)> {
    let total: usize = values.iter().map(|e| e.inner().linear_terms().len()).sum();
    let mut terms = Vec::with_capacity(total);
    for expr in values {
        terms.extend_from_slice(expr.inner().linear_terms());
    }
    terms
}

/// Extract objective terms directly from an array or expression, avoiding O(n^2) intermediate Expr.
fn extract_objective_terms(ob: &Bound<'_, PyAny>) -> PyResult<Vec<(VariableId, f64)>> {
    // Fast path: VariableArray -- collect terms directly (supports compact storage)
    if let Ok(va) = ob.extract::<PyRef<'_, PyVariableArray>>() {
        return Ok(va.collect_linear_terms_fast());
    }
    // Fast path: ExprArray -- check compact first, then fall back to full
    if let Ok(ea) = ob.extract::<PyRef<'_, PyExprArray>>() {
        if let Some(compact) = ea.as_compact() {
            return Ok(compact.collect_linear_terms());
        }
        return Ok(collect_linear_terms(&ea.get_values()));
    }
    // Fallback: extract as expression
    let linear_expr = extract_expr(ob)?;
    let (expr, _offset) = linear_expr.into_parts();
    Ok(expr.into_linear_terms())
}

fn parse_slack_bound(bound: &str) -> PyResult<SlackBound> {
    match bound {
        "lower" => Ok(SlackBound::Lower),
        "upper" => Ok(SlackBound::Upper),
        "both" => Ok(SlackBound::Both),
        _ => Err(pym::errors::SlackBoundError::new_err(format!(
            "Invalid slack bound '{}' (expected 'lower', 'upper', or 'both')",
            bound
        ))),
    }
}

fn bounds_from_sense(sense: ComparisonSense, rhs: f64) -> Bounds {
    match sense {
        ComparisonSense::LessEqual => Bounds::new(f64::NEG_INFINITY, rhs),
        ComparisonSense::GreaterEqual => Bounds::new(rhs, f64::INFINITY),
        ComparisonSense::Equal => Bounds::new(rhs, rhs),
    }
}

pub fn highs_version() -> Option<String> {
    arco_highs::highs_version()
}

/// Register the Arco Python module contents.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(diagnostic_codes, m)?)?;
    // Register all module classes
    m.add_class::<PyModel>()?;
    m.add_class::<PyBlockHandle>()?;
    m.add_class::<PyBlockPorts>()?;
    m.add_class::<PyBlockResults>()?;
    let typed_block_fn = wrap_pyfunction!(pym::model_blocks::typed_block, m)?;
    m.add_function(typed_block_fn.clone())?;

    // Register from submodules
    pym::enums::register(m)?;
    pym::errors::register(m)?;
    pym::solver::register(m)?;
    pym::solution::register(m)?;
    pym::bounds::register(m)?;
    pym::index_set::register(m)?;
    pym::expr::register(m)?;
    pym::arrays::register(m)?;
    pym::variable::register(m)?;
    pym::constraint::register(m)?;
    pym::handles::register(m)?;
    pym::slack_variable::register(m)?;
    pym::views::register(m)?;
    pym::snapshot::register(m)?;
    pym::iterators::register(m)?;
    #[cfg(feature = "ipopt")]
    pym::nonlinear::register(m)?;
    pym::bounds::export_bound_constants(m)?;
    m.setattr("block", typed_block_fn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SolverSettings;
    use std::collections::BTreeMap;

    #[test]
    fn solver_settings_rejects_zero_threads() {
        let result = SolverSettings::new(
            None,
            Some(0),
            None,
            None,
            None,
            None,
            None,
            None,
            BTreeMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_rejects_negative_tolerance() {
        let result = SolverSettings::new(
            None,
            None,
            Some(-0.5),
            None,
            None,
            None,
            None,
            None,
            BTreeMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_rejects_negative_time_limit() {
        let result = SolverSettings::new(
            None,
            None,
            None,
            Some(-1.0),
            None,
            None,
            None,
            None,
            BTreeMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_rejects_negative_mip_gap() {
        let result = SolverSettings::new(
            None,
            None,
            None,
            None,
            Some(-0.1),
            None,
            None,
            None,
            BTreeMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_accepts_defaults() {
        let result = SolverSettings::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            BTreeMap::new(),
        );
        assert!(result.is_ok());
    }
}
