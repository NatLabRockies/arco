//! Python-free views over block links.

use crate::BlockLink;
use std::collections::HashSet;

/// Convert links into dependency edges `(source_block, target_block)`.
pub(crate) fn dependency_edges(links: &[BlockLink]) -> Vec<(String, String)> {
    links
        .iter()
        .map(|link| {
            (
                link.source.block_name.clone(),
                link.target.block_name.clone(),
            )
        })
        .collect()
}

/// Collect linked input keys for a target block.
pub(crate) fn linked_input_keys(links: &[BlockLink], block_name: &str) -> HashSet<String> {
    links
        .iter()
        .filter(|link| link.target.block_name == block_name)
        .map(|link| link.target.key.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{dependency_edges, linked_input_keys};
    use crate::{BlockLink, BlockPort, Transform};

    fn make_link(
        source_block: &str,
        source_key: &str,
        target_block: &str,
        target_key: &str,
    ) -> BlockLink {
        BlockLink {
            source: BlockPort::new_output(source_block.to_string(), source_key.to_string()),
            target: BlockPort::new_input(target_block.to_string(), target_key.to_string()),
            transform: Transform::identity_internal(),
        }
    }

    #[test]
    fn dependency_edges_extract_source_target_block_pairs() {
        let links = vec![
            make_link("A", "out", "B", "in"),
            make_link("B", "out", "C", "in"),
        ];

        let edges = dependency_edges(&links);
        assert_eq!(
            edges,
            vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string())
            ]
        );
    }

    #[test]
    fn linked_input_keys_filters_by_target_block_name() {
        let links = vec![
            make_link("A", "out", "B", "in1"),
            make_link("C", "out", "B", "in2"),
            make_link("D", "out", "E", "in3"),
        ];

        let linked = linked_input_keys(&links, "B");
        assert_eq!(linked.len(), 2);
        assert!(linked.contains("in1"));
        assert!(linked.contains("in2"));
        assert!(!linked.contains("in3"));
    }
}
