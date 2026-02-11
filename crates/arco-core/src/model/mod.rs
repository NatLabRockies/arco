//! Model module for building optimization models.
//!
//! This module provides the core [`Model`] type and related structures for building
//! linear and mixed-integer programming models.
//!
//! # Module Organization
//!
//! - [`error`]: Model error types
//! - [`builder`]: Methods for adding variables, constraints, and objectives
//! - [`storage`]: Column-first sparse storage access
//! - [`metadata`]: Variable and constraint naming and metadata
//! - [`slack`]: Slack variable and elastic constraint support
//! - [`inspect`]: Model inspection and snapshots
//! - [`csc_import`]: CSC format import

mod builder;
mod csc_import;
mod error;
mod inspect;
mod metadata;
mod pretty;
mod slack;
mod storage;

use crate::slack::SlackHandle;
use crate::types::{Bounds, Constraint, Objective, SimplifyLevel, Variable};
use arco_expr::ids::{ConstraintId, VariableId};
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

pub use csc_import::CscInput;
pub use error::ModelError;
pub use inspect::{
    CoefficientView, ConstraintView, InspectOptions, ModelSnapshot, ObjectiveView, SlackView,
    SnapshotMetadata, VariableView,
};
pub use pretty::{
    DefaultPrettyPrintAdapter, PrettyBoundGroup, PrettyPrintAdapter, PrettyPrintOptions,
    PrettySection, format_ascii_number,
};

/// A lazy model builder for linear and mixed-integer programs.
///
/// Variables, constraints, and objectives can be added at any time.
/// The internal representation uses column-first sparse storage (CSC format).
#[derive(Debug, Clone)]
pub struct Model {
    pub(crate) variables: Vec<Bounds>,
    pub(crate) variable_is_integer_bits: Vec<u64>,
    pub(crate) variable_is_inactive_bits: Vec<u64>,
    pub(crate) constraints: Vec<Constraint>,
    pub(crate) objective: Objective,
    pub(crate) objective_name: Option<String>,
    simplify_level: SimplifyLevel,
    // Column-first sparse storage: variable_id -> vec of (constraint_id, coefficient)
    pub(crate) columns: HashMap<VariableId, ColumnData>,
    pub(crate) next_variable_id: u32,
    pub(crate) next_constraint_id: u32,
    pub(crate) slack_handles: Vec<SlackHandle>,
    // Lazy-allocated metadata storage
    pub(crate) variable_names: Option<BTreeMap<VariableId, String>>,
    pub(crate) constraint_names: Option<BTreeMap<ConstraintId, String>>,
    pub(crate) variable_metadata: Option<BTreeMap<VariableId, serde_json::Value>>,
    pub(crate) constraint_metadata: Option<BTreeMap<ConstraintId, serde_json::Value>>,
}

const BITS_PER_WORD: usize = u64::BITS as usize;

#[derive(Debug, Clone)]
pub(crate) enum ColumnData {
    Single((ConstraintId, f64)),
    Many(Vec<(ConstraintId, f64)>),
}

impl ColumnData {
    #[inline]
    fn from_entries(entries: Vec<(ConstraintId, f64)>) -> Self {
        if entries.len() == 1 {
            ColumnData::Single(entries[0])
        } else {
            ColumnData::Many(entries)
        }
    }

    #[inline]
    fn upsert(&mut self, constraint_id: ConstraintId, coefficient: f64) {
        match self {
            ColumnData::Single((existing_id, existing_coeff)) => {
                if *existing_id == constraint_id {
                    *existing_coeff = coefficient;
                } else {
                    let previous = (*existing_id, *existing_coeff);
                    *self = ColumnData::Many(vec![previous, (constraint_id, coefficient)]);
                }
            }
            ColumnData::Many(entries) => {
                if let Some(entry) = entries.iter_mut().find(|(cid, _)| *cid == constraint_id) {
                    entry.1 = coefficient;
                } else {
                    entries.push((constraint_id, coefficient));
                }
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            ColumnData::Single(_) => 1,
            ColumnData::Many(entries) => entries.len(),
        }
    }

    #[inline]
    fn as_slice(&self) -> &[(ConstraintId, f64)] {
        match self {
            ColumnData::Single(entry) => std::slice::from_ref(entry),
            ColumnData::Many(entries) => entries.as_slice(),
        }
    }
}

impl Model {
    /// Create a new empty model.
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            variable_is_integer_bits: Vec::new(),
            variable_is_inactive_bits: Vec::new(),
            constraints: Vec::new(),
            objective: Objective::new(),
            objective_name: None,
            simplify_level: SimplifyLevel::default(),
            columns: HashMap::new(),
            next_variable_id: 0,
            next_constraint_id: 0,
            slack_handles: Vec::new(),
            variable_names: None,
            constraint_names: None,
            variable_metadata: None,
            constraint_metadata: None,
        }
    }

    /// Create a new model with pre-allocated storage capacities.
    pub fn with_capacities(variable_capacity: usize, constraint_capacity: usize) -> Self {
        Self {
            variables: Vec::with_capacity(variable_capacity),
            variable_is_integer_bits: Vec::with_capacity(variable_capacity.div_ceil(BITS_PER_WORD)),
            variable_is_inactive_bits: Vec::new(),
            constraints: Vec::with_capacity(constraint_capacity),
            objective: Objective::new(),
            objective_name: None,
            simplify_level: SimplifyLevel::default(),
            columns: HashMap::new(),
            next_variable_id: 0,
            next_constraint_id: 0,
            slack_handles: Vec::new(),
            variable_names: None,
            constraint_names: None,
            variable_metadata: None,
            constraint_metadata: None,
        }
    }

    /// Create a new model with a specified expression simplification level.
    pub fn with_simplify_level(simplify_level: SimplifyLevel) -> Self {
        Self {
            simplify_level,
            ..Self::new()
        }
    }

    /// Get the current expression simplification level.
    pub fn simplify_level(&self) -> SimplifyLevel {
        self.simplify_level
    }

    /// Update the expression simplification level.
    pub fn set_expr_simplify(&mut self, simplify_level: SimplifyLevel) -> Result<(), ModelError> {
        self.simplify_level = simplify_level;
        tracing::debug!(
            component = "model",
            operation = "set_expr_simplify",
            status = "success",
            simplify_level = simplify_level.as_str(),
            "Updated expression simplification level"
        );
        Ok(())
    }

    /// Get the objective
    pub fn objective(&self) -> &Objective {
        &self.objective
    }

    #[inline]
    pub(crate) fn push_variable(&mut self, variable: Variable) {
        let idx = self.variables.len();
        self.variables.push(variable.bounds);
        if variable.is_integer {
            Self::write_packed_flag(&mut self.variable_is_integer_bits, idx, true);
        }
        if !variable.is_active {
            Self::write_packed_flag(&mut self.variable_is_inactive_bits, idx, true);
        }
    }

    #[inline]
    pub(crate) fn get_variable_by_index(&self, idx: usize) -> Option<Variable> {
        let bounds = *self.variables.get(idx)?;
        Some(Variable {
            bounds,
            is_integer: Self::read_packed_flag(&self.variable_is_integer_bits, idx),
            is_active: !Self::read_packed_flag(&self.variable_is_inactive_bits, idx),
        })
    }

    #[inline]
    pub(crate) fn set_variable_active_by_index(&mut self, idx: usize, active: bool) -> bool {
        if idx >= self.variables.len() {
            return false;
        }
        Self::write_packed_flag(&mut self.variable_is_inactive_bits, idx, !active);
        true
    }

    #[inline]
    pub(crate) fn variable_is_active_by_index(&self, idx: usize) -> Option<bool> {
        if idx >= self.variables.len() {
            return None;
        }
        Some(!Self::read_packed_flag(
            &self.variable_is_inactive_bits,
            idx,
        ))
    }

    #[inline]
    fn read_packed_flag(bits: &[u64], idx: usize) -> bool {
        let word = idx / BITS_PER_WORD;
        let bit = idx % BITS_PER_WORD;
        bits.get(word)
            .is_some_and(|entry| (entry & (1_u64 << bit)) != 0)
    }

    #[inline]
    fn write_packed_flag(bits: &mut Vec<u64>, idx: usize, value: bool) {
        let word = idx / BITS_PER_WORD;
        let bit = idx % BITS_PER_WORD;
        let mask = 1_u64 << bit;
        if value {
            if bits.len() <= word {
                bits.resize(word + 1, 0);
            }
            bits[word] |= mask;
        } else if bits.len() > word {
            bits[word] &= !mask;
        }
    }

    pub(crate) fn ensure_variable_exists(&self, id: VariableId) -> Result<(), ModelError> {
        if (id.inner() as usize) < self.variables.len() {
            Ok(())
        } else {
            Err(ModelError::InvalidVariableId(id))
        }
    }

    pub(crate) fn ensure_constraint_exists(&self, id: ConstraintId) -> Result<(), ModelError> {
        if (id.inner() as usize) < self.constraints.len() {
            Ok(())
        } else {
            Err(ModelError::InvalidConstraintId(id))
        }
    }

    pub(crate) fn normalize_terms(&self, terms: Vec<(VariableId, f64)>) -> Vec<(VariableId, f64)> {
        let started = Instant::now();
        let terms_in = terms.len();

        let mut merged: BTreeMap<VariableId, f64> = BTreeMap::new();
        match self.simplify_level {
            SimplifyLevel::None => {
                for (var_id, coeff) in terms {
                    *merged.entry(var_id).or_insert(0.0) += coeff;
                }
            }
            SimplifyLevel::Light => {
                for (var_id, coeff) in terms {
                    if coeff == 0.0 {
                        continue;
                    }
                    *merged.entry(var_id).or_insert(0.0) += coeff;
                }
            }
        }

        let normalized: Vec<(VariableId, f64)> = merged
            .into_iter()
            .filter(|(_, coeff)| *coeff != 0.0)
            .collect();

        tracing::debug!(
            component = "model",
            operation = "lower_expr",
            status = "success",
            simplify_level = self.simplify_level.as_str(),
            expr_terms_in = terms_in,
            expr_terms_out = normalized.len(),
            duration_ms = started.elapsed().as_secs_f64() * 1000.0,
            "Lowered linear expression"
        );

        normalized
    }

    pub(crate) fn add_objective_terms(&mut self, terms: Vec<(VariableId, f64)>) {
        if terms.is_empty() {
            return;
        }
        let mut merged = self.objective.terms.clone();
        merged.extend(terms);
        self.objective.terms = self.normalize_terms(merged);
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn slack_variable_name(
    explicit: Option<&str>,
    constraint_name: Option<&str>,
    suffix: &str,
    force_suffix: bool,
) -> Option<String> {
    match explicit {
        Some(name) if force_suffix => Some(format!("{name}:{suffix}")),
        Some(name) => Some(name.to_string()),
        None => constraint_name.map(|name| format!("{name}:{suffix}")),
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::types::{Bounds, Constraint, Objective, Sense, Variable};
    use arco_expr::expr::{ComparisonSense, ConstraintExpr, Expr};

    mod metadata_inspect;
    mod slack_csc;
    mod support;

    #[test]
    fn test_new_model_is_empty() {
        let model = Model::new();
        assert_eq!(model.num_variables(), 0);
        assert_eq!(model.num_constraints(), 0);
    }

    #[test]
    fn test_add_variable() {
        let mut model = Model::new();
        let var = Variable {
            bounds: Bounds::new(0.0, 10.0),
            is_integer: false,
            is_active: true,
        };

        let id = model.add_variable(var).unwrap();
        assert_eq!(model.num_variables(), 1);
        assert_eq!(model.get_variable(id).unwrap(), var);
    }

    #[test]
    fn test_model_with_capacities() {
        let model = Model::with_capacities(32, 16);
        assert_eq!(model.num_variables(), 0);
        assert_eq!(model.num_constraints(), 0);
        assert!(model.variables.capacity() >= 32);
        assert!(model.constraints.capacity() >= 16);
    }

    #[test]
    fn test_variable_flags_are_packed() {
        let mut model = Model::new();
        for idx in 0..130 {
            model
                .add_variable(Variable {
                    bounds: Bounds::new(0.0, 1.0),
                    is_integer: idx % 2 == 0,
                    is_active: idx % 3 != 0,
                })
                .unwrap();
        }

        assert_eq!(model.variable_is_integer_bits.len(), 3);
        assert_eq!(model.variable_is_inactive_bits.len(), 3);

        let var_64 = model.get_variable(VariableId::new(64)).unwrap();
        assert!(var_64.is_integer);
        assert!(var_64.is_active);

        let var_129 = model.get_variable(VariableId::new(129)).unwrap();
        assert!(!var_129.is_integer);
        assert!(!var_129.is_active);
    }

    #[test]
    fn test_default_variable_flags_do_not_allocate_words() {
        let mut model = Model::new();
        for _ in 0..1_024 {
            model
                .add_variable(Variable {
                    bounds: Bounds::new(0.0, 1.0),
                    is_integer: false,
                    is_active: true,
                })
                .unwrap();
        }
        assert!(model.variable_is_integer_bits.is_empty());
        assert!(model.variable_is_inactive_bits.is_empty());
    }

    #[test]
    fn test_variable_activation_toggle() {
        let mut model = Model::new();
        let var = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        assert!(model.is_variable_active(var).unwrap());
        model.deactivate_variable(var).unwrap();
        assert!(!model.is_variable_active(var).unwrap());
        model.activate_variable(var).unwrap();
        assert!(model.is_variable_active(var).unwrap());
    }

    #[test]
    fn test_add_constraint() {
        let mut model = Model::new();
        let constraint = Constraint {
            bounds: Bounds::new(0.0, 100.0),
        };

        let id = model.add_constraint(constraint).unwrap();
        assert_eq!(model.num_constraints(), 1);
        assert_eq!(model.get_constraint(id).unwrap(), &constraint);
    }

    #[test]
    fn test_set_objective() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        let objective = Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(var_id, 1.0)],
        };

        model.set_objective(objective).unwrap();
        assert_eq!(model.objective().sense, Some(Sense::Minimize));
        assert_eq!(model.objective().terms.len(), 1);
    }

    #[test]
    fn test_set_objective_rejects_missing_sense() {
        let mut model = Model::new();
        let objective = Objective {
            sense: None,
            terms: Vec::new(),
        };

        let result = model.set_objective(objective);
        assert_eq!(result, Err(ModelError::NoObjective));
    }

    #[test]
    fn test_objective_name_roundtrip() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 1.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(var_id, 1.0)],
            })
            .unwrap();
        model.set_objective_name(Some("cost".to_string())).unwrap();
        assert_eq!(model.get_objective_name(), Some("cost"));

        model
            .set_objective(Objective {
                sense: Some(Sense::Maximize),
                terms: vec![(var_id, 2.0)],
            })
            .unwrap();
        assert!(model.get_objective_name().is_none());
    }

    #[test]
    fn test_multiple_objectives_rejected() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        model.minimize(Expr::term(var_id, 1.0)).unwrap();

        let result = model.maximize(Expr::term(var_id, 1.0));
        assert_eq!(result, Err(ModelError::MultipleObjectives));
    }

    #[test]
    fn test_set_coefficient() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        let constraint_id = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 100.0),
            })
            .unwrap();

        model.set_coefficient(var_id, constraint_id, 2.5).unwrap();
    }

    #[test]
    fn test_set_coefficient_rejects_non_finite() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let constraint_id = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 100.0),
            })
            .unwrap();

        assert!(
            model
                .set_coefficient(var_id, constraint_id, f64::INFINITY)
                .is_err()
        );
        assert!(
            model
                .set_coefficient(var_id, constraint_id, f64::NAN)
                .is_err()
        );
    }

    #[test]
    fn test_set_coefficient_with_invalid_variable_fails() {
        let mut model = Model::new();
        let invalid_var_id = VariableId::new(999);
        let constraint_id = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 100.0),
            })
            .unwrap();

        let result = model.set_coefficient(invalid_var_id, constraint_id, 2.5);
        assert_eq!(result, Err(ModelError::InvalidVariableId(invalid_var_id)));
    }

    #[test]
    fn test_set_coefficient_with_invalid_constraint_fails() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        let invalid_constraint_id = ConstraintId::new(999);

        let result = model.set_coefficient(var_id, invalid_constraint_id, 2.5);
        assert_eq!(
            result,
            Err(ModelError::InvalidConstraintId(invalid_constraint_id))
        );
    }

    #[test]
    fn test_columns_are_lazily_allocated() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        assert!(model.columns.is_empty());
        assert_eq!(
            model.get_column(var_id).expect("column should exist"),
            &Vec::new()
        );

        let constraint_id = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 100.0),
            })
            .unwrap();
        model.set_coefficient(var_id, constraint_id, 1.0).unwrap();
        assert_eq!(model.columns.len(), 1);
    }

    #[test]
    fn test_coefficients_persist_in_columns() {
        let mut model = Model::new();
        let v1 = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let v2 = model
            .add_variable(Variable {
                bounds: Bounds::new(-5.0, 5.0),
                is_integer: true,
                is_active: true,
            })
            .unwrap();

        let c1 = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 15.0),
            })
            .unwrap();
        let c2 = model
            .add_constraint(Constraint {
                bounds: Bounds::new(-10.0, 10.0),
            })
            .unwrap();

        model.set_coefficient(v1, c1, 1.5).unwrap();
        model.set_coefficient(v1, c2, -2.0).unwrap();
        model.set_coefficient(v2, c2, 3.5).unwrap();

        let col_v1 = model.get_column(v1).expect("v1 column missing");
        assert_eq!(col_v1, &vec![(c1, 1.5), (c2, -2.0)]);

        let col_v2 = model.get_column(v2).expect("v2 column missing");
        assert_eq!(col_v2, &vec![(c2, 3.5)]);
    }

    #[test]
    fn test_single_entry_columns_use_inline_storage() {
        let mut model = Model::new();
        let var = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 1.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let con = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 1.0),
            })
            .unwrap();

        model.set_coefficient(var, con, 4.0).unwrap();

        let stored = model.columns.get(&var).expect("column exists");
        assert!(matches!(stored, ColumnData::Single(_)));
    }

    #[test]
    fn test_binary_variable_constructor() {
        let var = Variable::binary();
        assert_eq!(var.bounds.lower, 0.0);
        assert_eq!(var.bounds.upper, 1.0);
        assert!(var.is_integer);
    }

    #[test]
    fn test_continuous_variable_constructor() {
        let var = Variable::continuous(Bounds::new(2.5, 10.5));
        assert_eq!(var.bounds.lower, 2.5);
        assert_eq!(var.bounds.upper, 10.5);
        assert!(!var.is_integer);
    }

    #[test]
    fn test_integer_variable_constructor() {
        let var = Variable::integer(Bounds::new(0.0, 100.0));
        assert_eq!(var.bounds.lower, 0.0);
        assert_eq!(var.bounds.upper, 100.0);
        assert!(var.is_integer);
    }

    #[test]
    fn test_add_binary_variable() {
        let mut model = Model::new();
        let var_id = model.add_variable(Variable::binary()).unwrap();
        let var = model.get_variable(var_id).unwrap();
        assert_eq!(var.bounds.lower, 0.0);
        assert_eq!(var.bounds.upper, 1.0);
        assert!(var.is_integer);
    }

    #[test]
    fn test_multiple_variables_and_constraints() {
        let mut model = Model::new();

        let var1 = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        let var2 = model
            .add_variable(Variable {
                bounds: Bounds::new(-5.0, 5.0),
                is_integer: true,
                is_active: true,
            })
            .unwrap();

        let constraint1 = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 20.0),
            })
            .unwrap();

        let constraint2 = model
            .add_constraint(Constraint {
                bounds: Bounds::new(-10.0, 10.0),
            })
            .unwrap();

        model.set_coefficient(var1, constraint1, 1.0).unwrap();
        model.set_coefficient(var2, constraint1, 2.0).unwrap();
        model.set_coefficient(var1, constraint2, -1.0).unwrap();
        model.set_coefficient(var2, constraint2, 1.0).unwrap();

        assert_eq!(model.num_variables(), 2);
        assert_eq!(model.num_constraints(), 2);
    }

    #[test]
    fn test_add_constraint_expr() {
        let mut model = Model::new();
        let var = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .unwrap();
        let expr = Expr::term(var, 1.0);
        let constraint = ConstraintExpr::new(expr, ComparisonSense::GreaterEqual, 2.0);

        let con = model.add_constraint_expr(constraint).unwrap();
        let stored = model.get_constraint(con).unwrap();
        assert_eq!(stored.bounds.lower, 2.0);
        assert!(stored.bounds.upper.is_infinite());
    }

    #[test]
    fn test_variable_bounds_validation() {
        let mut model = Model::new();
        let result = model.add_variable(Variable {
            bounds: Bounds::new(5.0, 1.0),
            is_integer: false,
            is_active: true,
        });
        assert!(matches!(
            result,
            Err(ModelError::InvalidVariableBounds { .. })
        ));
    }

    #[test]
    fn test_constraint_bounds_validation() {
        let mut model = Model::new();
        let result = model.add_constraint(Constraint {
            bounds: Bounds::new(10.0, 0.0),
        });
        assert!(matches!(
            result,
            Err(ModelError::InvalidConstraintBounds { .. })
        ));
    }

    #[test]
    fn test_variable_bounds_reject_nan() {
        let mut model = Model::new();
        let result = model.add_variable(Variable {
            bounds: Bounds::new(f64::NAN, 1.0),
            is_integer: false,
            is_active: true,
        });
        assert!(matches!(
            result,
            Err(ModelError::InvalidVariableBounds { .. })
        ));
    }

    #[test]
    fn test_constraint_bounds_reject_nan() {
        let mut model = Model::new();
        let result = model.add_constraint(Constraint {
            bounds: Bounds::new(0.0, f64::NAN),
        });
        assert!(matches!(
            result,
            Err(ModelError::InvalidConstraintBounds { .. })
        ));
    }

    #[test]
    fn test_set_objective_rejects_non_finite_coefficients() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 1.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let result = model.set_objective(Objective {
            sense: Some(Sense::Minimize),
            terms: vec![(var_id, f64::INFINITY)],
        });
        assert!(result.is_err());
    }
}
