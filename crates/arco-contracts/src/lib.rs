//! Shared solver-facing contracts for Arco.

/// User-facing solver selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverSelection {
    /// Select a solver family directly (for example `highs`).
    Family(String),
    /// Select a named solver profile.
    Profile(String),
}

impl SolverSelection {
    /// Construct a family selection.
    pub fn family(name: impl Into<String>) -> Self {
        Self::Family(name.into())
    }

    /// Construct a profile selection.
    pub fn profile(name: impl Into<String>) -> Self {
        Self::Profile(name.into())
    }

    /// Return the selected name regardless of variant.
    pub fn name(&self) -> &str {
        match self {
            Self::Family(name) | Self::Profile(name) => name,
        }
    }
}

/// Minimal solve request contract.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SolveRequest {
    /// Optional solver selection for this request.
    pub selection: Option<SolverSelection>,
}

impl SolveRequest {
    /// Build a request without an explicit solver selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a solver selection.
    pub fn with_selection(mut self, selection: SolverSelection) -> Self {
        self.selection = Some(selection);
        self
    }
}

/// Minimal solve status contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    /// Solve has not run.
    NotRun,
    /// Solve completed with an optimal result.
    Optimal,
    /// Solve completed and proved infeasibility.
    Infeasible,
    /// Solve failed for any other reason.
    Failed,
}

/// Minimal solve result envelope contract.
#[derive(Debug, Clone, PartialEq)]
pub struct SolveResult {
    /// Solver-reported status.
    pub status: SolveStatus,
    /// Objective value if available.
    pub objective_value: Option<f64>,
}

impl SolveResult {
    /// Create a result with the provided status.
    pub fn with_status(status: SolveStatus) -> Self {
        Self {
            status,
            objective_value: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::{SolveRequest, SolveResult, SolveStatus, SolverSelection};

    #[test]
    fn selection_name_reflects_variant_payload() {
        let family = SolverSelection::family("highs");
        let profile = SolverSelection::profile("dev-highs");

        assert_eq!(family.name(), "highs");
        assert_eq!(profile.name(), "dev-highs");
    }

    #[test]
    fn solve_request_selection_builder_sets_selection() {
        let request = SolveRequest::new().with_selection(SolverSelection::family("scip"));

        assert_eq!(request.selection, Some(SolverSelection::family("scip")));
    }

    #[test]
    fn solve_result_defaults_objective_value_to_none() {
        let result = SolveResult::with_status(SolveStatus::Optimal);

        assert_eq!(result.status, SolveStatus::Optimal);
        assert_eq!(result.objective_value, None);
    }
}
