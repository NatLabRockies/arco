//! Python bindings for Arco optimization using PyO3
//!
//! This module exposes Arco's model builder and solver to Python with zero-copy access
//! to solution data through memoryview.

mod arrays;
mod bounds;
mod constraint;
mod enums;
mod errors;
mod expr;
mod handles;
mod helpers;
mod index_set;
mod iterators;
mod logging;
mod model_blocks;
mod model_pretty;
mod serde_bridge;
mod slack_variable;
mod snapshot;
mod solution;
mod solver;
mod variable;
mod views;

use arco_blocks::{BlockPort, add_blocks_submodule};
use arco_core::model::{CscInput, PrettyPrintOptions, SparseMatrixExport};
use arco_core::types::Bounds;
use arco_core::{InspectOptions, Model, Objective, Sense, SlackBound, Variable};
use arco_expr::{ComparisonSense, ConstraintId, VariableId};

use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

pub(crate) type PyObject = Py<PyAny>;

fn sparse_export_dict<F>(py: Python<'_>, shape: (usize, usize), fill: F) -> PyResult<PyObject>
where
    F: FnOnce(&Bound<'_, PyDict>) -> PyResult<()>,
{
    let dict = PyDict::new(py);
    fill(&dict)?;
    dict.set_item("shape", (shape.0 as u32, shape.1 as u32))?;
    Ok(dict.unbind().into())
}

// Re-export types from modules
pub use arrays::{PyConstraintArray, PyExprArray, PyVariableArray};
pub use bounds::{BoundsSpec, PyBounds};
pub use constraint::PyConstraint;
pub use enums::{PyComparisonSense, PySense, PySimplifyLevel};
pub use expr::{PyConstraintExpr, PyExpr};
pub use handles::{PyElasticHandle, PySlackHandle};
pub use index_set::PyIndexSet;
pub use model_blocks::{PyBlockHandle, PyBlockPorts, PyBlockResults};
pub use slack_variable::PySlackVariable;
pub use snapshot::{PyModelSnapshot, PySnapshotMetadata};
pub use solution::{PySolutionStatus, PySolveResult};
#[cfg(feature = "ipopt")]
pub use solver::PyIpopt;
pub use solver::{PyHiGHS, PySolver, PyXpress, SolveOverrides, SolverSettings};
pub use variable::PyVariable;
pub use views::{
    PyCoefficientView, PyConstraintView, PyObjectiveView, PySlackView, PyVariableView,
};

/// Python wrapper for the Arco optimization model
#[pyclass(name = "Model")]
pub struct PyModel {
    pub(crate) inner: Model,
    solver_settings: SolverSettings,
    default_backend: String,
    last_solution: Option<Py<PySolveResult>>,
    /// Block definitions added via add_block()
    block_defs: Vec<model_blocks::BlockDef>,
    /// Links between blocks
    link_defs: Vec<model_blocks::LinkDef>,
    /// Compact metadata for arrays created via add_variables() for pretty-printing.
    array_print_specs: Vec<model_pretty::ArrayPrintSpec>,
}

impl PyModel {
    /// Compute effective bounds spec, validating binary constraints.
    #[allow(clippy::float_cmp)]
    fn effective_bounds(
        bounds: &BoundsSpec,
        is_integer: bool,
        is_binary: bool,
    ) -> PyResult<BoundsSpec> {
        let effective_binary = is_binary || bounds.is_binary;
        let effective_integer = is_integer || bounds.is_integer || effective_binary;

        if effective_binary
            && !bounds.is_binary
            && (bounds.bounds.lower != 0.0 || bounds.bounds.upper != 1.0)
        {
            return Err(errors::ModelBinaryBoundsError::new_err(
                "Binary variables must use bounds=[0,1]",
            ));
        }

        Ok(BoundsSpec {
            bounds: bounds.bounds,
            is_integer: effective_integer,
            is_binary: effective_binary,
        })
    }

    /// Reconstruct a variable name from array print specs.
    /// First checks Model's explicit names, then falls back to array spec reconstruction.
    fn reconstruct_variable_name(&self, var_id: u32) -> Option<String> {
        let vid = VariableId::new(var_id);
        // Check Model's explicit names first (for individually named vars)
        if let Some(name) = self.inner.get_variable_name(vid) {
            return Some(name.to_string());
        }
        // Reconstruct from array_print_spec
        let spec = self.find_array_print_spec(vid)?;
        let offset = (var_id - spec.start_var_id) as usize;
        if spec.len == 1 {
            Some(spec.base_name.clone())
        } else {
            Some(format!("{}[{}]", spec.base_name, offset))
        }
    }

    /// Find a variable by name, checking both explicit names and array spec reconstruction.
    fn find_variable_by_name(&self, name: &str) -> Option<VariableId> {
        // Check Model's explicit names first
        if let Some(id) = self.inner.get_variable_by_name(name) {
            return Some(id);
        }
        // Try to parse as "base_name[offset]" and check array specs
        if let Some(bracket_pos) = name.rfind('[') {
            let base = &name[..bracket_pos];
            let idx_str = name[bracket_pos + 1..].strip_suffix(']')?;
            let offset: usize = idx_str.parse().ok()?;
            for spec in &self.array_print_specs {
                if spec.base_name == base && offset < spec.len {
                    return Some(VariableId::new(spec.start_var_id + offset as u32));
                }
            }
        } else {
            // Try scalar name match (spec.len == 1)
            for spec in &self.array_print_specs {
                if spec.len == 1 && spec.base_name == name {
                    return Some(VariableId::new(spec.start_var_id));
                }
            }
        }
        None
    }

    fn set_constraint_name_if_provided(
        &mut self,
        constraint_id: ConstraintId,
        name: Option<String>,
    ) -> PyResult<()> {
        if let Some(name) = name {
            self.inner
                .set_constraint_name(constraint_id, name)
                .map_err(errors::model_error_to_py)?;
        }
        Ok(())
    }

    /// Name a contiguous block of constraints starting at `first_id`.
    fn name_constraint_block(
        &mut self,
        first_id: ConstraintId,
        count: usize,
        name: Option<&str>,
    ) -> PyResult<()> {
        let Some(base) = name else { return Ok(()) };
        for index in 0..count {
            let constraint_id = ConstraintId::new(first_id.inner() + index as u32);
            let con_name = if count == 1 {
                base.to_string()
            } else {
                format!("{base}[{index}]")
            };
            self.inner
                .set_constraint_name(constraint_id, con_name)
                .map_err(errors::model_error_to_py)?;
        }
        Ok(())
    }

    /// Insert constraints via compact term patterns (zero per-element allocation).
    fn add_constraints_compact_internal(
        &mut self,
        compact: &arrays::CompactConstraintStorage,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        let count = compact.count;
        let term_patterns = compact.term_patterns();
        let sense = compact.sense;

        // Build per-element bounds from sense + rhs
        let bounds_list: Vec<Bounds> = match &compact.rhs {
            arrays::CompactRhs::Scalar(rhs_val) => {
                vec![bounds_from_sense(sense, *rhs_val); count]
            }
            arrays::CompactRhs::Vec(rhs_values) => rhs_values
                .iter()
                .map(|rhs_val| bounds_from_sense(sense, *rhs_val))
                .collect(),
        };

        let first_constraint_id = self
            .inner
            .add_constraints_compact(&term_patterns, &bounds_list)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, count, name.as_deref())?;

        let rhs_vec = compact.rhs_vec();
        Ok(PyConstraintArray::from_batch(
            first_constraint_id.inner(),
            count,
            sense,
            &rhs_vec,
        ))
    }

    /// Add constraints from an array expression (VariableArray or ExprArray) with a separate rhs.
    ///
    /// Tries the compact fast path first, then falls back to materialized comparison.
    fn add_constraints_from_array(
        &mut self,
        compact_expr: Option<arrays::CompactExprStorage>,
        core_fn: impl FnOnce() -> arrays::LinearArrayCore,
        rhs_obj: &Bound<'_, PyAny>,
        sense: ComparisonSense,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        // Fast path: compact expression
        if let Some(ref compact) = compact_expr {
            if let Some(compact_con) = arrays::try_make_compact_constraint(compact, rhs_obj, sense)
            {
                return self.add_constraints_compact_internal(&compact_con, name);
            }
        }

        // Full path: materialize and compare
        let core = core_fn();
        let constraints = if let Ok(index_set) = rhs_obj.extract::<PyRef<'_, PyIndexSet>>() {
            core.compare_index_set(&index_set, sense)?
        } else {
            let value = rhs_obj.extract::<f64>()?;
            core.compare_scalar(value, sense)
        };
        self.add_constraints_full_internal(
            constraints.exprs().to_vec(),
            constraints.get_sense(),
            constraints.get_rhs(),
            name,
        )
    }

    /// Insert constraints via materialized expressions (existing batch path).
    fn add_constraints_full_internal(
        &mut self,
        exprs: Vec<PyExpr>,
        sense: ComparisonSense,
        rhs: Vec<f64>,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        let total = exprs.len();

        let batch: Vec<(Vec<(VariableId, f64)>, Bounds)> = exprs
            .into_iter()
            .zip(rhs.iter())
            .map(|(expr, &rhs_val)| {
                let bounds = bounds_from_sense(sense, rhs_val);
                (expr.into_inner().normalized_terms(), bounds)
            })
            .collect();

        let first_constraint_id = self
            .inner
            .add_constraints_batch(&batch)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, total, name.as_deref())?;

        Ok(PyConstraintArray::from_batch(
            first_constraint_id.inner(),
            total,
            sense,
            &rhs,
        ))
    }

    fn set_objective_from_expr(
        &mut self,
        expr: &Bound<'_, PyAny>,
        sense: Sense,
        name: Option<String>,
    ) -> PyResult<()> {
        let terms = extract_objective_terms(expr)?;
        self.inner
            .set_objective(Objective {
                sense: Some(sense),
                terms,
            })
            .map_err(errors::model_error_to_py)?;
        self.inner
            .set_objective_name(name)
            .map_err(errors::model_error_to_py)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn add_variables_scalar_bounds(
        &mut self,
        py: Python<'_>,
        index_sets: Vec<Py<PyIndexSet>>,
        shape: &[usize],
        total: usize,
        bounds: BoundsSpec,
        is_integer: bool,
        is_binary: bool,
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let effective_bounds = Self::effective_bounds(&bounds, is_integer, is_binary)?;
        let start_var_id = self.inner.num_variables() as u32;
        self.inner.reserve_variables(total);

        // Add all variables to the model in a tight loop (no PyExpr/PyVariable allocation)
        let var_template = Variable {
            bounds: bounds.bounds,
            is_integer: effective_bounds.is_integer,
            is_active: true,
        };
        for _ in 0..total {
            self.inner
                .add_variable(var_template)
                .map_err(errors::model_error_to_py)?;
        }

        self.register_array_print_spec(
            py,
            start_var_id,
            total,
            &index_sets,
            shape,
            name.as_deref(),
        );

        // Use compact storage: no Vec<PyExpr> or Vec<PyVariable> allocated
        Ok(PyVariableArray::new_compact(
            index_sets,
            shape.to_vec(),
            start_var_id,
            total,
            effective_bounds,
            name,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn add_variables_array_bounds(
        &mut self,
        py: Python<'_>,
        index_sets: Vec<Py<PyIndexSet>>,
        shape: &[usize],
        total: usize,
        bounds_obj: &Bound<'_, PyAny>,
        is_integer: bool,
        is_binary: bool,
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let start_var_id = self.inner.num_variables() as u32;
        self.inner.reserve_variables(total);

        // Extract lower and upper as numpy arrays from a Bounds-like object
        let lo_attr = bounds_obj
            .getattr("lower")
            .or_else(|_| bounds_obj.getattr("lo"))?;
        let hi_attr = bounds_obj
            .getattr("upper")
            .or_else(|_| bounds_obj.getattr("hi"))?;

        let np = py.import("numpy")?;
        let lo_flat = np
            .call_method1("asarray", (&lo_attr,))?
            .call_method0("flatten")?;
        let hi_flat = np
            .call_method1("asarray", (&hi_attr,))?
            .call_method0("flatten")?;

        let lo_values: Vec<f64> = lo_flat.extract()?;
        let hi_values: Vec<f64> = hi_flat.extract()?;

        if lo_values.len() != total {
            return Err(errors::ArrayShapeMismatchError::new_err(format!(
                "lower bounds length {} does not match total variables {}",
                lo_values.len(),
                total
            )));
        }
        if hi_values.len() != total {
            return Err(errors::ArrayShapeMismatchError::new_err(format!(
                "upper bounds length {} does not match total variables {}",
                hi_values.len(),
                total
            )));
        }

        let effective_binary = is_binary;
        let effective_integer = is_integer || effective_binary;

        let mut values = Vec::with_capacity(total);
        let mut variables = Vec::with_capacity(total);
        for i in 0..total {
            let element_bounds = Bounds::new(lo_values[i], hi_values[i]);
            let var = Variable {
                bounds: element_bounds,
                is_integer: effective_integer,
                is_active: true,
            };
            let var_id = self
                .inner
                .add_variable(var)
                .map_err(errors::model_error_to_py)?;

            let var_name = name.as_ref().map(|base| {
                if total == 1 {
                    base.clone()
                } else {
                    format!("{base}[{i}]")
                }
            });

            let element_bounds_spec = BoundsSpec {
                bounds: element_bounds,
                is_integer: effective_integer,
                is_binary: effective_binary,
            };

            values.push(PyExpr::from_term(var_id.inner(), 1.0));
            variables.push(PyVariable::new(
                var_id.inner(),
                var_name,
                element_bounds_spec,
            ));
        }

        self.register_array_print_spec(
            py,
            start_var_id,
            total,
            &index_sets,
            shape,
            name.as_deref(),
        );

        Ok(PyVariableArray::new(
            index_sets,
            shape.to_vec(),
            values,
            variables,
        ))
    }
}

#[pymethods]
impl PyModel {
    /// Create a new model
    #[new]
    #[pyo3(signature = (*, simplify_level=None, solver=None))]
    fn new(
        simplify_level: Option<PySimplifyLevel>,
        solver: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let inner = if let Some(level) = simplify_level {
            Model::with_simplify_level(level.into())
        } else {
            Model::new()
        };
        let default_backend = detect_default_backend(solver);
        let solver_settings = extract_solver_settings(solver)?;
        Ok(PyModel {
            inner,
            solver_settings,
            default_backend,
            last_solution: None,
            block_defs: Vec::new(),
            link_defs: Vec::new(),
            array_print_specs: Vec::new(),
        })
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
        let col_ptrs = helpers::extract_indices(col_ptrs, "col_ptrs")?;
        let row_indices = helpers::extract_indices(row_indices, "row_indices")?;
        let values = helpers::extract_f32(values, "values")?;
        let var_lower = helpers::extract_f32(var_lower, "var_lower")?;
        let var_upper = helpers::extract_f32(var_upper, "var_upper")?;
        let con_lower = helpers::extract_f32(con_lower, "con_lower")?;
        let con_upper = helpers::extract_f32(con_upper, "con_upper")?;
        let is_integer = helpers::extract_bool(is_integer, "is_integer")?;
        let simplify_level = simplify_level.map(Into::into).unwrap_or_default();

        let inner = Model::from_csc(
            CscInput {
                num_constraints,
                num_variables,
                col_ptrs: &col_ptrs,
                row_indices: &row_indices,
                values: &values,
                var_lower: &var_lower,
                var_upper: &var_upper,
                con_lower: &con_lower,
                con_upper: &con_upper,
                is_integer: &is_integer,
            },
            simplify_level,
        )
        .map_err(errors::model_error_to_py)?;

        Ok(PyModel {
            inner,
            solver_settings: SolverSettings::default(),
            default_backend: "highs".to_string(),
            last_solution: None,
            block_defs: Vec::new(),
            link_defs: Vec::new(),
            array_print_specs: Vec::new(),
        })
    }

    /// Add a variable to the model.
    ///
    /// # Arguments
    /// * `bounds` - Bounds or bound constant (e.g. NonNegativeFloat, Binary)
    /// * `is_integer` - Whether the variable is integer-constrained
    /// * `is_binary` - Whether the variable is binary
    /// * `name` - Optional name for the variable
    ///
    /// # Returns
    /// A Variable object
    #[pyo3(signature = (bounds, *, is_integer=false, is_binary=false, name=None))]
    fn add_variable(
        &mut self,
        bounds: BoundsSpec,
        is_integer: bool,
        is_binary: bool,
        name: Option<String>,
    ) -> PyResult<PyVariable> {
        let effective_bounds = Self::effective_bounds(&bounds, is_integer, is_binary)?;

        let var = Variable {
            bounds: bounds.bounds,
            is_integer: effective_bounds.is_integer,
            is_active: true,
        };

        let var_id = self
            .inner
            .add_variable(var)
            .map_err(errors::model_error_to_py)?;

        if let Some(ref n) = name {
            self.inner
                .set_variable_name(var_id, n.clone())
                .map_err(errors::model_error_to_py)?;
        }

        Ok(PyVariable::new(var_id.inner(), name, effective_bounds))
    }

    /// Add a vector or grid of variables to the model.
    #[pyo3(signature = (*index_sets, bounds, is_integer=false, is_binary=false, name=None))]
    fn add_variables(
        &mut self,
        py: Python<'_>,
        index_sets: &Bound<'_, PyTuple>,
        bounds: &Bound<'_, PyAny>,
        is_integer: bool,
        is_binary: bool,
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let index_sets = extract_index_sets(index_sets)?;

        if index_sets.is_empty() {
            return Err(errors::IndexSetEmptyError::new_err(
                "index_sets must be non-empty",
            ));
        }

        let shape: Vec<usize> = index_sets
            .iter()
            .map(|s| {
                let size = s.borrow(py).members.len();
                if size == 0 {
                    return Err(errors::IndexSetEmptyError::new_err(
                        "index sets must be non-empty",
                    ));
                }
                Ok(size)
            })
            .collect::<PyResult<_>>()?;

        let total = shape.iter().try_fold(1usize, |acc, &size| {
            acc.checked_mul(size)
                .ok_or_else(|| errors::ArrayOverflowError::new_err("array size overflow"))
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
                name,
            );
        }

        // Try per-element array bounds: Bounds object with numpy array lo/hi
        self.add_variables_array_bounds(
            py, index_sets, &shape, total, bounds, is_integer, is_binary, name,
        )
    }

    /// Deactivate a variable without removing its column.
    #[pyo3(signature = (*, var_id))]
    fn deactivate_variable(&mut self, var_id: u32) -> PyResult<()> {
        self.inner
            .deactivate_variable(VariableId::new(var_id))
            .map_err(errors::model_error_to_py)
    }

    /// Activate a previously deactivated variable.
    #[pyo3(signature = (*, var_id))]
    fn activate_variable(&mut self, var_id: u32) -> PyResult<()> {
        self.inner
            .activate_variable(VariableId::new(var_id))
            .map_err(errors::model_error_to_py)
    }

    /// Check whether a variable is active.
    #[pyo3(signature = (*, var_id))]
    fn is_variable_active(&self, var_id: u32) -> PyResult<bool> {
        self.inner
            .is_variable_active(VariableId::new(var_id))
            .map_err(errors::model_error_to_py)
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
                    errors::ConstraintBoundsMissingError::new_err(
                        "bounds are required when expression has no comparison",
                    )
                })?;
                let (expr, offset) = linear_expr.into_parts();
                let bounds = Bounds::new(bounds.inner.lower - offset, bounds.inner.upper - offset);
                (expr, bounds)
            } else {
                return Err(errors::ConstraintTypeError::new_err(
                    "expected an Expr, Variable, or ConstraintExpr",
                ));
            };

        let constraint_id = self
            .inner
            .add_expr_constraint(expr, constraint_bounds)
            .map_err(errors::model_error_to_py)?;
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
    #[pyo3(signature = (expr, *, sense=PyComparisonSense::GreaterEqual, rhs=None, name=None))]
    fn add_constraints(
        &mut self,
        expr: &Bound<'_, PyAny>,
        sense: PyComparisonSense,
        rhs: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        // Branch 1: ConstraintArray input
        if let Ok(array) = expr.extract::<PyRef<'_, PyConstraintArray>>() {
            if rhs.is_some() || sense != PyComparisonSense::GreaterEqual {
                return Err(errors::ConstraintSenseError::new_err(
                    "sense/rhs are not supported for comparison arrays",
                ));
            }

            // Fast path: compact constraint storage
            if let Some(compact) = array.as_compact() {
                return self.add_constraints_compact_internal(compact, name);
            }

            // Full path
            return self.add_constraints_full_internal(
                array.exprs().to_vec(),
                array.get_sense(),
                array.get_rhs(),
                name,
            );
        }

        // Branch 2: VariableArray or ExprArray input
        let sense: ComparisonSense = sense.into();
        let rhs_obj = rhs.ok_or_else(|| {
            errors::ConstraintBoundsMissingError::new_err("rhs is required for add_constraints")
        })?;

        if let Ok(array) = expr.extract::<PyRef<'_, PyVariableArray>>() {
            let compact = array.as_compact_expr();
            return self.add_constraints_from_array(
                compact,
                || array.to_core(),
                rhs_obj,
                sense,
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
                name,
            );
        }

        Err(errors::ConstraintTypeError::new_err(
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
            .map_err(errors::model_error_to_py)?;
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
                return Err(PyRuntimeError::new_err(format!(
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
                .map_err(errors::model_error_to_py)?;

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
            .map_err(errors::model_error_to_py)?;
        Ok(PyElasticHandle::from_handle(handle))
    }

    /// Set a coefficient in the constraint matrix
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
            .map_err(errors::model_error_to_py)
    }

    /// Set the objective function
    ///
    /// # Arguments
    /// * `sense` - The optimization sense (Minimize or Maximize)
    /// * `terms` - List of (variable_index, coefficient) tuples
    #[pyo3(signature = (sense, terms, *, name=None))]
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
            .map_err(errors::model_error_to_py)?;

        self.inner
            .set_objective_name(name)
            .map_err(errors::model_error_to_py)?;
        Ok(())
    }

    /// Minimize a linear expression.
    #[pyo3(signature = (expr, *, name=None))]
    fn minimize(&mut self, expr: &Bound<'_, PyAny>, name: Option<String>) -> PyResult<()> {
        self.set_objective_from_expr(expr, Sense::Minimize, name)
    }

    /// Maximize a linear expression.
    #[pyo3(signature = (expr, *, name=None))]
    fn maximize(&mut self, expr: &Bound<'_, PyAny>, name: Option<String>) -> PyResult<()> {
        self.set_objective_from_expr(expr, Sense::Maximize, name)
    }

    /// Set the objective name stored in model metadata.
    #[pyo3(signature = (*, name))]
    fn set_objective_name(&mut self, name: Option<String>) -> PyResult<()> {
        self.inner
            .set_objective_name(name)
            .map_err(errors::model_error_to_py)
    }

    /// Get the objective name stored in model metadata.
    fn get_objective_name(&self) -> Option<String> {
        self.inner
            .get_objective_name()
            .map(|value| value.to_string())
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
            .map_err(errors::model_error_to_py)
    }

    /// Solve the model and return a solution.
    ///
    /// Set `log_to_console=True` to enable solver logs.
    /// Set `primal_start` to a list of (variable_id, value) tuples for warm-start hints.
    /// Optional solver controls include `time_limit`, `mip_gap`, and `verbosity`.
    /// Pass `solver=arco.Xpress()` to use FICO Xpress instead of the default HiGHS solver.
    #[pyo3(
        signature = (*, solver=None, log_to_console=None, primal_start=None, time_limit=None, mip_gap=None, verbosity=None)
    )]
    fn solve(
        &mut self,
        py: Python<'_>,
        solver: Option<&Bound<'_, PyAny>>,
        log_to_console: Option<bool>,
        primal_start: Option<Vec<(u32, f64)>>,
        time_limit: Option<f64>,
        mip_gap: Option<f64>,
        verbosity: Option<u32>,
    ) -> PyResult<Py<PySolveResult>> {
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
            );
        }

        let overrides = SolveOverrides {
            log_to_console,
            time_limit,
            mip_gap,
            verbosity,
        };

        let hints: Option<Vec<(VariableId, f64)>> = primal_start.map(|ps| {
            ps.into_iter()
                .map(|(var_id, value)| (VariableId::new(var_id), value))
                .collect()
        });

        let effective_settings = if let Some(s) = solver {
            extract_solver_settings(Some(s))?
        } else {
            self.solver_settings.clone()
        };
        let effective_settings = effective_settings.with_overrides(overrides)?;

        let backend = resolve_backend(solver, &self.default_backend)?;
        let config = effective_settings.to_solver_config();

        let result = match backend.solve(&self.inner, &config, hints.as_deref()) {
            Ok(solution) => Ok(PySolveResult::new(solution)),
            Err(arco_solver::SolverError::SolveFailure { status }) => {
                Ok(PySolveResult::new(solve_failure_solution(status)))
            }
            Err(e) => Err(errors::generic_solver_error_to_py(e)),
        }?;

        let py_result = Py::new(py, result)?;
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
                let name = self
                    .inner
                    .get_constraint_name(con_id)
                    .map(|s| s.to_string());
                result.push(PyConstraint::new(i as u32, name, con.bounds));
            }
        }
        result
    }

    /// Returns an iterator over all constraints in the model.
    fn list_constraints(slf: PyRef<'_, Self>) -> iterators::PyConstraintIterator {
        let total = slf.inner.num_constraints();
        iterators::PyConstraintIterator::new(slf.into(), total)
    }

    /// Returns an iterator over all variables in the model.
    fn list_variables(slf: PyRef<'_, Self>) -> iterators::PyVariableIterator {
        let total = slf.inner.num_variables();
        iterators::PyVariableIterator::new(slf.into(), total)
    }

    /// Returns a constraint by exact name match.
    ///
    /// Raises KeyError if no constraint with the given name exists.
    #[pyo3(signature = (*, name))]
    fn get_constraint(&self, name: &str) -> PyResult<PyConstraint> {
        let con_id = self
            .inner
            .get_constraint_by_name(name)
            .ok_or_else(|| PyKeyError::new_err(name.to_string()))?;

        let con = self
            .inner
            .get_constraint(con_id)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(PyConstraint::new(
            con_id.inner(),
            Some(name.to_string()),
            con.bounds,
        ))
    }

    /// Returns a variable by exact name match.
    ///
    /// Raises KeyError if no variable with the given name exists.
    #[pyo3(signature = (*, name))]
    fn get_variable(&self, name: &str) -> PyResult<PyVariable> {
        let var_id = self
            .find_variable_by_name(name)
            .ok_or_else(|| PyKeyError::new_err(name.to_string()))?;
        let var = self
            .inner
            .get_variable(var_id)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyVariable::from_model_variable(
            var_id.inner(),
            Some(name.to_string()),
            &var,
        ))
    }

    fn __str__(&self) -> String {
        let adapter = model_pretty::PythonPrettyAdapter { model: self };
        self.inner
            .format_ascii_with_adapter(&adapter, PrettyPrintOptions::preview())
    }

    /// Pretty-print the model in a human-readable algebraic form.
    fn pprint(&self, py: Python<'_>) -> PyResult<()> {
        let adapter = model_pretty::PythonPrettyAdapter { model: self };
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

    /// Get sparse matrix columns as dict mapping variable_id -> [(constraint_id, coefficient), ...]
    fn get_columns(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new(py);

        for (var_id, coeffs) in self.inner.columns() {
            let coeff_list: Vec<(u32, f64)> = coeffs
                .iter()
                .map(|(cid, coeff)| (cid.inner(), *coeff))
                .collect();
            dict.set_item(var_id.inner(), coeff_list)?;
        }

        Ok(dict.unbind().into())
    }

    /// Export CSC matrix in a sparse-matrix compatible format.
    ///
    /// Returns dict with keys:
    /// - col_ptrs: list of column pointers (length = num_variables + 1)
    /// - row_indices: list of row indices
    /// - values: list of non-zero values
    /// - shape: tuple (num_constraints, num_variables)
    fn export_csc(&self, py: Python<'_>) -> PyResult<PyObject> {
        let matrix = self.inner.export_csc();
        sparse_export_dict(py, matrix.shape, |dict| {
            dict.set_item("col_ptrs", matrix.col_ptrs)?;
            dict.set_item("row_indices", matrix.row_indices)?;
            dict.set_item("values", matrix.values)
        })
    }

    /// Export CRS matrix in a sparse-matrix compatible format.
    ///
    /// Returns dict with keys:
    /// - row_ptrs: list of row pointers (length = num_constraints + 1)
    /// - col_indices: list of column indices
    /// - values: list of non-zero values
    /// - shape: tuple (num_constraints, num_variables)
    fn export_crs(&self, py: Python<'_>) -> PyResult<PyObject> {
        let matrix = self.inner.export_crs();
        sparse_export_dict(py, matrix.shape, |dict| {
            dict.set_item("row_ptrs", matrix.row_ptrs)?;
            dict.set_item("col_indices", matrix.col_indices)?;
            dict.set_item("values", matrix.values)
        })
    }

    /// Export COO matrix in a sparse-matrix compatible format.
    ///
    /// Returns dict with keys:
    /// - rows: list of row indices
    /// - cols: list of column indices
    /// - values: list of non-zero values
    /// - shape: tuple (num_constraints, num_variables)
    fn export_coo(&self, py: Python<'_>) -> PyResult<PyObject> {
        let matrix = self.inner.export_coo();
        sparse_export_dict(py, matrix.shape, |dict| {
            dict.set_item("rows", matrix.rows)?;
            dict.set_item("cols", matrix.cols)?;
            dict.set_item("values", matrix.values)
        })
    }

    #[allow(clippy::unused_self)]
    fn export_arrow(&self) -> PyResult<PyObject> {
        Err(PyRuntimeError::new_err(
            "Arrow export is not enabled in this build",
        ))
    }

    /// Set name for a variable
    ///
    /// # Arguments
    /// * `var_id` - Index of the variable
    /// * `name` - Name to assign to the variable
    #[pyo3(signature = (var_id, *, name))]
    fn set_variable_name(&mut self, var_id: u32, name: String) -> PyResult<()> {
        let id = VariableId::new(var_id);
        self.inner
            .set_variable_name(id, name)
            .map_err(errors::model_error_to_py)
    }

    /// Get name for a variable
    ///
    /// # Arguments
    /// * `var_id` - Index of the variable
    ///
    /// # Returns
    /// The name if set, None otherwise
    fn get_variable_name(&self, var_id: u32) -> Option<String> {
        self.reconstruct_variable_name(var_id)
    }

    /// Lookup a variable by name.
    #[pyo3(signature = (name, /))]
    fn get_variable_by_name(&self, name: String) -> Option<u32> {
        self.find_variable_by_name(&name).map(|id| id.inner())
    }

    /// Set metadata for a variable
    ///
    /// # Arguments
    /// * `var_id` - Index of the variable
    /// * `metadata` - Dictionary of metadata to attach
    #[pyo3(signature = (var_id, *, metadata))]
    fn set_variable_metadata(
        &mut self,
        var_id: u32,
        metadata: &Bound<'_, pyo3::types::PyDict>,
    ) -> PyResult<()> {
        let id = VariableId::new(var_id);
        let value = serde_bridge::py_any_to_json(&metadata.clone().into_any())?;
        self.inner
            .set_variable_metadata(id, value)
            .map_err(errors::model_error_to_py)
    }

    /// Get metadata for a variable
    ///
    /// # Arguments
    /// * `var_id` - Index of the variable
    ///
    /// # Returns
    /// The metadata dictionary if set, None otherwise
    fn get_variable_metadata(&self, py: Python<'_>, var_id: u32) -> Option<PyObject> {
        let id = VariableId::new(var_id);
        self.inner
            .get_variable_metadata(id)
            .and_then(|v| serde_bridge::json_to_py(py, v).ok())
    }

    /// Set name for a constraint
    ///
    /// # Arguments
    /// * `con_id` - Index of the constraint
    /// * `name` - Name to assign to the constraint
    #[pyo3(signature = (con_id, *, name))]
    fn set_constraint_name(&mut self, con_id: u32, name: String) -> PyResult<()> {
        let id = ConstraintId::new(con_id);
        self.inner
            .set_constraint_name(id, name)
            .map_err(errors::model_error_to_py)
    }

    /// Get name for a constraint
    ///
    /// # Arguments
    /// * `con_id` - Index of the constraint
    ///
    /// # Returns
    /// The name if set, None otherwise
    fn get_constraint_name(&self, con_id: u32) -> Option<String> {
        let id = ConstraintId::new(con_id);
        self.inner.get_constraint_name(id).map(|s| s.to_string())
    }

    /// Lookup a constraint by name.
    #[pyo3(signature = (name, /))]
    fn get_constraint_by_name(&self, name: String) -> Option<u32> {
        self.inner
            .get_constraint_by_name(&name)
            .map(|id| id.inner())
    }

    /// Set metadata for a constraint
    ///
    /// # Arguments
    /// * `con_id` - Index of the constraint
    /// * `metadata` - Dictionary of metadata to attach
    #[pyo3(signature = (con_id, *, metadata))]
    fn set_constraint_metadata(
        &mut self,
        con_id: u32,
        metadata: &Bound<'_, pyo3::types::PyDict>,
    ) -> PyResult<()> {
        let id = ConstraintId::new(con_id);
        let value = serde_bridge::py_any_to_json(&metadata.clone().into_any())?;
        self.inner
            .set_constraint_metadata(id, value)
            .map_err(errors::model_error_to_py)
    }

    /// Get metadata for a constraint
    ///
    /// # Arguments
    /// * `con_id` - Index of the constraint
    ///
    /// # Returns
    /// The metadata dictionary if set, None otherwise
    fn get_constraint_metadata(&self, py: Python<'_>, con_id: u32) -> Option<PyObject> {
        let id = ConstraintId::new(con_id);
        self.inner
            .get_constraint_metadata(id)
            .and_then(|v| serde_bridge::json_to_py(py, v).ok())
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

        let snapshot = self.inner.inspect(options);
        PyModelSnapshot::from_snapshot(py, snapshot)
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

#[cfg(test)]
mod tests {
    use super::SolverSettings;

    #[test]
    fn solver_settings_rejects_zero_threads() {
        let result = SolverSettings::new(None, Some(0), None, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_rejects_negative_tolerance() {
        let result = SolverSettings::new(None, None, Some(-0.5), None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_rejects_negative_time_limit() {
        let result = SolverSettings::new(None, None, None, Some(-1.0), None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_rejects_negative_mip_gap() {
        let result = SolverSettings::new(None, None, None, None, Some(-0.1), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn solver_settings_accepts_defaults() {
        let result = SolverSettings::new(None, None, None, None, None, None, None);
        assert!(result.is_ok());
    }
}

/// Create a Solution for a solve failure (infeasible, unbounded, etc.)
fn solve_failure_solution(status: arco_core::solver::SolverStatus) -> arco_core::solver::Solution {
    arco_core::solver::Solution {
        primal_values: Vec::new(),
        variable_duals: Vec::new(),
        constraint_duals: Vec::new(),
        row_values: Vec::new(),
        objective_value: f64::NAN,
        status,
        solve_time_seconds: 0.0,
        metadata: std::collections::BTreeMap::new(),
    }
}

/// Detect which backend name a solver object represents.
fn detect_default_backend(solver: Option<&Bound<'_, PyAny>>) -> String {
    let Some(solver) = solver else {
        return "highs".to_string();
    };
    #[cfg(feature = "ipopt")]
    if solver.cast::<PyIpopt>().is_ok() {
        return "ipopt".to_string();
    }
    if solver.cast::<PyXpress>().is_ok() {
        return "xpress".to_string();
    }
    "highs".to_string()
}

/// Extract `SolverSettings` from an optional Python solver object (`HiGHS`, `Ipopt`, `Xpress`, or `Solver`).
fn extract_solver_settings(solver: Option<&Bound<'_, PyAny>>) -> PyResult<SolverSettings> {
    let Some(solver) = solver else {
        return Ok(SolverSettings::default());
    };
    if let Ok(highs) = solver.cast::<PyHiGHS>() {
        return Ok(highs.borrow().into_super().settings.clone());
    }
    #[cfg(feature = "ipopt")]
    if let Ok(ipopt) = solver.cast::<PyIpopt>() {
        return Ok(ipopt.borrow().into_super().settings.clone());
    }
    if let Ok(xpress) = solver.cast::<PyXpress>() {
        return Ok(xpress.borrow().into_super().settings.clone());
    }
    if let Ok(base) = solver.cast::<PySolver>() {
        return Ok(base.borrow().settings.clone());
    }
    Err(errors::SolverTypeError::new_err(
        "solver must be a Solver, HiGHS, Ipopt, or Xpress instance",
    ))
}

/// Resolve which `SolverBackend` implementation to use.
fn resolve_backend(
    solver: Option<&Bound<'_, PyAny>>,
    default_backend: &str,
) -> PyResult<Box<dyn arco_solver::SolverBackend>> {
    #[cfg(feature = "ipopt")]
    if solver.is_some_and(|s| s.cast::<PyIpopt>().is_ok()) || default_backend == "ipopt" {
        return Ok(Box::new(arco_ipopt::IpoptBackend));
    }
    #[cfg(not(feature = "ipopt"))]
    if default_backend == "ipopt" {
        return Err(errors::SolverInternalError::new_err(
            "Ipopt backend is not enabled in this build",
        ));
    }
    #[cfg(feature = "xpress")]
    if solver.is_some_and(|s| s.cast::<PyXpress>().is_ok()) || default_backend == "xpress" {
        return Ok(Box::new(arco_xpress::XpressBackend));
    }
    #[cfg(not(feature = "xpress"))]
    if solver.is_some_and(|s| s.cast::<PyXpress>().is_ok()) || default_backend == "xpress" {
        return Err(errors::SolverInternalError::new_err(
            "Xpress backend is not enabled in this build",
        ));
    }
    // Default: HiGHS
    Ok(Box::new(arco_highs::HiGHSBackend))
}

/// Extract a `PyConstraint` from a Python object.
fn extract_constraint(ob: &Bound<'_, PyAny>) -> PyResult<Py<PyConstraint>> {
    ob.extract::<Py<PyConstraint>>()
        .map_err(|_| errors::ConstraintTypeError::new_err("expected a Constraint"))
}
/// Extract a `ConstraintId` from a `PyConstraint`.
fn extract_constraint_id(ob: &Bound<'_, PyAny>) -> PyResult<ConstraintId> {
    let constraint = extract_constraint(ob)?;
    let constraint_ref = constraint.bind(ob.py()).borrow();
    Ok(ConstraintId::new(constraint_ref.constraint_id))
}

/// Extract a `Vec<Py<PyIndexSet>>` from the positional `*index_sets` tuple.
fn extract_index_sets(tuple: &Bound<'_, PyTuple>) -> PyResult<Vec<Py<PyIndexSet>>> {
    tuple
        .iter()
        .map(|item| {
            item.extract::<Py<PyIndexSet>>().map_err(|_| {
                PyTypeError::new_err(
                    "add_variables() expects IndexSet arguments, \
                     e.g. model.add_variables(T, G, bounds=...)",
                )
            })
        })
        .collect()
}

/// Extract a `PyExpr` from a Python object that may be a `PyExpr`, `PyVariable`, or scalar.
fn extract_expr(ob: &Bound<'_, PyAny>) -> PyResult<PyExpr> {
    Ok(ob.extract::<crate::expr::ExprLike>()?.0)
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
        _ => Err(errors::SlackBoundError::new_err(format!(
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

/// The Arco Python module
#[pymodule]
fn arco(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register all module classes
    m.add_class::<PyModel>()?;
    m.add_class::<PyBlockHandle>()?;
    m.add_class::<PyBlockPorts>()?;
    m.add_class::<PyBlockResults>()?;
    let typed_block_fn = wrap_pyfunction!(model_blocks::typed_block, m)?;
    m.add_function(typed_block_fn.clone())?;

    // Register from submodules
    enums::register(m)?;
    errors::register(m)?;
    solver::register(m)?;
    solution::register(m)?;
    bounds::register(m)?;
    index_set::register(m)?;
    expr::register(m)?;
    arrays::register(m)?;
    variable::register(m)?;
    constraint::register(m)?;
    handles::register(m)?;
    slack_variable::register(m)?;
    views::register(m)?;
    snapshot::register(m)?;
    logging::register(m)?;
    iterators::register(m)?;
    bounds::export_bound_constants(m)?;
    add_blocks_submodule(m.py(), m)?;
    m.setattr("block", typed_block_fn)?;

    Ok(())
}
