//! Python wrapper for solver solutions.

use crate::pym::errors::{SolverIndexError, SolverTypeError};
use arco_solver::{Solution, SolverStatus};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::types::{PyAny, PyBool, PyInt};

use crate::PyObject;
use crate::pym::arrays::PyVariableArray;
use crate::pym::constraint::PyConstraint;
use crate::pym::variable::PyVariable;

/// Python enum for solution status.
#[pyo3_macros::pyclass(from_py_object, name = "SolutionStatus", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PySolutionStatus {
    #[pyo3(name = "OPTIMAL")]
    Optimal,
    #[pyo3(name = "INFEASIBLE")]
    Infeasible,
    #[pyo3(name = "UNBOUNDED")]
    Unbounded,
    #[pyo3(name = "TIME_LIMIT")]
    TimeLimit,
    #[pyo3(name = "ERROR")]
    Error,
}

impl From<SolverStatus> for PySolutionStatus {
    fn from(status: SolverStatus) -> Self {
        match status {
            SolverStatus::Optimal => PySolutionStatus::Optimal,
            SolverStatus::Infeasible => PySolutionStatus::Infeasible,
            SolverStatus::Unbounded => PySolutionStatus::Unbounded,
            SolverStatus::TimeLimit => PySolutionStatus::TimeLimit,
            SolverStatus::IterationLimit => PySolutionStatus::TimeLimit,
            SolverStatus::Unknown => PySolutionStatus::Error,
        }
    }
}

/// Python wrapper for a solver solution result.
#[pyo3_macros::pyclass(name = "SolveResult")]
pub struct PySolveResult {
    inner: Solution,
    /// Per-block results for composed models; None for simple models.
    blocks_ref: Option<PyObject>,
}

impl PySolveResult {
    pub(crate) fn new(inner: Solution) -> Self {
        Self {
            inner,
            blocks_ref: None,
        }
    }

    pub(crate) fn with_blocks(inner: Solution, blocks: PyObject) -> Self {
        Self {
            inner,
            blocks_ref: Some(blocks),
        }
    }

    /// Access the inner solution (for use by solution_summary).
    pub fn inner(&self) -> &Solution {
        &self.inner
    }

    fn value_for_variable(
        &self,
        py: Python<'_>,
        variable: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        if let Ok(var) = variable.extract::<PyRef<'_, PyVariable>>() {
            let index = var.var_id as usize;
            let value = self.inner.get_primal(index).ok_or_else(|| {
                SolverIndexError::new_err(format!(
                    "Variable index {} out of bounds for {} primal values",
                    index,
                    self.inner.primal_values.len()
                ))
            })?;
            return Ok(value.into_pyobject(py)?.into_any().unbind());
        }
        if let Ok(arr) = variable.extract::<PyRef<'_, PyVariableArray>>() {
            let variable_slots = arr.get_variable_slots();
            let shape = arr.get_shape();
            let mut values = Vec::with_capacity(variable_slots.len());
            for slot in variable_slots {
                if let Some(var) = slot {
                    let index = var.var_id as usize;
                    let value = self.inner.get_primal(index).ok_or_else(|| {
                        SolverIndexError::new_err(format!(
                            "Variable index {} out of bounds for {} primal values",
                            index,
                            self.inner.primal_values.len()
                        ))
                    })?;
                    values.push(value);
                } else {
                    values.push(f64::NAN);
                }
            }
            let np = py.import("numpy")?;
            let flat = PyList::new(py, &values)?;
            let array = np.call_method1("array", (flat,))?;
            let shape_tuple = PyList::new(py, shape)?;
            let reshaped = array.call_method1("reshape", (shape_tuple,))?;
            return Ok(reshaped.unbind());
        }
        Err(SolverTypeError::new_err(
            "value() expects a Variable or VariableArray",
        ))
    }

    fn dual_for_constraint(&self, constraint: PyRef<'_, PyConstraint>) -> PyResult<f64> {
        let index = constraint.constraint_id as usize;
        self.inner.get_constraint_dual(index).ok_or_else(|| {
            SolverIndexError::new_err(format!(
                "Constraint index {} out of bounds for {} constraint duals",
                index,
                self.inner.constraint_duals.len()
            ))
        })
    }

    fn reduced_cost_for_variable(&self, variable: PyRef<'_, PyVariable>) -> PyResult<f64> {
        let index = variable.var_id as usize;
        self.inner.get_variable_dual(index).ok_or_else(|| {
            SolverIndexError::new_err(format!(
                "Variable index {} out of bounds for {} variable duals",
                index,
                self.inner.variable_duals.len()
            ))
        })
    }

    fn slack_for_constraint(&self, constraint: PyRef<'_, PyConstraint>) -> PyResult<f64> {
        let index = constraint.constraint_id as usize;
        let activity = self.inner.get_row_value(index).ok_or_else(|| {
            SolverIndexError::new_err(format!(
                "Constraint index {} out of bounds for {} row values",
                index,
                self.inner.row_values.len()
            ))
        })?;
        let bounds = constraint.constraint_bounds;
        let lower = bounds.lower;
        let upper = bounds.upper;
        let slack = if lower.is_finite() && upper.is_finite() {
            // Ranged constraint: return minimum slack to nearest bound
            (upper - activity).min(activity - lower)
        } else if upper.is_finite() {
            // expr <= ub
            upper - activity
        } else if lower.is_finite() {
            // expr >= lb
            activity - lower
        } else {
            // Free constraint (no bounds): slack is infinite
            f64::INFINITY
        };
        Ok(slack)
    }

    fn parse_index(index: &Bound<'_, PyAny>) -> PyResult<usize> {
        if index.is_instance_of::<PyBool>() {
            return Err(SolverTypeError::new_err(
                "index must be an integer for raw result accessors",
            ));
        }
        let signed_any = index.call_method0("__index__").map_err(|_| {
            SolverTypeError::new_err("index must be an integer for raw result accessors")
        })?;
        if !signed_any.is_instance_of::<PyInt>() {
            return Err(SolverTypeError::new_err(
                "index must be an integer for raw result accessors",
            ));
        }
        if signed_any.is_instance_of::<PyBool>() {
            return Err(SolverTypeError::new_err(
                "index must be an integer for raw result accessors",
            ));
        }
        let signed = signed_any.extract::<isize>().map_err(|_| {
            SolverIndexError::new_err("Index out of bounds for raw result accessors")
        })?;
        if signed < 0 {
            return Err(SolverIndexError::new_err(format!(
                "Index {} out of bounds for raw result accessors",
                signed
            )));
        }
        Ok(signed as usize)
    }
}

#[pyo3_macros::pymethods]
impl PySolveResult {
    /// The status of the solve.
    #[getter]
    fn status(&self) -> PySolutionStatus {
        PySolutionStatus::from(self.inner.status)
    }

    /// The objective value of the solution.
    #[getter]
    fn objective_value(&self) -> f64 {
        self.inner.objective_value
    }

    /// Get the value of a variable or variable array from the solution.
    ///
    /// For a single Variable, returns a float.
    /// For a VariableArray, returns a numpy ndarray.
    #[pyo3(signature = (variable, /))]
    fn value(&self, py: Python<'_>, variable: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        self.value_for_variable(py, variable)
    }

    /// Get the dual value (shadow price) for a constraint.
    #[pyo3(signature = (constraint))]
    fn dual(&self, constraint: PyRef<'_, PyConstraint>) -> PyResult<f64> {
        self.dual_for_constraint(constraint)
    }

    /// Get the reduced cost for a variable.
    #[pyo3(signature = (variable))]
    fn reduced_cost(&self, variable: PyRef<'_, PyVariable>) -> PyResult<f64> {
        self.reduced_cost_for_variable(variable)
    }

    /// Get the slack value for a constraint.
    ///
    /// Slack is the distance from the constraint activity to the nearest bound:
    /// - For `expr <= ub`: slack = ub - activity
    /// - For `expr >= lb`: slack = activity - lb
    /// - For ranged constraints: min(ub - activity, activity - lb)
    #[pyo3(signature = (constraint))]
    fn slack(&self, constraint: PyRef<'_, PyConstraint>) -> PyResult<f64> {
        self.slack_for_constraint(constraint)
    }

    // Expert raw-vector accessors for integrations that already track solver order.

    /// Get primal values as a list.
    #[getter]
    fn primal_values(&self) -> Vec<f64> {
        self.inner.primal_values.clone()
    }

    /// Get variable dual values (reduced costs) as a list.
    #[getter]
    fn variable_duals(&self) -> Vec<f64> {
        self.inner.variable_duals.clone()
    }

    /// Get constraint dual values (shadow prices) as a list.
    #[getter]
    fn constraint_duals(&self) -> Vec<f64> {
        self.inner.constraint_duals.clone()
    }

    /// Get a specific primal value by index.
    #[pyo3(signature = (*, index))]
    fn get_primal(&self, index: &Bound<'_, PyAny>) -> PyResult<f64> {
        let index = Self::parse_index(index)?;
        let len = self.inner.primal_values.len();
        self.inner.get_primal(index).ok_or_else(|| {
            SolverIndexError::new_err(format!(
                "Index {} out of bounds for {} primal values",
                index, len
            ))
        })
    }

    /// Get a specific variable dual value by index.
    #[pyo3(signature = (*, index))]
    fn get_variable_dual(&self, index: &Bound<'_, PyAny>) -> PyResult<f64> {
        let index = Self::parse_index(index)?;
        let len = self.inner.variable_duals.len();
        self.inner.get_variable_dual(index).ok_or_else(|| {
            SolverIndexError::new_err(format!(
                "Index {} out of bounds for {} variable duals",
                index, len
            ))
        })
    }

    /// Get a specific constraint dual value by index.
    #[pyo3(signature = (*, index))]
    fn get_constraint_dual(&self, index: &Bound<'_, PyAny>) -> PyResult<f64> {
        let index = Self::parse_index(index)?;
        let len = self.inner.constraint_duals.len();
        self.inner.get_constraint_dual(index).ok_or_else(|| {
            SolverIndexError::new_err(format!(
                "Index {} out of bounds for {} constraint duals",
                index, len
            ))
        })
    }

    /// Get the number of primal values.
    fn num_primal_values(&self) -> usize {
        self.inner.primal_values.len()
    }

    /// Get the number of variable dual values.
    fn num_variable_duals(&self) -> usize {
        self.inner.variable_duals.len()
    }

    /// Get the number of constraint dual values.
    fn num_constraint_duals(&self) -> usize {
        self.inner.constraint_duals.len()
    }

    /// Check if solution is optimal.
    fn is_optimal(&self) -> bool {
        self.inner.is_optimal()
    }

    /// Check if solution is feasible (optimal, time limit, or iteration limit).
    fn is_feasible(&self) -> bool {
        self.inner.is_feasible()
    }

    /// Check if solution is infeasible.
    fn is_infeasible(&self) -> bool {
        self.inner.is_infeasible()
    }

    /// Check if solution is unbounded.
    fn is_unbounded(&self) -> bool {
        self.inner.is_unbounded()
    }

    /// Get solution status as a human-readable string.
    fn status_string(&self) -> &'static str {
        self.inner.status_string()
    }

    /// Get solve time in seconds.
    fn solve_time_seconds(&self) -> f64 {
        self.inner.solve_time_seconds
    }

    /// Numeric backend metadata, including optional timings and matrix stats.
    #[getter]
    fn metadata(&self) -> std::collections::BTreeMap<String, f64> {
        self.inner.metadata.clone()
    }

    /// Get number of simplex iterations (from metadata, 0 if not available).
    fn simplex_iterations(&self) -> u64 {
        self.inner
            .metadata
            .get("simplex_iterations")
            .copied()
            .unwrap_or(0.0) as u64
    }

    /// Get number of barrier iterations (from metadata, 0 if not available).
    fn barrier_iterations(&self) -> u64 {
        self.inner
            .metadata
            .get("barrier_iterations")
            .copied()
            .unwrap_or(0.0) as u64
    }

    /// Get total iterations (simplex + barrier).
    fn total_iterations(&self) -> u64 {
        self.simplex_iterations() + self.barrier_iterations()
    }

    /// Get relative MIP gap (from metadata, 0.0 if not available).
    fn mip_gap(&self) -> f64 {
        self.inner.metadata.get("mip_gap").copied().unwrap_or(0.0)
    }

    /// Get primal feasibility tolerance achieved (from metadata, 0.0 if not available).
    fn primal_feasibility_tolerance(&self) -> f64 {
        self.inner
            .metadata
            .get("primal_feasibility_tolerance")
            .copied()
            .unwrap_or(0.0)
    }

    /// Get dual feasibility tolerance achieved (from metadata, 0.0 if not available).
    fn dual_feasibility_tolerance(&self) -> f64 {
        self.inner
            .metadata
            .get("dual_feasibility_tolerance")
            .copied()
            .unwrap_or(0.0)
    }

    /// Access per-block results for composed models.
    /// Returns None for simple (non-composed) models.
    #[getter]
    fn blocks(&self, py: Python<'_>) -> Option<PyObject> {
        self.blocks_ref.as_ref().map(|b| b.clone_ref(py))
    }

    fn __repr__(&self) -> String {
        format!(
            "SolveResult(status={:?}, objective_value={})",
            PySolutionStatus::from(self.inner.status),
            self.inner.objective_value
        )
    }
}

/// Register solution classes with the Python module.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySolveResult>()?;
    m.add_class::<PySolutionStatus>()?;
    Ok(())
}
