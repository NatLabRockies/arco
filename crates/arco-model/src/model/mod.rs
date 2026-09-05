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
//! - [`sparse`]: Sparse matrix exports (CSC/CRS/COO)

mod builder;
mod csc_import;
mod error;
mod inspect;
mod metadata;
mod pretty;
mod slack;
mod sparse;
mod storage;
mod view;

use crate::ids::{ConstraintId, VariableId};
use crate::slack::SlackHandle;
use crate::types::{Bounds, Constraint, Objective, SimplifyLevel, Variable};
use smallvec::SmallVec;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

/// Column storage type: inline for ≤2 constraint entries (the common case),
/// heap-allocated otherwise.
pub(crate) type ColumnVec = SmallVec<[(ConstraintId, f64); 2]>;

pub use csc_import::CscInput;
pub use error::ModelError;
pub use inspect::{
    CoefficientView, ConstraintView, InspectOptions, ModelSnapshot, ObjectiveView, SlackView,
    SnapshotMemoryEstimate, SnapshotMetadata, VariableView,
};
pub use pretty::{
    DefaultPrettyPrintAdapter, PrettyBoundGroup, PrettyPrintAdapter, PrettyPrintOptions,
    PrettySection, format_ascii_number,
};
pub use sparse::{CooMatrix, CrsMatrix, CscMatrix, SparseMatrixExport};
pub use view::{ModelFingerprint, ModelPatch, ModelView, PatchedModelView, StructuralFacts};

/// A lazy model builder for linear and mixed-integer programs.
///
/// Variables, constraints, and objectives can be added at any time.
/// The internal representation uses column-first sparse storage (CSC format).
#[derive(Debug, Clone)]
pub struct Model {
    variables: Vec<Bounds>,
    variable_is_integer_bits: Vec<u64>,
    variable_is_inactive_bits: Vec<u64>,
    constraints: Vec<Constraint>,
    objective: Objective,
    objective_name: Option<String>,
    simplify_level: SimplifyLevel,
    // Column-first sparse storage: indexed by variable_id, each entry is a list of
    // (constraint_id, coefficient) pairs. Uses SmallVec for inline storage of ≤2 entries.
    columns: Vec<ColumnVec>,
    // Number of stored matrix entries, including duplicate rows and explicit zeros.
    coefficient_count: usize,
    next_variable_id: u32,
    next_constraint_id: u32,
    slack_handles: Vec<SlackHandle>,
    // Lazy-allocated metadata storage
    variable_names: Option<BTreeMap<VariableId, String>>,
    constraint_names: Option<BTreeMap<ConstraintId, String>>,
    variable_metadata: Option<BTreeMap<VariableId, serde_json::Value>>,
    constraint_metadata: Option<BTreeMap<ConstraintId, serde_json::Value>>,
    // Reverse lookup for O(1) name-to-id resolution
    variable_name_to_id: Option<HashMap<String, VariableId>>,
    constraint_name_to_id: Option<HashMap<String, ConstraintId>>,
}

pub(crate) const BITS_PER_WORD: usize = u64::BITS as usize;

#[inline]
pub(crate) fn bounds_are_valid(lower: f64, upper: f64) -> bool {
    !lower.is_nan() && !upper.is_nan() && lower <= upper
}

#[inline]
pub(crate) fn coefficient_is_valid(coefficient: f64) -> bool {
    coefficient.is_finite()
}

#[inline]
pub(crate) fn slack_penalty_is_valid(penalty: f64) -> bool {
    penalty.is_finite() && penalty >= 0.0
}

/// Upsert a coefficient in a column: update existing entry or append.
///
/// Returns `true` when a new entry was appended and `false` when an existing
/// entry was updated.
#[inline]
pub(crate) fn column_upsert(
    column: &mut ColumnVec,
    constraint_id: ConstraintId,
    coefficient: f64,
) -> bool {
    if let Some(entry) = column.iter_mut().find(|(cid, _)| *cid == constraint_id) {
        entry.1 = coefficient;
        false
    } else {
        column.push((constraint_id, coefficient));
        true
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
            columns: Vec::new(),
            coefficient_count: 0,
            next_variable_id: 0,
            next_constraint_id: 0,
            slack_handles: Vec::new(),
            variable_names: None,
            constraint_names: None,
            variable_metadata: None,
            constraint_metadata: None,
            variable_name_to_id: None,
            constraint_name_to_id: None,
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
            columns: Vec::with_capacity(variable_capacity),
            coefficient_count: 0,
            next_variable_id: 0,
            next_constraint_id: 0,
            slack_handles: Vec::new(),
            variable_names: None,
            constraint_names: None,
            variable_metadata: None,
            constraint_metadata: None,
            variable_name_to_id: None,
            constraint_name_to_id: None,
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
    fn push_variable(&mut self, variable: Variable) {
        let idx = self.variables.len();
        self.variables.push(variable.bounds);
        self.columns.push(ColumnVec::new());
        if variable.is_integer {
            Self::write_packed_flag(&mut self.variable_is_integer_bits, idx, true);
        }
        if !variable.is_active {
            Self::write_packed_flag(&mut self.variable_is_inactive_bits, idx, true);
        }
    }

    #[inline]
    fn get_variable_by_index(&self, idx: usize) -> Option<Variable> {
        let bounds = *self.variables.get(idx)?;
        Some(Variable {
            bounds,
            is_integer: Self::read_packed_flag(&self.variable_is_integer_bits, idx),
            is_active: !Self::read_packed_flag(&self.variable_is_inactive_bits, idx),
        })
    }

    #[inline]
    fn set_variable_active_by_index(&mut self, idx: usize, active: bool) -> bool {
        if idx >= self.variables.len() {
            return false;
        }
        Self::write_packed_flag(&mut self.variable_is_inactive_bits, idx, !active);
        true
    }

    #[inline]
    fn variable_is_active_by_index(&self, idx: usize) -> Option<bool> {
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

    fn ensure_variable_exists(&self, id: VariableId) -> Result<(), ModelError> {
        if (id.inner() as usize) < self.variables.len() {
            Ok(())
        } else {
            Err(ModelError::InvalidVariableId(id))
        }
    }

    fn ensure_constraint_exists(&self, id: ConstraintId) -> Result<(), ModelError> {
        if (id.inner() as usize) < self.constraints.len() {
            Ok(())
        } else {
            Err(ModelError::InvalidConstraintId(id))
        }
    }

    /// Normalize terms with minimal allocations.
    /// Uses in-place deduplication instead of HashMap to reduce allocations.
    fn normalize_terms(&self, mut terms: Vec<(VariableId, f64)>) -> Vec<(VariableId, f64)> {
        let started = Instant::now();
        let terms_in = terms.len();

        let skip_zeros = matches!(self.simplify_level, SimplifyLevel::Light);

        // Pre-filter zeros if needed (in-place)
        if skip_zeros {
            terms.retain(|(_, coeff)| *coeff != 0.0);
        }

        // Sort by variable ID to enable in-place deduplication
        terms.sort_unstable_by_key(|(id, _)| id.inner());

        // In-place deduplication: accumulate coefficients for same variable
        if terms.len() > 1 {
            let mut write_idx = 0;
            for read_idx in 1..terms.len() {
                if terms[read_idx].0 == terms[write_idx].0 {
                    // Same variable: accumulate coefficient
                    let coeff = terms[read_idx].1;
                    terms[write_idx].1 += coeff;
                } else {
                    // Different variable: move to next write position
                    write_idx += 1;
                    terms[write_idx] = terms[read_idx];
                }
            }
            // Truncate to remove duplicates
            terms.truncate(write_idx + 1);
        }

        // Filter out zero coefficients post-merge
        terms.retain(|(_, coeff)| *coeff != 0.0);

        tracing::debug!(
            component = "model",
            operation = "lower_expr",
            status = "success",
            simplify_level = self.simplify_level.as_str(),
            expr_terms_in = terms_in,
            expr_terms_out = terms.len(),
            duration_ms = started.elapsed().as_secs_f64() * 1000.0,
            "Lowered linear expression"
        );

        terms
    }

    pub fn add_objective_terms(&mut self, terms: Vec<(VariableId, f64)>) -> Result<(), ModelError> {
        if self.objective.sense.is_none() {
            return Err(ModelError::NoObjective);
        }
        if terms.is_empty() {
            return Ok(());
        }
        for (var_id, coeff) in &terms {
            self.ensure_variable_exists(*var_id)?;
            if !coefficient_is_valid(*coeff) {
                return Err(ModelError::InvalidCoefficient {
                    coefficient: *coeff,
                });
            }
        }
        let mut merged = std::mem::take(&mut self.objective.terms);
        merged.extend(terms);
        self.objective.terms = self.normalize_terms(merged);
        Ok(())
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
    use crate::expr::{ComparisonSense, ConstraintExpr, Expr};
    use crate::types::{Bounds, Constraint, Objective, Sense, Variable};
    use std::mem::{align_of, size_of};

    mod coefficient_count;
    mod metadata_inspect;
    mod slack_csc;
    mod sparse_export;
    mod support;

    #[test]
    fn column_storage_fits_its_pointer_aligned_memory_budget() {
        type Entry = (ConstraintId, f64);

        let data_alignment = align_of::<[Entry; 2]>().max(align_of::<(*mut Entry, usize)>());
        let data_offset = size_of::<usize>().div_ceil(data_alignment) * data_alignment;
        let data_size = size_of::<[Entry; 2]>().max(size_of::<(*mut Entry, usize)>());
        let expected_size = (data_offset + data_size).div_ceil(data_alignment) * data_alignment;

        assert!(size_of::<ColumnVec>() <= expected_size);
    }

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
    fn test_reserve_constraints_preallocates_without_adding_rows() {
        let mut model = Model::new();

        model.reserve_constraints(16);

        assert_eq!(model.num_constraints(), 0);
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
    fn test_add_objective_terms_merges_with_existing_objective_terms() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let y = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.0)],
            })
            .unwrap();

        model
            .add_objective_terms(vec![(x, 2.0), (y, 3.0), (x, -1.0)])
            .unwrap();

        assert_eq!(model.objective().sense, Some(Sense::Minimize));
        let mut terms = model.objective().terms.clone();
        terms.sort_by_key(|(id, _)| id.inner());
        assert_eq!(terms, vec![(x, 2.0), (y, 3.0)]);
    }

    #[test]
    fn test_add_objective_terms_noop_on_empty_terms() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.0)],
            })
            .unwrap();

        model.add_objective_terms(Vec::new()).unwrap();

        assert_eq!(model.objective().terms, vec![(x, 1.0)]);
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
    fn test_columns_grow_with_variables() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();

        assert_eq!(model.columns.len(), 1);
        assert!(model.columns[var_id.inner() as usize].is_empty());
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
        assert_eq!(model.columns[var_id.inner() as usize].len(), 1);
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

        let stored = &model.columns[var.inner() as usize];
        assert_eq!(stored.len(), 1);
        // SmallVec with ≤2 entries uses inline storage (no heap allocation)
        assert!(!stored.spilled());
    }

    #[test]
    fn test_column_storage_spills_after_two_entries_without_changing_slices() {
        let mut model = Model::new();
        let variables: Vec<_> = (0..5)
            .map(|_| {
                model
                    .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
                    .unwrap()
            })
            .collect();
        let constraints: Vec<_> = (0..9)
            .map(|_| {
                model
                    .add_constraint(Constraint {
                        bounds: Bounds::new(0.0, 1.0),
                    })
                    .unwrap()
            })
            .collect();

        for (variable, entry_count) in variables.iter().zip([0, 1, 2, 3, 9]) {
            for (index, constraint) in constraints.iter().take(entry_count).enumerate() {
                model
                    .set_coefficient(*variable, *constraint, (index + 1) as f64)
                    .unwrap();
            }
        }

        for (variable, entry_count) in variables.iter().zip([0, 1, 2, 3, 9]) {
            let column = model.get_column(*variable).expect("column should exist");
            assert_eq!(column.len(), entry_count);
            let expected = constraints
                .iter()
                .take(entry_count)
                .enumerate()
                .map(|(index, constraint)| (*constraint, (index + 1) as f64))
                .collect::<Vec<_>>();
            assert_eq!(column, expected.as_slice());
            assert_eq!(
                model.columns[variable.inner() as usize].spilled(),
                entry_count > 2
            );
        }
        assert_eq!(model.num_coefficients(), 15);
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
