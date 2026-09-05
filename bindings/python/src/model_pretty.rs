use crate::{PyIndexSet, PyModel};
use arco_model::Bounds;
use arco_model::{ConstraintId, Model, VariableId};
use arco_model::{PrettyBoundGroup, PrettyPrintAdapter, PrettySection, format_ascii_number};
use pyo3::prelude::*;
use std::collections::HashSet;
use std::fmt::Write as _;

const FLOAT_EQ_EPSILON: f64 = 1e-12;

#[derive(Debug, Clone, Copy)]
struct DenseIndexRun {
    active_start: usize,
    dense_start: usize,
}

#[derive(Debug)]
enum DenseIndexMapping {
    Identity,
    Runs(Box<[DenseIndexRun]>),
    Explicit(Box<[usize]>),
}

impl DenseIndexMapping {
    fn from_indices(indices: &[usize]) -> Self {
        let mut is_identity = true;
        let mut is_strictly_increasing = true;
        let mut run_count = 0usize;
        let mut previous = None;

        for (active_start, &dense_start) in indices.iter().enumerate() {
            if dense_start != active_start {
                is_identity = false;
            }
            if let Some(previous_dense) = previous {
                if dense_start <= previous_dense {
                    is_strictly_increasing = false;
                }
                if previous_dense
                    .checked_add(1)
                    .is_none_or(|next_dense| dense_start != next_dense)
                {
                    run_count = run_count.saturating_add(1);
                }
            } else {
                run_count = 1;
            }
            previous = Some(dense_start);
        }

        if is_identity {
            return Self::Identity;
        }
        if !is_strictly_increasing {
            return Self::Explicit(indices.to_vec().into_boxed_slice());
        }

        let Some(run_bytes) = run_count.checked_mul(std::mem::size_of::<DenseIndexRun>())
        else {
            return Self::Explicit(indices.to_vec().into_boxed_slice());
        };
        let Some(explicit_bytes) = indices.len().checked_mul(std::mem::size_of::<usize>())
        else {
            return Self::Explicit(indices.to_vec().into_boxed_slice());
        };
        if run_bytes >= explicit_bytes {
            return Self::Explicit(indices.to_vec().into_boxed_slice());
        }

        let mut runs = Vec::with_capacity(run_count);
        let mut previous = None;
        for (active_start, &dense_start) in indices.iter().enumerate() {
            let starts_new_run = previous
                .and_then(|previous_dense: usize| previous_dense.checked_add(1))
                .is_none_or(|next_dense| dense_start != next_dense);
            if starts_new_run {
                runs.push(DenseIndexRun {
                    active_start,
                    dense_start,
                });
            }
            previous = Some(dense_start);
        }
        Self::Runs(runs.into_boxed_slice())
    }

    fn dense_offset(&self, active_offset: usize, active_len: usize) -> Option<usize> {
        match self {
            Self::Identity => (active_offset < active_len).then_some(active_offset),
            Self::Explicit(indices) => indices.get(active_offset).copied(),
            Self::Runs(runs) => {
                let run_index = runs.partition_point(|run| run.active_start <= active_offset);
                let run_index = run_index.checked_sub(1)?;
                let run = runs.get(run_index)?;
                let run_end = run_index
                    .checked_add(1)
                    .and_then(|next| runs.get(next).map(|next_run| next_run.active_start))
                    .unwrap_or(active_len);
                let run_len = run_end.checked_sub(run.active_start)?;
                let delta = active_offset.checked_sub(run.active_start)?;
                (delta < run_len).then(|| run.dense_start.checked_add(delta))?
            }
        }
    }

    fn active_offset(&self, dense_offset: usize, active_len: usize) -> Option<usize> {
        match self {
            Self::Identity => (dense_offset < active_len).then_some(dense_offset),
            Self::Explicit(indices) => indices.iter().position(|index| *index == dense_offset),
            Self::Runs(runs) => {
                let run_index = runs.partition_point(|run| run.dense_start <= dense_offset);
                let run_index = run_index.checked_sub(1)?;
                let run = runs.get(run_index)?;
                let run_end = run_index
                    .checked_add(1)
                    .and_then(|next| runs.get(next).map(|next_run| next_run.active_start))
                    .unwrap_or(active_len);
                let run_len = run_end.checked_sub(run.active_start)?;
                let delta = dense_offset.checked_sub(run.dense_start)?;
                (delta < run_len).then(|| run.active_start.checked_add(delta))?
            }
        }
    }

    #[cfg(test)]
    fn payload_bytes(&self) -> usize {
        match self {
            Self::Identity => 0,
            Self::Runs(runs) => runs.len() * std::mem::size_of::<DenseIndexRun>(),
            Self::Explicit(indices) => indices.len() * std::mem::size_of::<usize>(),
        }
    }
}

pub(crate) struct ArrayPrintSpec {
    pub(crate) start_var_id: u32,
    pub(crate) len: usize,
    pub(crate) base_name: String,
    shape: Vec<usize>,
    strides: Vec<usize>,
    index_sets: Vec<Py<PyIndexSet>>,
    dense_mapping: DenseIndexMapping,
}

pub(crate) struct ConstraintPrintSpec {
    pub(crate) start_constraint_id: u32,
    pub(crate) len: usize,
    pub(crate) base_name: String,
}

pub(crate) struct PythonPrettyAdapter<'a> {
    pub(crate) model: &'a PyModel,
}

impl PrettyPrintAdapter for PythonPrettyAdapter<'_> {
    fn variable_label(&self, model: &Model, var_id: VariableId) -> Option<String> {
        let spec = self.model.find_array_print_spec(var_id)?;
        let array_label = PyModel::array_label_for_var(spec, var_id)?;
        if let Some(name) = model.get_variable_name(var_id) {
            if !PyModel::is_autogenerated_array_name(spec, var_id, name) {
                return Some(name.to_string());
            }
        }
        Some(array_label)
    }

    fn constraint_label(&self, _model: &Model, constraint_id: ConstraintId) -> Option<String> {
        self.model
            .reconstruct_constraint_name(constraint_id.inner())
    }

    fn sections(&self, _model: &Model) -> Vec<PrettySection> {
        let lines = self.model.render_index_set_lines();
        if lines.is_empty() {
            Vec::new()
        } else {
            vec![PrettySection {
                heading: "Index sets".to_string(),
                entries: lines,
            }]
        }
    }

    fn grouped_bounds(&self, _model: &Model) -> Vec<PrettyBoundGroup> {
        self.model.render_grouped_array_bounds()
    }
}

impl ArrayPrintSpec {
    fn offset_of(&self, var_id: VariableId) -> Option<usize> {
        let raw = var_id.inner();
        if raw < self.start_var_id {
            return None;
        }
        let offset = (raw - self.start_var_id) as usize;
        (offset < self.len).then_some(offset)
    }

    pub(crate) fn dense_offset(&self, active_offset: usize) -> Option<usize> {
        self.dense_mapping.dense_offset(active_offset, self.len)
    }

    pub(crate) fn active_offset(&self, dense_offset: usize) -> Option<usize> {
        self.dense_mapping.active_offset(dense_offset, self.len)
    }
}

impl ConstraintPrintSpec {
    pub(crate) fn offset_of(&self, constraint_id: ConstraintId) -> Option<usize> {
        let raw = constraint_id.inner();
        if raw < self.start_constraint_id {
            return None;
        }
        let offset = (raw - self.start_constraint_id) as usize;
        (offset < self.len).then_some(offset)
    }
}

impl PyModel {
    pub(crate) fn register_array_print_spec(
        &mut self,
        py: Python<'_>,
        start_var_id: u32,
        total: usize,
        index_sets: &[Py<PyIndexSet>],
        shape: &[usize],
        base_name: Option<&str>,
    ) {
        if total == 0 || index_sets.is_empty() {
            return;
        }
        let strides = (0..shape.len())
            .map(|axis| shape[axis + 1..].iter().product::<usize>().max(1))
            .collect::<Vec<_>>();

        self.array_print_specs.push(ArrayPrintSpec {
            start_var_id,
            len: total,
            base_name: base_name.unwrap_or("x").to_string(),
            shape: shape.to_vec(),
            strides,
            index_sets: index_sets.iter().map(|set| set.clone_ref(py)).collect(),
            dense_mapping: DenseIndexMapping::Identity,
        });
    }

    pub(crate) fn register_sparse_array_print_spec(
        &mut self,
        py: Python<'_>,
        start_var_id: u32,
        active_indices: &[usize],
        index_sets: &[Py<PyIndexSet>],
        shape: &[usize],
        base_name: Option<&str>,
    ) {
        if active_indices.is_empty() || index_sets.is_empty() {
            return;
        }
        let strides = (0..shape.len())
            .map(|axis| shape[axis + 1..].iter().product::<usize>().max(1))
            .collect::<Vec<_>>();

        self.array_print_specs.push(ArrayPrintSpec {
            start_var_id,
            len: active_indices.len(),
            base_name: base_name.unwrap_or("x").to_string(),
            shape: shape.to_vec(),
            strides,
            index_sets: index_sets.iter().map(|set| set.clone_ref(py)).collect(),
            dense_mapping: DenseIndexMapping::from_indices(active_indices),
        });
    }

    pub(crate) fn find_array_print_spec(&self, var_id: VariableId) -> Option<&ArrayPrintSpec> {
        self.array_print_specs
            .iter()
            .find(|spec| spec.offset_of(var_id).is_some())
    }

    fn array_label_for_var(spec: &ArrayPrintSpec, var_id: VariableId) -> Option<String> {
        let offset = spec.offset_of(var_id)?;
        let dense_offset = spec.dense_offset(offset).unwrap_or(offset);
        if spec.shape.is_empty() {
            return Some(spec.base_name.clone());
        }
        if spec.index_sets.len() != spec.shape.len() {
            return None;
        }

        Python::attach(|py| {
            let mut parts = Vec::with_capacity(spec.shape.len());
            for axis in 0..spec.shape.len() {
                let stride = *spec.strides.get(axis)?;
                let size = *spec.shape.get(axis)?;
                if size == 0 {
                    return None;
                }
                let coord = (dense_offset / stride) % size;
                let set_ref = spec.index_sets.get(axis)?.borrow(py);
                let member = set_ref.members.get(coord)?;
                parts.push(format_index_member(member));
            }
            Some(format!("{}[{}]", spec.base_name, parts.join(",")))
        })
    }

    fn is_autogenerated_array_name(
        spec: &ArrayPrintSpec,
        var_id: VariableId,
        name: &str,
    ) -> bool {
        let Some(offset) = spec.offset_of(var_id) else {
            return false;
        };
        if spec.len == 1 {
            return name == spec.base_name || name == format!("{}[0]", spec.base_name);
        }
        let dense_offset = spec.dense_offset(offset).unwrap_or(offset);
        name == format!("{}[{dense_offset}]", spec.base_name)
    }

    fn render_index_set_lines(&self) -> Vec<String> {
        if self.array_print_specs.is_empty() {
            return Vec::new();
        }
        Python::attach(|py| {
            let mut seen = HashSet::new();
            let mut lines = Vec::new();
            for spec in &self.array_print_specs {
                for index_set in &spec.index_sets {
                    let set_ref = index_set.borrow(py);
                    let members = set_ref
                        .members
                        .iter()
                        .map(format_index_member)
                        .collect::<Vec<_>>()
                        .join(", ");
                    let line = format!("{} = [{}]", set_ref.name, members);
                    if seen.insert(line.clone()) {
                        lines.push(line);
                    }
                }
            }
            lines
        })
    }

    fn render_grouped_array_bounds(&self) -> Vec<PrettyBoundGroup> {
        let mut groups = Vec::new();

        for spec in &self.array_print_specs {
            if spec.len <= 1 || spec.index_sets.len() != spec.shape.len() {
                continue;
            }

            let first_id = VariableId::new(spec.start_var_id);
            let Ok(first_var) = self.inner.get_variable(first_id) else {
                continue;
            };
            if is_binary_variable(first_var.bounds, first_var.is_integer) {
                continue;
            }

            let uniform = (1..spec.len).all(|offset| {
                let var_id = VariableId::new(spec.start_var_id + offset as u32);
                if let Ok(var) = self.inner.get_variable(var_id) {
                    var.is_integer == first_var.is_integer
                        && float_approx_equal(var.bounds.lower, first_var.bounds.lower)
                        && float_approx_equal(var.bounds.upper, first_var.bounds.upper)
                } else {
                    false
                }
            });
            if !uniform {
                continue;
            }

            let Some(bounds_label) = Self::array_bounds_label(spec) else {
                continue;
            };
            let Some(mut line) = format_variable_bounds_line(&bounds_label, first_var.bounds)
            else {
                continue;
            };
            if let Some(quantifier) = Self::array_bounds_quantifier(spec) {
                let _ = write!(line, "  {quantifier}");
            }
            let vars = (0..spec.len)
                .map(|offset| VariableId::new(spec.start_var_id + offset as u32))
                .collect();
            groups.push(PrettyBoundGroup { text: line, vars });
        }

        groups
    }

    fn array_bounds_label(spec: &ArrayPrintSpec) -> Option<String> {
        if spec.shape.is_empty() {
            return Some(spec.base_name.clone());
        }
        if spec.index_sets.len() != spec.shape.len() {
            return None;
        }
        Python::attach(|py| {
            let mut axis_symbols = Vec::with_capacity(spec.index_sets.len());
            for (axis, index_set) in spec.index_sets.iter().enumerate() {
                let set_ref = index_set.borrow(py);
                axis_symbols.push(axis_symbol_from_name(&set_ref.name, axis));
            }
            Some(format!("{}[{}]", spec.base_name, axis_symbols.join(",")))
        })
    }

    fn array_bounds_quantifier(spec: &ArrayPrintSpec) -> Option<String> {
        if spec.shape.is_empty() || spec.index_sets.len() != spec.shape.len() {
            return None;
        }
        Python::attach(|py| {
            let mut parts = Vec::with_capacity(spec.index_sets.len());
            for (axis, index_set) in spec.index_sets.iter().enumerate() {
                let set_ref = index_set.borrow(py);
                let symbol = axis_symbol_from_name(&set_ref.name, axis);
                parts.push(format!("{symbol} in {}", set_ref.name));
            }
            Some(format!("for {}", parts.join(", ")))
        })
    }
}

fn float_approx_equal(lhs: f64, rhs: f64) -> bool {
    if lhs.to_bits() == rhs.to_bits() {
        return true;
    }
    if !lhs.is_finite() || !rhs.is_finite() {
        return false;
    }
    let scale = lhs.abs().max(rhs.abs()).max(1.0);
    (lhs - rhs).abs() <= FLOAT_EQ_EPSILON * scale
}

fn is_binary_variable(bounds: Bounds, is_integer: bool) -> bool {
    is_integer && float_approx_equal(bounds.lower, 0.0) && float_approx_equal(bounds.upper, 1.0)
}

fn format_variable_bounds_line(label: &str, bounds: Bounds) -> Option<String> {
    let lower_finite = bounds.lower.is_finite();
    let upper_finite = bounds.upper.is_finite();
    if !lower_finite && !upper_finite {
        return None;
    }

    if lower_finite && upper_finite {
        return Some(format!(
            "{} <= {label} <= {}",
            format_ascii_number(bounds.lower),
            format_ascii_number(bounds.upper)
        ));
    }
    if lower_finite {
        return Some(format!("{} <= {label}", format_ascii_number(bounds.lower)));
    }
    Some(format!("{label} <= {}", format_ascii_number(bounds.upper)))
}

fn format_index_member(member: &crate::py_modules::index_set::IndexMember) -> String {
    match member {
        crate::py_modules::index_set::IndexMember::Int(value) => value.to_string(),
        crate::py_modules::index_set::IndexMember::Float(value) => format_ascii_number(*value),
        crate::py_modules::index_set::IndexMember::Str(value) => value.clone(),
        crate::py_modules::index_set::IndexMember::Tuple(items) => {
            let parts = items.iter().map(format_index_member).collect::<Vec<_>>();
            format!("({})", parts.join(", "))
        }
    }
}

fn axis_symbol_from_name(name: &str, axis: usize) -> String {
    if let Some(ch) = name.chars().find(|ch| ch.is_ascii_alphabetic()) {
        return ch.to_ascii_lowercase().to_string();
    }
    format!("i{}", axis + 1)
}

#[cfg(test)]
mod tests {
    use crate::py_modules::model_pretty::{DenseIndexMapping, DenseIndexRun};
    use std::mem::size_of;

    #[test]
    fn dense_mapping_identity_has_no_payload_and_handles_empty_input() {
        let mapping = DenseIndexMapping::from_indices(&[]);
        assert!(matches!(mapping, DenseIndexMapping::Identity));
        assert_eq!(mapping.payload_bytes(), 0);
        assert_eq!(mapping.dense_offset(0, 0), None);
        assert_eq!(mapping.active_offset(0, 0), None);

        let mapping = DenseIndexMapping::from_indices(&[0, 1, 2, 3]);
        assert!(matches!(mapping, DenseIndexMapping::Identity));
        assert_eq!(mapping.payload_bytes(), 0);
        assert_eq!(mapping.dense_offset(3, 4), Some(3));
        assert_eq!(mapping.active_offset(3, 4), Some(3));
        assert_eq!(mapping.dense_offset(4, 4), None);
        assert_eq!(mapping.active_offset(4, 4), None);
    }

    #[test]
    fn dense_mapping_uses_runs_for_sparse_sorted_indices() {
        let indices = [5, 6, 7, 10, 11, 12, 20, 21];
        let mapping = DenseIndexMapping::from_indices(&indices);
        assert!(matches!(mapping, DenseIndexMapping::Runs(_)));
        assert!(mapping.payload_bytes() < indices.len() * size_of::<usize>());

        let expected_dense = [5, 6, 7, 10, 11, 12, 20, 21];
        for (active_offset, expected) in expected_dense.into_iter().enumerate() {
            assert_eq!(mapping.dense_offset(active_offset, indices.len()), Some(expected));
            assert_eq!(mapping.active_offset(expected, indices.len()), Some(active_offset));
        }
        for dense_offset in [0, 4, 8, 9, 13, 19, 22] {
            assert_eq!(mapping.active_offset(dense_offset, indices.len()), None);
        }
    }

    #[test]
    fn dense_mapping_keeps_explicit_fallback_semantics() {
        let indices = [5, 3, 4, 3];
        let mapping = DenseIndexMapping::from_indices(&indices);
        assert!(matches!(mapping, DenseIndexMapping::Explicit(_)));
        for (active_offset, expected) in indices.into_iter().enumerate() {
            assert_eq!(mapping.dense_offset(active_offset, indices.len()), Some(expected));
        }
        assert_eq!(mapping.active_offset(3, indices.len()), Some(1));
        assert_eq!(mapping.active_offset(6, indices.len()), None);

        let short_sorted = DenseIndexMapping::from_indices(&[5, 6]);
        assert!(matches!(short_sorted, DenseIndexMapping::Explicit(_)));

        let irregular_sorted = DenseIndexMapping::from_indices(&[1, 3, 5, 7]);
        assert!(matches!(irregular_sorted, DenseIndexMapping::Explicit(_)));
        for (active_offset, expected) in [1, 3, 5, 7].into_iter().enumerate() {
            assert_eq!(
                irregular_sorted.dense_offset(active_offset, 4),
                Some(expected)
            );
        }
    }

    #[test]
    fn dense_mapping_handles_usize_maximum_run_boundaries() {
        let first_dense = usize::MAX - 2;
        let indices = [first_dense, usize::MAX - 1, usize::MAX];
        let mapping = DenseIndexMapping::from_indices(&indices);
        assert!(matches!(mapping, DenseIndexMapping::Runs(_)));

        assert_eq!(mapping.dense_offset(0, indices.len()), Some(first_dense));
        assert_eq!(mapping.dense_offset(2, indices.len()), Some(usize::MAX));
        assert_eq!(mapping.dense_offset(3, indices.len()), None);
        assert_eq!(mapping.dense_offset(usize::MAX, indices.len()), None);
        assert_eq!(mapping.active_offset(first_dense, indices.len()), Some(0));
        assert_eq!(mapping.active_offset(usize::MAX, indices.len()), Some(2));
        assert_eq!(mapping.active_offset(usize::MAX - 3, indices.len()), None);
    }

    #[test]
    fn dense_mapping_header_fits_three_portable_words() {
        assert!(size_of::<DenseIndexRun>() == 2 * size_of::<usize>());
        assert!(size_of::<DenseIndexMapping>() <= 3 * size_of::<usize>());
    }
}
