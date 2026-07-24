//! Detached expression primitives owned by the model crate.
//!
//! These are the canonical expression types for primitive model construction.

//! Expression construction errors.

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned while building linear expressions from user input.
pub enum LinearExprError {
    /// Both paired `terms` and split `variables/coefficients` were provided.
    MixedInputs,
    /// Required inputs were omitted for the selected construction style.
    MissingInputs,
    /// `variables` and `coefficients` lengths differ.
    MismatchedLengths,
}

impl LinearExprError {
    /// Returns a semantic error code for programmatic handling.
    pub(crate) fn code(&self) -> &'static str {
        match self {
            LinearExprError::MixedInputs => "EXPR_MIXED_INPUTS",
            LinearExprError::MissingInputs => "EXPR_MISSING_INPUTS",
            LinearExprError::MismatchedLengths => "EXPR_MISMATCHED_LENGTHS",
        }
    }

    fn detail(&self) -> &'static str {
        match self {
            LinearExprError::MixedInputs => "Use either terms or variables/coefficients, not both",
            LinearExprError::MissingInputs => "variables and coefficients are required",
            LinearExprError::MismatchedLengths => {
                "variables and coefficients must have the same length"
            }
        }
    }
}

impl std::fmt::Display for LinearExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code(), self.detail())
    }
}

impl std::error::Error for LinearExprError {}

#[cfg(test)]
mod error_tests {
    use crate::expr::LinearExprError;

    #[test]
    fn error_code_is_stable() {
        assert_eq!(LinearExprError::MixedInputs.code(), "EXPR_MIXED_INPUTS");
        assert_eq!(LinearExprError::MissingInputs.code(), "EXPR_MISSING_INPUTS");
        assert_eq!(
            LinearExprError::MismatchedLengths.code(),
            "EXPR_MISMATCHED_LENGTHS"
        );
    }

    #[test]
    fn display_prefixes_error_code() {
        let rendered = LinearExprError::MixedInputs.to_string();
        assert!(rendered.starts_with("[EXPR_MIXED_INPUTS]"));
    }
}

// Core expression type: terms by degree + constant.
//
// Stores terms in separate Vecs per degree for optimal memory:
// - linear:    12 bytes/term  (VarId, f64)
// - quadratic: 16 bytes/term  (VarId, VarId, f64)
// - cubic:     20 bytes/term  (VarId, VarId, VarId, f64)
//
// User-facing API is degree-agnostic. Degree partitioning is an
// internal detail only exposed at the solver boundary.

pub use crate::ids::{ConstraintId, VariableId};
use std::collections::HashMap;

/// Sparse polynomial expression split by degree plus a constant term.
///
/// Terms are stored in separate vectors for linear, quadratic, and cubic
/// components to keep the hot path compact and predictable.
/// Quadratic and cubic fields use `Option<Box<Vec<T>>>` to avoid 24-byte
/// empty `Vec` overhead per expression (saves ~32 bytes each for the
/// common linear-only case).
#[derive(Debug, Clone, Default)]
#[allow(clippy::box_collection, clippy::type_complexity)]
pub struct Expr {
    constant: f64,
    linear: Vec<(VariableId, f64)>,
    quadratic: Option<Box<Vec<(VariableId, VariableId, f64)>>>,
    cubic: Option<Box<Vec<(VariableId, VariableId, VariableId, f64)>>>,
}

impl Expr {
    // ── Helpers ──────────────────────────────────────────────

    /// Wrap a non-empty vec into `Option<Box<Vec<T>>>`, returning `None` for empty.
    #[inline]
    #[allow(clippy::box_collection)]
    fn wrap_optional<T>(v: Vec<T>) -> Option<Box<Vec<T>>> {
        if v.is_empty() {
            None
        } else {
            Some(Box::new(v))
        }
    }

    /// Return quadratic terms as a slice (empty if `None`).
    #[inline]
    fn quad_slice(&self) -> &[(VariableId, VariableId, f64)] {
        self.quadratic.as_deref().map_or(&[], Vec::as_slice)
    }

    /// Return cubic terms as a slice (empty if `None`).
    #[inline]
    fn cubic_slice(&self) -> &[(VariableId, VariableId, VariableId, f64)] {
        self.cubic.as_deref().map_or(&[], Vec::as_slice)
    }

    /// Get or create the quadratic vec for mutation.
    #[inline]
    fn quad_mut(&mut self) -> &mut Vec<(VariableId, VariableId, f64)> {
        self.quadratic.get_or_insert_with(|| Box::new(Vec::new()))
    }

    /// Get or create the cubic vec for mutation.
    #[inline]
    fn cubic_mut(&mut self) -> &mut Vec<(VariableId, VariableId, VariableId, f64)> {
        self.cubic.get_or_insert_with(|| Box::new(Vec::new()))
    }

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
            quadratic: None,
            cubic: None,
        }
    }

    /// Just a constant, no variable terms.
    pub fn from_constant(constant: f64) -> Self {
        Self {
            constant,
            linear: Vec::new(),
            quadratic: None,
            cubic: None,
        }
    }

    /// Single linear term: coeff * var.
    pub fn term(var_id: VariableId, coeff: f64) -> Self {
        if coeff == 0.0 {
            return Self::default();
        }
        Self {
            constant: 0.0,
            linear: vec![(var_id, coeff)],
            quadratic: None,
            cubic: None,
        }
    }

    /// Single variable with coefficient 1.0.
    pub fn var(var_id: VariableId) -> Self {
        Self {
            constant: 0.0,
            linear: vec![(var_id, 1.0)],
            quadratic: None,
            cubic: None,
        }
    }

    /// From raw linear terms, no constant.
    pub fn from_linear(linear: Vec<(VariableId, f64)>) -> Self {
        Self {
            constant: 0.0,
            linear,
            quadratic: None,
            cubic: None,
        }
    }

    // ── Accessors ───────────────────────────────────────────

    /// Returns the scalar constant term.
    pub fn constant(&self) -> f64 {
        self.constant
    }

    /// Returns the raw linear terms `(var, coeff)` in insertion order.
    pub fn linear_terms(&self) -> &[(VariableId, f64)] {
        &self.linear
    }

    /// Returns the raw quadratic terms `(var_a, var_b, coeff)`.
    pub fn quadratic_terms(&self) -> &[(VariableId, VariableId, f64)] {
        self.quad_slice()
    }

    /// Returns the raw cubic terms `(var_a, var_b, var_c, coeff)`.
    pub fn cubic_terms(&self) -> &[(VariableId, VariableId, VariableId, f64)] {
        self.cubic_slice()
    }

    /// Consume and return linear terms.
    pub fn into_linear_terms(self) -> Vec<(VariableId, f64)> {
        self.linear
    }

    /// Consume and return (linear_terms, constant).
    pub fn into_parts(self) -> (Vec<(VariableId, f64)>, f64) {
        (self.linear, self.constant)
    }

    /// Total number of terms across all degrees.
    pub fn num_terms(&self) -> usize {
        self.linear.len() + self.quad_slice().len() + self.cubic_slice().len()
    }

    /// Max degree of any term (0 = constant only).
    pub(crate) fn degree(&self) -> usize {
        if !self.cubic_slice().is_empty() {
            3
        } else if !self.quad_slice().is_empty() {
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
            quadratic: Self::wrap_optional(
                self.quad_slice()
                    .iter()
                    .map(|(a, b, c)| (*a, *b, *c * by))
                    .filter(|(_, _, c)| *c != 0.0)
                    .collect(),
            ),
            cubic: Self::wrap_optional(
                self.cubic_slice()
                    .iter()
                    .map(|(a, b, c, d)| (*a, *b, *c, *d * by))
                    .filter(|(_, _, _, d)| *d != 0.0)
                    .collect(),
            ),
        }
    }

    /// Merge two slices into `Option<Box<Vec<T>>>`, returning `None` if both are empty.
    #[inline]
    #[allow(clippy::box_collection)]
    fn merge_slices<T: Copy>(a: &[T], b: &[T]) -> Option<Box<Vec<T>>> {
        if a.is_empty() && b.is_empty() {
            return None;
        }
        let mut v = Vec::with_capacity(a.len() + b.len());
        v.extend_from_slice(a);
        v.extend_from_slice(b);
        Some(Box::new(v))
    }

    /// Add another expression (merges all degree terms + constants).
    pub fn add(&self, other: &Expr) -> Self {
        let mut linear = Vec::with_capacity(self.linear.len() + other.linear.len());
        linear.extend_from_slice(&self.linear);
        linear.extend_from_slice(&other.linear);

        Self {
            constant: self.constant + other.constant,
            linear,
            quadratic: Self::merge_slices(self.quad_slice(), other.quad_slice()),
            cubic: Self::merge_slices(self.cubic_slice(), other.cubic_slice()),
        }
    }

    /// Add another expression in-place (O(other.len()), avoids cloning self).
    pub fn add_assign(&mut self, other: &Expr) {
        self.constant += other.constant;
        self.linear.extend_from_slice(&other.linear);
        let other_quad = other.quad_slice();
        if !other_quad.is_empty() {
            self.quad_mut().extend_from_slice(other_quad);
        }
        let other_cubic = other.cubic_slice();
        if !other_cubic.is_empty() {
            self.cubic_mut().extend_from_slice(other_cubic);
        }
    }

    /// Add another expression in-place, consuming `other` to avoid cloning.
    pub fn add_assign_owned(&mut self, mut other: Expr) {
        self.constant += other.constant;
        self.linear.append(&mut other.linear);
        if let Some(mut other_quad) = other.quadratic {
            if !other_quad.is_empty() {
                self.quad_mut().append(&mut other_quad);
            }
        }
        if let Some(mut other_cubic) = other.cubic {
            if !other_cubic.is_empty() {
                self.cubic_mut().append(&mut other_cubic);
            }
        }
    }

    /// Pre-allocate capacity for the internal Vecs.
    pub fn reserve(&mut self, linear: usize, quadratic: usize, cubic: usize) {
        self.linear.reserve(linear);
        if quadratic > 0 {
            self.quad_mut().reserve(quadratic);
        }
        if cubic > 0 {
            self.cubic_mut().reserve(cubic);
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
        let mut merged: HashMap<VariableId, f64> = HashMap::with_capacity(self.linear.len());
        for (var_id, coeff) in &self.linear {
            if *coeff == 0.0 {
                continue;
            }
            *merged.entry(*var_id).or_insert(0.0) += *coeff;
        }
        merged.into_iter().filter(|(_, c)| *c != 0.0).collect()
    }
}

// ── Operator overloads ──────────────────────────────────────

impl std::ops::Add for Expr {
    type Output = Expr;

    fn add(self, rhs: Expr) -> Self::Output {
        Expr::add(&self, &rhs)
    }
}

impl std::ops::AddAssign for Expr {
    fn add_assign(&mut self, rhs: Expr) {
        Expr::add_assign_owned(self, rhs);
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

// Constraint expressions: linear expression with comparison sense and RHS.

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
    pub(crate) fn new(expr: Expr, sense: ComparisonSense, rhs: f64) -> Self {
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
    pub(crate) fn into_parts(self) -> (Expr, ComparisonSense, f64) {
        (self.expr, self.sense, self.rhs)
    }
}

impl Expr {
    /// Builds `self (sense) rhs`, moving this expression's constant to the RHS.
    pub(crate) fn compare_scalar(&self, rhs: f64, sense: ComparisonSense) -> ConstraintExpr {
        ConstraintExpr::new(self.without_constant(), sense, rhs - self.constant())
    }

    /// Builds `self (sense) other` as a normalized single-sided constraint.
    pub fn compare_expr(&self, other: &Expr, sense: ComparisonSense) -> ConstraintExpr {
        let combined = self.add(&other.scale(-1.0));
        ConstraintExpr::new(combined.without_constant(), sense, -combined.constant())
    }

    /// Convenience wrapper for `self <= rhs`.
    pub(crate) fn le_scalar(&self, rhs: f64) -> ConstraintExpr {
        self.compare_scalar(rhs, ComparisonSense::LessEqual)
    }

    /// Convenience wrapper for `self >= rhs`.
    pub fn ge_scalar(&self, rhs: f64) -> ConstraintExpr {
        self.compare_scalar(rhs, ComparisonSense::GreaterEqual)
    }

    /// Convenience wrapper for `self == rhs`.
    pub(crate) fn eq_scalar(&self, rhs: f64) -> ConstraintExpr {
        self.compare_scalar(rhs, ComparisonSense::Equal)
    }

    /// Convenience wrapper for `self <= rhs_expr`.
    pub fn le_expr(&self, rhs: &Expr) -> ConstraintExpr {
        self.compare_expr(rhs, ComparisonSense::LessEqual)
    }

    /// Convenience wrapper for `self >= rhs_expr`.
    pub(crate) fn ge_expr(&self, rhs: &Expr) -> ConstraintExpr {
        self.compare_expr(rhs, ComparisonSense::GreaterEqual)
    }

    /// Convenience wrapper for `self == rhs_expr`.
    pub fn eq_expr(&self, rhs: &Expr) -> ConstraintExpr {
        self.compare_expr(rhs, ComparisonSense::Equal)
    }
}

// Builder functions for constructing linear expressions.

/// Build an Expr from flexible inputs.
///
/// Accepts either:
/// - `terms`: pre-paired (VariableId, f64) tuples
/// - `variables` + `coefficients`: separate vecs zipped together
///
/// Returns an error if both styles are mixed or if lengths mismatch.
pub(crate) fn linear_terms(
    terms: Option<Vec<(VariableId, f64)>>,
    variables: Option<Vec<VariableId>>,
    coefficients: Option<Vec<f64>>,
) -> Result<Expr, LinearExprError> {
    match (terms, variables, coefficients) {
        (Some(t), None, None) => {
            let filtered: Vec<_> = t.into_iter().filter(|(_, c)| *c != 0.0).collect();
            Ok(Expr::from_linear(filtered))
        }
        (None, Some(vars), Some(coeffs)) => {
            if vars.len() != coeffs.len() {
                return Err(LinearExprError::MismatchedLengths);
            }
            let filtered: Vec<_> = vars
                .into_iter()
                .zip(coeffs)
                .filter(|(_, c)| *c != 0.0)
                .collect();
            Ok(Expr::from_linear(filtered))
        }
        (None, None, None) => Err(LinearExprError::MissingInputs),
        (None, Some(_), None) | (None, None, Some(_)) => Err(LinearExprError::MissingInputs),
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => Err(LinearExprError::MixedInputs),
    }
}

/// Combines multiple expressions into a single expression by concatenating all terms.
///
/// Duplicate variable terms are NOT merged -- use `normalized_terms()` on the result
/// if term consolidation is needed.
pub(crate) fn linear_sum(exprs: Vec<Expr>) -> Expr {
    let total_linear: usize = exprs.iter().map(|e| e.linear_terms().len()).sum();
    let total_quadratic: usize = exprs.iter().map(|e| e.quadratic_terms().len()).sum();
    let total_cubic: usize = exprs.iter().map(|e| e.cubic_terms().len()).sum();

    let mut acc = Expr::new_empty();
    acc.reserve(total_linear, total_quadratic, total_cubic);
    for expr in exprs {
        acc.add_assign_owned(expr);
    }
    acc
}
