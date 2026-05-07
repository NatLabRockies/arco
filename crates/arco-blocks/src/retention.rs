//! Core artifact-retention policy logic for block execution.
//!
//! This module is intentionally Python-free so policy behavior can be tested as
//! plain Rust logic independent from PyO3 wrappers.

use crate::DropPolicy;

/// Retention decision for block-run artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArtifactRetention {
    pub(crate) keep_model: bool,
    pub(crate) keep_solution: bool,
}

impl ArtifactRetention {
    pub(crate) const KEEP_BOTH: Self = Self {
        keep_model: true,
        keep_solution: true,
    };

    pub(crate) const KEEP_SOLUTION_ONLY: Self = Self {
        keep_model: false,
        keep_solution: true,
    };

    pub(crate) const KEEP_NONE: Self = Self {
        keep_model: false,
        keep_solution: false,
    };
}

/// Decide which artifacts to retain for a completed block run.
pub(crate) fn retention_for_policy(policy: DropPolicy) -> ArtifactRetention {
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
