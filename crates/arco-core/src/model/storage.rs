//! Storage access methods for the model.

use crate::types::{Constraint, Variable};
use arco_expr::ids::{ConstraintId, VariableId};

use crate::model::Model;
use crate::model::error::ModelError;

impl Model {
    /// Get the number of variables
    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    /// Get the number of constraints
    pub fn num_constraints(&self) -> usize {
        self.constraints.len()
    }

    /// Get the number of coefficients in the model.
    pub fn num_coefficients(&self) -> usize {
        self.columns.iter().map(|col| col.len()).sum()
    }

    /// Get a variable by ID.
    pub fn get_variable(&self, id: VariableId) -> Result<Variable, ModelError> {
        self.get_variable_by_index(id.inner() as usize)
            .ok_or(ModelError::InvalidVariableId(id))
    }

    /// Get a constraint by ID.
    pub fn get_constraint(&self, id: ConstraintId) -> Result<&Constraint, ModelError> {
        self.constraints
            .get(id.inner() as usize)
            .ok_or(ModelError::InvalidConstraintId(id))
    }

    /// Get the coefficient matrix in CSC (column-sparse-compressed) format
    ///
    /// Returns an iterator over columns, where each column contains (constraint_id, coefficient) pairs.
    /// This enables zero-copy access to the sparse matrix structure.
    pub fn columns(&self) -> impl Iterator<Item = (VariableId, &[(ConstraintId, f64)])> + '_ {
        self.columns
            .iter()
            .enumerate()
            .map(|(idx, col)| (VariableId::new(idx as u32), col.as_slice()))
    }

    /// Get the coefficients for a specific variable (column)
    pub fn get_column(&self, var_id: VariableId) -> Option<&[(ConstraintId, f64)]> {
        self.columns
            .get(var_id.inner() as usize)
            .map(|col| col.as_slice())
    }
}
