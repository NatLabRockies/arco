use crate::py_modules::arrays;
use crate::py_modules::errors;
use crate::{
    BoundsSpec, PyConstraintArray, PyExpr, PyIndexSet, PyModel, PyVariable, PyVariableArray,
    bounds_from_sense, extract_objective_terms,
};
use arco_arrays::BroadcastPlan;
use arco_model::Bounds;
use arco_model::expr::{ComparisonSense, Expr};
use arco_model::{ConstraintId, Objective, Sense, Variable, VariableId};
use pyo3::prelude::*;
use pyo3::types::PyAny;

fn normalized_terms_for_batch(expr: Expr) -> Vec<(VariableId, f64)> {
    let linear_terms = expr.linear_terms();
    let can_reuse = linear_terms
        .iter()
        .all(|(_, coefficient)| coefficient.is_finite() && *coefficient != 0.0)
        && linear_terms
            .windows(2)
            .all(|window| window[0].0 < window[1].0);

    if can_reuse {
        expr.into_linear_terms()
    } else {
        expr.normalized_terms()
    }
}

fn intersect_sorted_row_positions(row_indices: &[usize], active_indices: &[usize]) -> Vec<usize> {
    let mut row_pos = 0usize;
    let mut active_pos = 0usize;
    let mut positions = Vec::new();

    while row_pos < row_indices.len() && active_pos < active_indices.len() {
        let row_idx = row_indices[row_pos];
        let active_idx = active_indices[active_pos];
        match row_idx.cmp(&active_idx) {
            std::cmp::Ordering::Equal => {
                positions.push(row_pos);
                row_pos += 1;
                active_pos += 1;
            }
            std::cmp::Ordering::Less => row_pos += 1,
            std::cmp::Ordering::Greater => active_pos += 1,
        }
    }

    positions
}

impl PyModel {
    fn resolve_labeled_f64_values(
        obj: &Bound<'_, PyAny>,
        index_sets: &[Py<PyIndexSet>],
        shape: &[usize],
        total: usize,
        label: &str,
    ) -> PyResult<Vec<f64>> {
        if obj.getattr("axes").is_ok() {
            let values_obj = obj.getattr("values").map_err(|_| {
                errors::ArrayTypeError::new_err(format!(
                    "labeled {label} must expose a values attribute"
                ))
            })?;
            let Some(source_shape) = arrays::labeled_shape_from_axes_attr(obj, label)? else {
                return Err(errors::ArrayTypeError::new_err(format!(
                    "labeled {label} must expose axes as IndexSet values"
                )));
            };
            let target_shape = arrays::labeled_shape_from_index_sets(index_sets)?;
            let plan = BroadcastPlan::new(source_shape, target_shape)
                .map_err(|err| errors::ArrayShapeMismatchError::new_err(err.to_string()))?;

            let py = obj.py();
            let np = py.import("numpy")?;
            let flat = np
                .call_method1("asarray", (&values_obj,))?
                .call_method0("flatten")?;
            let values: Vec<f64> = flat.extract()?;
            let aligned = plan
                .broadcast_dense(&values)
                .map_err(|err| errors::ArrayShapeMismatchError::new_err(err.to_string()))?;
            if aligned.len() != total {
                return Err(errors::ArrayShapeMismatchError::new_err(format!(
                    "{label} length {} does not match total variables {}",
                    aligned.len(),
                    total
                )));
            }
            return Ok(aligned);
        }

        let py = obj.py();
        let np = py.import("numpy")?;
        let arr = np.call_method1("asarray", (obj,))?;
        let flat = match np.call_method1("broadcast_to", (&arr, shape.to_vec())) {
            Ok(broadcast) => broadcast.call_method0("flatten")?,
            Err(_) => arr.call_method0("flatten")?,
        };
        let values: Vec<f64> = flat.extract()?;
        if values.len() != total {
            return Err(errors::ArrayShapeMismatchError::new_err(format!(
                "{label} length {} does not match total variables {}",
                values.len(),
                total
            )));
        }
        Ok(values)
    }

    fn resolve_labeled_f64_values_at_indices(
        obj: &Bound<'_, PyAny>,
        index_sets: &[Py<PyIndexSet>],
        shape: &[usize],
        total: usize,
        active_indices: &[usize],
        label: &str,
    ) -> PyResult<Vec<f64>> {
        if obj.getattr("axes").is_ok() {
            let values_obj = obj.getattr("values").map_err(|_| {
                errors::ArrayTypeError::new_err(format!(
                    "labeled {label} must expose a values attribute"
                ))
            })?;
            let Some(source_shape) = arrays::labeled_shape_from_axes_attr(obj, label)? else {
                return Err(errors::ArrayTypeError::new_err(format!(
                    "labeled {label} must expose axes as IndexSet values"
                )));
            };
            let target_shape = arrays::labeled_shape_from_index_sets(index_sets)?;
            let source_len = source_shape.total_len();
            let plan = BroadcastPlan::new(source_shape, target_shape)
                .map_err(|err| errors::ArrayShapeMismatchError::new_err(err.to_string()))?;

            let py = obj.py();
            let np = py.import("numpy")?;
            let flat = np
                .call_method1("asarray", (&values_obj,))?
                .call_method0("flatten")?;
            let values: Vec<f64> = flat.extract()?;
            if values.len() != source_len {
                return Err(errors::ArrayShapeMismatchError::new_err(format!(
                    "{label} source length {} does not match source shape length {}",
                    values.len(),
                    source_len
                )));
            }
            let mut resolved = Vec::with_capacity(active_indices.len());
            for &target_flat in active_indices {
                if target_flat >= total {
                    return Err(errors::ArrayShapeMismatchError::new_err(format!(
                        "{label} active index {target_flat} exceeds total variables {total}",
                    )));
                }
                let source_flat = plan.source_offset_for_target_flat(target_flat);
                let value = values.get(source_flat).ok_or_else(|| {
                    errors::ArrayShapeMismatchError::new_err(format!(
                        "{label} source index {source_flat} exceeds source length {}",
                        values.len()
                    ))
                })?;
                resolved.push(*value);
            }
            return Ok(resolved);
        }

        let py = obj.py();
        let np = py.import("numpy")?;
        let arr = np.call_method1("asarray", (obj,))?;
        let flat = arr.call_method0("flatten")?;
        let values: Vec<f64> = flat.extract()?;
        if values.len() == 1 {
            return Ok(vec![values[0]; active_indices.len()]);
        }
        if values.len() == total {
            return active_indices
                .iter()
                .map(|&idx| {
                    values.get(idx).copied().ok_or_else(|| {
                        errors::ArrayShapeMismatchError::new_err(format!(
                            "{label} active index {idx} exceeds total variables {total}",
                        ))
                    })
                })
                .collect();
        }

        let broadcast = np.call_method1("broadcast_to", (&arr, shape.to_vec()))?;
        let flat = broadcast.call_method0("flatten")?;
        let values: Vec<f64> = flat.extract()?;
        if values.len() != total {
            return Err(errors::ArrayShapeMismatchError::new_err(format!(
                "{label} length {} does not match total variables {}",
                values.len(),
                total
            )));
        }
        active_indices
            .iter()
            .map(|&idx| {
                values.get(idx).copied().ok_or_else(|| {
                    errors::ArrayShapeMismatchError::new_err(format!(
                        "{label} active index {idx} exceeds total variables {total}",
                    ))
                })
            })
            .collect()
    }

    fn resolve_active_indices(
        active: Option<&Bound<'_, PyAny>>,
        index_sets: Option<&[Py<PyIndexSet>]>,
        shape: &[usize],
        total: usize,
    ) -> PyResult<Vec<usize>> {
        let Some(active_obj) = active else {
            return Ok((0..total).collect());
        };

        if active_obj.is_instance_of::<pyo3::types::PyBool>() {
            let value: bool = active_obj.extract()?;
            return if value {
                Ok((0..total).collect())
            } else {
                Ok(Vec::new())
            };
        }

        if let Some(index_sets) = index_sets {
            if active_obj.getattr("axes").is_ok() {
                let values_obj = active_obj.getattr("values").map_err(|_| {
                    errors::ArrayTypeError::new_err(
                        "labeled active masks must expose a values attribute",
                    )
                })?;
                let Some(source_shape) =
                    arrays::labeled_shape_from_axes_attr(active_obj, "active masks")?
                else {
                    return Err(errors::ArrayTypeError::new_err(
                        "labeled active masks must expose axes as IndexSet values",
                    ));
                };
                let target_shape = arrays::labeled_shape_from_index_sets(index_sets)?;
                let plan = BroadcastPlan::new(source_shape, target_shape)
                    .map_err(|err| errors::ArrayShapeMismatchError::new_err(err.to_string()))?;

                let py = active_obj.py();
                let np = py.import("numpy")?;
                let flat = np
                    .call_method1("asarray", (&values_obj,))?
                    .call_method0("flatten")?;
                let mask_values: Vec<bool> = flat.extract()?;
                let indices = plan
                    .active_target_indices(&mask_values, |value| *value)
                    .map_err(|err| errors::ArrayShapeMismatchError::new_err(err.to_string()))?;
                return Ok(indices);
            }
        }

        let values_obj = match active_obj.getattr("values") {
            Ok(values) => values,
            Err(_) => active_obj.clone(),
        };
        let py = active_obj.py();
        let np = py.import("numpy")?;
        let arr = np.call_method1("asarray", (&values_obj,))?;
        let broadcast = np.call_method1("broadcast_to", (arr, shape.to_vec()))?;
        let flat = broadcast.call_method0("flatten")?;
        let mask: Vec<bool> = flat.extract()?;
        if mask.len() != total {
            return Err(errors::ArrayShapeMismatchError::new_err(format!(
                "active mask size {} does not match target size {}",
                mask.len(),
                total
            )));
        }
        Ok(mask
            .into_iter()
            .enumerate()
            .filter_map(|(idx, is_active)| is_active.then_some(idx))
            .collect())
    }

    /// Compute effective bounds spec, validating binary constraints.
    #[allow(clippy::float_cmp)]
    pub(crate) fn effective_bounds(
        bounds: &BoundsSpec,
        is_integer: bool,
        is_binary: bool,
    ) -> PyResult<BoundsSpec> {
        let effective_binary = is_binary || bounds.is_binary;
        let effective_integer = is_integer || bounds.is_integer || effective_binary;

        if effective_binary
            && !bounds.is_binary
            && (bounds.bounds.lower != 0.0 || bounds.bounds.upper != 1.0)
        {
            return Err(errors::ModelBinaryBoundsError::new_err(
                "Binary variables must use bounds=[0,1]",
            ));
        }

        Ok(BoundsSpec {
            bounds: bounds.bounds,
            is_integer: effective_integer,
            is_binary: effective_binary,
        })
    }

    /// Reconstruct a variable name from array print specs.
    /// First checks Model's explicit names, then falls back to array spec reconstruction.
    pub(crate) fn reconstruct_variable_name(&self, var_id: u32) -> Option<String> {
        let vid = VariableId::new(var_id);
        // Check Model's explicit names first (for individually named vars)
        if let Some(name) = self.inner.get_variable_name(vid) {
            return Some(name.to_string());
        }
        // Reconstruct from array_print_spec
        let spec = self.find_array_print_spec(vid)?;
        let offset = (var_id - spec.start_var_id) as usize;
        let dense_offset = spec.dense_offset(offset).unwrap_or(offset);
        if spec.len == 1 {
            Some(spec.base_name.clone())
        } else {
            Some(format!("{}[{}]", spec.base_name, dense_offset))
        }
    }

    /// Find a variable by name, checking both explicit names and array spec reconstruction.
    pub(crate) fn find_variable_by_name(&self, name: &str) -> Option<VariableId> {
        // Check Model's explicit names first
        if let Some(id) = self.inner.get_variable_by_name(name) {
            return Some(id);
        }
        // Try to parse as "base_name[offset]" and check array specs
        if let Some(bracket_pos) = name.rfind('[') {
            let base = &name[..bracket_pos];
            let idx_str = name[bracket_pos + 1..].strip_suffix(']')?;
            let offset: usize = idx_str.parse().ok()?;
            for spec in &self.array_print_specs {
                if spec.base_name != base {
                    continue;
                }
                if let Some(active_pos) = spec.active_offset(offset) {
                    let active_pos = u32::try_from(active_pos).ok()?;
                    let variable_id = spec.start_var_id.checked_add(active_pos)?;
                    return Some(VariableId::new(variable_id));
                }
            }
        } else {
            // Try scalar name match (spec.len == 1)
            for spec in &self.array_print_specs {
                if spec.len == 1 && spec.base_name == name {
                    return Some(VariableId::new(spec.start_var_id));
                }
            }
        }
        None
    }

    pub(crate) fn set_constraint_name_if_provided(
        &mut self,
        constraint_id: ConstraintId,
        name: Option<String>,
    ) -> PyResult<()> {
        if let Some(name) = name {
            self.inner
                .set_constraint_name(constraint_id, name)
                .map_err(errors::model_error_to_py)?;
        }
        Ok(())
    }

    pub(crate) fn reconstruct_constraint_name(&self, constraint_id: u32) -> Option<String> {
        let cid = ConstraintId::new(constraint_id);
        if let Some(name) = self.inner.get_constraint_name(cid) {
            return Some(name.to_string());
        }
        let spec = self
            .constraint_print_specs
            .iter()
            .find(|spec| spec.offset_of(cid).is_some())?;
        let offset = (constraint_id - spec.start_constraint_id) as usize;
        if spec.len == 1 {
            Some(spec.base_name.clone())
        } else {
            Some(format!("{}[{}]", spec.base_name, offset))
        }
    }

    pub(crate) fn find_constraint_by_name(&self, name: &str) -> Option<ConstraintId> {
        if let Some(id) = self.inner.get_constraint_by_name(name) {
            return Some(id);
        }
        if let Some(bracket_pos) = name.rfind('[') {
            let base = &name[..bracket_pos];
            let idx_str = name[bracket_pos + 1..].strip_suffix(']')?;
            let offset: usize = idx_str.parse().ok()?;
            for spec in &self.constraint_print_specs {
                if spec.base_name == base && offset < spec.len {
                    return Some(ConstraintId::new(spec.start_constraint_id + offset as u32));
                }
            }
        } else {
            for spec in &self.constraint_print_specs {
                if spec.len == 1 && spec.base_name == name {
                    return Some(ConstraintId::new(spec.start_constraint_id));
                }
            }
        }
        None
    }

    /// Name a contiguous block of constraints starting at `first_id`.
    fn name_constraint_block(
        &mut self,
        first_id: ConstraintId,
        count: usize,
        name: Option<&str>,
    ) {
        let Some(base) = name else { return };
        if count == 0 {
            return;
        }
        self.constraint_print_specs
            .push(crate::py_modules::model_pretty::ConstraintPrintSpec {
                start_constraint_id: first_id.inner(),
                len: count,
                base_name: base.to_string(),
            });
    }

    /// Insert constraints via compact term patterns (zero per-element allocation).
    #[allow(clippy::too_many_arguments)]
    fn add_constraints_compact_internal(
        &mut self,
        compact: &arrays::CompactConstraintStorage,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        self.add_constraints_compact_shaped_internal(compact, active, name, &[compact.count], &[])
    }

    pub(crate) fn add_constraints_compact_shaped_internal(
        &mut self,
        compact: &arrays::CompactConstraintStorage,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
        shape: &[usize],
        index_sets: &[Py<PyIndexSet>],
    ) -> PyResult<PyConstraintArray> {
        let count = compact.count;
        let sense = compact.sense;
        let term_patterns = compact.term_patterns();

        if active.is_none() && count == 0 {
            return Ok(PyConstraintArray::from_batch_shaped(
                0,
                shape.to_vec(),
                Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
                sense,
                &[],
                name,
            ));
        }

        if active.is_none() {
            let mut filtered_rhs = Vec::with_capacity(count);
            let mut bounds_list = Vec::with_capacity(count);
            for idx in 0..count {
                let rhs_val = match &compact.rhs {
                    arrays::CompactRhs::Scalar(value) => *value,
                    arrays::CompactRhs::Vec(values) => values[idx],
                };
                filtered_rhs.push(rhs_val);
                bounds_list.push(bounds_from_sense(sense, rhs_val));
            }

            let first_constraint_id = self
                .inner
                .add_constraints_compact(&term_patterns, &bounds_list)
                .map_err(errors::model_error_to_py)?;

            self.name_constraint_block(first_constraint_id, count, name.as_deref());

            return Ok(PyConstraintArray::from_batch_shaped(
                first_constraint_id.inner(),
                shape.to_vec(),
                Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
                sense,
                &filtered_rhs,
                name,
            ));
        }

        let active_indices = Self::resolve_active_indices(
            active,
            (!index_sets.is_empty()).then_some(index_sets),
            shape,
            count,
        )?;

        if active_indices.is_empty() {
            return Ok(PyConstraintArray::from_batch_shaped(
                0,
                shape.to_vec(),
                Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
                sense,
                &[],
                name,
            ));
        }

        let mut filtered_rhs = Vec::with_capacity(active_indices.len());
        let mut bounds_list = Vec::with_capacity(active_indices.len());

        for idx in active_indices.iter().copied() {
            let rhs_val = match &compact.rhs {
                arrays::CompactRhs::Scalar(value) => *value,
                arrays::CompactRhs::Vec(values) => values[idx],
            };
            filtered_rhs.push(rhs_val);
            bounds_list.push(bounds_from_sense(sense, rhs_val));
        }

        let first_constraint_id = self
            .inner
            .add_constraints_compact_indexed(&term_patterns, &active_indices, &bounds_list)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, active_indices.len(), name.as_deref());

        Ok(PyConstraintArray::from_batch_shaped(
            first_constraint_id.inner(),
            shape.to_vec(),
            Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
            sense,
            &filtered_rhs,
            name,
        ))
    }

    pub(crate) fn add_constraints_lazy_compare_shaped_internal(
        &mut self,
        left: &arrays::LinearArrayCore,
        right: &arrays::LinearArrayCore,
        sense: ComparisonSense,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
        shape: &[usize],
        index_sets: &[Py<PyIndexSet>],
    ) -> PyResult<PyConstraintArray> {
        let total = left.values.len();
        let active_indices = Self::resolve_active_indices(
            active,
            (!index_sets.is_empty()).then_some(index_sets),
            shape,
            total,
        )?;

        if active_indices.is_empty() {
            return Ok(PyConstraintArray::from_batch_shaped(
                0,
                shape.to_vec(),
                Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
                sense,
                &[],
                name,
            ));
        }

        let mut filtered_rhs: Vec<f64> = Vec::with_capacity(active_indices.len());
        let row_count = active_indices.len();
        let rows = active_indices.into_iter().map(|idx| {
            let diff = left.values[idx]
                .inner()
                .add(&right.values[idx].inner().scale(-1.0));
            let rhs_value = -diff.constant();
            filtered_rhs.push(rhs_value);
            (
                normalized_terms_for_batch(diff),
                bounds_from_sense(sense, rhs_value),
            )
        });

        let first_constraint_id = self
            .inner
            .add_constraints_batch_streaming(row_count, rows)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, row_count, name.as_deref());

        Ok(PyConstraintArray::from_batch_shaped(
            first_constraint_id.inner(),
            shape.to_vec(),
            Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
            sense,
            &filtered_rhs,
            name,
        ))
    }

    /// Add constraints from an array expression (VariableArray or ExprArray) with a separate rhs.
    ///
    /// Tries the compact fast path first, then falls back to materialized comparison.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_constraints_from_array(
        &mut self,
        compact_expr: Option<arrays::CompactExprStorage>,
        core_fn: impl FnOnce() -> arrays::LinearArrayCore,
        rhs_obj: &Bound<'_, PyAny>,
        sense: ComparisonSense,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        // Fast path: compact expression
        if let Some(ref compact) = compact_expr {
            if let Some(compact_con) = arrays::try_make_compact_constraint(compact, rhs_obj, sense)
            {
                return self.add_constraints_compact_internal(&compact_con, active, name);
            }
        }

        // Full path: materialize and compare
        let core = core_fn();
        let constraints = if let Ok(index_set) = rhs_obj.extract::<PyRef<'_, PyIndexSet>>() {
            core.compare_index_set(&index_set, sense)?
        } else {
            let value = rhs_obj.extract::<f64>()?;
            core.compare_scalar(value, sense)
        };
        self.add_constraints_shaped_internal(
            constraints.exprs().iter().cloned(),
            constraints.get_sense(),
            constraints.get_rhs(),
            active,
            name,
            &core.shape,
            &core.index_sets,
        )
    }

    /// Insert constraints via materialized expressions (existing batch path).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_constraints_shaped_internal<I>(
        &mut self,
        exprs: I,
        sense: ComparisonSense,
        rhs: Vec<f64>,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
        shape: &[usize],
        index_sets: &[Py<PyIndexSet>],
    ) -> PyResult<PyConstraintArray>
    where
        I: ExactSizeIterator<Item = PyExpr>,
    {
        let total = exprs.len();
        let active_indices = Self::resolve_active_indices(
            active,
            (!index_sets.is_empty()).then_some(index_sets),
            shape,
            total,
        )?;

        let mut active_lookup = vec![false; total];
        let active_count = active_indices.len();
        for idx in active_indices {
            active_lookup[idx] = true;
        }

        let mut filtered_rhs: Vec<f64> = Vec::with_capacity(active_count);
        let rows = exprs
            .into_iter()
            .zip(rhs.iter().copied())
            .enumerate()
            .filter_map(|(idx, (expr, rhs_val))| {
                if !active_lookup[idx] {
                    return None;
                }
                filtered_rhs.push(rhs_val);
                Some((
                    normalized_terms_for_batch(expr.into_inner()),
                    bounds_from_sense(sense, rhs_val),
                ))
            });

        if active_count == 0 {
            return Ok(PyConstraintArray::from_batch_shaped(
                0,
                shape.to_vec(),
                Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
                sense,
                &[],
                name,
            ));
        }

        let first_constraint_id = self
            .inner
            .add_constraints_batch_streaming(active_count, rows)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, active_count, name.as_deref());

        Ok(PyConstraintArray::from_batch_shaped(
            first_constraint_id.inner(),
            shape.to_vec(),
            Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
            sense,
            &filtered_rhs,
            name,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_constraints_sparse_rows_internal(
        &mut self,
        exprs: &[PyExpr],
        sense: ComparisonSense,
        rhs: &[f64],
        row_indices: &[usize],
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
        shape: &[usize],
        index_sets: &[Py<PyIndexSet>],
    ) -> PyResult<PyConstraintArray> {
        let total = shape.iter().product();
        for row_idx in row_indices.iter().copied() {
            if row_idx >= total {
                return Err(errors::ArrayShapeMismatchError::new_err(format!(
                    "sparse constraint row index {row_idx} out of range for constraint array of size {total}"
                )));
            }
        }

        let selected_positions = if active.is_none() {
            None
        } else {
            let active_indices = Self::resolve_active_indices(
                active,
                (!index_sets.is_empty()).then_some(index_sets),
                shape,
                total,
            )?;
            for idx in active_indices.iter().copied() {
                if idx >= total {
                    return Err(errors::ArrayShapeMismatchError::new_err(format!(
                        "active row index {idx} out of range for constraint array of size {total}"
                    )));
                }
            }
            Some(intersect_sorted_row_positions(row_indices, &active_indices))
        };

        let row_count = selected_positions
            .as_ref()
            .map_or(row_indices.len(), Vec::len);

        if row_count == 0 {
            return Ok(PyConstraintArray::from_batch_shaped(
                0,
                shape.to_vec(),
                Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
                sense,
                &[],
                name,
            ));
        }

        let mut filtered_rhs: Vec<f64> = Vec::with_capacity(row_count);
        let first_constraint_id = if let Some(selected_positions) = selected_positions {
            let rows = selected_positions.into_iter().map(|position| {
                let expr = &exprs[position];
                let rhs_val = rhs[position];
                filtered_rhs.push(rhs_val);
                (
                    expr.inner().normalized_terms(),
                    bounds_from_sense(sense, rhs_val),
                )
            });
            self.inner
                .add_constraints_batch_streaming(row_count, rows)
                .map_err(errors::model_error_to_py)?
        } else {
            let rows = exprs
                .iter()
                .zip(rhs.iter().copied())
                .map(|(expr, rhs_val)| {
                    filtered_rhs.push(rhs_val);
                    (
                        expr.inner().normalized_terms(),
                        bounds_from_sense(sense, rhs_val),
                    )
                });
            self.inner
                .add_constraints_batch_streaming(row_count, rows)
                .map_err(errors::model_error_to_py)?
        };

        self.name_constraint_block(first_constraint_id, row_count, name.as_deref());

        Ok(PyConstraintArray::from_batch_shaped(
            first_constraint_id.inner(),
            shape.to_vec(),
            Python::attach(|py| index_sets.iter().map(|set| set.clone_ref(py)).collect()),
            sense,
            &filtered_rhs,
            name,
        ))
    }
    pub(crate) fn set_objective_from_expr(
        &mut self,
        expr: &Bound<'_, PyAny>,
        sense: Sense,
        name: Option<String>,
    ) -> PyResult<()> {
        let terms = extract_objective_terms(expr)?;
        self.inner
            .set_objective(Objective {
                sense: Some(sense),
                terms,
            })
            .map_err(errors::model_error_to_py)?;
        self.inner
            .set_objective_name(name)
            .map_err(errors::model_error_to_py)?;
        Ok(())
    }

    pub(crate) fn add_objective_terms_from_expr(
        &mut self,
        expr: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        if self.inner.objective().sense.is_none() {
            return Err(errors::model_error_to_py(
                arco_model::ModelError::NoObjective,
            ));
        }
        let terms = extract_objective_terms(expr)?;
        self.inner
            .add_objective_terms(terms)
            .map_err(errors::model_error_to_py)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_variables_scalar_bounds(
        &mut self,
        py: Python<'_>,
        index_sets: Vec<Py<PyIndexSet>>,
        shape: &[usize],
        total: usize,
        bounds: BoundsSpec,
        is_integer: bool,
        is_binary: bool,
        _active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let effective_bounds = Self::effective_bounds(&bounds, is_integer, is_binary)?;
        let active_indices =
            Self::resolve_active_indices(_active, Some(&index_sets), shape, total)?;

        if active_indices.len() == total {
            let start_var_id = self.inner.num_variables() as u32;
            self.inner.reserve_variables(total);

            let var_template = Variable {
                bounds: bounds.bounds,
                is_integer: effective_bounds.is_integer,
                is_active: true,
            };
            self.inner
                .add_variables_uniform(var_template, total)
                .map_err(errors::model_error_to_py)?;

            self.register_array_print_spec(
                py,
                start_var_id,
                total,
                &index_sets,
                shape,
                name.as_deref(),
            );

            return Ok(PyVariableArray::new_compact(
                index_sets,
                shape.to_vec(),
                start_var_id,
                total,
                effective_bounds,
                name,
            ));
        }

        let active_count = active_indices.len();
        let start_var_id = self.inner.num_variables() as u32;
        self.inner.reserve_variables(active_count);
        let var_template = Variable {
            bounds: bounds.bounds,
            is_integer: effective_bounds.is_integer,
            is_active: true,
        };
        self.inner
            .add_variables_uniform(var_template, active_count)
            .map_err(errors::model_error_to_py)?;
        let var_ids = (0..active_count)
            .map(|offset| start_var_id + offset as u32)
            .collect::<Vec<_>>();

        self.register_sparse_array_print_spec(
            py,
            start_var_id,
            &active_indices,
            &index_sets,
            shape,
            name.as_deref(),
        );

        Ok(PyVariableArray::new_active_sparse(
            index_sets,
            shape.to_vec(),
            active_indices,
            var_ids,
            effective_bounds,
            name,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_variables_array_bounds(
        &mut self,
        py: Python<'_>,
        index_sets: Vec<Py<PyIndexSet>>,
        shape: &[usize],
        total: usize,
        bounds_obj: &Bound<'_, PyAny>,
        is_integer: bool,
        is_binary: bool,
        _active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let start_var_id = self.inner.num_variables() as u32;
        let active_indices =
            Self::resolve_active_indices(_active, Some(&index_sets), shape, total)?;
        let active_count = active_indices.len();
        self.inner.reserve_variables(active_count);

        // Extract lower and upper as numpy arrays from a Bounds-like object
        let lo_attr = bounds_obj
            .getattr("lower")
            .or_else(|_| bounds_obj.getattr("lo"))?;
        let hi_attr = bounds_obj
            .getattr("upper")
            .or_else(|_| bounds_obj.getattr("hi"))?;
        let effective_binary = is_binary;
        let effective_integer = is_integer || effective_binary;

        if active_count == total {
            let lo_values = Self::resolve_labeled_f64_values(
                &lo_attr,
                &index_sets,
                shape,
                total,
                "lower bounds",
            )?;
            let hi_values = Self::resolve_labeled_f64_values(
                &hi_attr,
                &index_sets,
                shape,
                total,
                "upper bounds",
            )?;
            let mut values = Vec::with_capacity(total);
            let mut variables = Vec::with_capacity(total);
            for i in 0..total {
                let element_bounds = Bounds::new(lo_values[i], hi_values[i]);
                let var = Variable {
                    bounds: element_bounds,
                    is_integer: effective_integer,
                    is_active: true,
                };
                let var_id = self
                    .inner
                    .add_variable(var)
                    .map_err(errors::model_error_to_py)?;

                let var_name = name.as_ref().map(|base| {
                    if total == 1 {
                        base.clone()
                    } else {
                        format!("{base}[{i}]")
                    }
                });

                let element_bounds_spec = BoundsSpec {
                    bounds: element_bounds,
                    is_integer: effective_integer,
                    is_binary: effective_binary,
                };

                values.push(PyExpr::from_term(var_id.inner(), 1.0));
                variables.push(PyVariable::new(
                    var_id.inner(),
                    var_name,
                    element_bounds_spec,
                ));
            }

            self.register_array_print_spec(
                py,
                start_var_id,
                total,
                &index_sets,
                shape,
                name.as_deref(),
            );

            return Ok(PyVariableArray::new(
                index_sets,
                shape.to_vec(),
                values,
                variables,
            ));
        }

        let lo_values = Self::resolve_labeled_f64_values_at_indices(
            &lo_attr,
            &index_sets,
            shape,
            total,
            &active_indices,
            "lower bounds",
        )?;
        let hi_values = Self::resolve_labeled_f64_values_at_indices(
            &hi_attr,
            &index_sets,
            shape,
            total,
            &active_indices,
            "upper bounds",
        )?;
        let mut model_bounds = Vec::with_capacity(active_count);
        let mut bounds_specs = Vec::with_capacity(active_count);
        for (lo, hi) in lo_values.into_iter().zip(hi_values) {
            let element_bounds = Bounds::new(lo, hi);
            let element_bounds_spec = BoundsSpec {
                bounds: element_bounds,
                is_integer: effective_integer,
                is_binary: effective_binary,
            };

            model_bounds.push(element_bounds);
            bounds_specs.push(element_bounds_spec);
        }
        self.inner
            .add_variables_with_bounds(&model_bounds, effective_integer, true)
            .map_err(errors::model_error_to_py)?;
        let var_ids = (0..active_count)
            .map(|offset| start_var_id + offset as u32)
            .collect::<Vec<_>>();

        self.register_sparse_array_print_spec(
            py,
            start_var_id,
            &active_indices,
            &index_sets,
            shape,
            name.as_deref(),
        );

        Ok(PyVariableArray::new_active_sparse_with_bounds(
            index_sets,
            shape.to_vec(),
            active_indices,
            var_ids,
            bounds_specs,
            name,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::py_modules::model_edit::normalized_terms_for_batch;
    use crate::{PyExpr, PyModel};
    use arco_model::VariableId;
    use arco_model::expr::{ComparisonSense, Expr};
    use pyo3::types::PyList;
    use std::cell::Cell;
    use std::rc::Rc;

    struct CountingIterator<I> {
        inner: I,
        consumed: Rc<Cell<usize>>,
    }

    impl<I: Iterator> Iterator for CountingIterator<I> {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            let item = self.inner.next();
            if item.is_some() {
                self.consumed.set(self.consumed.get() + 1);
            }
            item
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.inner.size_hint()
        }
    }

    impl<I: ExactSizeIterator> ExactSizeIterator for CountingIterator<I> {}

    #[test]
    fn shaped_rows_are_deferred_until_active_mask_validation() {
        pyo3::Python::initialize();
        pyo3::Python::attach(|py| {
            let mut model = PyModel::new(None, None).unwrap();
            let consumed = Rc::new(Cell::new(0));
            let rows = CountingIterator {
                inner: vec![PyExpr::from_term(0, 1.0)].into_iter(),
                consumed: Rc::clone(&consumed),
            };
            let active = PyList::new(py, [true, false]).unwrap();

            let result = model.add_constraints_shaped_internal(
                rows,
                ComparisonSense::GreaterEqual,
                vec![0.0],
                Some(active.as_any()),
                None,
                &[1],
                &[],
            );

            assert!(result.is_err());
            assert_eq!(consumed.get(), 0);
            assert_eq!(model.inner.num_constraints(), 0);
        });
    }

    #[test]
    fn owned_ordered_terms_reuse_the_linear_buffer() {
        let mut terms = Vec::with_capacity(3);
        terms.extend([
            (VariableId::new(1), 1.0),
            (VariableId::new(4), -2.0),
            (VariableId::new(9), 3.0),
        ]);
        let pointer = terms.as_ptr();
        let capacity = terms.capacity();

        let normalized = normalized_terms_for_batch(Expr::from_linear(terms));

        assert_eq!(normalized.as_ptr(), pointer);
        assert_eq!(normalized.capacity(), capacity);
        assert_eq!(normalized.len(), 3);
        assert_eq!(
            normalized,
            vec![
                (VariableId::new(1), 1.0),
                (VariableId::new(4), -2.0),
                (VariableId::new(9), 3.0),
            ]
        );
    }

    fn sorted_terms(mut terms: Vec<(VariableId, f64)>) -> Vec<(VariableId, f64)> {
        terms.sort_by_key(|(variable, _)| *variable);
        terms
    }

    fn assert_same_terms_by_bits(left: Vec<(VariableId, f64)>, right: Vec<(VariableId, f64)>) {
        let left = sorted_terms(left);
        let right = sorted_terms(right);
        assert_eq!(left.len(), right.len());
        for ((left_variable, left_coefficient), (right_variable, right_coefficient)) in
            left.into_iter().zip(right)
        {
            assert_eq!(left_variable, right_variable);
            if left_coefficient.is_nan() {
                assert!(right_coefficient.is_nan());
            } else {
                assert_eq!(left_coefficient.to_bits(), right_coefficient.to_bits());
            }
        }
    }

    #[test]
    fn ineligible_terms_use_the_existing_normalizer() {
        let cases = [
            Expr::from_linear(vec![
                (VariableId::new(3), 2.0),
                (VariableId::new(1), 4.0),
                (VariableId::new(3), -0.5),
                (VariableId::new(2), 0.0),
                (VariableId::new(5), -0.0),
            ]),
            Expr::from_linear(vec![
                (VariableId::new(3), 1.0e16),
                (VariableId::new(3), 1.0),
                (VariableId::new(3), -1.0e16),
            ]),
            Expr::from_linear(vec![
                (VariableId::new(3), 1.0),
                (VariableId::new(1), 2.0),
                (VariableId::new(2), 3.0),
            ]),
            Expr::from_linear(vec![
                (VariableId::new(1), f64::NAN),
                (VariableId::new(2), f64::INFINITY),
                (VariableId::new(3), -f64::INFINITY),
            ]),
        ];

        for expr in cases {
            let expected = expr.normalized_terms();
            let actual = normalized_terms_for_batch(expr);
            assert_same_terms_by_bits(actual, expected);
        }
    }

    #[test]
    fn nonfinite_terms_still_reach_model_coefficient_validation() {
        for coefficient in [f64::NAN, f64::INFINITY, -f64::INFINITY] {
            let mut model = arco_model::Model::new();
            assert!(model
                .add_variable(arco_model::Variable::continuous(arco_model::Bounds::new(
                    0.0,
                    1.0,
                )))
                .is_ok());
            let terms = normalized_terms_for_batch(Expr::from_linear(vec![
                (VariableId::new(0), coefficient),
            ]));
            let result = model.add_constraints_batch_streaming(
                1,
                std::iter::once((terms, arco_model::Bounds::new(0.0, 1.0))),
            );

            assert!(matches!(
                result,
                Err(arco_model::ModelError::InvalidCoefficient { .. })
            ));
        }
    }

    #[test]
    fn nonfinite_batch_terms_preserve_partial_insertion() {
        pyo3::Python::initialize();
        pyo3::Python::attach(|_| {
            let mut model = PyModel::new(None, None).unwrap();
            assert!(model
                .inner
                .add_variable(arco_model::Variable::continuous(
                    arco_model::Bounds::new(0.0, 1.0),
                ))
                .is_ok());
            let rows = vec![
                PyExpr::from_expr(Expr::from_linear(vec![(VariableId::new(0), 1.0)])),
                PyExpr::from_expr(Expr::from_linear(vec![(VariableId::new(0), f64::NAN)])),
            ];

            let result = model.add_constraints_shaped_internal(
                rows.into_iter(),
                ComparisonSense::Equal,
                vec![0.0, 0.0],
                None,
                None,
                &[2],
                &[],
            );

            assert!(result.is_err());
            assert_eq!(model.inner.num_constraints(), 2);
            assert_eq!(model.inner.num_coefficients(), 1);
        });
    }
}
