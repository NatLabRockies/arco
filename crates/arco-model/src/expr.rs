//! Detached expression primitives exposed from the model crate.
//!
//! This is the canonical import location for expression construction during the
//! migration away from the standalone expression crate.

pub use arco_expr::expr::{ComparisonSense, ConstraintExpr, Expr};
