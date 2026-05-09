//! Python-free helpers for block input satisfaction checks.

use std::collections::HashSet;

/// Return a missing required input key if any required key is absent from both
/// provided and linked input sets.
pub(crate) fn first_missing_required_input(
    required: &HashSet<String>,
    provided: &HashSet<String>,
    linked: &HashSet<String>,
) -> Option<String> {
    let mut keys = required.iter().collect::<Vec<_>>();
    keys.sort();
    keys.into_iter()
        .find(|key| !provided.contains(*key) && !linked.contains(*key))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::first_missing_required_input;
    use std::collections::HashSet;

    fn set(values: &[&str]) -> HashSet<String> {
        values.iter().copied().map(str::to_string).collect()
    }

    #[test]
    fn reports_none_when_all_required_inputs_are_covered() {
        let required = set(&["a", "b", "c"]);
        let provided = set(&["a"]);
        let linked = set(&["b", "c"]);

        assert_eq!(
            first_missing_required_input(&required, &provided, &linked),
            None
        );
    }

    #[test]
    fn reports_first_missing_key_in_sorted_order() {
        let required = set(&["z", "a", "m"]);
        let provided = set(&["z"]);
        let linked = set(&[]);

        assert_eq!(
            first_missing_required_input(&required, &provided, &linked),
            Some("a".to_string())
        );
    }
}
