//! Canonical algebra seam for Arco.
//!
//! Expression internals live in `arco-model`; this crate is a stable algebra
//! import seam while legacy callers migrate.

pub use arco_model::expr::{ComparisonSense, ConstraintExpr, Expr, LinearExprError};
pub use arco_model::{ConstraintId, ExpressionId, VariableId};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebra_seam_reexports_variable_id() {
        let id = VariableId::new(0);
        assert_eq!(id.inner(), 0);
    }
}
