//! Shared status conversions for HiGHS integration.

use crate::ffi::HighsStatus;
use arco_core::solver::SolverStatus as CoreSolverStatus;
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

pub(crate) fn core_to_generic_status(status: CoreSolverStatus) -> SolverStatus {
    match status {
        CoreSolverStatus::Optimal => SolverStatus::Optimal,
        CoreSolverStatus::Infeasible => SolverStatus::Infeasible,
        CoreSolverStatus::Unbounded => SolverStatus::Unbounded,
        CoreSolverStatus::TimeLimit => SolverStatus::ReachedTimeLimit,
        CoreSolverStatus::IterationLimit => SolverStatus::ReachedIterationLimit,
        CoreSolverStatus::Unknown => SolverStatus::Unknown,
    }
}

pub(crate) fn highs_to_generic_status(status: HighsStatus) -> SolverStatus {
    core_to_generic_status(highs_to_core_status(status))
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
    fn test_core_to_generic_mapping() {
        assert_eq!(
            core_to_generic_status(CoreSolverStatus::Optimal),
            SolverStatus::Optimal
        );
        assert_eq!(
            core_to_generic_status(CoreSolverStatus::TimeLimit),
            SolverStatus::ReachedTimeLimit
        );
        assert_eq!(
            core_to_generic_status(CoreSolverStatus::IterationLimit),
            SolverStatus::ReachedIterationLimit
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
