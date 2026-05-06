//! Model inspection and snapshot methods.

use std::collections::HashSet;

use crate::slack::{SlackBound, SlackVariables};
use crate::types::{Bounds, Sense};
use arco_expr::ids::{ConstraintId, VariableId};

use crate::model::Model;

/// View of a variable in a model snapshot.
#[derive(Debug, Clone)]
pub struct VariableView {
    pub id: VariableId,
    pub name: Option<String>,
    pub bounds: Bounds,
    pub is_integer: bool,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
}

/// View of a constraint in a model snapshot.
#[derive(Debug, Clone)]
pub struct ConstraintView {
    pub id: ConstraintId,
    pub name: Option<String>,
    pub bounds: Bounds,
    pub nnz: usize,
    pub metadata: Option<serde_json::Value>,
}

/// View of a coefficient in a model snapshot.
#[derive(Debug, Clone)]
pub struct CoefficientView {
    pub variable_id: VariableId,
    pub constraint_id: ConstraintId,
    pub value: f64,
}

/// View of the objective in a model snapshot.
#[derive(Debug, Clone)]
pub struct ObjectiveView {
    pub sense: Option<Sense>,
    pub terms: Vec<(VariableId, f64)>,
    pub name: Option<String>,
}

/// View of a slack variable in a model snapshot.
#[derive(Debug, Clone)]
pub struct SlackView {
    pub constraint_id: ConstraintId,
    pub bound: SlackBound,
    pub penalty: f64,
    pub variable_ids: SlackVariables,
    pub name: Option<String>,
}

/// Metadata about a model snapshot.
#[derive(Debug, Clone, Copy)]
pub struct SnapshotMetadata {
    pub variables: usize,
    pub constraints: usize,
    pub coefficients: usize,
}

/// A complete snapshot of a model.
#[derive(Debug, Clone)]
pub struct ModelSnapshot {
    pub variables: Vec<VariableView>,
    pub constraints: Vec<ConstraintView>,
    pub coefficients: Option<Vec<CoefficientView>>,
    pub objective: Option<ObjectiveView>,
    pub slacks: Option<Vec<SlackView>>,
    pub metadata: SnapshotMetadata,
}

/// Options for inspecting a model.
#[derive(Debug, Clone)]
pub struct InspectOptions {
    pub include_coefficients: bool,
    pub include_slacks: bool,
    pub variable_filter: Option<Vec<VariableId>>,
    pub constraint_filter: Option<Vec<ConstraintId>>,
}

impl Default for InspectOptions {
    fn default() -> Self {
        Self {
            include_coefficients: false,
            include_slacks: true,
            variable_filter: None,
            constraint_filter: None,
        }
    }
}

impl Model {
    /// Inspect the model structure and return a structured snapshot.
    pub fn inspect(&self, options: InspectOptions) -> ModelSnapshot {
        let InspectOptions {
            include_coefficients,
            include_slacks,
            variable_filter,
            constraint_filter,
        } = options;

        let var_filter: Option<HashSet<VariableId>> =
            variable_filter.map(|ids| ids.into_iter().collect());
        let con_filter: Option<HashSet<ConstraintId>> =
            constraint_filter.map(|ids| ids.into_iter().collect());

        let mut nnz_map: Vec<usize> = vec![0; self.num_constraints()];
        let mut coefficients: Vec<CoefficientView> = Vec::new();

        for (var_id, coeffs) in self.columns() {
            if let Some(filter) = var_filter.as_ref() {
                if !filter.contains(&var_id) {
                    continue;
                }
            }
            for (constraint_id, coeff) in coeffs {
                if let Some(filter) = con_filter.as_ref() {
                    if !filter.contains(constraint_id) {
                        continue;
                    }
                }
                if let Some(entry) = nnz_map.get_mut(constraint_id.inner() as usize) {
                    *entry += 1;
                }
                if include_coefficients {
                    coefficients.push(CoefficientView {
                        variable_id: var_id,
                        constraint_id: *constraint_id,
                        value: *coeff,
                    });
                }
            }
        }

        let variables = self
            .variables
            .iter()
            .enumerate()
            .map(|(idx, bounds)| (VariableId::new(idx as u32), bounds))
            .filter(|(id, _)| var_filter.as_ref().is_none_or(|filter| filter.contains(id)))
            .map(|(id, bounds)| {
                let idx = id.inner() as usize;
                VariableView {
                    id,
                    name: self
                        .variable_names
                        .as_ref()
                        .and_then(|names| names.get(&id).cloned()),
                    bounds: *bounds,
                    is_integer: Self::read_packed_flag(&self.variable_is_integer_bits, idx),
                    is_active: !Self::read_packed_flag(&self.variable_is_inactive_bits, idx),
                    metadata: self
                        .variable_metadata
                        .as_ref()
                        .and_then(|meta| meta.get(&id).cloned()),
                }
            })
            .collect();

        let constraints = self
            .constraints
            .iter()
            .enumerate()
            .map(|(idx, constraint)| (ConstraintId::new(idx as u32), constraint))
            .filter(|(id, _)| con_filter.as_ref().is_none_or(|filter| filter.contains(id)))
            .map(|(id, constraint)| ConstraintView {
                id,
                name: self
                    .constraint_names
                    .as_ref()
                    .and_then(|names| names.get(&id).cloned()),
                bounds: constraint.bounds,
                nnz: *nnz_map.get(id.inner() as usize).unwrap_or(&0),
                metadata: self
                    .constraint_metadata
                    .as_ref()
                    .and_then(|meta| meta.get(&id).cloned()),
            })
            .collect();

        let objective = if self.objective.sense.is_some() || !self.objective.terms.is_empty() {
            Some(ObjectiveView {
                sense: self.objective.sense,
                terms: self.objective.terms.clone(),
                name: self.objective_name.clone(),
            })
        } else {
            None
        };

        let slacks = if include_slacks {
            let filtered: Vec<SlackView> = self
                .slack_handles
                .iter()
                .filter(|handle| {
                    con_filter
                        .as_ref()
                        .is_none_or(|filter| filter.contains(&handle.constraint_id))
                })
                .map(|handle| SlackView {
                    constraint_id: handle.constraint_id,
                    bound: handle.bound,
                    penalty: handle.penalty,
                    variable_ids: handle.var_ids,
                    name: handle.name.clone(),
                })
                .collect();
            Some(filtered)
        } else {
            None
        };

        ModelSnapshot {
            variables,
            constraints,
            coefficients: if include_coefficients {
                Some(coefficients)
            } else {
                None
            },
            objective,
            slacks,
            metadata: SnapshotMetadata {
                variables: self.num_variables(),
                constraints: self.num_constraints(),
                coefficients: self.num_coefficients(),
            },
        }
    }
}
