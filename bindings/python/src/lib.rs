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
use arco_core::model::{CscInput, PrettyPrintOptions};
use arco_core::types::Bounds;
use arco_core::{InspectOptions, Model, Objective, Sense, SlackBound, Variable};
use arco_expr::{ComparisonSense, ConstraintId, VariableId};

use pyo3::exceptions::{PyKeyError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};

pub(crate) type PyObject = Py<PyAny>;

// Re-export types from modules
pub use arrays::{PyConstraintArray, PyExprArray, PyVariableArray};
pub use bounds::{BoundsSpec, PyBounds};
pub use constraint::PyConstraint;
pub use enums::{PySense, PySimplifyLevel};
pub use expr::{PyConstraintExpr, PyExpr};
pub use handles::{PyElasticHandle, PySlackHandle};
pub use index_set::PyIndexSet;
pub use model_blocks::{PyBlockHandle, PyBlockPorts, PyBlockResults};
pub use slack_variable::PySlackVariable;
pub use snapshot::{PyModelSnapshot, PySnapshotMetadata};
pub use solution::{PySolutionStatus, PySolveResult};
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
    use_xpress: bool,
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

        let mut values = Vec::with_capacity(total);
        let mut variables = Vec::with_capacity(total);
        for i in 0..total {
            let var = Variable {
                bounds: bounds.bounds,
                is_integer: effective_bounds.is_integer,
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
            if let Some(ref n) = var_name {
                self.inner
                    .set_variable_name(var_id, n.clone())
                    .map_err(errors::model_error_to_py)?;
            }

            values.push(PyExpr::from_term(var_id.inner(), 1.0));
            variables.push(PyVariable::new(var_id.inner(), var_name, effective_bounds));
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
            if let Some(ref n) = var_name {
                self.inner
                    .set_variable_name(var_id, n.clone())
                    .map_err(errors::model_error_to_py)?;
            }

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
        let use_xpress = solver.is_some_and(|s| s.cast::<PyXpress>().is_ok());
        let solver_settings = extract_solver_settings(solver)?;
        Ok(PyModel {
            inner,
            solver_settings,
            use_xpress,
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
            use_xpress: false,
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
    #[pyo3(signature = (index_sets, bounds, *, is_integer=false, is_binary=false, name=None))]
    fn add_variables(
        &mut self,
        py: Python<'_>,
        index_sets: Vec<Py<PyIndexSet>>,
        bounds: &Bound<'_, PyAny>,
        is_integer: bool,
        is_binary: bool,
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        if index_sets.is_empty() {
            return Err(errors::IndexSetEmptyError::new_err(
                "index_sets must be non-empty",
            ));
        }

        let mut shape = Vec::with_capacity(index_sets.len());
        for index_set in &index_sets {
            let size = index_set.borrow(py).members.len();
            if size == 0 {
                return Err(errors::IndexSetEmptyError::new_err(
                    "index sets must be non-empty",
                ));
            }
            shape.push(size);
        }

        let total = shape.iter().try_fold(1usize, |acc, size| {
            acc.checked_mul(*size)
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
    #[pyo3(signature = (expr, *, sense="ge", rhs=None, name=None))]
    fn add_constraints(
        &mut self,
        expr: &Bound<'_, PyAny>,
        sense: &str,
        rhs: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<Vec<PyConstraint>> {
        let (exprs, sense, rhs) = if let Ok(array) = expr.extract::<PyRef<'_, PyConstraintArray>>()
        {
            if rhs.is_some() || !sense.eq_ignore_ascii_case("ge") {
                return Err(errors::ConstraintSenseError::new_err(
                    "sense/rhs are not supported for comparison arrays",
                ));
            }
            (
                array.exprs().to_vec(),
                array.get_sense(),
                array.get_rhs().to_vec(),
            )
        } else if let Ok(array) = expr.extract::<PyRef<'_, PyVariableArray>>() {
            let sense = parse_comparison_sense(sense)?;
            let rhs = rhs.ok_or_else(|| {
                errors::ConstraintBoundsMissingError::new_err("rhs is required for add_constraints")
            })?;
            let constraints = if let Ok(index_set) = rhs.extract::<PyRef<'_, PyIndexSet>>() {
                array.core.compare_index_set(&index_set, sense)?
            } else {
                let value = rhs.extract::<f64>()?;
                array.core.compare_scalar(value, sense)
            };
            (
                constraints.exprs().to_vec(),
                constraints.get_sense(),
                constraints.get_rhs().to_vec(),
            )
        } else if let Ok(array) = expr.extract::<PyRef<'_, PyExprArray>>() {
            let sense = parse_comparison_sense(sense)?;
            let rhs = rhs.ok_or_else(|| {
                errors::ConstraintBoundsMissingError::new_err("rhs is required for add_constraints")
            })?;
            let constraints = if let Ok(index_set) = rhs.extract::<PyRef<'_, PyIndexSet>>() {
                array.core.compare_index_set(&index_set, sense)?
            } else {
                let value = rhs.extract::<f64>()?;
                array.core.compare_scalar(value, sense)
            };
            (
                constraints.exprs().to_vec(),
                constraints.get_sense(),
                constraints.get_rhs().to_vec(),
            )
        } else {
            return Err(errors::ConstraintTypeError::new_err(
                "expected ConstraintArray, VariableArray, or ExprArray",
            ));
        };

        let total = exprs.len();
        let mut constraints = Vec::with_capacity(total);
        for (index, expr) in exprs.into_iter().enumerate() {
            let con_bounds = bounds_from_sense(sense, rhs[index]);
            let constraint_id = self
                .inner
                .add_expr_constraint(expr.into_inner(), con_bounds)
                .map_err(errors::model_error_to_py)?;
            let con_name = name.as_ref().map(|base| {
                if total == 1 {
                    base.clone()
                } else {
                    format!("{base}[{index}]")
                }
            });
            if let Some(ref label) = con_name {
                self.inner
                    .set_constraint_name(constraint_id, label.clone())
                    .map_err(errors::model_error_to_py)?;
            }
            constraints.push(PyConstraint::new(
                constraint_id.inner(),
                con_name,
                con_bounds,
            ));
        }

        Ok(constraints)
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
        let constraint_id = extract_constraint_id(constraint)?;

        let handle = slf
            .borrow_mut()
            .inner
            .add_slack(constraint_id, parsed_bound, penalty, name.clone())
            .map_err(errors::model_error_to_py)?;

        // Get the PyConstraint reference for identity preservation
        let py_constraint: Py<PyConstraint> =
            if let Ok(con) = constraint.extract::<Py<PyConstraint>>() {
                con
            } else {
                // Fallback: create a new PyConstraint if a raw u32 was passed
                let con = PyConstraint::new(
                    constraint_id.inner(),
                    None,
                    Bounds::new(f64::NEG_INFINITY, f64::INFINITY),
                );
                Py::new(constraint.py(), con)?
            };

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
            let constraint_id = extract_constraint_id(con_any)?;

            let handle = slf
                .borrow_mut()
                .inner
                .add_slack(constraint_id, parsed_bound, *pen, name.clone())
                .map_err(errors::model_error_to_py)?;

            let py_constraint: Py<PyConstraint> =
                if let Ok(con) = con_any.extract::<Py<PyConstraint>>() {
                    con
                } else {
                    let con = PyConstraint::new(
                        constraint_id.inner(),
                        None,
                        Bounds::new(f64::NEG_INFINITY, f64::INFINITY),
                    );
                    Py::new(py, con)?
                };

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
        let linear_expr = extract_expr(expr)?;
        let (expr, _offset) = linear_expr.into_parts();
        self.inner
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: expr.into_linear_terms(),
            })
            .map_err(errors::model_error_to_py)?;
        self.inner
            .set_objective_name(name)
            .map_err(errors::model_error_to_py)?;
        Ok(())
    }

    /// Maximize a linear expression.
    #[pyo3(signature = (expr, *, name=None))]
    fn maximize(&mut self, expr: &Bound<'_, PyAny>, name: Option<String>) -> PyResult<()> {
        let linear_expr = extract_expr(expr)?;
        let (expr, _offset) = linear_expr.into_parts();
        self.inner
            .set_objective(Objective {
                sense: Some(Sense::Maximize),
                terms: expr.into_linear_terms(),
            })
            .map_err(errors::model_error_to_py)?;
        self.inner
            .set_objective_name(name)
            .map_err(errors::model_error_to_py)?;
        Ok(())
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

        let solver_backend = if let Some(s) = solver {
            resolve_solver_backend(Some(s))?
        } else if self.use_xpress {
            SolverBackend::Xpress(self.solver_settings.clone())
        } else {
            SolverBackend::HiGHS(self.solver_settings.clone())
        };

        let result = match solver_backend {
            SolverBackend::Xpress(_settings) => Err(errors::SolverInternalError::new_err(
                "Xpress backend is not enabled in this build",
            )),
            SolverBackend::HiGHS(settings) => {
                let settings = settings.with_overrides(overrides)?;
                let mut highs = arco_highs::Solver::new(self.inner.clone())
                    .map_err(errors::solver_error_to_py)?;
                settings.apply_highs(&mut highs);

                if let Some(ref hints) = hints {
                    highs
                        .set_primal_start(hints)
                        .map_err(errors::solver_error_to_py)?;
                }

                match highs.solve() {
                    Ok(solution) => Ok(PySolveResult::new(solution.into_core_solution())),
                    Err(arco_core::SolverError::SolveFailure { status }) => {
                        Ok(PySolveResult::new(solve_failure_solution(status)))
                    }
                    Err(e) => Err(errors::solver_error_to_py(e)),
                }
            }
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
                let name = self.inner.get_variable_name(var_id).map(|s| s.to_string());
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
            .inner
            .get_variable_by_name(name)
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
        let num_constraints = self.inner.num_constraints();
        let num_variables = self.inner.num_variables();

        let mut col_ptrs = vec![0usize];
        let mut row_indices = Vec::new();
        let mut values = Vec::new();

        for (_var_id, coeffs) in self.inner.columns() {
            for (constraint_id, coeff) in coeffs {
                row_indices.push(constraint_id.inner());
                values.push(*coeff);
            }
            col_ptrs.push(row_indices.len());
        }

        let dict = PyDict::new(py);
        dict.set_item("col_ptrs", col_ptrs)?;
        dict.set_item("row_indices", row_indices)?;
        dict.set_item("values", values)?;
        dict.set_item("shape", (num_constraints as u32, num_variables as u32))?;

        Ok(dict.unbind().into())
    }

    /// Export CRS matrix in a sparse-matrix compatible format.
    ///
    /// Returns dict with keys:
    /// - row_ptrs: list of row pointers (length = num_constraints + 1)
    /// - col_indices: list of column indices
    /// - values: list of non-zero values
    /// - shape: tuple (num_constraints, num_variables)
    fn export_crs(&self, py: Python<'_>) -> PyResult<PyObject> {
        let num_constraints = self.inner.num_constraints();
        let num_variables = self.inner.num_variables();
        let rows = self.inner.rows();

        let mut row_ptrs = vec![0usize];
        let mut col_indices = Vec::new();
        let mut values = Vec::new();

        for row in &rows {
            for (var_id, coeff) in row {
                col_indices.push(var_id.inner());
                values.push(*coeff);
            }
            row_ptrs.push(col_indices.len());
        }

        let dict = PyDict::new(py);
        dict.set_item("row_ptrs", row_ptrs)?;
        dict.set_item("col_indices", col_indices)?;
        dict.set_item("values", values)?;
        dict.set_item("shape", (num_constraints as u32, num_variables as u32))?;

        Ok(dict.unbind().into())
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
        let id = VariableId::new(var_id);
        self.inner.get_variable_name(id).map(|s| s.to_string())
    }

    /// Lookup a variable by name.
    #[pyo3(signature = (name, /))]
    fn get_variable_by_name(&self, name: String) -> Option<u32> {
        self.inner.get_variable_by_name(&name).map(|id| id.inner())
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

/// Which solver backend to use when solving.
enum SolverBackend {
    HiGHS(SolverSettings),
    Xpress(SolverSettings),
}

/// Extract `SolverSettings` from an optional Python solver object (`HiGHS`, `Xpress`, or `Solver`).
fn extract_solver_settings(solver: Option<&Bound<'_, PyAny>>) -> PyResult<SolverSettings> {
    let Some(solver) = solver else {
        return Ok(SolverSettings::default());
    };
    if let Ok(highs) = solver.cast::<PyHiGHS>() {
        return Ok(highs.borrow().into_super().settings.clone());
    }
    if let Ok(xpress) = solver.cast::<PyXpress>() {
        return Ok(xpress.borrow().into_super().settings.clone());
    }
    if let Ok(base) = solver.cast::<PySolver>() {
        return Ok(base.borrow().settings.clone());
    }
    Err(errors::SolverTypeError::new_err(
        "solver must be a Solver, HiGHS, or Xpress instance",
    ))
}

/// Determine the solver backend from the `solver` parameter passed to `solve()`.
fn resolve_solver_backend(solver: Option<&Bound<'_, PyAny>>) -> PyResult<SolverBackend> {
    let Some(solver) = solver else {
        return Ok(SolverBackend::HiGHS(SolverSettings::default()));
    };
    if let Ok(xpress) = solver.cast::<PyXpress>() {
        let settings = xpress.borrow().into_super().settings.clone();
        return Ok(SolverBackend::Xpress(settings));
    }
    if let Ok(highs) = solver.cast::<PyHiGHS>() {
        let settings = highs.borrow().into_super().settings.clone();
        return Ok(SolverBackend::HiGHS(settings));
    }
    if let Ok(base) = solver.cast::<PySolver>() {
        let settings = base.borrow().settings.clone();
        return Ok(SolverBackend::HiGHS(settings));
    }
    Err(errors::SolverTypeError::new_err(
        "solver must be a Solver, HiGHS, or Xpress instance",
    ))
}

/// Extract a `ConstraintId` from a Python object that may be a `PyConstraint` or `u32`.
fn extract_constraint_id(ob: &Bound<'_, PyAny>) -> PyResult<ConstraintId> {
    if let Ok(con) = ob.extract::<PyRef<'_, PyConstraint>>() {
        return Ok(ConstraintId::new(con.constraint_id));
    }
    if let Ok(id) = ob.extract::<u32>() {
        return Ok(ConstraintId::new(id));
    }
    Err(errors::ConstraintTypeError::new_err(
        "expected a Constraint or integer constraint ID",
    ))
}

/// Extract a `PyExpr` from a Python object that may be a `PyExpr`, `PyVariable`, or scalar.
fn extract_expr(ob: &Bound<'_, PyAny>) -> PyResult<PyExpr> {
    Ok(ob.extract::<crate::expr::ExprLike>()?.0)
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

fn parse_comparison_sense(sense: &str) -> PyResult<ComparisonSense> {
    match sense.to_lowercase().as_str() {
        "ge" | ">=" => Ok(ComparisonSense::GreaterEqual),
        "le" | "<=" => Ok(ComparisonSense::LessEqual),
        "eq" | "==" => Ok(ComparisonSense::Equal),
        _ => Err(errors::ConstraintSenseError::new_err(format!(
            "Invalid sense '{sense}' (expected 'ge', 'le', or 'eq')",
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
