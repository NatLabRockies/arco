//! Language-neutral block composition core.

mod dag;
mod error;
mod execution_plan;
mod retention;

pub use crate::error::BlockError;
pub use crate::retention::{ArtifactRetention, DropPolicy, retention_for_policy};

/// Directed dependency edge: `(source, target)` means `target` depends on `source`.
pub type DependencyEdge = (String, String);

/// Build topological execution levels for a block graph.
///
/// Returned levels contain indices into `block_names`.
pub fn build_execution_levels(
    block_names: &[String],
    links: &[DependencyEdge],
) -> Result<Vec<Vec<usize>>, BlockError> {
    let plan = execution_plan::build_execution_plan(block_names, links)?;
    Ok(plan.execution_levels)
}
