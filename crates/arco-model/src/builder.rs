//! Frozen primitive construction path.
//!
//! This module provides the `ModelBuilder<S> -> FrozenModel<S>` seam used by
//! new consumers. Builders are mutable; finished models are read-only views over
//! compact primitive storage.

use crate::ids::{ConstraintId, VariableId};
use crate::{Constraint, Model, ModelError, ModelView, Objective, Variable};
use std::marker::PhantomData;

/// Immutable scalar-tagged model produced by [`ModelBuilder`].
#[derive(Debug, Clone)]
pub struct FrozenModel<S = f64> {
    model: Model,
    _scalar: PhantomData<S>,
}

/// Scalar-generic model builder facade.
#[derive(Debug, Clone)]
pub struct ModelBuilder<S = f64> {
    model: Model,
    _scalar: PhantomData<S>,
}

/// Default frozen model type.
pub type Model64 = FrozenModel<f64>;

/// f32-tagged frozen model. Numeric storage is still canonical f64 until the
/// solver/export hot paths gain native f32 kernels.
pub type Model32 = FrozenModel<f32>;

impl<S> FrozenModel<S> {
    fn new(model: Model) -> Self {
        Self {
            model,
            _scalar: PhantomData,
        }
    }

    pub fn as_model(&self) -> &Model {
        &self.model
    }

    pub fn into_model(self) -> Model {
        self.model
    }

    pub fn num_variables(&self) -> usize {
        self.model.num_variables()
    }

    pub fn num_constraints(&self) -> usize {
        self.model.num_constraints()
    }

    pub fn num_coefficients(&self) -> usize {
        self.model.num_coefficients()
    }

    pub fn objective(&self) -> &Objective {
        self.model.objective()
    }
}

impl<S> ModelView for FrozenModel<S> {
    fn num_variables(&self) -> usize {
        self.model.num_variables()
    }

    fn num_constraints(&self) -> usize {
        self.model.num_constraints()
    }

    fn num_coefficients(&self) -> usize {
        self.model.num_coefficients()
    }

    fn variable(&self, id: VariableId) -> Option<Variable> {
        self.model.variable(id)
    }

    fn constraint(&self, id: ConstraintId) -> Option<Constraint> {
        self.model.constraint(id)
    }

    fn objective(&self) -> &Objective {
        self.model.objective()
    }

    fn column(&self, id: VariableId) -> Option<&[(ConstraintId, f64)]> {
        self.model.column(id)
    }

    fn variable_name(&self, id: VariableId) -> Option<&str> {
        self.model.get_variable_name(id)
    }

    fn constraint_name(&self, id: ConstraintId) -> Option<&str> {
        self.model.get_constraint_name(id)
    }

    fn objective_name(&self) -> Option<&str> {
        self.model.get_objective_name()
    }

    fn variable_metadata(&self, id: VariableId) -> Option<&serde_json::Value> {
        self.model.get_variable_metadata(id)
    }

    fn constraint_metadata(&self, id: ConstraintId) -> Option<&serde_json::Value> {
        self.model.get_constraint_metadata(id)
    }
}

impl<S> Default for ModelBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> ModelBuilder<S> {
    pub(crate) fn new() -> Self {
        Self {
            model: Model::new(),
            _scalar: PhantomData,
        }
    }

    pub(crate) fn add_variable(&mut self, variable: Variable) -> Result<VariableId, ModelError> {
        self.model.add_variable(variable)
    }

    pub(crate) fn add_constraint(
        &mut self,
        constraint: Constraint,
    ) -> Result<ConstraintId, ModelError> {
        self.model.add_constraint(constraint)
    }

    pub(crate) fn set_coefficient(
        &mut self,
        variable_id: VariableId,
        constraint_id: ConstraintId,
        coefficient: f64,
    ) -> Result<(), ModelError> {
        self.model
            .set_coefficient(variable_id, constraint_id, coefficient)
    }

    pub(crate) fn set_variable_name(
        &mut self,
        variable_id: VariableId,
        name: impl Into<String>,
    ) -> Result<(), ModelError> {
        self.model.set_variable_name(variable_id, name.into())
    }

    pub(crate) fn set_constraint_name(
        &mut self,
        constraint_id: ConstraintId,
        name: impl Into<String>,
    ) -> Result<(), ModelError> {
        self.model.set_constraint_name(constraint_id, name.into())
    }

    pub(crate) fn set_objective(&mut self, objective: Objective) -> Result<(), ModelError> {
        self.model.set_objective(objective)
    }

    pub(crate) fn set_objective_name(&mut self, name: Option<String>) -> Result<(), ModelError> {
        self.model.set_objective_name(name)
    }

    pub(crate) fn finish(self) -> FrozenModel<S> {
        FrozenModel::new(self.model)
    }

    pub(crate) fn finish_legacy_model(self) -> Model {
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
