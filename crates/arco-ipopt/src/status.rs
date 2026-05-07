//! IPOPT status to Arco status mapping.

use arco_solver::SolverStatus;

type CoreSolverStatus = SolverStatus;
use ipopt::SolveStatus;

pub(crate) fn ipopt_to_core_status(status: SolveStatus) -> CoreSolverStatus {
    match status {
        SolveStatus::SolveSucceeded | SolveStatus::SolvedToAcceptableLevel => {
            CoreSolverStatus::Optimal
        }
        SolveStatus::InfeasibleProblemDetected | SolveStatus::RestorationFailed => {
            CoreSolverStatus::Infeasible
        }
        SolveStatus::DivergingIterates => CoreSolverStatus::Unbounded,
        SolveStatus::MaximumIterationsExceeded => CoreSolverStatus::IterationLimit,
        SolveStatus::MaximumCpuTimeExceeded => CoreSolverStatus::TimeLimit,
        unknown => {
            tracing::debug!("Unknown IPOPT status: {:?}", unknown);
            CoreSolverStatus::Unknown
        }
    }
}

pub(crate) fn ipopt_to_generic_status(status: SolveStatus) -> SolverStatus {
    ipopt_to_core_status(status)
}

pub(crate) fn ipopt_status_string(status: SolveStatus) -> &'static str {
    match status {
        SolveStatus::SolveSucceeded => "optimal",
        SolveStatus::SolvedToAcceptableLevel => "acceptable",
        SolveStatus::InfeasibleProblemDetected => "infeasible",
        SolveStatus::DivergingIterates => "unbounded",
        SolveStatus::MaximumIterationsExceeded => "iteration_limit",
        SolveStatus::MaximumCpuTimeExceeded => "time_limit",
        SolveStatus::RestorationFailed => "restoration_failed",
        SolveStatus::UserRequestedStop => "user_stopped",
        _ => "unknown",
    }
}

pub(crate) fn ipopt_has_solution(status: SolveStatus) -> bool {
    matches!(
        status,
        SolveStatus::SolveSucceeded
            | SolveStatus::SolvedToAcceptableLevel
            | SolveStatus::MaximumIterationsExceeded
            | SolveStatus::MaximumCpuTimeExceeded
            | SolveStatus::FeasiblePointFound
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipopt_to_core_mapping() {
        assert_eq!(
            ipopt_to_core_status(SolveStatus::SolveSucceeded),
            CoreSolverStatus::Optimal
        );
        assert_eq!(
            ipopt_to_core_status(SolveStatus::SolvedToAcceptableLevel),
            CoreSolverStatus::Optimal
        );
        assert_eq!(
            ipopt_to_core_status(SolveStatus::InfeasibleProblemDetected),
            CoreSolverStatus::Infeasible
        );
        assert_eq!(
            ipopt_to_core_status(SolveStatus::DivergingIterates),
            CoreSolverStatus::Unbounded
        );
        assert_eq!(
            ipopt_to_core_status(SolveStatus::MaximumIterationsExceeded),
            CoreSolverStatus::IterationLimit
        );
        assert_eq!(
            ipopt_to_core_status(SolveStatus::MaximumCpuTimeExceeded),
            CoreSolverStatus::TimeLimit
        );
    }

    #[test]
    fn test_status_helpers() {
        assert!(ipopt_has_solution(SolveStatus::SolveSucceeded));
        assert!(ipopt_has_solution(SolveStatus::SolvedToAcceptableLevel));
        assert!(ipopt_has_solution(SolveStatus::MaximumIterationsExceeded));
        assert!(!ipopt_has_solution(SolveStatus::InfeasibleProblemDetected));
        assert!(!ipopt_has_solution(SolveStatus::DivergingIterates));
        assert_eq!(ipopt_status_string(SolveStatus::SolveSucceeded), "optimal");
        assert_eq!(
            ipopt_status_string(SolveStatus::InfeasibleProblemDetected),
            "infeasible"
        );
    }
}
