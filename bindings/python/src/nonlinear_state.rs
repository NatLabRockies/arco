//! State for nonlinear (IPOPT) constraints and objective on `Model`.
//!
//! Only present when the `ipopt` feature is enabled. The Python `Model`
//! accumulates these alongside its linear constraints; when `Model.solve` is
//! invoked with `solver=arco.Ipopt(...)`, the linear and nonlinear pieces are
//! lowered into a single `NonlinearProblem` and solved.

use arco_ops::nlp::NonlinearExpr as NlExpr;

use crate::py_modules::nonlinear::NlSense;

/// One nonlinear constraint stored on the model.
#[derive(Debug, Clone)]
pub struct NonlinearConstraintEntry {
    /// Expression equal to `lhs - rhs`; constraint reads `expr <sense> 0`.
    pub expr: NlExpr,
    pub sense: NlSense,
    pub name: Option<String>,
}

/// Nonlinear objective override. When `Some`, supersedes the linear objective
/// for the IPOPT solve path.
#[derive(Debug, Clone)]
pub struct NonlinearObjectiveEntry {
    pub expr: NlExpr,
    /// `true` = minimize, `false` = maximize.
    pub minimize: bool,
    pub name: Option<String>,
}

/// Aggregate nonlinear state attached to a `PyModel`.
#[derive(Debug, Default, Clone)]
pub struct NonlinearState {
    pub constraints: Vec<NonlinearConstraintEntry>,
    pub objective: Option<NonlinearObjectiveEntry>,
}

impl NonlinearState {
    /// True if any nonlinear constraint or a nonlinear objective is registered.
    pub fn has_any(&self) -> bool {
        !self.constraints.is_empty() || self.objective.is_some()
    }
}
