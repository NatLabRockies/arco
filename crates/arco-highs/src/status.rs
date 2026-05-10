//! Shared status conversions for HiGHS integration.

use crate::ffi::HighsStatus;
use arco_solver::SolverStatus as CoreSolverStatus;
use arco_solver::SolverStatus;

pub(crate) fn highs_to_core_status(status: HighsStatus) -> CoreSolverStatus {
    match status {
        HighsStatus::Optimal => CoreSolverStatus::Optimal,
        HighsStatus::Infeasible => CoreSolverStatus::Infeasible,
        HighsStatus::Unbounded => CoreSolverStatus::Unbounded,
        HighsStatus::UnboundedOrInfeasible => CoreSolverStatus::Unknown,
        HighsStatus::ReachedTimeLimit => CoreSolverStatus::TimeLimit,
        HighsStatus::ReachedIterationLimit => CoreSolverStatus::IterationLimit,
        HighsStatus::Unknown => CoreSolverStatus::Unknown,
    }
}

pub(crate) fn highs_to_generic_status(status: HighsStatus) -> SolverStatus {
    highs_to_core_status(status)
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
    matches!(
        status,
        HighsStatus::Optimal | HighsStatus::ReachedTimeLimit | HighsStatus::ReachedIterationLimit
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highs_to_core_mapping() {
        assert_eq!(
            highs_to_core_status(HighsStatus::Optimal),
            CoreSolverStatus::Optimal
        );
        assert_eq!(
            highs_to_core_status(HighsStatus::ReachedTimeLimit),
            CoreSolverStatus::TimeLimit
        );
        assert_eq!(
            highs_to_core_status(HighsStatus::ReachedIterationLimit),
            CoreSolverStatus::IterationLimit
        );
    }

    #[test]
    fn test_solver_status_is_same_as_core() {
        // Since SolverStatus is now re-exported from arco-model, these are the same type
        assert_eq!(SolverStatus::Optimal, CoreSolverStatus::Optimal);
        assert_eq!(SolverStatus::TimeLimit, CoreSolverStatus::TimeLimit);
        assert_eq!(
            SolverStatus::IterationLimit,
            CoreSolverStatus::IterationLimit
        );
    }

    #[test]
    fn test_status_helpers() {
        assert!(highs_has_solution(HighsStatus::Optimal));
        assert!(highs_has_solution(HighsStatus::ReachedTimeLimit));
        assert!(!highs_has_solution(HighsStatus::Infeasible));
        assert!(!highs_has_solution(HighsStatus::UnboundedOrInfeasible));
        assert_eq!(
            highs_to_core_status(HighsStatus::UnboundedOrInfeasible),
            CoreSolverStatus::Unknown
        );
        assert_eq!(
            highs_status_string(HighsStatus::UnboundedOrInfeasible),
            "unbounded_or_infeasible"
        );
        assert_eq!(highs_status_string(HighsStatus::Unknown), "unknown");
    }
}
