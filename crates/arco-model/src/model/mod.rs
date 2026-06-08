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

const VARIABLE_BOUNDS_FREE: u32 = 0;
const VARIABLE_BOUNDS_NONNEGATIVE: u32 = 1;
const VARIABLE_BOUNDS_UNIT: u32 = 2;
const VARIABLE_BOUNDS_CUSTOM_BASE: u32 = 3;

const CONSTRAINT_BOUNDS_FREE: u32 = 0;
const CONSTRAINT_BOUNDS_EQ_ZERO: u32 = 1;
const CONSTRAINT_BOUNDS_UPPER_ZERO: u32 = 2;
const CONSTRAINT_BOUNDS_LOWER_ZERO: u32 = 3;
const CONSTRAINT_BOUNDS_CUSTOM_BASE: u32 = 4;

const FREE_BOUNDS: Bounds = Bounds {
    lower: f64::NEG_INFINITY,
    upper: f64::INFINITY,
};
const NONNEGATIVE_BOUNDS: Bounds = Bounds {
    lower: 0.0,
    upper: f64::INFINITY,
};
const UNIT_BOUNDS: Bounds = Bounds {
    lower: 0.0,
    upper: 1.0,
};

const FREE_CONSTRAINT: Constraint = Constraint {
    bounds: FREE_BOUNDS,
};
const EQ_ZERO_CONSTRAINT: Constraint = Constraint {
    bounds: Bounds {
        lower: 0.0,
        upper: 0.0,
    },
};
const UPPER_ZERO_CONSTRAINT: Constraint = Constraint {
    bounds: Bounds {
        lower: f64::NEG_INFINITY,
        upper: 0.0,
    },
};
const LOWER_ZERO_CONSTRAINT: Constraint = Constraint {
    bounds: Bounds {
        lower: 0.0,
        upper: f64::INFINITY,
    },
};

/// Compact variable-bound storage.
///
/// Common bounds are represented as one u32 code per variable. Custom bounds
/// are deduplicated by exact f64 bit pattern so repeated array-bound variables
/// retain one side-table entry instead of one full `Bounds` per variable.
#[derive(Debug, Clone, Default)]
pub(crate) struct VariableStore {
    codes: Vec<u32>,
    custom: Vec<Bounds>,
    custom_lookup: HashMap<(u64, u64), u32>,
}

impl VariableStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            codes: Vec::with_capacity(capacity),
            custom: Vec::new(),
            custom_lookup: HashMap::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.codes.len()
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.codes.capacity()
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        self.codes.reserve(additional);
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.codes.shrink_to_fit();
        self.custom.shrink_to_fit();
        self.custom_lookup.shrink_to_fit();
    }

    pub(crate) fn push_bounds(&mut self, bounds: Bounds) -> Result<(), ModelError> {
        let code = self.code_for_bounds(bounds)?;
        self.codes.push(code);
        Ok(())
    }

    pub(crate) fn extend_repeated(
        &mut self,
        bounds: Bounds,
        count: usize,
    ) -> Result<(), ModelError> {
        let code = self.code_for_bounds(bounds)?;
        self.codes.resize(self.codes.len() + count, code);
        Ok(())
    }

    pub(crate) fn extend_from_slice(&mut self, bounds: &[Bounds]) -> Result<(), ModelError> {
        self.codes.reserve(bounds.len());
        let mut last_key: Option<(u64, u64)> = None;
        let mut last_code = 0;
        for bound in bounds {
            let code = if let Some(code) = common_variable_bound_code(*bound) {
                code
            } else {
                let key = bounds_key(*bound);
                if Some(key) == last_key {
                    last_code
                } else {
                    let code = self.custom_code_for_key(key, *bound)?;
                    last_key = Some(key);
                    last_code = code;
                    code
                }
            };
            self.codes.push(code);
        }
        Ok(())
    }

    pub(crate) fn get_bounds(&self, index: usize) -> Option<Bounds> {
        let code = *self.codes.get(index)?;
        self.bounds_for_code(code)
    }

    pub(crate) fn iter_bounds(&self) -> impl Iterator<Item = Bounds> + '_ {
        self.codes
            .iter()
            .filter_map(|code| self.bounds_for_code(*code))
    }

    fn code_for_bounds(&mut self, bounds: Bounds) -> Result<u32, ModelError> {
        if let Some(code) = common_variable_bound_code(bounds) {
            return Ok(code);
        }
        self.custom_code_for_key(bounds_key(bounds), bounds)
    }

    fn custom_code_for_key(&mut self, key: (u64, u64), bounds: Bounds) -> Result<u32, ModelError> {
        if let Some(code) = self.custom_lookup.get(&key) {
            return Ok(*code);
        }
        let custom_idx =
            u32::try_from(self.custom.len()).map_err(|_| ModelError::InvalidCscData {
                reason: "too many custom variable bounds for compact storage".to_string(),
            })?;
        let code = VARIABLE_BOUNDS_CUSTOM_BASE
            .checked_add(custom_idx)
            .ok_or_else(|| ModelError::InvalidCscData {
                reason: "too many custom variable bounds for compact storage".to_string(),
            })?;
        self.custom.push(bounds);
        self.custom_lookup.insert(key, code);
        Ok(code)
    }

    fn bounds_for_code(&self, code: u32) -> Option<Bounds> {
        match code {
            VARIABLE_BOUNDS_FREE => Some(FREE_BOUNDS),
            VARIABLE_BOUNDS_NONNEGATIVE => Some(NONNEGATIVE_BOUNDS),
            VARIABLE_BOUNDS_UNIT => Some(UNIT_BOUNDS),
            custom_code => {
                let custom_idx = (custom_code - VARIABLE_BOUNDS_CUSTOM_BASE) as usize;
                self.custom.get(custom_idx).copied()
            }
        }
    }
}

#[inline]
fn bounds_key(bounds: Bounds) -> (u64, u64) {
    (bounds.lower.to_bits(), bounds.upper.to_bits())
}

#[inline]
fn common_variable_bound_code(bounds: Bounds) -> Option<u32> {
    if bounds == FREE_BOUNDS {
        Some(VARIABLE_BOUNDS_FREE)
    } else if bounds == NONNEGATIVE_BOUNDS {
        Some(VARIABLE_BOUNDS_NONNEGATIVE)
    } else if bounds == UNIT_BOUNDS {
        Some(VARIABLE_BOUNDS_UNIT)
    } else {
        None
    }
}

/// Compact constraint-bound storage.
///
/// Common row bounds are represented as one u32 code per constraint. Uncommon
/// bounds are stored once in a side table and referenced by code. This preserves
/// stable constraint IDs while avoiding a 16-byte `Constraint` for every row.
#[derive(Debug, Clone, Default)]
pub(crate) struct ConstraintStore {
    codes: Vec<u32>,
    custom: Vec<Constraint>,
    recent_custom: SmallVec<[((u64, u64), u32); 8]>,
}

impl ConstraintStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            codes: Vec::with_capacity(capacity),
            custom: Vec::new(),
            recent_custom: SmallVec::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.codes.len()
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.codes.capacity()
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        self.codes.reserve(additional);
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.codes.shrink_to_fit();
        self.custom.shrink_to_fit();
    }

    pub(crate) fn push(&mut self, constraint: Constraint) {
        let code = common_constraint_bound_code(constraint.bounds)
            .unwrap_or_else(|| self.custom_code_for_constraint(constraint));
        self.codes.push(code);
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Constraint> {
        let code = *self.codes.get(index)?;
        Some(self.constraint_for_code(code))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Constraint> + '_ {
        self.codes
            .iter()
            .map(|code| self.constraint_for_code(*code))
    }

    fn constraint_for_code(&self, code: u32) -> &Constraint {
        match code {
            CONSTRAINT_BOUNDS_FREE => &FREE_CONSTRAINT,
            CONSTRAINT_BOUNDS_EQ_ZERO => &EQ_ZERO_CONSTRAINT,
            CONSTRAINT_BOUNDS_UPPER_ZERO => &UPPER_ZERO_CONSTRAINT,
            CONSTRAINT_BOUNDS_LOWER_ZERO => &LOWER_ZERO_CONSTRAINT,
            custom_code => {
                let custom_idx = (custom_code - CONSTRAINT_BOUNDS_CUSTOM_BASE) as usize;
                &self.custom[custom_idx]
            }
        }
    }

    fn custom_code_for_constraint(&mut self, constraint: Constraint) -> u32 {
        let key = bounds_key(constraint.bounds);
        if let Some((_, code)) = self
            .recent_custom
            .iter()
            .find(|(recent_key, _)| *recent_key == key)
        {
            return *code;
        }

        let custom_idx = self.custom.len() as u32;
        let code = CONSTRAINT_BOUNDS_CUSTOM_BASE + custom_idx;
        self.custom.push(constraint);
        if self.recent_custom.len() == self.recent_custom.inline_size() {
            self.recent_custom.remove(0);
        }
        self.recent_custom.push((key, code));
        code
    }
}

#[inline]
fn common_constraint_bound_code(bounds: Bounds) -> Option<u32> {
    match bounds {
        Bounds {
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        } => Some(CONSTRAINT_BOUNDS_FREE),
        Bounds {
            lower: 0.0,
            upper: 0.0,
        } => Some(CONSTRAINT_BOUNDS_EQ_ZERO),
        Bounds {
            lower: f64::NEG_INFINITY,
            upper: 0.0,
        } => Some(CONSTRAINT_BOUNDS_UPPER_ZERO),
        Bounds {
            lower: 0.0,
            upper: f64::INFINITY,
        } => Some(CONSTRAINT_BOUNDS_LOWER_ZERO),
        _ => None,
    }
}

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
    pub(crate) variables: VariableStore,
    pub(crate) variable_is_integer_bits: Vec<u64>,
    pub(crate) variable_is_inactive_bits: Vec<u64>,
    pub(crate) constraints: ConstraintStore,
    pub(crate) objective: Objective,
    pub(crate) objective_name: Option<String>,
    simplify_level: SimplifyLevel,
    // Column-first sparse storage: indexed by variable_id, each entry is a list of
    // (constraint_id, coefficient) pairs. Uses SmallVec for inline storage of ≤2 entries.
    pub(crate) columns: Vec<ColumnVec>,
    pub(crate) next_variable_id: u32,
    pub(crate) next_constraint_id: u32,
    pub(crate) slack_handles: Vec<SlackHandle>,
    // Lazy-allocated metadata storage
    pub(crate) variable_names: Option<BTreeMap<VariableId, String>>,
    pub(crate) constraint_names: Option<BTreeMap<ConstraintId, String>>,
    pub(crate) variable_metadata: Option<BTreeMap<VariableId, serde_json::Value>>,
    pub(crate) constraint_metadata: Option<BTreeMap<ConstraintId, serde_json::Value>>,
    // Reverse lookup for O(1) name-to-id resolution
    pub(crate) variable_name_to_id: Option<HashMap<String, VariableId>>,
    pub(crate) constraint_name_to_id: Option<HashMap<String, ConstraintId>>,
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
#[inline]
pub(crate) fn column_upsert(column: &mut ColumnVec, constraint_id: ConstraintId, coefficient: f64) {
    if let Some(entry) = column.iter_mut().find(|(cid, _)| *cid == constraint_id) {
        entry.1 = coefficient;
    } else {
        column.push((constraint_id, coefficient));
    }
}

impl Model {
    /// Create a new empty model.
    pub fn new() -> Self {
        Self {
            variables: VariableStore::new(),
            variable_is_integer_bits: Vec::new(),
            variable_is_inactive_bits: Vec::new(),
            constraints: ConstraintStore::new(),
            objective: Objective::new(),
            objective_name: None,
            simplify_level: SimplifyLevel::default(),
            columns: Vec::new(),
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
            variables: VariableStore::with_capacity(variable_capacity),
            variable_is_integer_bits: Vec::with_capacity(variable_capacity.div_ceil(BITS_PER_WORD)),
            variable_is_inactive_bits: Vec::new(),
            constraints: ConstraintStore::with_capacity(constraint_capacity),
            objective: Objective::new(),
            objective_name: None,
            simplify_level: SimplifyLevel::default(),
            columns: Vec::with_capacity(variable_capacity),
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

    #[doc(hidden)]
    pub fn take_objective_terms_for_consumed_solve(&mut self) -> Vec<(VariableId, f64)> {
        std::mem::take(&mut self.objective.terms)
    }

    #[doc(hidden)]
    pub fn drain_column_for_consumed_solve(
        &mut self,
        variable_id: VariableId,
        mut visit: impl FnMut(ConstraintId, f64),
    ) -> bool {
        let idx = variable_id.inner() as usize;
        let Some(column) = self.columns.get_mut(idx) else {
            return false;
        };
        for (constraint_id, coefficient) in std::mem::take(column) {
            visit(constraint_id, coefficient);
        }
        true
    }

    pub(crate) fn shrink_retained_storage(&mut self) {
        self.variables.shrink_to_fit();
        self.variable_is_integer_bits.shrink_to_fit();
        self.variable_is_inactive_bits.shrink_to_fit();
        self.constraints.shrink_to_fit();
        self.columns.shrink_to_fit();
        for column in &mut self.columns {
            column.shrink_to_fit();
        }
        self.slack_handles.shrink_to_fit();
    }

    #[inline]
    pub(crate) fn push_variable(&mut self, variable: Variable) -> Result<(), ModelError> {
        let idx = self.variables.len();
        self.variables.push_bounds(variable.bounds)?;
        self.columns.push(ColumnVec::new());
        if variable.is_integer {
            Self::write_packed_flag(&mut self.variable_is_integer_bits, idx, true);
        }
        if !variable.is_active {
            Self::write_packed_flag(&mut self.variable_is_inactive_bits, idx, true);
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn get_variable_by_index(&self, idx: usize) -> Option<Variable> {
        let bounds = self.variables.get_bounds(idx)?;
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

    /// Normalize terms with minimal allocations.
    /// Uses in-place deduplication instead of HashMap to reduce allocations.
    pub(crate) fn normalize_terms(
        &self,
        mut terms: Vec<(VariableId, f64)>,
    ) -> Vec<(VariableId, f64)> {
        let started = Instant::now();
        let terms_in = terms.len();

        let skip_zeros = matches!(self.simplify_level, SimplifyLevel::Light);

        if terms_are_sorted_unique_nonzero(&terms) {
            tracing::debug!(
                component = "model",
                operation = "lower_expr",
                status = "success",
                simplify_level = self.simplify_level.as_str(),
                expr_terms_in = terms_in,
                expr_terms_out = terms.len(),
                duration_ms = started.elapsed().as_secs_f64() * 1000.0,
                "Lowered already-normalized linear expression"
            );
            return terms;
        }

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
        if terms.is_empty() {
            return Ok(());
        }
        if self.objective.sense.is_none() {
            return Err(ModelError::NoObjective);
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

#[inline]
fn terms_are_sorted_unique_nonzero(terms: &[(VariableId, f64)]) -> bool {
    let mut previous: Option<u32> = None;
    for (var_id, coefficient) in terms {
        if *coefficient == 0.0 {
            return false;
        }
        let current = var_id.inner();
        if previous.is_some_and(|previous| current <= previous) {
            return false;
        }
        previous = Some(current);
    }
    true
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

    mod metadata_inspect;
    mod slack_csc;
    mod sparse_export;
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
    fn test_constraint_store_compacts_common_bounds_and_roundtrips_custom_bounds() {
        let mut model = Model::new();
        let equality = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 0.0),
            })
            .unwrap();
        let upper_zero = model
            .add_constraint(Constraint {
                bounds: Bounds::new(f64::NEG_INFINITY, 0.0),
            })
            .unwrap();
        let custom = model
            .add_constraint(Constraint {
                bounds: Bounds::new(3.0, 7.0),
            })
            .unwrap();

        assert_eq!(model.constraints.custom.len(), 1);
        assert_eq!(
            model.get_constraint(equality).unwrap().bounds,
            Bounds::new(0.0, 0.0)
        );
        assert_eq!(
            model.get_constraint(upper_zero).unwrap().bounds,
            Bounds::new(f64::NEG_INFINITY, 0.0)
        );
        assert_eq!(
            model.get_constraint(custom).unwrap().bounds,
            Bounds::new(3.0, 7.0)
        );
    }

    #[test]
    fn test_constraint_store_reuses_recent_custom_bounds() {
        let mut model = Model::new();
        let first = model
            .add_constraint(Constraint {
                bounds: Bounds::new(3.0, 7.0),
            })
            .unwrap();
        let second = model
            .add_constraint(Constraint {
                bounds: Bounds::new(3.0, 7.0),
            })
            .unwrap();
        let third = model
            .add_constraint(Constraint {
                bounds: Bounds::new(5.0, 9.0),
            })
            .unwrap();
        let fourth = model
            .add_constraint(Constraint {
                bounds: Bounds::new(3.0, 7.0),
            })
            .unwrap();

        assert_eq!(model.constraints.custom.len(), 2);
        assert_eq!(
            model.get_constraint(first).unwrap().bounds,
            Bounds::new(3.0, 7.0)
        );
        assert_eq!(
            model.get_constraint(second).unwrap().bounds,
            Bounds::new(3.0, 7.0)
        );
        assert_eq!(
            model.get_constraint(third).unwrap().bounds,
            Bounds::new(5.0, 9.0)
        );
        assert_eq!(
            model.get_constraint(fourth).unwrap().bounds,
            Bounds::new(3.0, 7.0)
        );
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
    fn test_set_objective_trims_excess_term_capacity() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let mut terms = Vec::with_capacity(16);
        terms.push((var_id, 1.0));

        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms,
            })
            .unwrap();

        assert_eq!(
            model.objective().terms.capacity(),
            model.objective().terms.len()
        );
    }

    #[test]
    fn test_set_objective_shrinks_retained_column_capacity() {
        let mut model = Model::new();
        let var_id = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();
        for _ in 0..3 {
            let constraint_id = model
                .add_constraint(Constraint {
                    bounds: Bounds::new(0.0, 1.0),
                })
                .unwrap();
            model.set_coefficient(var_id, constraint_id, 1.0).unwrap();
        }
        model.columns[var_id.inner() as usize].reserve(16);

        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(var_id, 1.0)],
            })
            .unwrap();

        let column = &model.columns[var_id.inner() as usize];
        assert_eq!(column.len(), 3);
        assert_eq!(column.capacity(), 3);
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
    fn test_take_objective_terms_preserves_sense_and_drains_terms() {
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
                terms: vec![(x, 2.0)],
            })
            .unwrap();

        let terms = model.take_objective_terms_for_consumed_solve();

        assert_eq!(terms, vec![(x, 2.0)]);
        assert_eq!(model.objective().sense, Some(Sense::Minimize));
        assert!(model.objective().terms.is_empty());
    }

    #[test]
    fn test_drain_column_for_consumed_solve_visits_entries_and_clears_column() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 10.0),
                is_integer: false,
                is_active: true,
            })
            .unwrap();
        let row = model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 100.0),
            })
            .unwrap();
        model.set_coefficient(x, row, 3.5).unwrap();

        let mut drained = Vec::new();
        assert!(
            model.drain_column_for_consumed_solve(x, |constraint_id, coefficient| {
                drained.push((constraint_id, coefficient));
            })
        );

        assert_eq!(drained, vec![(row, 3.5)]);
        assert!(model.get_column(x).is_some_and(|column| column.is_empty()));
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
    fn test_streaming_constraint_batch_inserts_rows_without_retained_batch() {
        let mut model = Model::new();
        let v1 = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();
        let v2 = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();

        let rows = vec![
            (vec![(v1, 1.0)], Bounds::new(0.0, 5.0)),
            (vec![(v1, -1.0), (v2, 2.0)], Bounds::new(3.0, 3.0)),
        ];

        let first = model
            .add_constraints_batch_streaming(rows.len(), rows)
            .unwrap();

        assert_eq!(first, ConstraintId::new(0));
        assert_eq!(model.num_constraints(), 2);
        assert_eq!(
            model.get_column(v1).expect("v1 column missing"),
            &vec![(ConstraintId::new(0), 1.0), (ConstraintId::new(1), -1.0)]
        );
        assert_eq!(
            model.get_column(v2).expect("v2 column missing"),
            &vec![(ConstraintId::new(1), 2.0)]
        );
    }

    #[test]
    fn test_compact_indexed_constraints_use_dense_row_indices() {
        let mut model = Model::new();
        for _ in 0..6 {
            model
                .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
                .unwrap();
        }

        let first = model
            .add_constraints_compact_indexed(
                &[(0, 1.0), (3, -2.0)],
                &[0, 2],
                &[Bounds::new(0.0, 1.0), Bounds::new(4.0, 4.0)],
            )
            .unwrap();

        assert_eq!(first, ConstraintId::new(0));
        assert_eq!(model.num_constraints(), 2);
        assert_eq!(
            model
                .get_column(VariableId::new(0))
                .expect("v0 column missing"),
            &vec![(ConstraintId::new(0), 1.0)]
        );
        assert_eq!(
            model
                .get_column(VariableId::new(2))
                .expect("v2 column missing"),
            &vec![(ConstraintId::new(1), 1.0)]
        );
        assert_eq!(
            model
                .get_column(VariableId::new(3))
                .expect("v3 column missing"),
            &vec![(ConstraintId::new(0), -2.0)]
        );
        assert_eq!(
            model
                .get_column(VariableId::new(5))
                .expect("v5 column missing"),
            &vec![(ConstraintId::new(1), -2.0)]
        );
    }

    #[test]
    fn test_add_variables_uniform_assigns_contiguous_ids() {
        let mut model = Model::new();
        let first = model
            .add_variables_uniform(Variable::continuous(Bounds::new(0.0, 10.0)), 3)
            .unwrap();

        assert_eq!(first, VariableId::new(0));
        assert_eq!(model.num_variables(), 3);
        assert_eq!(model.next_variable_id, 3);
        for idx in 0..3 {
            let variable = model.get_variable(VariableId::new(idx)).unwrap();
            assert_eq!(variable.bounds, Bounds::new(0.0, 10.0));
            assert!(variable.is_active);
            assert!(!variable.is_integer);
        }

        let second = model
            .add_variables_uniform(Variable::integer(Bounds::new(-1.0, 4.0)), 2)
            .unwrap();

        assert_eq!(second, VariableId::new(3));
        assert_eq!(model.num_variables(), 5);
        for idx in 3..5 {
            let variable = model.get_variable(VariableId::new(idx)).unwrap();
            assert_eq!(variable.bounds, Bounds::new(-1.0, 4.0));
            assert!(variable.is_integer);
        }
    }

    #[test]
    fn test_add_variables_uniform_rejects_invalid_bounds() {
        let mut model = Model::new();
        let result = model.add_variables_uniform(Variable::continuous(Bounds::new(5.0, 1.0)), 2);

        assert_eq!(
            result,
            Err(ModelError::InvalidVariableBounds {
                lower: 5.0,
                upper: 1.0
            })
        );
        assert_eq!(model.num_variables(), 0);
    }

    #[test]
    fn test_add_variables_with_bounds_assigns_contiguous_ids() {
        let mut model = Model::new();
        let bounds = [
            Bounds::new(0.0, 1.0),
            Bounds::new(2.0, 5.0),
            Bounds::new(-3.0, 4.0),
        ];

        let first = model
            .add_variables_with_bounds(&bounds, true, true)
            .unwrap();

        assert_eq!(first, VariableId::new(0));
        assert_eq!(model.num_variables(), 3);
        assert_eq!(model.next_variable_id, 3);
        for (idx, expected_bounds) in bounds.iter().copied().enumerate() {
            let variable = model.get_variable(VariableId::new(idx as u32)).unwrap();
            assert_eq!(variable.bounds, expected_bounds);
            assert!(variable.is_integer);
            assert!(variable.is_active);
        }
    }

    #[test]
    fn test_variable_store_compacts_common_bounds_and_deduplicates_custom_bounds() {
        let mut model = Model::new();
        model
            .add_variables_uniform(Variable::continuous(Bounds::new(0.0, f64::INFINITY)), 3)
            .unwrap();
        model
            .add_variables_with_bounds(
                &[
                    Bounds::new(0.0, 7.0),
                    Bounds::new(0.0, 7.0),
                    Bounds::new(0.0, 9.0),
                    Bounds::new(0.0, 7.0),
                ],
                false,
                true,
            )
            .unwrap();

        assert_eq!(model.variables.custom.len(), 2);
        assert_eq!(
            model.get_variable(VariableId::new(0)).unwrap().bounds,
            Bounds::new(0.0, f64::INFINITY)
        );
        assert_eq!(
            model.get_variable(VariableId::new(3)).unwrap().bounds,
            Bounds::new(0.0, 7.0)
        );
        assert_eq!(
            model.get_variable(VariableId::new(5)).unwrap().bounds,
            Bounds::new(0.0, 9.0)
        );
    }

    #[test]
    fn test_add_variables_with_bounds_rejects_invalid_bounds() {
        let mut model = Model::new();
        let bounds = [Bounds::new(0.0, 1.0), Bounds::new(5.0, 1.0)];
        let result = model.add_variables_with_bounds(&bounds, false, true);

        assert_eq!(
            result,
            Err(ModelError::InvalidVariableBounds {
                lower: 5.0,
                upper: 1.0
            })
        );
        assert_eq!(model.num_variables(), 0);
    }

    #[test]
    fn normalize_terms_keeps_already_sorted_unique_terms() {
        let model = Model::new();
        let terms = vec![
            (VariableId::new(1), 2.0),
            (VariableId::new(3), -4.0),
            (VariableId::new(8), 1.5),
        ];

        assert!(super::terms_are_sorted_unique_nonzero(&terms));
        assert_eq!(model.normalize_terms(terms.clone()), terms);
    }

    #[test]
    fn normalize_terms_still_merges_duplicates_and_removes_zeros() {
        let model = Model::new();
        let terms = vec![
            (VariableId::new(3), 1.0),
            (VariableId::new(1), 2.0),
            (VariableId::new(3), -1.0),
            (VariableId::new(2), 0.0),
        ];

        assert!(!super::terms_are_sorted_unique_nonzero(&terms));
        assert_eq!(
            model.normalize_terms(terms),
            vec![(VariableId::new(1), 2.0)]
        );
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
