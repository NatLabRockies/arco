//! IPOPT status to Arco status mapping.
//!
//! Uses a local [`IpoptSolveStatus`] enum instead of the `ipopt::SolveStatus`
//! type so this crate compiles without linking native IPOPT. The local enum
//! mirrors the IPOPT return statuses. When the native IPOPT adapter is built
//! (via the `arco-ops ipopt` feature), its code bridges the real
//! `ipopt::SolveStatus` to this enum.

use arco_solver::SolverStatus;

type CoreSolverStatus = SolverStatus;

/// A local mirror of IPOPT's `SolveStatus` that does not require linking the
/// native `ipopt` crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpoptSolveStatus {
    SolveSucceeded,
    SolvedToAcceptableLevel,
    InfeasibleProblemDetected,
    DivergingIterates,
    MaximumIterationsExceeded,
    MaximumCpuTimeExceeded,
    RestorationFailed,
    UserRequestedStop,
    FeasiblePointFound,
    Unknown,
}

impl IpoptSolveStatus {
    /// Create from the string returned by the native IPOPT status.
    pub(crate) fn from_ipopt_debug(debug: &str) -> Self {
        match debug {
            "SolveSucceeded" => Self::SolveSucceeded,
            "SolvedToAcceptableLevel" => Self::SolvedToAcceptableLevel,
            "InfeasibleProblemDetected" => Self::InfeasibleProblemDetected,
            "DivergingIterates" => Self::DivergingIterates,
            "MaximumIterationsExceeded" => Self::MaximumIterationsExceeded,
            "MaximumCpuTimeExceeded" => Self::MaximumCpuTimeExceeded,
            "RestorationFailed" => Self::RestorationFailed,
            "UserRequestedStop" => Self::UserRequestedStop,
            "FeasiblePointFound" => Self::FeasiblePointFound,
            _ => Self::Unknown,
        }
    }
}

pub(crate) fn ipopt_to_core_status(status: IpoptSolveStatus) -> CoreSolverStatus {
    match status {
        IpoptSolveStatus::SolveSucceeded | IpoptSolveStatus::SolvedToAcceptableLevel => {
            CoreSolverStatus::Optimal
        }
        IpoptSolveStatus::InfeasibleProblemDetected | IpoptSolveStatus::RestorationFailed => {
            CoreSolverStatus::Infeasible
        }
        IpoptSolveStatus::DivergingIterates => CoreSolverStatus::Unbounded,
        IpoptSolveStatus::MaximumIterationsExceeded => CoreSolverStatus::IterationLimit,
        IpoptSolveStatus::MaximumCpuTimeExceeded => CoreSolverStatus::TimeLimit,
        _ => CoreSolverStatus::Unknown,
    }
}

pub(crate) fn ipopt_to_generic_status(status: IpoptSolveStatus) -> SolverStatus {
    ipopt_to_core_status(status)
}

pub(crate) fn ipopt_status_string(status: IpoptSolveStatus) -> &'static str {
    match status {
        IpoptSolveStatus::SolveSucceeded => "optimal",
        IpoptSolveStatus::SolvedToAcceptableLevel => "acceptable",
        IpoptSolveStatus::InfeasibleProblemDetected => "infeasible",
        IpoptSolveStatus::DivergingIterates => "unbounded",
        IpoptSolveStatus::MaximumIterationsExceeded => "iteration_limit",
        IpoptSolveStatus::MaximumCpuTimeExceeded => "time_limit",
        IpoptSolveStatus::RestorationFailed => "restoration_failed",
        IpoptSolveStatus::UserRequestedStop => "user_stopped",
        _ => "unknown",
    }
}

pub(crate) fn ipopt_has_solution(status: IpoptSolveStatus) -> bool {
    matches!(
        status,
        IpoptSolveStatus::SolveSucceeded
            | IpoptSolveStatus::SolvedToAcceptableLevel
            | IpoptSolveStatus::MaximumIterationsExceeded
            | IpoptSolveStatus::MaximumCpuTimeExceeded
            | IpoptSolveStatus::FeasiblePointFound
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipopt_to_core_mapping() {
        assert_eq!(
            ipopt_to_core_status(IpoptSolveStatus::SolveSucceeded),
            CoreSolverStatus::Optimal
        );
        assert_eq!(
            ipopt_to_core_status(IpoptSolveStatus::SolvedToAcceptableLevel),
            CoreSolverStatus::Optimal
        );
        assert_eq!(
            ipopt_to_core_status(IpoptSolveStatus::InfeasibleProblemDetected),
            CoreSolverStatus::Infeasible
        );
        assert_eq!(
            ipopt_to_core_status(IpoptSolveStatus::DivergingIterates),
            CoreSolverStatus::Unbounded
        );
        assert_eq!(
            ipopt_to_core_status(IpoptSolveStatus::MaximumIterationsExceeded),
            CoreSolverStatus::IterationLimit
        );
        assert_eq!(
            ipopt_to_core_status(IpoptSolveStatus::MaximumCpuTimeExceeded),
            CoreSolverStatus::TimeLimit
        );
    }

    #[test]
    fn test_unknown_maps_to_unknown() {
        assert_eq!(
            ipopt_to_core_status(IpoptSolveStatus::Unknown),
            CoreSolverStatus::Unknown
        );
    }

    #[test]
    fn test_status_helpers() {
        assert!(ipopt_has_solution(IpoptSolveStatus::SolveSucceeded));
        assert!(ipopt_has_solution(
            IpoptSolveStatus::SolvedToAcceptableLevel
        ));
        assert!(ipopt_has_solution(
            IpoptSolveStatus::MaximumIterationsExceeded
        ));
        assert!(!ipopt_has_solution(
            IpoptSolveStatus::InfeasibleProblemDetected
        ));
        assert!(!ipopt_has_solution(IpoptSolveStatus::DivergingIterates));
        assert_eq!(
            ipopt_status_string(IpoptSolveStatus::SolveSucceeded),
            "optimal"
        );
        assert_eq!(
            ipopt_status_string(IpoptSolveStatus::InfeasibleProblemDetected),
            "infeasible"
        );
    }

    #[test]
    fn test_from_ipopt_debug() {
        assert!(matches!(
            IpoptSolveStatus::from_ipopt_debug("SolveSucceeded"),
            IpoptSolveStatus::SolveSucceeded
        ));
        assert!(matches!(
            IpoptSolveStatus::from_ipopt_debug("UnknownStatus"),
            IpoptSolveStatus::Unknown
        ));
    }
}
