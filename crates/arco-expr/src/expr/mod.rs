//! Expression types for optimization modeling.
//!
//! This module provides the core expression types used to build optimization models:
//!
//! - [`Expr`] — A polynomial expression with terms organized by degree (linear, quadratic, cubic) plus a constant
//! - [`ConstraintExpr`] — An expression with a comparison sense (≤, =, ≥) and right-hand side value
//! - [`LinearExprError`] — Error types for expression construction failures
//! - [`linear_sum`] — Builder function to combine multiple expressions
//! - [`linear_terms`] — Builder function to create expressions from flexible inputs

pub mod builders;
pub mod constraint;
pub mod core;
pub mod error;

pub use builders::{linear_sum, linear_terms};
pub use constraint::{ComparisonSense, ConstraintExpr};
pub use core::Expr;
pub use error::LinearExprError;
