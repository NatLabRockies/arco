//! CSC format import for building models from sparse matrices.

use crate::types::{Bounds, Constraint, SimplifyLevel, Variable};
use arco_expr::ids::{ConstraintId, VariableId};

use crate::model::error::ModelError;
use crate::model::{ColumnData, Model};

/// Input data for building a model from CSC format.
pub struct CscInput<'a> {
    pub num_constraints: usize,
    pub num_variables: usize,
    pub col_ptrs: &'a [usize],
    pub row_indices: &'a [usize],
    pub values: &'a [f32],
    pub var_lower: &'a [f32],
    pub var_upper: &'a [f32],
    pub con_lower: &'a [f32],
    pub con_upper: &'a [f32],
    pub is_integer: &'a [bool],
}

impl Model {
    /// Build a model directly from CSC data.
    pub fn from_csc(
        input: CscInput<'_>,
        simplify_level: SimplifyLevel,
    ) -> Result<Self, ModelError> {
        let CscInput {
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
        } = input;

        if col_ptrs.len() != num_variables + 1 {
            return Err(ModelError::InvalidCscData {
                reason: "col_ptrs length must be num_variables + 1".to_string(),
            });
        }
        if row_indices.len() != values.len() {
            return Err(ModelError::InvalidCscData {
                reason: "row_indices and values must be the same length".to_string(),
            });
        }
        if var_lower.len() != num_variables || var_upper.len() != num_variables {
            return Err(ModelError::InvalidCscData {
                reason: "variable bounds length must match num_variables".to_string(),
            });
        }
        if con_lower.len() != num_constraints || con_upper.len() != num_constraints {
            return Err(ModelError::InvalidCscData {
                reason: "constraint bounds length must match num_constraints".to_string(),
            });
        }
        if is_integer.len() != num_variables {
            return Err(ModelError::InvalidCscData {
                reason: "is_integer length must match num_variables".to_string(),
            });
        }
        if col_ptrs.first().copied().unwrap_or(0) != 0 {
            return Err(ModelError::InvalidCscData {
                reason: "col_ptrs must start at 0".to_string(),
            });
        }
        if col_ptrs.last().copied().unwrap_or(0) != values.len() {
            return Err(ModelError::InvalidCscData {
                reason: "col_ptrs last entry must equal values length".to_string(),
            });
        }

        let mut model = Model::with_capacities(num_variables, num_constraints);
        model.set_expr_simplify(simplify_level)?;

        for idx in 0..num_variables {
            let lower = var_lower[idx] as f64;
            let upper = var_upper[idx] as f64;
            if lower.is_nan() || upper.is_nan() || lower > upper {
                return Err(ModelError::InvalidVariableBounds { lower, upper });
            }
            model.push_variable(Variable {
                bounds: Bounds::new(lower, upper),
                is_integer: is_integer[idx],
                is_active: true,
            });
        }

        for idx in 0..num_constraints {
            let lower = con_lower[idx] as f64;
            let upper = con_upper[idx] as f64;
            if lower.is_nan() || upper.is_nan() || lower > upper {
                return Err(ModelError::InvalidConstraintBounds { lower, upper });
            }
            model.constraints.push(Constraint {
                bounds: Bounds::new(lower, upper),
            });
        }

        for col in 0..num_variables {
            let start = col_ptrs[col];
            let end = col_ptrs[col + 1];
            if start > end || end > values.len() {
                return Err(ModelError::InvalidCscData {
                    reason: format!("col_ptrs must be non-decreasing (col {col})"),
                });
            }
            if start == end {
                continue;
            }
            let mut column: Vec<(ConstraintId, f64)> = Vec::with_capacity(end - start);
            for idx in start..end {
                let row = row_indices[idx];
                if row >= num_constraints {
                    return Err(ModelError::InvalidCscData {
                        reason: format!("row index out of bounds at position {idx}"),
                    });
                }
                let coefficient = values[idx] as f64;
                if !coefficient.is_finite() {
                    return Err(ModelError::InvalidCscData {
                        reason: format!(
                            "coefficient at position {idx} must be finite (got {coefficient})"
                        ),
                    });
                }
                column.push((ConstraintId::new(row as u32), coefficient));
            }
            model.columns.insert(
                VariableId::new(col as u32),
                ColumnData::from_entries(column),
            );
        }

        model.next_variable_id = num_variables as u32;
        model.next_constraint_id = num_constraints as u32;

        Ok(model)
    }
}
