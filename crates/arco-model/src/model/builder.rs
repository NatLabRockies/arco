//! Model builder methods for adding variables, constraints, and objectives.

use crate::expr::{ComparisonSense, ConstraintExpr, Expr};
use crate::ids::{ConstraintId, VariableId};
use crate::model::error::ModelError;
use crate::model::{
    BITS_PER_WORD, ColumnVec, Model, bounds_are_valid, coefficient_is_valid, column_upsert,
};
use crate::types::{Bounds, Constraint, Objective, Sense, Variable};

impl Model {
    /// Pre-allocate capacity for the given number of additional variables.
    pub fn reserve_variables(&mut self, count: usize) {
        self.variables.reserve(count);
        self.columns.reserve(count);
        let needed_words = (self.variables.len() + count).div_ceil(BITS_PER_WORD);
        for bits in [
            &mut self.variable_is_integer_bits,
            &mut self.variable_is_inactive_bits,
        ] {
            if needed_words > bits.len() {
                bits.reserve(needed_words - bits.len());
            }
        }
    }

    /// Pre-allocate capacity for the given number of additional constraints.
    pub fn reserve_constraints(&mut self, count: usize) {
        self.constraints.reserve(count);
    }

    /// Add a variable to the model.
    pub fn add_variable(&mut self, variable: Variable) -> Result<VariableId, ModelError> {
        if !bounds_are_valid(variable.bounds.lower, variable.bounds.upper) {
            return Err(ModelError::InvalidVariableBounds {
                lower: variable.bounds.lower,
                upper: variable.bounds.upper,
            });
        }

        let id = VariableId::new(self.next_variable_id);
        self.next_variable_id += 1;

        self.push_variable(variable)?;

        Ok(id)
    }

    /// Add a contiguous block of variables that share bounds and integrality.
    ///
    /// This avoids repeated validation and per-variable method dispatch in large
    /// array builders while preserving the same contiguous ID assignment as
    /// calling `add_variable` repeatedly.
    pub fn add_variables_uniform(
        &mut self,
        variable: Variable,
        count: usize,
    ) -> Result<VariableId, ModelError> {
        if !bounds_are_valid(variable.bounds.lower, variable.bounds.upper) {
            return Err(ModelError::InvalidVariableBounds {
                lower: variable.bounds.lower,
                upper: variable.bounds.upper,
            });
        }
        if count == 0 {
            return Ok(VariableId::new(self.next_variable_id));
        }

        let count_u32 = u32::try_from(count).map_err(|_| ModelError::InvalidCscData {
            reason: "variable block size exceeds u32 id space".to_string(),
        })?;
        let first_id = self.next_variable_id;
        let next_id =
            first_id
                .checked_add(count_u32)
                .ok_or_else(|| ModelError::InvalidCscData {
                    reason: "variable block would exceed u32 id space".to_string(),
                })?;

        self.reserve_variables(count);

        let start_idx = self.variables.len();
        self.variables.extend_repeated(variable.bounds, count)?;
        self.columns.resize_with(start_idx + count, ColumnVec::new);
        if variable.is_integer {
            for idx in start_idx..start_idx + count {
                Self::write_packed_flag(&mut self.variable_is_integer_bits, idx, true);
            }
        }
        if !variable.is_active {
            for idx in start_idx..start_idx + count {
                Self::write_packed_flag(&mut self.variable_is_inactive_bits, idx, true);
            }
        }
        self.next_variable_id = next_id;

        Ok(VariableId::new(first_id))
    }

    /// Add a contiguous block of variables with per-variable bounds and shared flags.
    ///
    /// This is the array-bounds companion to `add_variables_uniform`: callers can
    /// validate and insert a large variable block without per-variable method
    /// dispatch while preserving contiguous IDs.
    pub fn add_variables_with_bounds(
        &mut self,
        bounds: &[Bounds],
        is_integer: bool,
        is_active: bool,
    ) -> Result<VariableId, ModelError> {
        for bound in bounds {
            if !bounds_are_valid(bound.lower, bound.upper) {
                return Err(ModelError::InvalidVariableBounds {
                    lower: bound.lower,
                    upper: bound.upper,
                });
            }
        }
        if bounds.is_empty() {
            return Ok(VariableId::new(self.next_variable_id));
        }

        let count_u32 = u32::try_from(bounds.len()).map_err(|_| ModelError::InvalidCscData {
            reason: "variable block size exceeds u32 id space".to_string(),
        })?;
        let first_id = self.next_variable_id;
        let next_id =
            first_id
                .checked_add(count_u32)
                .ok_or_else(|| ModelError::InvalidCscData {
                    reason: "variable block would exceed u32 id space".to_string(),
                })?;

        self.reserve_variables(bounds.len());

        let start_idx = self.variables.len();
        self.variables.extend_from_slice(bounds)?;
        self.columns
            .resize_with(start_idx + bounds.len(), ColumnVec::new);
        if is_integer {
            for idx in start_idx..start_idx + bounds.len() {
                Self::write_packed_flag(&mut self.variable_is_integer_bits, idx, true);
            }
        }
        if !is_active {
            for idx in start_idx..start_idx + bounds.len() {
                Self::write_packed_flag(&mut self.variable_is_inactive_bits, idx, true);
            }
        }
        self.next_variable_id = next_id;

        Ok(VariableId::new(first_id))
    }

    /// Add a constraint to the model.
    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<ConstraintId, ModelError> {
        if !bounds_are_valid(constraint.bounds.lower, constraint.bounds.upper) {
            return Err(ModelError::InvalidConstraintBounds {
                lower: constraint.bounds.lower,
                upper: constraint.bounds.upper,
            });
        }

        let id = ConstraintId::new(self.next_constraint_id);
        self.next_constraint_id += 1;

        self.constraints.push(constraint);

        Ok(id)
    }

    /// Set the objective function.
    pub fn set_objective(&mut self, objective: Objective) -> Result<(), ModelError> {
        let sense = objective.sense.ok_or(ModelError::NoObjective)?;
        for (var_id, coeff) in &objective.terms {
            self.ensure_variable_exists(*var_id)?;
            if !coefficient_is_valid(*coeff) {
                return Err(ModelError::InvalidCoefficient {
                    coefficient: *coeff,
                });
            }
        }

        let mut normalized = self.normalize_terms(objective.terms);
        normalized.shrink_to_fit();
        self.objective = Objective {
            sense: Some(sense),
            terms: normalized,
        };
        self.objective_name = None;
        self.shrink_retained_storage();
        tracing::debug!(
            component = "model",
            operation = "set_objective",
            status = "success",
            sense = ?sense,
            terms = self.objective.terms.len(),
            "Set objective function"
        );
        Ok(())
    }

    /// Minimize a linear expression.
    ///
    /// Returns an error if the model already has an objective.
    pub fn minimize(&mut self, expr: Expr) -> Result<(), ModelError> {
        self.set_sense(expr, Sense::Minimize)
    }

    /// Maximize a linear expression.
    ///
    /// Returns an error if the model already has an objective.
    pub fn maximize(&mut self, expr: Expr) -> Result<(), ModelError> {
        self.set_sense(expr, Sense::Maximize)
    }

    fn set_sense(&mut self, expr: Expr, sense: Sense) -> Result<(), ModelError> {
        if self.objective.sense.is_some() {
            return Err(ModelError::MultipleObjectives);
        }
        self.set_objective(Objective {
            sense: Some(sense),
            terms: expr.into_linear_terms(),
        })
    }

    /// Add a constraint from an expression and explicit bounds.
    pub fn add_expr_constraint(
        &mut self,
        expr: Expr,
        bounds: Bounds,
    ) -> Result<ConstraintId, ModelError> {
        let constraint_id = self.add_constraint(Constraint { bounds })?;
        for (var_id, coeff) in self.normalize_terms(expr.into_linear_terms()) {
            self.set_coefficient(var_id, constraint_id, coeff)?;
        }
        Ok(constraint_id)
    }

    /// Add a constraint from a comparison expression (e.g., `x + y <= 10`).
    pub fn add_constraint_expr(
        &mut self,
        constraint: ConstraintExpr,
    ) -> Result<ConstraintId, ModelError> {
        let (expr, sense, rhs) = constraint.into_parts();
        let bounds = match sense {
            ComparisonSense::LessEqual => Bounds::new(f64::NEG_INFINITY, rhs),
            ComparisonSense::GreaterEqual => Bounds::new(rhs, f64::INFINITY),
            ComparisonSense::Equal => Bounds::new(rhs, rhs),
        };
        self.add_expr_constraint(expr, bounds)
    }

    /// Add constraints from compact term patterns (fastest path: zero per-constraint Vec allocation).
    pub fn add_constraints_compact(
        &mut self,
        term_patterns: &[(u32, f64)],
        bounds_list: &[Bounds],
    ) -> Result<ConstraintId, ModelError> {
        let count = bounds_list.len();
        if count == 0 {
            return Ok(ConstraintId::new(self.next_constraint_id));
        }

        self.constraints.reserve(count);
        let first_constraint_id = self.next_constraint_id;

        for (i, bounds) in bounds_list.iter().enumerate() {
            if !bounds_are_valid(bounds.lower, bounds.upper) {
                return Err(ModelError::InvalidConstraintBounds {
                    lower: bounds.lower,
                    upper: bounds.upper,
                });
            }

            let constraint_id = ConstraintId::new(self.next_constraint_id);
            self.next_constraint_id += 1;
            self.constraints.push(Constraint { bounds: *bounds });

            for &(start_var_id, coeff) in term_patterns {
                let var_idx = (start_var_id + i as u32) as usize;
                if var_idx >= self.variables.len() {
                    return Err(ModelError::InvalidVariableId(VariableId::new(
                        var_idx as u32,
                    )));
                }
                self.columns[var_idx].push((constraint_id, coeff));
            }
        }

        Ok(ConstraintId::new(first_constraint_id))
    }

    /// Add compact constraints where each row maps term patterns through a dense row index.
    ///
    /// For each inserted row, a pattern `(start_var_id, coeff)` is expanded to
    /// variable `start_var_id + row_index`. This keeps Python active-mask paths
    /// from allocating a per-row term vector when active rows are sparse.
    pub fn add_constraints_compact_indexed(
        &mut self,
        term_patterns: &[(u32, f64)],
        row_indices: &[usize],
        bounds_list: &[Bounds],
    ) -> Result<ConstraintId, ModelError> {
        let count = row_indices.len();
        if count == 0 {
            return Ok(ConstraintId::new(self.next_constraint_id));
        }
        if bounds_list.len() != count {
            return Err(ModelError::InvalidCscData {
                reason: "compact indexed constraint bounds length must match row indices"
                    .to_string(),
            });
        }

        self.constraints.reserve(count);
        let first_constraint_id = self.next_constraint_id;

        for (row_index, bounds) in row_indices.iter().copied().zip(bounds_list.iter()) {
            if !bounds_are_valid(bounds.lower, bounds.upper) {
                return Err(ModelError::InvalidConstraintBounds {
                    lower: bounds.lower,
                    upper: bounds.upper,
                });
            }

            let constraint_id = ConstraintId::new(self.next_constraint_id);
            self.next_constraint_id += 1;
            self.constraints.push(Constraint { bounds: *bounds });

            for &(start_var_id, coeff) in term_patterns {
                let Some(var_idx) = (start_var_id as usize).checked_add(row_index) else {
                    return Err(ModelError::InvalidVariableId(VariableId::new(u32::MAX)));
                };
                if var_idx >= self.variables.len() {
                    return Err(ModelError::InvalidVariableId(VariableId::new(
                        var_idx as u32,
                    )));
                }
                self.columns[var_idx].push((constraint_id, coeff));
            }
        }

        Ok(ConstraintId::new(first_constraint_id))
    }

    /// Add a batch of constraints with pre-normalized terms.
    ///
    /// Each constraint is given as a slice of `(VariableId, f64)` terms and a `Bounds`.
    /// Terms are assumed to be already normalized (no duplicate variable IDs, no zero
    /// coefficients). This skips per-constraint `normalize_terms` HashMap creation.
    ///
    /// Returns the first `ConstraintId` in the contiguous block of added constraints.
    pub fn add_constraints_batch(
        &mut self,
        constraints: &[(Vec<(VariableId, f64)>, Bounds)],
    ) -> Result<ConstraintId, ModelError> {
        self.add_constraints_batch_streaming(
            constraints.len(),
            constraints
                .iter()
                .map(|(terms, bounds)| (terms.as_slice(), *bounds)),
        )
    }

    /// Add a streamed batch of constraints with pre-normalized term slices.
    ///
    /// Rows are consumed one at a time, so callers do not need to retain a full
    /// batch of row vectors before inserting them into the model.
    pub fn add_constraints_batch_streaming<I, T>(
        &mut self,
        count: usize,
        constraints: I,
    ) -> Result<ConstraintId, ModelError>
    where
        I: IntoIterator<Item = (T, Bounds)>,
        T: AsRef<[(VariableId, f64)]>,
    {
        if count == 0 {
            return Ok(ConstraintId::new(self.next_constraint_id));
        }

        self.constraints.reserve(count);
        let first_constraint_id = self.next_constraint_id;

        for (terms, bounds) in constraints {
            let terms = terms.as_ref();
            if !bounds_are_valid(bounds.lower, bounds.upper) {
                return Err(ModelError::InvalidConstraintBounds {
                    lower: bounds.lower,
                    upper: bounds.upper,
                });
            }

            let constraint_id = ConstraintId::new(self.next_constraint_id);
            self.next_constraint_id += 1;
            self.constraints.push(Constraint { bounds });

            for &(var_id, coeff) in terms {
                // Skip validation for each coefficient since terms are pre-normalized.
                // We still check variable bounds.
                let var_idx = var_id.inner() as usize;
                if var_idx >= self.variables.len() {
                    return Err(ModelError::InvalidVariableId(var_id));
                }
                self.columns[var_idx].push((constraint_id, coeff));
            }
        }

        Ok(ConstraintId::new(first_constraint_id))
    }

    /// Add a coefficient to the constraint matrix.
    ///
    /// This adds a coefficient at the intersection of a variable column and constraint row.
    /// Returns an error if the variable or constraint IDs are invalid.
    pub fn set_coefficient(
        &mut self,
        var_id: VariableId,
        constraint_id: ConstraintId,
        coefficient: f64,
    ) -> Result<(), ModelError> {
        if !coefficient_is_valid(coefficient) {
            return Err(ModelError::InvalidCoefficient { coefficient });
        }
        self.ensure_variable_exists(var_id)?;
        self.ensure_constraint_exists(constraint_id)?;

        // Update or insert in column-first storage (Vec indexed by variable ID).
        column_upsert(
            &mut self.columns[var_id.inner() as usize],
            constraint_id,
            coefficient,
        );

        Ok(())
    }

    /// Check if a variable is active.
    pub fn is_variable_active(&self, id: VariableId) -> Result<bool, ModelError> {
        self.variable_is_active_by_index(id.inner() as usize)
            .ok_or(ModelError::InvalidVariableId(id))
    }

    /// Deactivate a variable without removing its column.
    pub fn deactivate_variable(&mut self, id: VariableId) -> Result<(), ModelError> {
        if self.set_variable_active_by_index(id.inner() as usize, false) {
            Ok(())
        } else {
            Err(ModelError::InvalidVariableId(id))
        }
    }

    /// Activate a previously deactivated variable.
    pub fn activate_variable(&mut self, id: VariableId) -> Result<(), ModelError> {
        if self.set_variable_active_by_index(id.inner() as usize, true) {
            Ok(())
        } else {
            Err(ModelError::InvalidVariableId(id))
        }
    }
}
