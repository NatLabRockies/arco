//! Block dependency graph (DAG) for execution ordering.

use std::collections::HashMap;

use crate::error::BlockError;

/// Dependency graph of blocks for determining execution order.
#[derive(Debug, Clone)]
pub struct BlockDag {
    /// For each block index, which blocks it depends on.
    dependencies: Vec<Vec<usize>>,
}

impl BlockDag {
    /// Build a DAG from block names and links.
    ///
    /// Links are `(source_block_name, target_block_name)` pairs,
    /// meaning `target` depends on `source`.
    pub fn from_links(
        block_names: &[String],
        links: &[(String, String)],
    ) -> Result<Self, BlockError> {
        let num_blocks = block_names.len();

        let mut name_to_index: HashMap<String, usize> = HashMap::new();
        for (idx, name) in block_names.iter().enumerate() {
            if name_to_index.insert(name.clone(), idx).is_some() {
                return Err(BlockError::DuplicateBlock(name.clone()));
            }
        }

        let mut dependencies: Vec<Vec<usize>> = vec![Vec::new(); num_blocks];

        for (source_name, target_name) in links {
            let source_idx = name_to_index
                .get(source_name)
                .ok_or_else(|| BlockError::BlockNotFound(source_name.clone()))?;
            let target_idx = name_to_index
                .get(target_name)
                .ok_or_else(|| BlockError::BlockNotFound(target_name.clone()))?;

            dependencies[*target_idx].push(*source_idx);
        }

        Ok(Self { dependencies })
    }

    /// Return execution levels where blocks at each level can run in parallel.
    ///
    /// Validates the DAG is acyclic before computing levels.
    pub fn execution_levels(&self) -> Result<Vec<Vec<usize>>, BlockError> {
        let num_blocks = self.dependencies.len();
        let mut indegree = self
            .dependencies
            .iter()
            .map(std::vec::Vec::len)
            .collect::<Vec<_>>();
        let mut dependents = vec![Vec::new(); num_blocks];
        for (block, deps) in self.dependencies.iter().enumerate() {
            for &dep in deps {
                dependents[dep].push(block);
            }
        }

        let mut levels: Vec<Vec<usize>> = Vec::new();
        let mut current = (0..num_blocks)
            .filter(|&block| indegree[block] == 0)
            .collect::<Vec<_>>();
        let mut visited = 0usize;

        while !current.is_empty() {
            visited += current.len();
            let mut next = Vec::new();
            for &block in &current {
                for &dependent in &dependents[block] {
                    indegree[dependent] -= 1;
                    if indegree[dependent] == 0 {
                        next.push(dependent);
                    }
                }
            }
            levels.push(current);
            current = next;
        }

        if visited != num_blocks {
            return Err(BlockError::CycleDetected(
                "Block DAG has a cycle".to_string(),
            ));
        }

        Ok(levels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(names: &[&str]) -> Vec<String> {
        names.iter().copied().map(str::to_string).collect()
    }

    fn links(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
            .collect()
    }

    #[test]
    fn test_linear_chain() {
        let dag =
            BlockDag::from_links(&blocks(&["A", "B", "C"]), &links(&[("A", "B"), ("B", "C")]))
                .unwrap();

        let levels = dag.execution_levels().unwrap();
        assert_eq!(levels, vec![vec![0], vec![1], vec![2]]);
    }

    #[test]
    fn test_independent_blocks() {
        let dag = BlockDag::from_links(&blocks(&["A", "B", "C"]), &[]).unwrap();

        let levels = dag.execution_levels().unwrap();
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 3);
    }

    #[test]
    fn test_diamond() {
        let dag = BlockDag::from_links(
            &blocks(&["A", "B", "C", "D"]),
            &links(&[("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")]),
        )
        .unwrap();

        let levels = dag.execution_levels().unwrap();
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec![0]); // A
        assert_eq!(levels[1].len(), 2); // B, C parallel
        assert_eq!(levels[2], vec![3]); // D
    }

    #[test]
    fn test_cycle_detection() {
        let dag = BlockDag::from_links(
            &blocks(&["A", "B", "C"]),
            &links(&[("A", "B"), ("B", "C"), ("C", "A")]),
        )
        .unwrap();

        assert!(dag.execution_levels().is_err());
    }

    #[test]
    fn test_unknown_block() {
        let result = BlockDag::from_links(&blocks(&["A", "B"]), &links(&[("A", "Unknown")]));
        assert!(matches!(result, Err(BlockError::BlockNotFound(_))));
    }

    #[test]
    fn test_duplicate_block() {
        let result = BlockDag::from_links(&blocks(&["A", "A"]), &[]);
        assert!(matches!(result, Err(BlockError::DuplicateBlock(_))));
    }
}
