//! Operations facade seam for Arco interaction surfaces.

use arco_contracts::{SolveRequest, SolverSelection};
use arco_targets::SolveTarget;
use arco_validate::{ValidationIssue, ValidationReport};

/// Thin operations facade used by interaction surfaces.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ArcoOps;

impl ArcoOps {
    /// Create a new operations facade.
    pub fn new() -> Self {
        Self
    }

    /// Build a minimal solve request from an optional solver selection.
    pub fn build_solve_request(selection: Option<SolverSelection>) -> SolveRequest {
        selection.map_or_else(SolveRequest::new, |value| {
            SolveRequest::new().with_selection(value)
        })
    }

    /// Run basic seam-level validation on a lowered solve target.
    pub fn validate_target(target: &SolveTarget) -> ValidationReport {
        let mut report = ValidationReport::new();
        if !target.has_variables() {
            report.push(ValidationIssue::error(
                "TARGET_EMPTY_VARIABLE_SET",
                "target has no decision variables",
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::ArcoOps;
    use arco_contracts::SolverSelection;
    use arco_targets::SolveTarget;

    #[test]
    fn build_solve_request_preserves_selection() {
        let request = ArcoOps::build_solve_request(Some(SolverSelection::profile("local-highs")));

        assert_eq!(
            request.selection,
            Some(SolverSelection::profile("local-highs"))
        );
    }

    #[test]
    fn validate_target_rejects_targets_without_variables() {
        let report = ArcoOps::validate_target(&SolveTarget::new("empty", 0, 0));

        assert!(!report.is_valid());
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].code, "TARGET_EMPTY_VARIABLE_SET");
    }

    #[test]
    fn validate_target_accepts_targets_with_variables() {
        let report = ArcoOps::validate_target(&SolveTarget::new("ok", 2, 1));

        assert!(report.is_valid());
        assert!(report.issues.is_empty());
    }
}
