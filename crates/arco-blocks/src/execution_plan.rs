//! Core block execution-plan builder.
//!
//! This module is Python-free and owns the dependency-planning logic used by
//! block execution.

use crate::dag::BlockDag;
use crate::error::BlockError;

/// Planned execution schedule for a block graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionPlan {
    pub(crate) execution_levels: Vec<Vec<usize>>,
}

/// Build an execution plan from block names and directed links.
///
/// Each link `(source, target)` means target depends on source.
pub(crate) fn build_execution_plan(
    block_names: &[String],
    links: &[(String, String)],
) -> Result<ExecutionPlan, BlockError> {
    let dag = BlockDag::from_links(block_names, links)?;
    let execution_levels = dag.execution_levels()?;
    Ok(ExecutionPlan { execution_levels })
}

#[cfg(test)]
mod tests {
    use super::build_execution_plan;

    fn blocks(names: &[&str]) -> Vec<String> {
        names.iter().copied().map(str::to_string).collect()
    }

    fn links(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(source, target)| ((*source).to_string(), (*target).to_string()))
            .collect()
    }

    #[test]
    fn build_execution_plan_orders_levels_for_diamond_graph() {
        let plan = build_execution_plan(
            &blocks(&["A", "B", "C", "D"]),
            &links(&[("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")]),
        )
        .expect("plan should build");

        assert_eq!(plan.execution_levels.len(), 3);
        assert_eq!(plan.execution_levels[0], vec![0]);
        assert_eq!(plan.execution_levels[2], vec![3]);
    }

    #[test]
    fn build_execution_plan_fails_for_cycles() {
        let result = build_execution_plan(
            &blocks(&["A", "B", "C"]),
            &links(&[("A", "B"), ("B", "C"), ("C", "A")]),
        );

        assert!(result.is_err());
    }
}
