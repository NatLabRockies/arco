//! Constraint expressions: linear expression with comparison sense and RHS.

use crate::expr::core::Expr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonSense {
    LessEqual,
    GreaterEqual,
    Equal,
}

impl ComparisonSense {
    pub fn as_str(self) -> &'static str {
        match self {
            ComparisonSense::LessEqual => "le",
            ComparisonSense::GreaterEqual => "ge",
            ComparisonSense::Equal => "eq",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstraintExpr {
    expr: Expr,
    sense: ComparisonSense,
    rhs: f64,
}

impl ConstraintExpr {
    pub fn new(expr: Expr, sense: ComparisonSense, rhs: f64) -> Self {
        Self { expr, sense, rhs }
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn sense(&self) -> ComparisonSense {
        self.sense
    }

    pub fn rhs(&self) -> f64 {
        self.rhs
    }

    pub fn into_parts(self) -> (Expr, ComparisonSense, f64) {
        (self.expr, self.sense, self.rhs)
    }
}
