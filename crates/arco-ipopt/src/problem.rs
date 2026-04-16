//! IPOPT problem adapter that maps an `arco_core::Model` to `ipopt::ConstrainedProblem`.

use arco_core::Model;
use arco_core::solver::SolverError;
use arco_expr::{ConstraintId, VariableId};
use ipopt::{BasicProblem, ConstrainedProblem, Index, Number};
use std::collections::BTreeMap;

/// Adapter that presents an `arco_core::Model` as an `ipopt::ConstrainedProblem`.
///
/// For LP problems, the Jacobian is constant and the Hessian is zero (we use
/// IPOPT's limited-memory BFGS approximation instead).
pub struct ArcoProblem {
    // Variable data
    pub(crate) num_vars: usize,
    pub(crate) x_lower: Vec<f64>,
    pub(crate) x_upper: Vec<f64>,
    pub(crate) x_init: Vec<f64>,

    // Objective: sign * c^T x
    pub(crate) obj_coeffs: Vec<f64>,
    pub(crate) obj_sign: f64, // +1 for minimize, -1 for maximize

    // Constraint data
    pub(crate) num_constraints: usize,
    pub(crate) g_lower: Vec<f64>,
    pub(crate) g_upper: Vec<f64>,

    // Jacobian in COO format (constant for LP)
    pub(crate) jac_rows: Vec<Index>,
    pub(crate) jac_cols: Vec<Index>,
    pub(crate) jac_vals: Vec<f64>,
}

/// IPOPT's conventional bounds for "infinity".
const IPOPT_INF: f64 = 1e19;

/// Clamp a bound value to IPOPT's finite range `[-IPOPT_INF, IPOPT_INF]`.
fn clamp_bound(val: f64) -> f64 {
    val.clamp(-IPOPT_INF, IPOPT_INF)
}

/// Pick a reasonable initial point: 0 when feasible, otherwise the nearest finite bound.
fn default_primal_value(lower: f64, upper: f64) -> f64 {
    if lower <= 0.0 && 0.0 <= upper {
        0.0
    } else if lower > 0.0 && lower.is_finite() {
        lower
    } else if upper < 0.0 && upper.is_finite() {
        upper
    } else {
        0.0
    }
}

impl ArcoProblem {
    /// Build an `ArcoProblem` from an Arco model.
    ///
    /// Returns an error if the model contains integer variables (IPOPT is
    /// continuous-only) or has no objective set.
    pub fn from_model(
        model: &Model,
        primal_start: Option<&[(VariableId, f64)]>,
    ) -> Result<Self, SolverError> {
        let num_vars = model.num_variables();
        if num_vars == 0 {
            return Err(SolverError::EmptyModel);
        }

        let objective = model.objective();
        let Some(sense) = objective.sense else {
            return Err(SolverError::NoObjective);
        };
        let obj_sign = match sense {
            arco_core::Sense::Minimize => 1.0,
            arco_core::Sense::Maximize => -1.0,
        };

        // Collect variable data
        let mut x_lower = Vec::with_capacity(num_vars);
        let mut x_upper = Vec::with_capacity(num_vars);
        let mut x_init = Vec::with_capacity(num_vars);
        let mut var_id_to_col: BTreeMap<VariableId, usize> = BTreeMap::new();

        for index in 0..num_vars {
            let var_id = VariableId::new(index as u32);
            let var = model
                .get_variable(var_id)
                .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;

            // IPOPT does not support integer variables
            if var.is_integer {
                return Err(SolverError::SolverSpecific(
                    "IPOPT does not support integer variables".to_string(),
                ));
            }

            let (lo, hi) = if var.is_active {
                (var.bounds.lower, var.bounds.upper)
            } else {
                (0.0, 0.0) // fixed at zero
            };

            x_lower.push(clamp_bound(lo));
            x_upper.push(clamp_bound(hi));
            x_init.push(default_primal_value(lo, hi));
            var_id_to_col.insert(var_id, index);
        }

        // Apply primal start hints
        if let Some(hints) = primal_start {
            for (var_id, value) in hints {
                let Some(&col) = var_id_to_col.get(var_id) else {
                    return Err(SolverError::InvalidVariableId(var_id.inner()));
                };
                x_init[col] = *value;
            }
        }

        // Build dense objective coefficient vector
        let mut obj_coeffs = vec![0.0; num_vars];
        for (var_id, coeff) in &objective.terms {
            let var = model
                .get_variable(*var_id)
                .map_err(|_| SolverError::InvalidVariableId(var_id.inner()))?;
            if !var.is_active {
                continue;
            }
            if let Some(&col) = var_id_to_col.get(var_id) {
                obj_coeffs[col] += obj_sign * coeff;
            }
        }

        // Collect constraint bounds and Jacobian in COO format
        let num_constraints = model.num_constraints();
        let mut g_lower = Vec::with_capacity(num_constraints);
        let mut g_upper = Vec::with_capacity(num_constraints);
        let mut jac_rows: Vec<Index> = Vec::new();
        let mut jac_cols: Vec<Index> = Vec::new();
        let mut jac_vals: Vec<f64> = Vec::new();

        for con_index in 0..num_constraints {
            let con_id = ConstraintId::new(con_index as u32);
            let constraint = model.get_constraint(con_id).map_err(|_| {
                SolverError::SolverSpecific(format!("constraint {} not found", con_index))
            })?;
            g_lower.push(clamp_bound(constraint.bounds.lower));
            g_upper.push(clamp_bound(constraint.bounds.upper));
        }

        // Iterate columns (CSC) to build COO Jacobian
        for (var_id, column) in model.columns() {
            let var = match model.get_variable(var_id) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !var.is_active {
                continue;
            }
            let Some(&col) = var_id_to_col.get(&var_id) else {
                continue;
            };
            for (con_id, coeff) in column {
                jac_rows.push(con_id.inner() as Index);
                jac_cols.push(col as Index);
                jac_vals.push(*coeff);
            }
        }

        Ok(ArcoProblem {
            num_vars,
            x_lower,
            x_upper,
            x_init,
            obj_coeffs,
            obj_sign,
            num_constraints,
            g_lower,
            g_upper,
            jac_rows,
            jac_cols,
            jac_vals,
        })
    }
}

impl BasicProblem for ArcoProblem {
    fn num_variables(&self) -> usize {
        self.num_vars
    }

    fn bounds(&self, x_l: &mut [Number], x_u: &mut [Number]) -> bool {
        x_l.copy_from_slice(&self.x_lower);
        x_u.copy_from_slice(&self.x_upper);
        true
    }

    fn initial_point(&self, x: &mut [Number]) -> bool {
        x.copy_from_slice(&self.x_init);
        true
    }

    fn objective(&self, x: &[Number], obj: &mut Number) -> bool {
        // c^T x (coeffs already include obj_sign)
        *obj = self.obj_coeffs.iter().zip(x).map(|(c, xi)| c * xi).sum();
        true
    }

    fn objective_grad(&self, _x: &[Number], grad_f: &mut [Number]) -> bool {
        // Constant gradient for LP (already includes obj_sign)
        grad_f.copy_from_slice(&self.obj_coeffs);
        true
    }
}

impl ConstrainedProblem for ArcoProblem {
    fn num_constraints(&self) -> usize {
        self.num_constraints
    }

    fn num_constraint_jacobian_non_zeros(&self) -> usize {
        self.jac_rows.len()
    }

    fn constraint(&self, x: &[Number], g: &mut [Number]) -> bool {
        // g = A * x (sparse matrix-vector product via COO)
        g.fill(0.0);
        for ((&row, &col), &val) in self
            .jac_rows
            .iter()
            .zip(self.jac_cols.iter())
            .zip(self.jac_vals.iter())
        {
            g[row as usize] += val * x[col as usize];
        }
        true
    }

    fn constraint_bounds(&self, g_l: &mut [Number], g_u: &mut [Number]) -> bool {
        g_l.copy_from_slice(&self.g_lower);
        g_u.copy_from_slice(&self.g_upper);
        true
    }

    fn constraint_jacobian_indices(&self, rows: &mut [Index], cols: &mut [Index]) -> bool {
        rows.copy_from_slice(&self.jac_rows);
        cols.copy_from_slice(&self.jac_cols);
        true
    }

    fn constraint_jacobian_values(&self, _x: &[Number], vals: &mut [Number]) -> bool {
        // Constant Jacobian for LP
        vals.copy_from_slice(&self.jac_vals);
        true
    }

    fn num_hessian_non_zeros(&self) -> usize {
        // LP has zero Hessian; we use limited-memory BFGS approximation
        0
    }

    fn hessian_indices(&self, _rows: &mut [Index], _cols: &mut [Index]) -> bool {
        true
    }

    fn hessian_values(
        &self,
        _x: &[Number],
        _obj_factor: Number,
        _lambda: &[Number],
        _vals: &mut [Number],
    ) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_core::types::Bounds;
    use arco_core::{Constraint, Objective, Sense, Variable};

    #[test]
    fn test_from_model_rejects_integer_variables() {
        let mut model = Model::new();
        let _x = model
            .add_variable(Variable::integer(Bounds::new(0.0, 10.0)))
            .unwrap();
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(_x, 1.0)],
            })
            .unwrap();

        let result = ArcoProblem::from_model(&model, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_model_simple_lp() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .unwrap();
        let y = model
            .add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))
            .unwrap();

        let con = model
            .add_constraint(Constraint {
                bounds: Bounds::new(5.0, f64::INFINITY),
            })
            .unwrap();
        model.set_coefficient(x, con, 1.0).unwrap();
        model.set_coefficient(y, con, 1.0).unwrap();

        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 2.0), (y, 3.0)],
            })
            .unwrap();

        let problem = ArcoProblem::from_model(&model, None).unwrap();
        assert_eq!(problem.num_vars, 2);
        assert_eq!(problem.num_constraints, 1);
        assert!((problem.obj_sign - 1.0).abs() < f64::EPSILON);
        assert_eq!(problem.jac_rows.len(), 2);
    }

    #[test]
    fn test_maximize_negates_coefficients() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();

        model
            .set_objective(Objective {
                sense: Some(Sense::Maximize),
                terms: vec![(x, 3.0)],
            })
            .unwrap();

        let problem = ArcoProblem::from_model(&model, None).unwrap();
        assert!((problem.obj_sign - (-1.0)).abs() < f64::EPSILON);
        // Coefficient should be negated: -1 * 3.0 = -3.0
        assert!((problem.obj_coeffs[0] - (-3.0)).abs() < 1e-12);
    }
}
