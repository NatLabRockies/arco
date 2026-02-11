//! Core expression type: terms by degree + constant.
//!
//! Stores terms in separate Vecs per degree for optimal memory:
//! - linear:    12 bytes/term  (VarId, f64)
//! - quadratic: 16 bytes/term  (VarId, VarId, f64)
//! - cubic:     20 bytes/term  (VarId, VarId, VarId, f64)
//!
//! User-facing API is degree-agnostic. Degree partitioning is an
//! internal detail only exposed at the solver boundary.

use crate::expr::constraint::{ComparisonSense, ConstraintExpr};
use crate::ids::VariableId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct Expr {
    constant: f64,
    linear: Vec<(VariableId, f64)>,
    quadratic: Vec<(VariableId, VariableId, f64)>,
    cubic: Vec<(VariableId, VariableId, VariableId, f64)>,
}

impl Expr {
    // ── Constructors ────────────────────────────────────────

    /// Empty expression (all zeros).
    pub fn new_empty() -> Self {
        Self::default()
    }

    /// Expression from linear terms and constant.
    pub fn new(linear: Vec<(VariableId, f64)>, constant: f64) -> Self {
        Self {
            constant,
            linear,
            ..Default::default()
        }
    }

    /// Just a constant, no variable terms.
    pub fn from_constant(constant: f64) -> Self {
        Self {
            constant,
            ..Default::default()
        }
    }

    /// Single linear term: coeff * var.
    pub fn term(var_id: VariableId, coeff: f64) -> Self {
        if coeff == 0.0 {
            return Self::default();
        }
        Self {
            linear: vec![(var_id, coeff)],
            ..Default::default()
        }
    }

    /// Single variable with coefficient 1.0.
    pub fn var(var_id: VariableId) -> Self {
        Self {
            linear: vec![(var_id, 1.0)],
            ..Default::default()
        }
    }

    /// From raw linear terms, no constant.
    pub fn from_linear(linear: Vec<(VariableId, f64)>) -> Self {
        Self {
            linear,
            ..Default::default()
        }
    }

    // ── Accessors ───────────────────────────────────────────

    pub fn constant(&self) -> f64 {
        self.constant
    }

    pub fn linear_terms(&self) -> &[(VariableId, f64)] {
        &self.linear
    }

    pub fn quadratic_terms(&self) -> &[(VariableId, VariableId, f64)] {
        &self.quadratic
    }

    pub fn cubic_terms(&self) -> &[(VariableId, VariableId, VariableId, f64)] {
        &self.cubic
    }

    /// Consume and return linear terms.
    pub fn into_linear_terms(self) -> Vec<(VariableId, f64)> {
        self.linear
    }

    /// Consume and return (linear_terms, constant).
    pub fn into_parts(self) -> (Vec<(VariableId, f64)>, f64) {
        (self.linear, self.constant)
    }

    /// Max degree of any term (0 = constant only).
    pub fn degree(&self) -> usize {
        if !self.cubic.is_empty() {
            3
        } else if !self.quadratic.is_empty() {
            2
        } else {
            usize::from(!self.linear.is_empty())
        }
    }

    // ── Operations (degree-agnostic) ────────────────────────

    /// Scale all terms and constant by a factor.
    pub fn scale(&self, by: f64) -> Self {
        Self {
            constant: self.constant * by,
            linear: self
                .linear
                .iter()
                .map(|(v, c)| (*v, *c * by))
                .filter(|(_, c)| *c != 0.0)
                .collect(),
            quadratic: self
                .quadratic
                .iter()
                .map(|(a, b, c)| (*a, *b, *c * by))
                .filter(|(_, _, c)| *c != 0.0)
                .collect(),
            cubic: self
                .cubic
                .iter()
                .map(|(a, b, c, d)| (*a, *b, *c, *d * by))
                .filter(|(_, _, _, d)| *d != 0.0)
                .collect(),
        }
    }

    /// Add another expression (merges all degree terms + constants).
    pub fn add(&self, other: &Expr) -> Self {
        let mut linear = Vec::with_capacity(self.linear.len() + other.linear.len());
        linear.extend_from_slice(&self.linear);
        linear.extend_from_slice(&other.linear);

        let mut quadratic = Vec::with_capacity(self.quadratic.len() + other.quadratic.len());
        quadratic.extend_from_slice(&self.quadratic);
        quadratic.extend_from_slice(&other.quadratic);

        let mut cubic = Vec::with_capacity(self.cubic.len() + other.cubic.len());
        cubic.extend_from_slice(&self.cubic);
        cubic.extend_from_slice(&other.cubic);

        Self {
            constant: self.constant + other.constant,
            linear,
            quadratic,
            cubic,
        }
    }

    /// Add a constant offset.
    pub fn add_constant(&self, value: f64) -> Self {
        Self {
            constant: self.constant + value,
            linear: self.linear.clone(),
            quadratic: self.quadratic.clone(),
            cubic: self.cubic.clone(),
        }
    }

    /// Copy with constant set to zero.
    pub fn without_constant(&self) -> Self {
        Self {
            constant: 0.0,
            linear: self.linear.clone(),
            quadratic: self.quadratic.clone(),
            cubic: self.cubic.clone(),
        }
    }

    /// Merged linear terms with duplicates combined.
    pub fn normalized_terms(&self) -> Vec<(VariableId, f64)> {
        let mut merged: BTreeMap<VariableId, f64> = BTreeMap::new();
        for (var_id, coeff) in &self.linear {
            if *coeff == 0.0 {
                continue;
            }
            *merged.entry(*var_id).or_insert(0.0) += *coeff;
        }
        merged.into_iter().filter(|(_, c)| *c != 0.0).collect()
    }

    // ── Comparison methods (produce ConstraintExpr) ─────────

    pub fn compare_scalar(&self, rhs: f64, sense: ComparisonSense) -> ConstraintExpr {
        ConstraintExpr::new(self.without_constant(), sense, rhs - self.constant)
    }

    pub fn compare_expr(&self, other: &Expr, sense: ComparisonSense) -> ConstraintExpr {
        let combined = self.add(&other.scale(-1.0));
        ConstraintExpr::new(combined.without_constant(), sense, -combined.constant)
    }

    pub fn le_scalar(&self, rhs: f64) -> ConstraintExpr {
        self.compare_scalar(rhs, ComparisonSense::LessEqual)
    }

    pub fn ge_scalar(&self, rhs: f64) -> ConstraintExpr {
        self.compare_scalar(rhs, ComparisonSense::GreaterEqual)
    }

    pub fn eq_scalar(&self, rhs: f64) -> ConstraintExpr {
        self.compare_scalar(rhs, ComparisonSense::Equal)
    }

    pub fn le_expr(&self, rhs: &Expr) -> ConstraintExpr {
        self.compare_expr(rhs, ComparisonSense::LessEqual)
    }

    pub fn ge_expr(&self, rhs: &Expr) -> ConstraintExpr {
        self.compare_expr(rhs, ComparisonSense::GreaterEqual)
    }

    pub fn eq_expr(&self, rhs: &Expr) -> ConstraintExpr {
        self.compare_expr(rhs, ComparisonSense::Equal)
    }
}

// ── Operator overloads ──────────────────────────────────────

impl std::ops::Add for Expr {
    type Output = Expr;

    fn add(self, rhs: Expr) -> Self::Output {
        Expr::add(&self, &rhs)
    }
}

impl std::ops::Sub for Expr {
    type Output = Expr;

    fn sub(self, rhs: Expr) -> Self::Output {
        Expr::add(&self, &rhs.scale(-1.0))
    }
}

impl std::ops::Mul<f64> for Expr {
    type Output = Expr;

    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

impl std::ops::Neg for Expr {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        self.scale(-1.0)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use crate::VariableId;
    use crate::expr::{
        ComparisonSense, ConstraintExpr, Expr, LinearExprError, linear_sum, linear_terms,
    };

    fn x() -> VariableId {
        VariableId::new(1)
    }

    fn y() -> VariableId {
        VariableId::new(2)
    }

    #[test]
    fn from_constant() {
        let e = Expr::from_constant(5.0);
        assert_eq!(e.constant(), 5.0);
        assert!(e.linear_terms().is_empty());
        assert_eq!(e.degree(), 0);
    }

    #[test]
    fn add_constant() {
        let e = Expr::var(x()).add_constant(3.0);
        assert_eq!(e.constant(), 3.0);
        assert_eq!(e.linear_terms().len(), 1);
    }

    #[test]
    fn scale_with_constant() {
        let e = Expr::new(vec![(x(), 2.0)], 3.0);
        let scaled = e.scale(2.0);
        assert_eq!(scaled.constant(), 6.0);
        assert_eq!(scaled.linear_terms()[0].1, 4.0);
    }

    #[test]
    fn add_exprs_with_constants() {
        let a = Expr::new(vec![(x(), 1.0)], 3.0);
        let b = Expr::new(vec![(y(), 2.0)], 7.0);
        let c = a.add(&b);
        assert_eq!(c.constant(), 10.0);
        assert_eq!(c.linear_terms().len(), 2);
    }

    #[test]
    fn le_scalar() {
        let e = Expr::new(vec![(x(), 1.0)], 3.0);
        let c = e.le_scalar(10.0);
        assert_eq!(c.sense(), ComparisonSense::LessEqual);
        assert_eq!(c.rhs(), 7.0); // 10.0 - 3.0
        assert_eq!(c.expr().constant(), 0.0);
    }

    #[test]
    fn ge_expr() {
        let lhs = Expr::new(vec![(x(), 1.0)], 3.0);
        let rhs = Expr::new(vec![(y(), 1.0)], 7.0);
        let c = lhs.ge_expr(&rhs);
        assert_eq!(c.sense(), ComparisonSense::GreaterEqual);
        assert_eq!(c.rhs(), 4.0); // 7.0 - 3.0
        assert_eq!(c.expr().linear_terms().len(), 2);
    }

    #[test]
    fn eq_scalar() {
        let e = Expr::from_linear(vec![(x(), 1.0)]);
        let c = e.eq_scalar(5.0);
        assert_eq!(c.sense(), ComparisonSense::Equal);
        assert_eq!(c.rhs(), 5.0);
    }

    #[test]
    fn degree_detection() {
        assert_eq!(Expr::from_constant(1.0).degree(), 0);
        assert_eq!(Expr::var(x()).degree(), 1);
    }

    #[test]
    fn without_constant() {
        let e = Expr::new(vec![(x(), 1.0)], 5.0);
        let stripped = e.without_constant();
        assert_eq!(stripped.constant(), 0.0);
        assert_eq!(stripped.linear_terms().len(), 1);
    }

    // ── Migrated tests from the old LinearExpr ─────────────

    #[test]
    fn linear_terms_rejects_mixed_inputs() {
        let result = linear_terms(
            Some(vec![(VariableId::new(1), 1.0)]),
            Some(vec![VariableId::new(1)]),
            None,
        );
        assert_eq!(result.unwrap_err(), LinearExprError::MixedInputs);
    }

    #[test]
    fn linear_terms_rejects_mismatched_lengths() {
        let result = linear_terms(
            None,
            Some(vec![VariableId::new(1), VariableId::new(2)]),
            Some(vec![1.0]),
        );
        assert_eq!(result.unwrap_err(), LinearExprError::MismatchedLengths);
    }

    #[test]
    fn linear_terms_filters_zero_coefficients() {
        let expr = linear_terms(
            Some(vec![(VariableId::new(1), 0.0), (VariableId::new(2), 3.5)]),
            None,
            None,
        )
        .expect("linear_terms should succeed");

        let terms = expr
            .linear_terms()
            .iter()
            .map(|(id, coeff)| (id.inner(), *coeff))
            .collect::<Vec<_>>();
        assert_eq!(terms, vec![(2, 3.5)]);
    }

    #[test]
    fn normalized_terms_merges_duplicates() {
        let expr = Expr::term(VariableId::new(1), 2.0)
            .add(&Expr::term(VariableId::new(1), -2.0))
            .add(&Expr::term(VariableId::new(2), 4.0));

        let normalized = expr
            .normalized_terms()
            .into_iter()
            .map(|(id, coeff)| (id.inner(), coeff))
            .collect::<Vec<_>>();
        assert_eq!(normalized, vec![(2, 4.0)]);
    }

    #[test]
    fn constraint_expr_exposes_parts() {
        let expr = Expr::term(VariableId::new(1), 1.0);
        let constraint = ConstraintExpr::new(expr.clone(), ComparisonSense::LessEqual, 10.0);

        assert_eq!(constraint.sense(), ComparisonSense::LessEqual);
        assert_eq!(constraint.rhs(), 10.0);
        assert_eq!(constraint.expr().linear_terms().len(), 1);

        let (inner, sense, rhs) = constraint.into_parts();
        assert_eq!(sense, ComparisonSense::LessEqual);
        assert_eq!(rhs, 10.0);
        assert_eq!(inner.linear_terms().len(), 1);
    }

    #[test]
    fn linear_sum_concatenates_terms() {
        let left = Expr::term(VariableId::new(1), 1.0);
        let right = Expr::term(VariableId::new(2), 2.0);
        let summed = linear_sum(vec![left, right]);
        let terms = summed
            .linear_terms()
            .iter()
            .map(|(id, coeff)| (id.inner(), *coeff))
            .collect::<Vec<_>>();
        assert_eq!(terms, vec![(1, 1.0), (2, 2.0)]);
    }
}
