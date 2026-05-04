//! Canonical model validation seam for Arco.

/// Severity level for a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Validation error that should block execution.
    Error,
    /// Validation warning that should be surfaced but may not block execution.
    Warning,
}

/// A single validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// Stable issue code for programmatic handling.
    pub code: &'static str,
    /// Human-readable issue message.
    pub message: String,
    /// Severity classification.
    pub severity: ValidationSeverity,
}

impl ValidationIssue {
    /// Construct an error issue.
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// Construct a warning issue.
    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            severity: ValidationSeverity::Warning,
        }
    }
}

/// Validation report containing all discovered issues.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationReport {
    /// Validation issues found during a validation pass.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Create an empty validation report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one issue to the report.
    pub fn push(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Returns true if the report contains no errors.
    pub fn is_valid(&self) -> bool {
        self.issues
            .iter()
            .all(|issue| issue.severity != ValidationSeverity::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::{ValidationIssue, ValidationReport, ValidationSeverity};

    #[test]
    fn issue_constructors_set_expected_severity() {
        assert_eq!(
            ValidationIssue::error("E001", "missing objective").severity,
            ValidationSeverity::Error
        );
        assert_eq!(
            ValidationIssue::warning("W001", "precision loss").severity,
            ValidationSeverity::Warning
        );
    }

    #[test]
    fn report_is_valid_when_only_warnings_exist() {
        let mut report = ValidationReport::new();
        report.push(ValidationIssue::warning("W001", "warning"));

        assert!(report.is_valid());
    }

    #[test]
    fn report_is_invalid_when_error_exists() {
        let mut report = ValidationReport::new();
        report.push(ValidationIssue::error("E001", "error"));

        assert!(!report.is_valid());
    }
}
