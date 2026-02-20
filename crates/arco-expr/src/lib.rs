//! Expression primitives and helpers for Arco models.

pub mod expr;
/// Lightweight typed identifiers used by expression and model APIs.
pub mod ids;

pub use expr::{ComparisonSense, ConstraintExpr, Expr, LinearExprError};
pub use ids::{ConstraintId, VariableId};
