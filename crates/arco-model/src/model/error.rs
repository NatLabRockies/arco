//! Model error types.

use arco_expr::ids::{ConstraintId, VariableId};

/// Errors that can occur during model operations
#[derive(Debug, Clone, PartialEq)]
pub enum ModelError {
    /// Model has no variables
    EmptyModel,
    /// Invalid variable ID
    InvalidVariableId(VariableId),
    /// Invalid variable bounds
    InvalidVariableBounds { lower: f64, upper: f64 },
    /// Invalid constraint ID
    InvalidConstraintId(ConstraintId),
    /// Invalid constraint bounds
    InvalidConstraintBounds { lower: f64, upper: f64 },
    /// No objective set
    NoObjective,
    /// Objective already set
    MultipleObjectives,
    /// Invalid slack penalty
    InvalidSlackPenalty { penalty: f64 },
    /// Invalid CSC ingest data
    InvalidCscData { reason: String },
    /// Non-finite matrix/objective coefficient
    InvalidCoefficient { coefficient: f64 },
}

impl ModelError {
    /// Returns a semantic error code for programmatic handling.
    pub fn code(&self) -> &'static str {
        match self {
            ModelError::EmptyModel => "MODEL_EMPTY",
            ModelError::InvalidVariableId(_) => "VARIABLE_INVALID_ID",
            ModelError::InvalidVariableBounds { .. } => "VARIABLE_INVALID_BOUNDS",
            ModelError::InvalidConstraintId(_) => "CONSTRAINT_INVALID_ID",
            ModelError::InvalidConstraintBounds { .. } => "CONSTRAINT_INVALID_BOUNDS",
            ModelError::NoObjective => "OBJECTIVE_MISSING",
            ModelError::MultipleObjectives => "OBJECTIVE_ALREADY_SET",
            ModelError::InvalidSlackPenalty { .. } => "SLACK_INVALID_PENALTY",
            ModelError::InvalidCscData { .. } => "CSC_INVALID_DATA",
            ModelError::InvalidCoefficient { .. } => "COEFFICIENT_INVALID",
        }
    }
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::EmptyModel => write!(f, "[{}] Model has no variables", self.code()),
            ModelError::InvalidVariableId(id) => write!(
                f,
                "[{}] Variable ID {} does not exist",
                self.code(),
                id.inner()
            ),
            ModelError::InvalidVariableBounds { lower, upper } => write!(
                f,
                "[{}] Variable bounds invalid: lower ({}) > upper ({})",
                self.code(),
                lower,
                upper
            ),
            ModelError::InvalidConstraintId(id) => write!(
                f,
                "[{}] Constraint ID {} does not exist",
                self.code(),
                id.inner()
            ),
            ModelError::InvalidConstraintBounds { lower, upper } => write!(
                f,
                "[{}] Constraint bounds invalid: lower ({}) > upper ({})",
                self.code(),
                lower,
                upper
            ),
            ModelError::NoObjective => {
                write!(f, "[{}] Model has no objective defined", self.code())
            }
            ModelError::MultipleObjectives => write!(
                f,
                "[{}] Model already has an objective; use set_objective to replace",
                self.code()
            ),
            ModelError::InvalidSlackPenalty { penalty } => write!(
                f,
                "[{}] Slack penalty must be finite and non-negative (got {})",
                self.code(),
                penalty
            ),
            ModelError::InvalidCscData { reason } => {
                write!(f, "[{}] CSC ingest invalid: {}", self.code(), reason)
            }
            ModelError::InvalidCoefficient { coefficient } => write!(
                f,
                "[{}] Coefficient must be finite (got {})",
                self.code(),
                coefficient
            ),
        }
    }
}

impl std::error::Error for ModelError {}
