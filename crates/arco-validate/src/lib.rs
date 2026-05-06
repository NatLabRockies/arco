//! Canonical model validation seam for Arco.

use std::collections::BTreeMap;

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

/// Run canonical validation on a lowered solve target.
pub fn validate_solve_target(has_variables: bool) -> ValidationReport {
    let mut report = ValidationReport::new();
    if !has_variables {
        report.push(ValidationIssue::error(
            "TARGET_EMPTY_VARIABLE_SET",
            "target has no decision variables",
        ));
    }

    report
}

/// Returns true when a lower/upper pair can be used as canonical model bounds.
pub fn bounds_are_valid(lower: f64, upper: f64) -> bool {
    !lower.is_nan() && !upper.is_nan() && lower <= upper
}

/// Returns true when a linear matrix or objective coefficient is finite.
pub fn coefficient_is_valid(coefficient: f64) -> bool {
    coefficient.is_finite()
}

/// Returns true when a slack penalty is finite and non-negative.
pub fn slack_penalty_is_valid(penalty: f64) -> bool {
    penalty.is_finite() && penalty >= 0.0
}

/// Render duplicate tuple rows and their provenance in deterministic diagnostic form.
pub fn duplicate_tuple_row_messages<T>(
    tuple_occurrences: &BTreeMap<Vec<String>, Vec<T>>,
) -> Vec<String>
where
    T: AsRef<str>,
{
    tuple_occurrences
        .iter()
        .filter(|(_, provenance)| provenance.len() > 1)
        .map(|(tuple, provenance)| {
            let provenance = provenance
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<_>>()
                .join("; ");
            format!("`{}` -> {provenance}", tuple.join(","))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ValidationIssue, ValidationReport, ValidationSeverity, bounds_are_valid,
        coefficient_is_valid, duplicate_tuple_row_messages, slack_penalty_is_valid,
        validate_solve_target,
    };
    use std::collections::BTreeMap;

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

    #[test]
    fn validate_solve_target_rejects_targets_without_variables() {
        let report = validate_solve_target(false);

        assert!(!report.is_valid());
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].code, "TARGET_EMPTY_VARIABLE_SET");
        assert_eq!(report.issues[0].message, "target has no decision variables");
    }

    #[test]
    fn validate_solve_target_accepts_targets_with_variables() {
        let report = validate_solve_target(true);

        assert!(report.is_valid());
        assert!(report.issues.is_empty());
    }

    #[test]
    fn canonical_model_scalar_helpers_classify_valid_inputs() {
        assert!(bounds_are_valid(f64::NEG_INFINITY, f64::INFINITY));
        assert!(!bounds_are_valid(5.0, 1.0));
        assert!(!bounds_are_valid(f64::NAN, 1.0));
        assert!(coefficient_is_valid(-3.5));
        assert!(!coefficient_is_valid(f64::INFINITY));
        assert!(slack_penalty_is_valid(0.0));
        assert!(!slack_penalty_is_valid(-1.0));
        assert!(!slack_penalty_is_valid(f64::NAN));
    }

    #[test]
    fn duplicate_tuple_row_messages_preserve_provenance() {
        let mut occurrences = BTreeMap::new();
        occurrences.insert(
            vec!["north".to_string(), "solar".to_string()],
            vec!["data `assets.csv` row 1", "data `assets.csv` row 3"],
        );
        occurrences.insert(vec!["south".to_string(), "wind".to_string()], vec!["row 2"]);

        assert_eq!(
            duplicate_tuple_row_messages(&occurrences),
            vec!["`north,solar` -> data `assets.csv` row 1; data `assets.csv` row 3"]
        );
    }
}
