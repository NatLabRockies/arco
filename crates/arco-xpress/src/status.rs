//! Xpress status to Arco status mapping.
#![allow(dead_code)]

use arco_core::solver::SolverStatus as CoreSolverStatus;
use arco_solver::SolverStatus;

// LP status constants (from Xpress C API)
const XPRS_LP_UNSTARTED: i32 = 0;
const XPRS_LP_OPTIMAL: i32 = 1;
const XPRS_LP_INFEAS: i32 = 2;
const XPRS_LP_CUTOFF: i32 = 3;
const XPRS_LP_UNFINISHED: i32 = 4;
const XPRS_LP_UNBOUNDED: i32 = 5;
const XPRS_LP_CUTOFF_IN_DUAL: i32 = 6;
const XPRS_LP_UNSOLVED: i32 = 7;
const XPRS_LP_NONCONVEX: i32 = 8;

// MIP status constants
const XPRS_MIP_NOT_LOADED: i32 = 0;
const XPRS_MIP_LP_NOT_OPTIMAL: i32 = 1;
const XPRS_MIP_LP_OPTIMAL: i32 = 2;
const XPRS_MIP_NO_SOL_FOUND: i32 = 3;
const XPRS_MIP_SOLUTION: i32 = 4;
const XPRS_MIP_INFEAS: i32 = 5;
const XPRS_MIP_OPTIMAL: i32 = 6;
const XPRS_MIP_UNBOUNDED: i32 = 7;

pub(crate) fn lp_status_to_core(status: i32) -> CoreSolverStatus {
    match status {
        XPRS_LP_OPTIMAL => CoreSolverStatus::Optimal,
        XPRS_LP_INFEAS => CoreSolverStatus::Infeasible,
        XPRS_LP_UNBOUNDED => CoreSolverStatus::Unbounded,
        XPRS_LP_UNFINISHED => CoreSolverStatus::TimeLimit,
        _ => CoreSolverStatus::Unknown,
    }
}

pub(crate) fn mip_status_to_core(status: i32) -> CoreSolverStatus {
    match status {
        XPRS_MIP_OPTIMAL => CoreSolverStatus::Optimal,
        XPRS_MIP_SOLUTION => CoreSolverStatus::TimeLimit,
        XPRS_MIP_INFEAS => CoreSolverStatus::Infeasible,
        XPRS_MIP_UNBOUNDED => CoreSolverStatus::Unbounded,
        _ => CoreSolverStatus::Unknown,
    }
}

pub(crate) fn core_to_generic(status: CoreSolverStatus) -> SolverStatus {
    status.into()
}

pub(crate) fn lp_has_solution(status: i32) -> bool {
    matches!(status, XPRS_LP_OPTIMAL | XPRS_LP_UNFINISHED)
}

pub(crate) fn mip_has_solution(status: i32) -> bool {
    matches!(status, XPRS_MIP_OPTIMAL | XPRS_MIP_SOLUTION)
}

pub(crate) fn lp_status_string(status: i32) -> &'static str {
    match status {
        XPRS_LP_UNSTARTED => "unstarted",
        XPRS_LP_OPTIMAL => "optimal",
        XPRS_LP_INFEAS => "infeasible",
        XPRS_LP_CUTOFF => "cutoff",
        XPRS_LP_UNFINISHED => "unfinished",
        XPRS_LP_UNBOUNDED => "unbounded",
        XPRS_LP_CUTOFF_IN_DUAL => "cutoff_in_dual",
        XPRS_LP_UNSOLVED => "unsolved",
        XPRS_LP_NONCONVEX => "nonconvex",
        _ => "unknown",
    }
}

pub(crate) fn mip_status_string(status: i32) -> &'static str {
    match status {
        XPRS_MIP_NOT_LOADED => "not_loaded",
        XPRS_MIP_LP_NOT_OPTIMAL => "lp_not_optimal",
        XPRS_MIP_LP_OPTIMAL => "lp_optimal",
        XPRS_MIP_NO_SOL_FOUND => "no_solution_found",
        XPRS_MIP_SOLUTION => "solution_found",
        XPRS_MIP_INFEAS => "infeasible",
        XPRS_MIP_OPTIMAL => "optimal",
        XPRS_MIP_UNBOUNDED => "unbounded",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lp_optimal_maps_to_core_optimal() {
        assert_eq!(
            lp_status_to_core(XPRS_LP_OPTIMAL),
            CoreSolverStatus::Optimal
        );
    }

    #[test]
    fn lp_infeas_maps_to_core_infeasible() {
        assert_eq!(
            lp_status_to_core(XPRS_LP_INFEAS),
            CoreSolverStatus::Infeasible
        );
    }

    #[test]
    fn lp_unbounded_maps_to_core_unbounded() {
        assert_eq!(
            lp_status_to_core(XPRS_LP_UNBOUNDED),
            CoreSolverStatus::Unbounded
        );
    }

    #[test]
    fn lp_unfinished_maps_to_time_limit() {
        assert_eq!(
            lp_status_to_core(XPRS_LP_UNFINISHED),
            CoreSolverStatus::TimeLimit
        );
    }

    #[test]
    fn lp_unknown_status_maps_to_unknown() {
        assert_eq!(lp_status_to_core(999), CoreSolverStatus::Unknown);
    }

    #[test]
    fn mip_optimal_maps_to_core_optimal() {
        assert_eq!(
            mip_status_to_core(XPRS_MIP_OPTIMAL),
            CoreSolverStatus::Optimal
        );
    }

    #[test]
    fn mip_solution_maps_to_core_time_limit() {
        assert_eq!(
            mip_status_to_core(XPRS_MIP_SOLUTION),
            CoreSolverStatus::TimeLimit
        );
    }

    #[test]
    fn mip_infeas_maps_to_core_infeasible() {
        assert_eq!(
            mip_status_to_core(XPRS_MIP_INFEAS),
            CoreSolverStatus::Infeasible
        );
    }

    #[test]
    fn mip_unbounded_maps_to_core_unbounded() {
        assert_eq!(
            mip_status_to_core(XPRS_MIP_UNBOUNDED),
            CoreSolverStatus::Unbounded
        );
    }

    #[test]
    fn mip_no_sol_maps_to_unknown() {
        assert_eq!(
            mip_status_to_core(XPRS_MIP_NO_SOL_FOUND),
            CoreSolverStatus::Unknown
        );
    }

    #[test]
    fn lp_has_solution_accepts_optimal_and_unfinished() {
        assert!(lp_has_solution(XPRS_LP_OPTIMAL));
        assert!(lp_has_solution(XPRS_LP_UNFINISHED));
        assert!(!lp_has_solution(XPRS_LP_INFEAS));
        assert!(!lp_has_solution(XPRS_LP_UNBOUNDED));
        assert!(!lp_has_solution(XPRS_LP_UNSTARTED));
    }

    #[test]
    fn mip_has_solution_accepts_optimal_and_solution() {
        assert!(mip_has_solution(XPRS_MIP_OPTIMAL));
        assert!(mip_has_solution(XPRS_MIP_SOLUTION));
        assert!(!mip_has_solution(XPRS_MIP_INFEAS));
        assert!(!mip_has_solution(XPRS_MIP_NO_SOL_FOUND));
    }

    #[test]
    fn status_strings_return_expected_labels() {
        assert_eq!(lp_status_string(XPRS_LP_OPTIMAL), "optimal");
        assert_eq!(lp_status_string(XPRS_LP_INFEAS), "infeasible");
        assert_eq!(lp_status_string(XPRS_LP_UNBOUNDED), "unbounded");
        assert_eq!(mip_status_string(XPRS_MIP_OPTIMAL), "optimal");
        assert_eq!(mip_status_string(XPRS_MIP_INFEAS), "infeasible");
        assert_eq!(mip_status_string(999), "unknown");
    }
}
