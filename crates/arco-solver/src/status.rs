//! Solver status types.

/// Common status values that solvers may return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolverStatus {
    /// Optimal solution found.
    Optimal,
    /// Problem is infeasible.
    Infeasible,
    /// Problem is unbounded.
    Unbounded,
    /// Solver reached time limit (may have feasible solution).
    ReachedTimeLimit,
    /// Solver reached iteration limit (may have feasible solution).
    ReachedIterationLimit,
    /// Status is unknown or solver did not complete.
    Unknown,
}

impl SolverStatus {
    /// Check if the status indicates an optimal solution.
    pub fn is_optimal(self) -> bool {
        matches!(self, SolverStatus::Optimal)
    }

    /// Check if the status indicates a feasible solution (optimal or limit-reached with solution).
    pub fn is_feasible(self) -> bool {
        matches!(
            self,
            SolverStatus::Optimal
                | SolverStatus::ReachedTimeLimit
                | SolverStatus::ReachedIterationLimit
        )
    }

    /// Check if the status indicates infeasibility.
    pub fn is_infeasible(self) -> bool {
        matches!(self, SolverStatus::Infeasible)
    }

    /// Check if the status indicates unboundedness.
    pub fn is_unbounded(self) -> bool {
        matches!(self, SolverStatus::Unbounded)
    }

    /// Get a human-readable string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            SolverStatus::Optimal => "optimal",
            SolverStatus::Infeasible => "infeasible",
            SolverStatus::Unbounded => "unbounded",
            SolverStatus::ReachedTimeLimit => "time_limit",
            SolverStatus::ReachedIterationLimit => "iteration_limit",
            SolverStatus::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for SolverStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_is_optimal() {
        assert!(SolverStatus::Optimal.is_optimal());
        assert!(!SolverStatus::Infeasible.is_optimal());
        assert!(!SolverStatus::Unbounded.is_optimal());
        assert!(!SolverStatus::ReachedTimeLimit.is_optimal());
        assert!(!SolverStatus::Unknown.is_optimal());
    }

    #[test]
    fn test_status_is_feasible() {
        assert!(SolverStatus::Optimal.is_feasible());
        assert!(SolverStatus::ReachedTimeLimit.is_feasible());
        assert!(SolverStatus::ReachedIterationLimit.is_feasible());
        assert!(!SolverStatus::Infeasible.is_feasible());
        assert!(!SolverStatus::Unbounded.is_feasible());
        assert!(!SolverStatus::Unknown.is_feasible());
    }

    #[test]
    fn test_status_is_infeasible() {
        assert!(SolverStatus::Infeasible.is_infeasible());
        assert!(!SolverStatus::Optimal.is_infeasible());
    }

    #[test]
    fn test_status_is_unbounded() {
        assert!(SolverStatus::Unbounded.is_unbounded());
        assert!(!SolverStatus::Optimal.is_unbounded());
    }

    #[test]
    fn test_status_as_str() {
        assert_eq!(SolverStatus::Optimal.as_str(), "optimal");
        assert_eq!(SolverStatus::Infeasible.as_str(), "infeasible");
        assert_eq!(SolverStatus::Unbounded.as_str(), "unbounded");
        assert_eq!(SolverStatus::ReachedTimeLimit.as_str(), "time_limit");
        assert_eq!(
            SolverStatus::ReachedIterationLimit.as_str(),
            "iteration_limit"
        );
        assert_eq!(SolverStatus::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", SolverStatus::Optimal), "optimal");
        assert_eq!(format!("{}", SolverStatus::Infeasible), "infeasible");
    }
}
