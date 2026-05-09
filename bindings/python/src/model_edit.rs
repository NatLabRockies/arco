use crate::py_modules::arrays;
use crate::py_modules::errors;
use crate::{
    BoundsSpec, PyConstraintArray, PyExpr, PyIndexSet, PyModel, PyVariable, PyVariableArray,
    bounds_from_sense, extract_objective_terms,
};
use arco_ops::expression::{ComparisonSense, ConstraintId, VariableId};
use arco_ops::modeling::types::Bounds;
use arco_ops::modeling::{Objective, Sense, Variable};
use pyo3::prelude::*;
use pyo3::types::PyAny;

impl PyModel {
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
    pub(crate) fn add_constraints_compact_internal(
        &mut self,
        compact: &arrays::CompactConstraintStorage,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        let count = compact.count;
        let term_patterns = compact.term_patterns();
        let sense = compact.sense;

        // Build per-element bounds from sense + rhs
        let bounds_list: Vec<Bounds> = match &compact.rhs {
            arrays::CompactRhs::Scalar(rhs_val) => {
                vec![bounds_from_sense(sense, *rhs_val); count]
            }
            arrays::CompactRhs::Vec(rhs_values) => rhs_values
                .iter()
                .map(|rhs_val| bounds_from_sense(sense, *rhs_val))
                .collect(),
        };

        let first_constraint_id = self
            .inner
            .add_constraints_compact(&term_patterns, &bounds_list)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, count, name.as_deref())?;

        let rhs_vec = compact.rhs_vec();
        Ok(PyConstraintArray::from_batch(
            first_constraint_id.inner(),
            count,
            sense,
            &rhs_vec,
        ))
    }

    /// Add constraints from an array expression (VariableArray or ExprArray) with a separate rhs.
    ///
    /// Tries the compact fast path first, then falls back to materialized comparison.
    pub(crate) fn add_constraints_from_array(
        &mut self,
        compact_expr: Option<arrays::CompactExprStorage>,
        core_fn: impl FnOnce() -> arrays::LinearArrayCore,
        rhs_obj: &Bound<'_, PyAny>,
        sense: ComparisonSense,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        // Fast path: compact expression
        if let Some(ref compact) = compact_expr {
            if let Some(compact_con) = arrays::try_make_compact_constraint(compact, rhs_obj, sense)
            {
                return self.add_constraints_compact_internal(&compact_con, name);
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
        self.add_constraints_full_internal(
            constraints.exprs().to_vec(),
            constraints.get_sense(),
            constraints.get_rhs(),
            name,
        )
    }

    /// Insert constraints via materialized expressions (existing batch path).
    pub(crate) fn add_constraints_full_internal(
        &mut self,
        exprs: Vec<PyExpr>,
        sense: ComparisonSense,
        rhs: Vec<f64>,
        name: Option<String>,
    ) -> PyResult<PyConstraintArray> {
        let total = exprs.len();

        let batch: Vec<(Vec<(VariableId, f64)>, Bounds)> = exprs
            .into_iter()
            .zip(rhs.iter())
            .map(|(expr, &rhs_val)| {
                let bounds = bounds_from_sense(sense, rhs_val);
                (expr.into_inner().normalized_terms(), bounds)
            })
            .collect();

        let first_constraint_id = self
            .inner
            .add_constraints_batch(&batch)
            .map_err(errors::model_error_to_py)?;

        self.name_constraint_block(first_constraint_id, total, name.as_deref())?;

        Ok(PyConstraintArray::from_batch(
            first_constraint_id.inner(),
            total,
            sense,
            &rhs,
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
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let effective_bounds = Self::effective_bounds(&bounds, is_integer, is_binary)?;
        let start_var_id = self.inner.num_variables() as u32;
        self.inner.reserve_variables(total);

        // Add all variables to the model in a tight loop (no PyExpr/PyVariable allocation)
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

        // Use compact storage: no Vec<PyExpr> or Vec<PyVariable> allocated
        Ok(PyVariableArray::new_compact(
            index_sets,
            shape.to_vec(),
            start_var_id,
            total,
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
        name: Option<String>,
    ) -> PyResult<PyVariableArray> {
        let start_var_id = self.inner.num_variables() as u32;
        self.inner.reserve_variables(total);

        // Extract lower and upper as numpy arrays from a Bounds-like object
        let lo_attr = bounds_obj
            .getattr("lower")
            .or_else(|_| bounds_obj.getattr("lo"))?;
        let hi_attr = bounds_obj
            .getattr("upper")
            .or_else(|_| bounds_obj.getattr("hi"))?;

        let np = py.import("numpy")?;
        let lo_flat = np
            .call_method1("asarray", (&lo_attr,))?
            .call_method0("flatten")?;
        let hi_flat = np
            .call_method1("asarray", (&hi_attr,))?
            .call_method0("flatten")?;

        let lo_values: Vec<f64> = lo_flat.extract()?;
        let hi_values: Vec<f64> = hi_flat.extract()?;

        if lo_values.len() != total {
            return Err(errors::ArrayShapeMismatchError::new_err(format!(
                "lower bounds length {} does not match total variables {}",
                lo_values.len(),
                total
            )));
        }
        if hi_values.len() != total {
            return Err(errors::ArrayShapeMismatchError::new_err(format!(
                "upper bounds length {} does not match total variables {}",
                hi_values.len(),
                total
            )));
        }

        let effective_binary = is_binary;
        let effective_integer = is_integer || effective_binary;

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

        Ok(PyVariableArray::new(
            index_sets,
            shape.to_vec(),
            values,
            variables,
        ))
    }
}
