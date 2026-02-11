//! Python wrapper for solver solutions.

use crate::errors::SolverIndexError;
use arco_core::solver::{Solution, SolverStatus};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::PyObject;
use crate::arrays::PyVariableArray;
use crate::constraint::PyConstraint;
use crate::variable::PyVariable;

/// Python enum for solution status.
#[pyclass(from_py_object, name = "SolutionStatus", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum PySolutionStatus {
    OPTIMAL,
    INFEASIBLE,
    UNBOUNDED,
    TIME_LIMIT,
    ERROR,
}

impl From<SolverStatus> for PySolutionStatus {
    fn from(status: SolverStatus) -> Self {
        match status {
            SolverStatus::Optimal => PySolutionStatus::OPTIMAL,
            SolverStatus::Infeasible => PySolutionStatus::INFEASIBLE,
            SolverStatus::Unbounded => PySolutionStatus::UNBOUNDED,
            SolverStatus::TimeLimit => PySolutionStatus::TIME_LIMIT,
            SolverStatus::IterationLimit => PySolutionStatus::TIME_LIMIT,
            SolverStatus::Unknown => PySolutionStatus::ERROR,
        }
    }
}

/// Python wrapper for a solver solution result.
#[pyclass(name = "SolveResult")]
pub struct PySolveResult {
    inner: Solution,
    /// Per-block results for composed models; None for simple models.
    blocks_ref: Option<PyObject>,
}

impl PySolveResult {
    pub fn new(inner: Solution) -> Self {
        Self {
            inner,
            blocks_ref: None,
        }
    }

    pub fn with_blocks(inner: Solution, blocks: PyObject) -> Self {
        Self {
            inner,
            blocks_ref: Some(blocks),
        }
    }

    /// Access the inner solution (for use by solution_summary).
    pub fn inner(&self) -> &Solution {
        &self.inner
    }
}

#[pymethods]
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
    fn get_value(&self, py: Python<'_>, variable: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(var) = variable.extract::<PyRef<'_, PyVariable>>() {
            let index = var.var_id as usize;
            let value = self.inner.get_primal(index).ok_or_else(|| {
                PyRuntimeError::new_err(format!(
                    "Variable index {} out of bounds for {} primal values",
                    index,
                    self.inner.primal_values.len()
                ))
            })?;
            return Ok(value.into_pyobject(py)?.into_any().unbind());
        }
        if let Ok(arr) = variable.extract::<PyRef<'_, PyVariableArray>>() {
            let variables = arr.get_variable_refs();
            let shape = arr.get_shape();
            let mut values = Vec::with_capacity(variables.len());
            for var in variables {
                let index = var.var_id as usize;
                let value = self.inner.get_primal(index).ok_or_else(|| {
                    PyRuntimeError::new_err(format!(
                        "Variable index {} out of bounds for {} primal values",
                        index,
                        self.inner.primal_values.len()
                    ))
                })?;
                values.push(value);
            }
            // Create numpy array with the correct shape
            let np = py.import("numpy")?;
            let flat = PyList::new(py, &values)?;
            let array = np.call_method1("array", (flat,))?;
            let shape_tuple = PyList::new(py, shape)?;
            let reshaped = array.call_method1("reshape", (shape_tuple,))?;
            return Ok(reshaped.unbind());
        }
        Err(PyRuntimeError::new_err(
            "get_value() expects a Variable or VariableArray",
        ))
    }

    /// Get the dual value (shadow price) for a constraint.
    #[pyo3(signature = (constraint))]
    fn get_dual(&self, constraint: PyRef<'_, PyConstraint>) -> PyResult<f64> {
        let index = constraint.constraint_id as usize;
        self.inner.get_constraint_dual(index).ok_or_else(|| {
            PyRuntimeError::new_err(format!(
                "Constraint index {} out of bounds for {} constraint duals",
                index,
                self.inner.constraint_duals.len()
            ))
        })
    }

    /// Get the reduced cost for a variable.
    #[pyo3(signature = (variable))]
    fn get_reduced_cost(&self, variable: PyRef<'_, PyVariable>) -> PyResult<f64> {
        let index = variable.var_id as usize;
        self.inner.get_variable_dual(index).ok_or_else(|| {
            PyRuntimeError::new_err(format!(
                "Variable index {} out of bounds for {} variable duals",
                index,
                self.inner.variable_duals.len()
            ))
        })
    }

    /// Get the slack value for a constraint.
    ///
    /// Slack is the distance from the constraint activity to the nearest bound:
    /// - For `expr <= ub`: slack = ub - activity
    /// - For `expr >= lb`: slack = activity - lb
    /// - For ranged constraints: min(ub - activity, activity - lb)
    #[pyo3(signature = (constraint))]
    fn get_slack(&self, constraint: PyRef<'_, PyConstraint>) -> PyResult<f64> {
        let index = constraint.constraint_id as usize;
        let activity = self.inner.get_row_value(index).ok_or_else(|| {
            PyRuntimeError::new_err(format!(
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
            // Free constraint (no bounds) — slack is infinite
            f64::INFINITY
        };
        Ok(slack)
    }

    // ── Legacy API (kept for backward compatibility) ─────────────────

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
    fn get_primal(&self, index: usize) -> PyResult<f64> {
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
    fn get_variable_dual(&self, index: usize) -> PyResult<f64> {
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
    fn get_constraint_dual(&self, index: usize) -> PyResult<f64> {
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

// ── solution_summary formatting helpers ──────────────────────────────

fn format_solve_time(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.2}ms", seconds * 1000.0)
    } else {
        format!("{:.2}s", seconds)
    }
}

fn format_sci(val: f64) -> String {
    if val.is_nan() {
        return "NaN".to_string();
    }
    if val.is_infinite() {
        return if val > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    // Rust's {:.5e} omits the + sign on exponents. We want e.g. 2.60000e+03.
    let s = format!("{:.5e}", val);
    // Replace "e" with proper sign-padded exponent
    if let Some(pos) = s.rfind('e') {
        let mantissa = &s[..pos];
        let exp_str = &s[pos + 1..];
        let exp: i32 = exp_str.parse().unwrap_or(0);
        format!("{}e{:+03}", mantissa, exp)
    } else {
        s
    }
}

fn status_str(status: SolverStatus) -> &'static str {
    match status {
        SolverStatus::Optimal => "OPTIMAL",
        SolverStatus::Infeasible => "INFEASIBLE",
        SolverStatus::Unbounded => "UNBOUNDED",
        SolverStatus::TimeLimit => "TIME_LIMIT",
        SolverStatus::IterationLimit => "ITERATION_LIMIT",
        SolverStatus::Unknown => "ERROR",
    }
}

/// Pretty-print a tree-formatted solution summary.
#[pyfunction]
#[pyo3(signature = (result, *, verbose=false))]
pub fn solution_summary(
    py: Python<'_>,
    result: PyRef<'_, PySolveResult>,
    verbose: bool,
) -> PyResult<()> {
    let sol = result.inner();
    let mut lines = Vec::new();

    lines.push("Solution Summary".to_string());

    // Solver line
    lines.push("\u{251c} solver          : HiGHS".to_string());

    // Termination section
    let is_last_section = !verbose;
    let term_prefix = if is_last_section {
        "\u{2514}"
    } else {
        "\u{251c}"
    };
    let term_cont = if is_last_section { " " } else { "\u{2502}" };
    lines.push(format!("{} Termination", term_prefix));
    lines.push(format!(
        "{} \u{251c} status        : {}",
        term_cont,
        status_str(sol.status)
    ));
    lines.push(format!(
        "{} \u{2514} objective     : {}",
        term_cont,
        format_sci(sol.objective_value)
    ));

    if verbose {
        // Solution section
        lines.push("\u{251c} Solution".to_string());

        // Variable values
        let has_duals = !sol.constraint_duals.is_empty();
        let values_prefix = if has_duals { "\u{251c}" } else { "\u{2514}" };
        lines.push(format!("\u{2502} {} values", values_prefix));

        let val_cont = if has_duals { "\u{2502}" } else { " " };
        let num_vals = sol.primal_values.len();
        for (i, val) in sol.primal_values.iter().enumerate() {
            let is_last = i + 1 == num_vals;
            let branch = if is_last { "\u{2514}" } else { "\u{251c}" };
            lines.push(format!(
                "\u{2502} {}  {} x{:<12}: {}",
                val_cont,
                branch,
                i,
                format_sci(*val)
            ));
        }

        // Constraint duals
        if has_duals {
            lines.push("\u{2502} \u{2514} duals".to_string());
            let num_duals = sol.constraint_duals.len();
            for (i, val) in sol.constraint_duals.iter().enumerate() {
                let is_last = i + 1 == num_duals;
                let branch = if is_last { "\u{2514}" } else { "\u{251c}" };
                lines.push(format!(
                    "\u{2502}   {} c{:<12}: {}",
                    branch,
                    i,
                    format_sci(*val)
                ));
            }
        }

        // Work section
        let iterations = sol
            .metadata
            .get("simplex_iterations")
            .copied()
            .unwrap_or(0.0) as u64
            + sol
                .metadata
                .get("barrier_iterations")
                .copied()
                .unwrap_or(0.0) as u64;
        let nodes = sol.metadata.get("nodes").copied().unwrap_or(0.0) as u64;

        lines.push("\u{2514} Work".to_string());
        lines.push(format!(
            "  \u{251c} solve_time    : {}",
            format_solve_time(sol.solve_time_seconds)
        ));
        lines.push(format!("  \u{251c} iterations    : {}", iterations));
        lines.push(format!("  \u{2514} nodes         : {}", nodes));
    }

    let output = lines.join("\n");
    let builtins = py.import("builtins")?;
    builtins.call_method1("print", (output,))?;
    Ok(())
}

/// Register solution classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySolveResult>()?;
    m.add_class::<PySolutionStatus>()?;
    m.add_function(pyo3::wrap_pyfunction!(solution_summary, m)?)?;
    Ok(())
}
