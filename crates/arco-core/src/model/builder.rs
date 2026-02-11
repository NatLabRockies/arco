//! Model builder methods for adding variables, constraints, and objectives.

use crate::types::{Bounds, Constraint, Objective, Sense, Variable};
use arco_expr::expr::{ComparisonSense, ConstraintExpr, Expr};
use arco_expr::ids::{ConstraintId, VariableId};

use crate::model::error::ModelError;
use crate::model::{ColumnData, Model};

impl Model {
    /// Add a variable to the model.
    pub fn add_variable(&mut self, variable: Variable) -> Result<VariableId, ModelError> {
        if variable.bounds.lower.is_nan()
            || variable.bounds.upper.is_nan()
            || variable.bounds.lower > variable.bounds.upper
        {
            return Err(ModelError::InvalidVariableBounds {
                lower: variable.bounds.lower,
                upper: variable.bounds.upper,
            });
        }

        let id = VariableId::new(self.next_variable_id);
        self.next_variable_id += 1;

        self.push_variable(variable);

        Ok(id)
    }

    /// Add a constraint to the model.
    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<ConstraintId, ModelError> {
        if constraint.bounds.lower.is_nan()
            || constraint.bounds.upper.is_nan()
            || constraint.bounds.lower > constraint.bounds.upper
        {
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
            if !coeff.is_finite() {
                return Err(ModelError::InvalidCoefficient {
                    coefficient: *coeff,
                });
            }
        }

        let normalized = self.normalize_terms(objective.terms);
        self.objective = Objective {
            sense: Some(sense),
            terms: normalized,
        };
        self.objective_name = None;
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
        if self.objective.sense.is_some() {
            return Err(ModelError::MultipleObjectives);
        }
        self.set_objective(Objective {
            sense: Some(Sense::Minimize),
            terms: expr.into_linear_terms(),
        })
    }

    /// Maximize a linear expression.
    ///
    /// Returns an error if the model already has an objective.
    pub fn maximize(&mut self, expr: Expr) -> Result<(), ModelError> {
        if self.objective.sense.is_some() {
            return Err(ModelError::MultipleObjectives);
        }
        self.set_objective(Objective {
            sense: Some(Sense::Maximize),
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
        if !coefficient.is_finite() {
            return Err(ModelError::InvalidCoefficient { coefficient });
        }
        self.ensure_variable_exists(var_id)?;
        self.ensure_constraint_exists(constraint_id)?;

        // Update or insert in column-first storage.
        match self.columns.entry(var_id) {
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(ColumnData::Single((constraint_id, coefficient)));
            }
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                occupied.get_mut().upsert(constraint_id, coefficient);
            }
        }

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
