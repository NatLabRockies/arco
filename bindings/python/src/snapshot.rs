//! Python wrappers for model snapshot types.

use arco_model::Sense;
use pyo3::prelude::*;

use crate::py_modules::views::{
    PyCoefficientView, PyConstraintView, PyObjectiveView, PySlackView, PyVariableView,
};

/// Conservative memory estimate for sparse matrix storage.
#[pyo3_macros::pyclass(from_py_object, name = "SnapshotMemoryEstimate")]
#[derive(Clone)]
pub struct PySnapshotMemoryEstimate {
    pub coefficient_value_bytes: usize,
    pub coefficient_index_bytes: usize,
    pub variable_column_pointer_bytes: usize,
    pub sparse_matrix_bytes: usize,
}

#[pyo3_macros::pymethods]
impl PySnapshotMemoryEstimate {
    #[getter]
    fn coefficient_value_bytes(&self) -> usize {
        self.coefficient_value_bytes
    }

    #[getter]
    fn coefficient_index_bytes(&self) -> usize {
        self.coefficient_index_bytes
    }

    #[getter]
    fn variable_column_pointer_bytes(&self) -> usize {
        self.variable_column_pointer_bytes
    }

    #[getter]
    fn sparse_matrix_bytes(&self) -> usize {
        self.sparse_matrix_bytes
    }
}

/// Metadata about a model snapshot.
#[pyo3_macros::pyclass(from_py_object, name = "SnapshotMetadata")]
#[derive(Clone)]
pub struct PySnapshotMetadata {
    pub variables: usize,
    pub constraints: usize,
    pub coefficients: usize,
    pub memory: PySnapshotMemoryEstimate,
}

#[pyo3_macros::pymethods]
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

    #[getter]
    fn memory(&self) -> PySnapshotMemoryEstimate {
        self.memory.clone()
    }
}

/// A snapshot of a model's state.
#[pyo3_macros::pyclass(from_py_object, name = "ModelSnapshot")]
#[derive(Clone)]
pub struct PyModelSnapshot {
    pub variables: Vec<PyVariableView>,
    pub constraints: Vec<PyConstraintView>,
    pub coefficients: Option<Vec<PyCoefficientView>>,
    pub objective: Option<PyObjectiveView>,
    pub slacks: Option<Vec<PySlackView>>,
    pub metadata: PySnapshotMetadata,
}

#[pyo3_macros::pymethods]
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
    pub fn from_snapshot(_py: Python<'_>, snapshot: arco_model::ModelSnapshot) -> PyResult<Self> {
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
                memory: PySnapshotMemoryEstimate {
                    coefficient_value_bytes: snapshot.metadata.memory.coefficient_value_bytes,
                    coefficient_index_bytes: snapshot.metadata.memory.coefficient_index_bytes,
                    variable_column_pointer_bytes: snapshot
                        .metadata
                        .memory
                        .variable_column_pointer_bytes,
                    sparse_matrix_bytes: snapshot.metadata.memory.sparse_matrix_bytes,
                },
            },
        })
    }
}

/// Register snapshot classes with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySnapshotMemoryEstimate>()?;
    m.add_class::<PySnapshotMetadata>()?;
    m.add_class::<PyModelSnapshot>()?;
    Ok(())
}
