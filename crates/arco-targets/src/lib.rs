//! Solver-facing compile target seam for Arco.

/// Minimal lowered target summary passed to solver-side orchestration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveTarget {
    /// Human-readable target identifier.
    pub name: String,
    /// Number of decision variables.
    pub variable_count: usize,
    /// Number of constraints.
    pub constraint_count: usize,
}

impl SolveTarget {
    /// Build a new target summary.
    pub fn new(name: impl Into<String>, variable_count: usize, constraint_count: usize) -> Self {
        Self {
            name: name.into(),
            variable_count,
            constraint_count,
        }
    }

    /// Whether the target has any decision variables.
    pub fn has_variables(&self) -> bool {
        self.variable_count > 0
    }
}

#[cfg(test)]
mod tests {
    use super::SolveTarget;

    #[test]
    fn solve_target_constructor_sets_fields() {
        let target = SolveTarget::new("demo", 3, 2);

        assert_eq!(target.name, "demo");
        assert_eq!(target.variable_count, 3);
        assert_eq!(target.constraint_count, 2);
    }

    #[test]
    fn has_variables_reflects_variable_count() {
        assert!(SolveTarget::new("a", 1, 0).has_variables());
        assert!(!SolveTarget::new("b", 0, 0).has_variables());
    }
}
