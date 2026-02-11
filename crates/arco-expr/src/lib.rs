pub mod expr;
pub mod ids;

pub use expr::{ComparisonSense, ConstraintExpr, Expr, LinearExprError};
pub use ids::{ConstraintId, VariableId};
