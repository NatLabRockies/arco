//! Shared status conversions for HiGHS integration.

use crate::ffi::HighsStatus;
use arco_solver::{SolverStatus, SolverStatusMapping};
use highs::HighsModelStatus;

impl SolverStatusMapping for HighsStatus {
    fn to_solver_status(self) -> SolverStatus {
        match self {
            HighsStatus::Optimal => SolverStatus::Optimal,
            HighsStatus::Infeasible => SolverStatus::Infeasible,
            HighsStatus::Unbounded => SolverStatus::Unbounded,
            HighsStatus::UnboundedOrInfeasible => SolverStatus::Unknown,
            HighsStatus::ReachedTimeLimit => SolverStatus::TimeLimit,
            HighsStatus::ReachedIterationLimit => SolverStatus::IterationLimit,
            HighsStatus::Unknown => SolverStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HighsModelStatusMapping(pub(crate) HighsModelStatus);

impl SolverStatusMapping for HighsModelStatusMapping {
    fn to_solver_status(self) -> SolverStatus {
        match self.0 {
            HighsModelStatus::Optimal => SolverStatus::Optimal,
            HighsModelStatus::Infeasible => SolverStatus::Infeasible,
            HighsModelStatus::Unbounded => SolverStatus::Unbounded,
            HighsModelStatus::ReachedTimeLimit => SolverStatus::TimeLimit,
            HighsModelStatus::ReachedIterationLimit => SolverStatus::IterationLimit,
            _ => SolverStatus::Unknown,
        }
    }
}

pub(crate) fn highs_model_status(status: HighsModelStatus) -> HighsModelStatusMapping {
    HighsModelStatusMapping(status)
}

pub(crate) fn highs_to_core_status(status: HighsStatus) -> SolverStatus {
    status.to_solver_status()
}

pub(crate) fn highs_status_string(status: HighsStatus) -> &'static str {
    match status {
        HighsStatus::Optimal => "optimal",
        HighsStatus::Infeasible => "infeasible",
        HighsStatus::Unbounded => "unbounded",
        HighsStatus::UnboundedOrInfeasible => "unbounded_or_infeasible",
        HighsStatus::ReachedTimeLimit => "time_limit",
        HighsStatus::ReachedIterationLimit => "iteration_limit",
        HighsStatus::Unknown => "unknown",
    }
}

pub(crate) fn highs_has_solution(status: HighsStatus) -> bool {
    status.has_solution()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highs_to_core_mapping() {
        assert_eq!(
            highs_to_core_status(HighsStatus::Optimal),
            SolverStatus::Optimal
        );
        assert_eq!(
            highs_to_core_status(HighsStatus::ReachedTimeLimit),
            SolverStatus::TimeLimit
        );
        assert_eq!(
            highs_to_core_status(HighsStatus::ReachedIterationLimit),
            SolverStatus::IterationLimit
        );
    }

    #[test]
    fn test_highs_model_status_mapping() {
        assert_eq!(
            highs_model_status(HighsModelStatus::Optimal).to_solver_status(),
            SolverStatus::Optimal
        );
        assert_eq!(
            highs_model_status(HighsModelStatus::ReachedTimeLimit).to_solver_status(),
            SolverStatus::TimeLimit
        );
        assert!(highs_model_status(HighsModelStatus::Optimal).has_solution());
        assert!(!highs_model_status(HighsModelStatus::Infeasible).has_solution());
    }

    #[test]
    fn test_status_helpers() {
        assert!(highs_has_solution(HighsStatus::Optimal));
        assert!(highs_has_solution(HighsStatus::ReachedTimeLimit));
        assert!(!highs_has_solution(HighsStatus::Infeasible));
        assert!(!highs_has_solution(HighsStatus::UnboundedOrInfeasible));
        assert_eq!(
            highs_to_core_status(HighsStatus::UnboundedOrInfeasible),
            SolverStatus::Unknown
        );
        assert_eq!(
            highs_status_string(HighsStatus::UnboundedOrInfeasible),
            "unbounded_or_infeasible"
        );
        assert_eq!(highs_status_string(HighsStatus::Unknown), "unknown");
    }
}
