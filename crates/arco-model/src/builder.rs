//! Frozen primitive construction path.
//!
//! This module provides the `ModelBuilder<S> -> Model` seam used by new
//! consumers. The concrete storage is currently the compact `f64` model; `f32`
//! builders convert explicitly at finish time.

use crate::ids::{ConstraintId, VariableId};
use crate::{Constraint, Model, ModelError, Objective, Variable};
use std::marker::PhantomData;

/// Scalar-generic model builder facade.
#[derive(Debug, Clone)]
pub struct ModelBuilder<S = f64> {
    model: Model,
    _scalar: PhantomData<S>,
}

/// Default frozen model type.
pub type Model64 = Model;

/// Transitional f32 builder output uses f64 storage until a full f32 hot path lands.
pub type Model32 = Model;

impl<S> Default for ModelBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> ModelBuilder<S> {
    pub fn new() -> Self {
        Self {
            model: Model::new(),
            _scalar: PhantomData,
        }
    }

    pub fn add_variable(&mut self, variable: Variable) -> Result<VariableId, ModelError> {
        self.model.add_variable(variable)
    }

    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<ConstraintId, ModelError> {
        self.model.add_constraint(constraint)
    }

    pub fn set_coefficient(
        &mut self,
        variable_id: VariableId,
        constraint_id: ConstraintId,
        coefficient: f64,
    ) -> Result<(), ModelError> {
        self.model
            .set_coefficient(variable_id, constraint_id, coefficient)
    }

    pub fn set_variable_name(
        &mut self,
        variable_id: VariableId,
        name: impl Into<String>,
    ) -> Result<(), ModelError> {
        self.model.set_variable_name(variable_id, name.into())
    }

    pub fn set_constraint_name(
        &mut self,
        constraint_id: ConstraintId,
        name: impl Into<String>,
    ) -> Result<(), ModelError> {
        self.model.set_constraint_name(constraint_id, name.into())
    }

    pub fn set_objective(&mut self, objective: Objective) -> Result<(), ModelError> {
        self.model.set_objective(objective)
    }

    pub fn set_objective_name(&mut self, name: Option<String>) -> Result<(), ModelError> {
        self.model.set_objective_name(name)
    }

    pub fn finish(self) -> Model {
        self.model
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bounds, Constraint, ModelBuilder, ModelView, Variable};

    #[test]
    fn builder_finishes_frozen_viewable_model() {
        let mut builder = ModelBuilder::<f64>::new();
        let x = builder
            .add_variable(Variable::continuous(Bounds::new(0.0, 4.0)))
            .unwrap();
        let row = builder
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, 10.0),
            })
            .unwrap();
        builder.set_coefficient(x, row, 2.0).unwrap();
        let model = builder.finish();

        assert_eq!(model.structural_facts().variables, 1);
        assert_eq!(model.structural_facts().coefficients, 1);
    }
}
