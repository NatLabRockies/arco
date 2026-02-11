//! Expression construction errors.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinearExprError {
    MixedInputs,
    MissingInputs,
    MismatchedLengths,
}

impl LinearExprError {
    /// Returns a semantic error code for programmatic handling.
    pub fn code(&self) -> &'static str {
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
mod tests {
    use super::LinearExprError;

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
