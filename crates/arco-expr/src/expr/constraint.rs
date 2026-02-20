//! Constraint expressions: linear expression with comparison sense and RHS.

use crate::expr::core::Expr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Comparison operator used to form a model constraint.
pub enum ComparisonSense {
    /// Left-hand side is less than or equal to the right-hand side.
    LessEqual,
    /// Left-hand side is greater than or equal to the right-hand side.
    GreaterEqual,
    /// Left-hand side is exactly equal to the right-hand side.
    Equal,
}

impl ComparisonSense {
    /// Returns the compact solver-facing token (`"le"`, `"ge"`, or `"eq"`).
    pub fn as_str(self) -> &'static str {
        match self {
            ComparisonSense::LessEqual => "le",
            ComparisonSense::GreaterEqual => "ge",
            ComparisonSense::Equal => "eq",
        }
    }
}

#[derive(Debug, Clone)]
/// Normalized constraint expression in the form `expr (sense) rhs`.
pub struct ConstraintExpr {
    expr: Expr,
    sense: ComparisonSense,
    rhs: f64,
}

impl ConstraintExpr {
    /// Creates a constraint from a normalized expression, comparison sense, and RHS.
    pub fn new(expr: Expr, sense: ComparisonSense, rhs: f64) -> Self {
        Self { expr, sense, rhs }
    }

    /// Returns the left-hand expression (typically with constant term removed).
    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    /// Returns the comparison sense.
    pub fn sense(&self) -> ComparisonSense {
        self.sense
    }

    /// Returns the right-hand side scalar.
    pub fn rhs(&self) -> f64 {
        self.rhs
    }

    /// Decomposes the constraint into `(expr, sense, rhs)`.
    pub fn into_parts(self) -> (Expr, ComparisonSense, f64) {
        (self.expr, self.sense, self.rhs)
    }
}
