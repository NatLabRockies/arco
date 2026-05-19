//! Core artifact-retention policy logic for block execution.
//!
//! This module is intentionally Python-free so policy behavior can be tested as
//! plain Rust logic independent from PyO3 wrappers.

/// Policy for retaining block-run artifacts after a composed solve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPolicy {
    /// Retain both model snapshots and solution summaries.
    KeepModel,
    /// Retain compact solution summaries only.
    KeepSummary,
    /// Drop all per-block artifacts after aggregation.
    DropAll,
}

/// Retention decision for block-run artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactRetention {
    pub keep_diagnostics: bool,
    pub keep_model: bool,
    pub keep_solution: bool,
}

impl ArtifactRetention {
    pub const KEEP_BOTH: Self = Self {
        keep_diagnostics: true,
        keep_model: true,
        keep_solution: true,
    };

    pub const KEEP_SOLUTION_ONLY: Self = Self {
        keep_diagnostics: true,
        keep_model: false,
        keep_solution: true,
    };

    pub const KEEP_NONE: Self = Self {
        keep_diagnostics: false,
        keep_model: false,
        keep_solution: false,
    };
}

/// Decide which artifacts to retain for a completed block run.
pub fn retention_for_policy(policy: DropPolicy) -> ArtifactRetention {
    match policy {
        DropPolicy::KeepModel => ArtifactRetention::KEEP_BOTH,
        DropPolicy::KeepSummary => ArtifactRetention::KEEP_SOLUTION_ONLY,
        DropPolicy::DropAll => ArtifactRetention::KEEP_NONE,
    }
}

#[cfg(test)]
mod tests {
    use super::{ArtifactRetention, retention_for_policy};
    use crate::DropPolicy;

    #[test]
    fn keep_model_policy_retains_model_and_solution() {
        assert_eq!(
            retention_for_policy(DropPolicy::KeepModel),
            ArtifactRetention::KEEP_BOTH
        );
    }

    #[test]
    fn keep_summary_policy_retains_solution_only() {
        assert_eq!(
            retention_for_policy(DropPolicy::KeepSummary),
            ArtifactRetention::KEEP_SOLUTION_ONLY
        );
    }

    #[test]
    fn drop_all_policy_retains_nothing() {
        assert_eq!(
            retention_for_policy(DropPolicy::DropAll),
            ArtifactRetention::KEEP_NONE
        );
    }
}
