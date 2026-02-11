//! Builder functions for constructing linear expressions.

use crate::expr::core::Expr;
use crate::expr::error::LinearExprError;
use crate::ids::VariableId;

/// Build an Expr from flexible inputs.
///
/// Accepts either:
/// - `terms`: pre-paired (VariableId, f64) tuples
/// - `variables` + `coefficients`: separate vecs zipped together
///
/// Returns an error if both styles are mixed or if lengths mismatch.
pub fn linear_terms(
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

/// Combines multiple expressions into a single expression by concatenating all their linear terms.
///
/// This function merges the linear terms from each input expression into a new expression.
/// Note that duplicate variable terms are NOT merged - use `normalized_terms()` on the result
/// if term consolidation is needed.
pub fn linear_sum(exprs: Vec<Expr>) -> Expr {
    let mut terms = Vec::new();
    for expr in exprs {
        terms.extend(expr.into_linear_terms());
    }
    Expr::from_linear(terms)
}
