//! Storage access methods for the model.

use crate::types::{Constraint, Variable};
use arco_expr::ids::{ConstraintId, VariableId};

use crate::model::Model;
use crate::model::error::ModelError;

fn empty_column() -> &'static [(ConstraintId, f64)] {
    &[]
}

impl Model {
    #[inline]
    fn column_slice(&self, var_id: VariableId) -> &[(ConstraintId, f64)] {
        match self.columns.get(&var_id) {
            Some(column) => column.as_slice(),
            None => empty_column(),
        }
    }

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
        self.columns.values().map(|coeffs| coeffs.len()).sum()
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
        (0..self.variables.len()).map(move |idx| {
            let var_id = VariableId::new(idx as u32);
            (var_id, self.column_slice(var_id))
        })
    }

    /// Get the coefficient matrix in CRS (row-sparse-compressed) form.
    ///
    /// Returns a vector of rows, each containing (variable_id, coefficient) pairs.
    pub fn rows(&self) -> Vec<Vec<(VariableId, f64)>> {
        let mut rows = vec![Vec::new(); self.num_constraints()];
        for (var_id, coeffs) in self.columns() {
            for (constraint_id, coeff) in coeffs {
                let idx = constraint_id.inner() as usize;
                if let Some(row) = rows.get_mut(idx) {
                    row.push((var_id, *coeff));
                }
            }
        }
        rows
    }

    /// Get the coefficients for a specific variable (column)
    pub fn get_column(&self, var_id: VariableId) -> Option<&[(ConstraintId, f64)]> {
        if (var_id.inner() as usize) < self.variables.len() {
            Some(self.column_slice(var_id))
        } else {
            None
        }
    }
}
