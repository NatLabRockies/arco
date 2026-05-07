//! Read-only model views, value patches, and structural fingerprints.

use crate::ids::{ConstraintId, VariableId};
use crate::model::Model;
use crate::types::{Bounds, Constraint, Objective, Variable};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// Cheap facts about a finite optimization model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralFacts {
    pub variables: usize,
    pub constraints: usize,
    pub coefficients: usize,
    pub integer_variables: usize,
}

/// Stable fingerprint for structure and numeric values visible through a model view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelFingerprint(pub u64);

/// Read-only boundary consumed by validation, format, and solver layers.
pub trait ModelView {
    fn num_variables(&self) -> usize;
    fn num_constraints(&self) -> usize;
    fn num_coefficients(&self) -> usize;
    fn variable(&self, id: VariableId) -> Option<Variable>;
    fn constraint(&self, id: ConstraintId) -> Option<Constraint>;
    fn objective(&self) -> &Objective;
    fn column(&self, id: VariableId) -> Option<&[(ConstraintId, f64)]>;

    fn structural_facts(&self) -> StructuralFacts {
        let integer_variables = (0..self.num_variables())
            .filter(|idx| {
                self.variable(VariableId::new(*idx as u32))
                    .is_some_and(|variable| variable.is_integer)
            })
            .count();
        StructuralFacts {
            variables: self.num_variables(),
            constraints: self.num_constraints(),
            coefficients: self.num_coefficients(),
            integer_variables,
        }
    }

    fn fingerprint(&self) -> ModelFingerprint {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.num_variables().hash(&mut hasher);
        self.num_constraints().hash(&mut hasher);
        self.num_coefficients().hash(&mut hasher);
        for idx in 0..self.num_variables() {
            let id = VariableId::new(idx as u32);
            if let Some(variable) = self.variable(id) {
                variable.bounds.lower.to_bits().hash(&mut hasher);
                variable.bounds.upper.to_bits().hash(&mut hasher);
                variable.is_integer.hash(&mut hasher);
                variable.is_active.hash(&mut hasher);
            }
            if let Some(column) = self.column(id) {
                for (constraint_id, coefficient) in column {
                    constraint_id.inner().hash(&mut hasher);
                    (*coefficient).to_bits().hash(&mut hasher);
                }
            }
        }
        for idx in 0..self.num_constraints() {
            if let Some(constraint) = self.constraint(ConstraintId::new(idx as u32)) {
                constraint.bounds.lower.to_bits().hash(&mut hasher);
                constraint.bounds.upper.to_bits().hash(&mut hasher);
            }
        }
        self.objective().sense.hash(&mut hasher);
        for (variable_id, coefficient) in &self.objective().terms {
            variable_id.inner().hash(&mut hasher);
            (*coefficient).to_bits().hash(&mut hasher);
        }
        ModelFingerprint(hasher.finish())
    }
}

impl ModelView for Model {
    fn num_variables(&self) -> usize {
        Model::num_variables(self)
    }

    fn num_constraints(&self) -> usize {
        Model::num_constraints(self)
    }

    fn num_coefficients(&self) -> usize {
        Model::num_coefficients(self)
    }

    fn variable(&self, id: VariableId) -> Option<Variable> {
        self.get_variable_by_index(id.inner() as usize)
    }

    fn constraint(&self, id: ConstraintId) -> Option<Constraint> {
        self.get_constraint(id).copied().ok()
    }

    fn objective(&self) -> &Objective {
        &self.objective
    }

    fn column(&self, id: VariableId) -> Option<&[(ConstraintId, f64)]> {
        self.get_column(id)
    }
}

/// Value-only patch. It cannot add/remove variables, constraints, or matrix entries.
#[derive(Debug, Clone, Default)]
pub struct ModelPatch {
    variable_bounds: BTreeMap<VariableId, Bounds>,
    constraint_bounds: BTreeMap<ConstraintId, Bounds>,
    coefficients: BTreeMap<(VariableId, ConstraintId), f64>,
    objective_terms: BTreeMap<VariableId, f64>,
}

impl ModelPatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_variable_bounds(&mut self, id: VariableId, bounds: Bounds) {
        self.variable_bounds.insert(id, bounds);
    }

    pub fn set_constraint_bounds(&mut self, id: ConstraintId, bounds: Bounds) {
        self.constraint_bounds.insert(id, bounds);
    }

    pub fn set_coefficient(
        &mut self,
        variable_id: VariableId,
        constraint_id: ConstraintId,
        value: f64,
    ) {
        self.coefficients
            .insert((variable_id, constraint_id), value);
    }

    pub fn set_objective_term(&mut self, variable_id: VariableId, value: f64) {
        self.objective_terms.insert(variable_id, value);
    }
}

/// Read-only patched view over a base model view.
pub struct PatchedModelView<'a, V: ModelView + ?Sized> {
    base: &'a V,
    patch: &'a ModelPatch,
}

impl<'a, V: ModelView + ?Sized> PatchedModelView<'a, V> {
    pub fn new(base: &'a V, patch: &'a ModelPatch) -> Self {
        Self { base, patch }
    }
}

impl<V: ModelView + ?Sized> ModelView for PatchedModelView<'_, V> {
    fn num_variables(&self) -> usize {
        self.base.num_variables()
    }

    fn num_constraints(&self) -> usize {
        self.base.num_constraints()
    }

    fn num_coefficients(&self) -> usize {
        self.base.num_coefficients()
    }

    fn variable(&self, id: VariableId) -> Option<Variable> {
        let mut variable = self.base.variable(id)?;
        if let Some(bounds) = self.patch.variable_bounds.get(&id) {
            variable.bounds = *bounds;
        }
        Some(variable)
    }

    fn constraint(&self, id: ConstraintId) -> Option<Constraint> {
        let mut constraint = self.base.constraint(id)?;
        if let Some(bounds) = self.patch.constraint_bounds.get(&id) {
            constraint.bounds = *bounds;
        }
        Some(constraint)
    }

    fn objective(&self) -> &Objective {
        self.base.objective()
    }

    fn column(&self, id: VariableId) -> Option<&[(ConstraintId, f64)]> {
        self.base.column(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::ids::{ConstraintId, VariableId};
    use crate::{Bounds, Constraint, Model, ModelPatch, ModelView, PatchedModelView, Variable};

    #[test]
    fn model_view_reports_facts_and_fingerprint() {
        let mut model = Model::new();
        let var = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();
        let con = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, 2.0),
            })
            .unwrap();
        model.set_coefficient(var, con, 3.0).unwrap();

        let facts = model.structural_facts();
        assert_eq!(facts.variables, 1);
        assert_eq!(facts.constraints, 1);
        assert_eq!(facts.coefficients, 1);
        assert_ne!(model.fingerprint().0, 0);
    }

    #[test]
    fn patched_view_overrides_bounds_without_mutating_base() {
        let mut model = Model::new();
        let var = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();
        let mut patch = ModelPatch::new();
        patch.set_variable_bounds(var, Bounds::new(2.0, 8.0));
        let patched = PatchedModelView::new(&model, &patch);

        assert_eq!(patched.variable(var).unwrap().bounds, Bounds::new(2.0, 8.0));
        assert_eq!(model.variable(var).unwrap().bounds, Bounds::new(0.0, 10.0));
        assert!(patched.constraint(ConstraintId::new(0)).is_none());
        assert!(patched.variable(VariableId::new(99)).is_none());
    }
}
