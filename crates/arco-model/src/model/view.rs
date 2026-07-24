//! Read-only model views, value patches, and structural fingerprints.

use crate::ids::{ConstraintId, VariableId};
use crate::model::Model;
use crate::types::{Bounds, Constraint, Objective, Variable};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

/// Cheap facts about a finite optimization model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralFacts {
    pub(crate) variables: usize,
    pub(crate) constraints: usize,
    pub(crate) coefficients: usize,
    pub(crate) integer_variables: usize,
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

    fn variable_name(&self, _id: VariableId) -> Option<&str> {
        None
    }

    fn constraint_name(&self, _id: ConstraintId) -> Option<&str> {
        None
    }

    fn objective_name(&self) -> Option<&str> {
        None
    }

    fn variable_metadata(&self, _id: VariableId) -> Option<&serde_json::Value> {
        None
    }

    fn constraint_metadata(&self, _id: ConstraintId) -> Option<&serde_json::Value> {
        None
    }

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

    fn variable_name(&self, id: VariableId) -> Option<&str> {
        self.get_variable_name(id)
    }

    fn constraint_name(&self, id: ConstraintId) -> Option<&str> {
        self.get_constraint_name(id)
    }

    fn objective_name(&self) -> Option<&str> {
        self.get_objective_name()
    }

    fn variable_metadata(&self, id: VariableId) -> Option<&serde_json::Value> {
        self.get_variable_metadata(id)
    }

    fn constraint_metadata(&self, id: ConstraintId) -> Option<&serde_json::Value> {
        self.get_constraint_metadata(id)
    }
}

/// Value-only patch. It cannot add/remove variables, constraints, or matrix entries.
#[derive(Debug, Clone, Default)]
pub struct ModelPatch {
    variable_bounds: BTreeMap<VariableId, Bounds>,
    constraint_bounds: BTreeMap<ConstraintId, Bounds>,
    coefficients: BTreeMap<(VariableId, ConstraintId), f64>,
    objective_terms: BTreeMap<VariableId, f64>,
    variable_names: BTreeMap<VariableId, String>,
    constraint_names: BTreeMap<ConstraintId, String>,
    objective_name: ObjectiveNamePatch,
    variable_metadata: BTreeMap<VariableId, serde_json::Value>,
    constraint_metadata: BTreeMap<ConstraintId, serde_json::Value>,
}

impl ModelPatch {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_variable_bounds(&mut self, id: VariableId, bounds: Bounds) {
        self.variable_bounds.insert(id, bounds);
    }

    pub fn set_constraint_bounds(&mut self, id: ConstraintId, bounds: Bounds) {
        self.constraint_bounds.insert(id, bounds);
    }

    pub(crate) fn set_coefficient(
        &mut self,
        variable_id: VariableId,
        constraint_id: ConstraintId,
        value: f64,
    ) {
        self.coefficients
            .insert((variable_id, constraint_id), value);
    }

    pub(crate) fn set_objective_term(&mut self, variable_id: VariableId, value: f64) {
        self.objective_terms.insert(variable_id, value);
    }

    pub(crate) fn set_variable_name(&mut self, id: VariableId, name: impl Into<String>) {
        self.variable_names.insert(id, name.into());
    }

    pub(crate) fn set_constraint_name(&mut self, id: ConstraintId, name: impl Into<String>) {
        self.constraint_names.insert(id, name.into());
    }

    pub(crate) fn set_objective_name(&mut self, name: Option<String>) {
        self.objective_name = ObjectiveNamePatch::Set(name);
    }

    pub(crate) fn set_variable_metadata(&mut self, id: VariableId, metadata: serde_json::Value) {
        self.variable_metadata.insert(id, metadata);
    }

    pub fn set_constraint_metadata(&mut self, id: ConstraintId, metadata: serde_json::Value) {
        self.constraint_metadata.insert(id, metadata);
    }
}

#[derive(Debug, Clone, Default)]
enum ObjectiveNamePatch {
    #[default]
    Unchanged,
    Set(Option<String>),
}

/// Read-only patched view over a base model view.
pub struct PatchedModelView<'a, V: ModelView + ?Sized> {
    base: &'a V,
    patch: &'a ModelPatch,
    objective: Objective,
    columns: BTreeMap<VariableId, Vec<(ConstraintId, f64)>>,
}

impl<'a, V: ModelView + ?Sized> PatchedModelView<'a, V> {
    pub(crate) fn new(base: &'a V, patch: &'a ModelPatch) -> Self {
        let objective = patched_objective(base.objective(), &patch.objective_terms);
        let columns = patched_columns(base, &patch.coefficients);
        Self {
            base,
            patch,
            objective,
            columns,
        }
    }
}

fn patched_objective(base: &Objective, patch: &BTreeMap<VariableId, f64>) -> Objective {
    let mut terms_by_variable = BTreeMap::new();
    for (variable_id, coefficient) in &base.terms {
        terms_by_variable.insert(*variable_id, *coefficient);
    }
    for (variable_id, coefficient) in patch {
        terms_by_variable.insert(*variable_id, *coefficient);
    }
    Objective {
        sense: base.sense,
        terms: terms_by_variable.into_iter().collect(),
    }
}

fn patched_columns<V: ModelView + ?Sized>(
    base: &V,
    patch: &BTreeMap<(VariableId, ConstraintId), f64>,
) -> BTreeMap<VariableId, Vec<(ConstraintId, f64)>> {
    let mut patched_variables = BTreeSet::new();
    for (variable_id, _) in patch.keys() {
        patched_variables.insert(*variable_id);
    }

    let mut columns = BTreeMap::new();
    for variable_id in patched_variables {
        if let Some(column) = base.column(variable_id) {
            let mut patched_column = column.to_vec();
            for (constraint_id, coefficient) in &mut patched_column {
                if let Some(value) = patch.get(&(variable_id, *constraint_id)) {
                    *coefficient = *value;
                }
            }
            columns.insert(variable_id, patched_column);
        }
    }
    columns
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
        &self.objective
    }

    fn column(&self, id: VariableId) -> Option<&[(ConstraintId, f64)]> {
        self.columns
            .get(&id)
            .map(Vec::as_slice)
            .or_else(|| self.base.column(id))
    }

    fn variable_name(&self, id: VariableId) -> Option<&str> {
        self.patch
            .variable_names
            .get(&id)
            .map(String::as_str)
            .or_else(|| self.base.variable_name(id))
    }

    fn constraint_name(&self, id: ConstraintId) -> Option<&str> {
        self.patch
            .constraint_names
            .get(&id)
            .map(String::as_str)
            .or_else(|| self.base.constraint_name(id))
    }

    fn objective_name(&self) -> Option<&str> {
        match &self.patch.objective_name {
            ObjectiveNamePatch::Unchanged => self.base.objective_name(),
            ObjectiveNamePatch::Set(name) => name.as_deref(),
        }
    }

    fn variable_metadata(&self, id: VariableId) -> Option<&serde_json::Value> {
        self.patch
            .variable_metadata
            .get(&id)
            .or_else(|| self.base.variable_metadata(id))
    }

    fn constraint_metadata(&self, id: ConstraintId) -> Option<&serde_json::Value> {
        self.patch
            .constraint_metadata
            .get(&id)
            .or_else(|| self.base.constraint_metadata(id))
    }
}

#[cfg(test)]
mod tests {
    use crate::ids::{ConstraintId, VariableId};
    use crate::{
        Bounds, Constraint, Model, ModelPatch, ModelView, Objective, PatchedModelView, Sense,
        Variable,
    };

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

    #[test]
    fn patched_view_overrides_coefficients_objective_and_sidecars() {
        let mut model = Model::new();
        let var = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();
        let row = model
            .add_constraint(Constraint {
                bounds: Bounds::new(1.0, 2.0),
            })
            .unwrap();
        model.set_coefficient(var, row, 3.0).unwrap();
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(var, 4.0)],
            })
            .unwrap();
        model.set_variable_name(var, "x".to_string()).unwrap();
        model.set_constraint_name(row, "limit".to_string()).unwrap();
        model.set_objective_name(Some("cost".to_string())).unwrap();
        model
            .set_variable_metadata(var, serde_json::json!({"base": true}))
            .unwrap();

        let mut patch = ModelPatch::new();
        patch.set_coefficient(var, row, 5.0);
        patch.set_objective_term(var, 7.0);
        patch.set_variable_name(var, "patched_x");
        patch.set_constraint_name(row, "patched_limit");
        patch.set_objective_name(Some("patched_cost".to_string()));
        patch.set_variable_metadata(var, serde_json::json!({"patched": true}));
        let patched = PatchedModelView::new(&model, &patch);

        assert_eq!(patched.column(var), Some(&[(row, 5.0)][..]));
        assert_eq!(patched.objective().terms, vec![(var, 7.0)]);
        assert_eq!(patched.variable_name(var), Some("patched_x"));
        assert_eq!(patched.constraint_name(row), Some("patched_limit"));
        assert_eq!(patched.objective_name(), Some("patched_cost"));
        assert_eq!(
            patched.variable_metadata(var),
            Some(&serde_json::json!({"patched": true}))
        );

        assert_eq!(model.column(var), Some(&[(row, 3.0)][..]));
        assert_eq!(model.objective().terms, vec![(var, 4.0)]);
        assert_eq!(model.get_variable_name(var), Some("x"));
    }
}
