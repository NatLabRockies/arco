//! Python wrapper for SlackVariable objects returned by add_slack/add_slacks.

use arco_core::slack::SlackVariables;
use pyo3::prelude::*;

use crate::PyObject;
use crate::constraint::PyConstraint;

/// A slack variable returned by `add_slack()`.
///
/// Wraps the underlying slack variable IDs with cached metadata (constraint, bound, penalty, name).
/// The `.value` property is available after solve and returns the slack amount used.
#[pyclass(name = "SlackVariable")]
pub struct PySlackVariable {
    /// The constraint this slack softens (stored as a Python reference for identity).
    constraint: Py<PyConstraint>,
    /// Which bound is relaxed: "lower", "upper", or "both".
    bound_str: String,
    /// The penalty coefficient.
    penalty: f64,
    /// Optional name for the slack variable.
    name_str: Option<String>,
    /// Internal variable IDs for the slack variables (lower and/or upper).
    var_ids: SlackVariables,
    /// Reference to the model for accessing primal values after solve.
    model: PyObject,
}

impl PySlackVariable {
    pub fn new(
        constraint: Py<PyConstraint>,
        bound_str: String,
        penalty: f64,
        name_str: Option<String>,
        var_ids: SlackVariables,
        model: PyObject,
    ) -> Self {
        Self {
            constraint,
            bound_str,
            penalty,
            name_str,
            var_ids,
            model,
        }
    }
}

#[pymethods]
impl PySlackVariable {
    /// The constraint this slack softens.
    #[getter]
    fn constraint(&self, py: Python<'_>) -> Py<PyConstraint> {
        self.constraint.clone_ref(py)
    }

    /// Which bound is relaxed: "lower", "upper", or "both".
    #[getter]
    fn bound(&self) -> &str {
        &self.bound_str
    }

    /// The penalty coefficient.
    #[getter]
    fn penalty(&self) -> f64 {
        self.penalty
    }

    /// The slack variable name.
    #[getter]
    fn name(&self) -> Option<&str> {
        self.name_str.as_deref()
    }

    /// The slack amount used (available after solve).
    ///
    /// For "lower" or "upper" bound, returns the primal value of the single slack variable.
    /// For "both", returns the sum of lower and upper slack variable values.
    #[getter]
    fn value(&self, py: Python<'_>) -> PyResult<f64> {
        // Access the model's last_solution to get primal values
        let model = self.model.bind(py);
        let last_solution = model.getattr("_last_solution")?;
        if last_solution.is_none() {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "SlackVariable.value is only available after solve()",
            ));
        }
        let primal_values: Vec<f64> = last_solution.getattr("primal_values")?.extract()?;

        let mut total = 0.0;
        if let Some(lower_id) = self.var_ids.lower {
            let idx = lower_id.inner() as usize;
            if idx < primal_values.len() {
                total += primal_values[idx];
            }
        }
        if let Some(upper_id) = self.var_ids.upper {
            let idx = upper_id.inner() as usize;
            if idx < primal_values.len() {
                total += primal_values[idx];
            }
        }
        Ok(total)
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        let con = self.constraint.borrow(py);
        let con_name = con
            .name
            .as_deref()
            .map_or_else(|| con.constraint_id.to_string(), |n| format!("'{}'", n));
        let name_part = self
            .name_str
            .as_deref()
            .map(|n| format!(", name='{}'", n))
            .unwrap_or_default();
        format!(
            "SlackVariable(constraint={}, bound='{}', penalty={}{})",
            con_name, self.bound_str, self.penalty, name_part
        )
    }
}

/// Register SlackVariable class with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySlackVariable>()?;
    Ok(())
}
