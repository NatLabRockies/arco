//! Python wrappers for model snapshot types.

use arco_core::Sense;
use pyo3::prelude::*;

use crate::views::{
    PyCoefficientView, PyConstraintView, PyObjectiveView, PySlackView, PyVariableView,
};

/// Metadata about a model snapshot.
#[pyclass(from_py_object, name = "SnapshotMetadata")]
#[derive(Clone)]
pub struct PySnapshotMetadata {
    pub variables: usize,
    pub constraints: usize,
    pub coefficients: usize,
}

#[pymethods]
impl PySnapshotMetadata {
    #[getter]
    fn variables(&self) -> usize {
        self.variables
    }

    #[getter]
    fn constraints(&self) -> usize {
        self.constraints
    }

    #[getter]
    fn coefficients(&self) -> usize {
        self.coefficients
    }
}

/// A snapshot of a model's state.
#[pyclass(from_py_object, name = "ModelSnapshot")]
#[derive(Clone)]
pub struct PyModelSnapshot {
    pub variables: Vec<PyVariableView>,
    pub constraints: Vec<PyConstraintView>,
    pub coefficients: Option<Vec<PyCoefficientView>>,
    pub objective: Option<PyObjectiveView>,
    pub slacks: Option<Vec<PySlackView>>,
    pub metadata: PySnapshotMetadata,
}

#[pymethods]
impl PyModelSnapshot {
    #[getter]
    fn variables(&self) -> Vec<PyVariableView> {
        self.variables.clone()
    }

    #[getter]
    fn constraints(&self) -> Vec<PyConstraintView> {
        self.constraints.clone()
    }

    #[getter]
    fn coefficients(&self) -> Option<Vec<PyCoefficientView>> {
        self.coefficients.clone()
    }

    #[getter]
    fn objective(&self) -> Option<PyObjectiveView> {
        self.objective.clone()
    }

    #[getter]
    fn slacks(&self) -> Option<Vec<PySlackView>> {
        self.slacks.clone()
    }

    #[getter]
    fn metadata(&self) -> PySnapshotMetadata {
        self.metadata.clone()
    }
}

impl PyModelSnapshot {
    pub fn from_snapshot(_py: Python<'_>, snapshot: arco_core::ModelSnapshot) -> PyResult<Self> {
        let variables = snapshot
            .variables
            .into_iter()
            .map(|v| PyVariableView {
                id: v.id.inner(),
                name: v.name,
                bounds: v.bounds,
                is_integer: v.is_integer,
                is_active: v.is_active,
                metadata: v.metadata,
            })
            .collect();

        let constraints = snapshot
            .constraints
            .into_iter()
            .map(|c| PyConstraintView {
                id: c.id.inner(),
                name: c.name,
                bounds: c.bounds,
                nnz: c.nnz,
                metadata: c.metadata,
            })
            .collect();

        let coefficients = snapshot.coefficients.map(|coeffs| {
            coeffs
                .into_iter()
                .map(|c| PyCoefficientView {
                    variable_id: c.variable_id.inner(),
                    constraint_id: c.constraint_id.inner(),
                    value: c.value,
                })
                .collect()
        });

        let objective = snapshot.objective.map(|obj| PyObjectiveView {
            sense: obj.sense.map(|s| match s {
                Sense::Minimize => "MINIMIZE".to_string(),
                Sense::Maximize => "MAXIMIZE".to_string(),
            }),
            terms: obj
                .terms
                .into_iter()
                .map(|(id, c)| (id.inner(), c))
                .collect(),
            name: obj.name,
        });

        let slacks = snapshot.slacks.map(|views| {
            views
                .into_iter()
                .map(|v| PySlackView {
                    constraint_id: v.constraint_id.inner(),
                    bound: v.bound.as_str().to_string(),
                    penalty: v.penalty,
                    lower_variable: v.variable_ids.lower.map(|id| id.inner()),
                    upper_variable: v.variable_ids.upper.map(|id| id.inner()),
                    name: v.name,
                })
                .collect()
        });

        Ok(PyModelSnapshot {
            variables,
            constraints,
            coefficients,
            objective,
            slacks,
            metadata: PySnapshotMetadata {
                variables: snapshot.metadata.variables,
                constraints: snapshot.metadata.constraints,
                coefficients: snapshot.metadata.coefficients,
            },
        })
    }
}

/// Register snapshot classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySnapshotMetadata>()?;
    m.add_class::<PyModelSnapshot>()?;
    Ok(())
}
