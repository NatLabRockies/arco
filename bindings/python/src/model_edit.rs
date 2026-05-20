use crate::py_modules::arrays;
use crate::py_modules::errors;
use crate::{
    BoundsSpec, PyConstraintArray, PyExpr, PyIndexSet, PyModel, PyVariable, PyVariableArray,
    bounds_from_sense, extract_objective_terms,
};
use arco_arrays::BroadcastPlan;
use arco_ops::expression::{ComparisonSense, ConstraintId, Expr, VariableId};
use arco_ops::modeling::types::Bounds;
use arco_ops::modeling::{Objective, Sense, Variable};
use pyo3::prelude::*;
use pyo3::types::PyAny;

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
        if spec.len == 1 {
            Some(spec.base_name.clone())
        } else {
            Some(format!("{}[{}]", spec.base_name, offset))
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
                if spec.base_name == base && offset < spec.len {
                    return Some(VariableId::new(spec.start_var_id + offset as u32));
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

    /// Name a contiguous block of constraints starting at `first_id`.
    pub(crate) fn name_constraint_block(
        &mut self,
        first_id: ConstraintId,
        count: usize,
        name: Option<&str>,
    ) -> PyResult<()> {
        let Some(base) = name else { return Ok(()) };
        for index in 0..count {
            let constraint_id = ConstraintId::new(first_id.inner() + index as u32);
            let con_name = if count == 1 {
                base.to_string()
            } else {
                format!("{base}[{index}]")
            };
            self.inner
                .set_constraint_name(constraint_id, con_name)
                .map_err(errors::model_error_to_py)?;
        }
        Ok(())
    }

    /// Insert constraints via compact term patterns (zero per-element allocation).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_constraints_compact_internal(
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

        let term_patterns = compact.term_patterns();
        let rhs_vec = compact.rhs_vec();

        let mut batch: Vec<(Vec<(VariableId, f64)>, Bounds)> = Vec::new();
        let mut filtered_rhs: Vec<f64> = Vec::new();

        for idx in active_indices {
            let rhs_val = rhs_vec[idx];
            let terms: Vec<(VariableId, f64)> = term_patterns
                .iter()
                .map(|(start_var_id, coeff)| (VariableId::new(start_var_id + idx as u32), *coeff))
                .collect();
            batch.push((terms, bounds_from_sense(sense, rhs_val)));
            filtered_rhs.push(rhs_val);
        }

        let first_constraint_id = self
            .inner
            .add_constraints_batch(&batch)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, batch.len(), name.as_deref())?;

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

        let mut batch: Vec<(Vec<(VariableId, f64)>, Bounds)> =
            Vec::with_capacity(active_indices.len());
        let mut filtered_rhs: Vec<f64> = Vec::with_capacity(active_indices.len());
        for idx in active_indices {
            let diff = left.values[idx]
                .inner()
                .add(&right.values[idx].inner().scale(-1.0));
            let diff_expr = PyExpr::from_expr(diff);
            let rhs_value = -diff_expr.constant();
            batch.push((
                diff_expr.without_constant().into_inner().normalized_terms(),
                bounds_from_sense(sense, rhs_value),
            ));
            filtered_rhs.push(rhs_value);
        }

        let first_constraint_id = self
            .inner
            .add_constraints_batch(&batch)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, batch.len(), name.as_deref())?;

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
            constraints.exprs().to_vec(),
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
    pub(crate) fn add_constraints_shaped_internal(
        &mut self,
        exprs: Vec<PyExpr>,
        sense: ComparisonSense,
        rhs: Vec<f64>,
        active: Option<&Bound<'_, PyAny>>,
        name: Option<String>,
        shape: &[usize],
        index_sets: &[Py<PyIndexSet>],
    ) -> PyResult<PyConstraintArray> {
        let total = exprs.len();
        let active_indices = Self::resolve_active_indices(
            active,
            (!index_sets.is_empty()).then_some(index_sets),
            shape,
            total,
        )?;

        let mut active_lookup = vec![false; total];
        for idx in active_indices {
            active_lookup[idx] = true;
        }

        let mut batch: Vec<(Vec<(VariableId, f64)>, Bounds)> = Vec::new();
        let mut filtered_rhs: Vec<f64> = Vec::new();
        for (idx, (expr, rhs_val)) in exprs.into_iter().zip(rhs.iter().copied()).enumerate() {
            if active_lookup[idx] {
                let bounds = bounds_from_sense(sense, rhs_val);
                batch.push((expr.into_inner().normalized_terms(), bounds));
                filtered_rhs.push(rhs_val);
            }
        }

        if batch.is_empty() {
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
            .add_constraints_batch(&batch)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, batch.len(), name.as_deref())?;

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
            for _ in 0..total {
                self.inner
                    .add_variable(var_template)
                    .map_err(errors::model_error_to_py)?;
            }

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
        self.inner.reserve_variables(active_count);
        let var_template = Variable {
            bounds: bounds.bounds,
            is_integer: effective_bounds.is_integer,
            is_active: true,
        };
        let mut next_active = active_indices.into_iter().peekable();
        let mut values = Vec::with_capacity(total);
        let mut variables = Vec::with_capacity(total);

        for idx in 0..total {
            if next_active.peek().copied() == Some(idx) {
                let var_id = self
                    .inner
                    .add_variable(var_template)
                    .map_err(errors::model_error_to_py)?;
                let var_name = name.as_ref().map(|base| {
                    if total == 1 {
                        base.clone()
                    } else {
                        format!("{base}[{idx}]")
                    }
                });
                if let Some(var_name_ref) = var_name.as_ref() {
                    self.inner
                        .set_variable_name(var_id, var_name_ref.clone())
                        .map_err(errors::model_error_to_py)?;
                }
                values.push(PyExpr::from_term(var_id.inner(), 1.0));
                variables.push(Some(PyVariable::new(
                    var_id.inner(),
                    var_name,
                    effective_bounds,
                )));
                let _ = next_active.next();
            } else {
                values.push(PyExpr::from_expr(Expr::from_constant(0.0)));
                variables.push(None);
            }
        }

        Ok(PyVariableArray::new_sparse(
            index_sets,
            shape.to_vec(),
            values,
            variables,
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
        let lo_values =
            Self::resolve_labeled_f64_values(&lo_attr, &index_sets, shape, total, "lower bounds")?;
        let hi_values =
            Self::resolve_labeled_f64_values(&hi_attr, &index_sets, shape, total, "upper bounds")?;

        let effective_binary = is_binary;
        let effective_integer = is_integer || effective_binary;

        if active_count == total {
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

        let mut values = Vec::with_capacity(total);
        let mut variables = Vec::with_capacity(total);
        let mut next_active = active_indices.into_iter().peekable();
        for i in 0..total {
            if next_active.peek().copied() == Some(i) {
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
                if let Some(var_name_ref) = var_name.as_ref() {
                    self.inner
                        .set_variable_name(var_id, var_name_ref.clone())
                        .map_err(errors::model_error_to_py)?;
                }

                let element_bounds_spec = BoundsSpec {
                    bounds: element_bounds,
                    is_integer: effective_integer,
                    is_binary: effective_binary,
                };

                values.push(PyExpr::from_term(var_id.inner(), 1.0));
                variables.push(Some(PyVariable::new(
                    var_id.inner(),
                    var_name,
                    element_bounds_spec,
                )));
                let _ = next_active.next();
            } else {
                values.push(PyExpr::from_expr(Expr::from_constant(0.0)));
                variables.push(None);
            }
        }

        Ok(PyVariableArray::new_sparse(
            index_sets,
            shape.to_vec(),
            values,
            variables,
        ))
    }
}
