use crate::ids::{ConstraintId, VariableId};

/// Which bound(s) a slack variable relaxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlackBound {
    Lower,
    Upper,
    Both,
}

impl SlackBound {
    pub fn as_str(self) -> &'static str {
        match self {
            SlackBound::Lower => "lower",
            SlackBound::Upper => "upper",
            SlackBound::Both => "both",
        }
    }

    pub(crate) fn has_lower(self) -> bool {
        matches!(self, SlackBound::Lower | SlackBound::Both)
    }

    pub(crate) fn has_upper(self) -> bool {
        matches!(self, SlackBound::Upper | SlackBound::Both)
    }
}

/// Slack variable IDs grouped by bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SlackVariables {
    pub lower: Option<VariableId>,
    pub upper: Option<VariableId>,
}

impl SlackVariables {
    pub(crate) fn new(lower: Option<VariableId>, upper: Option<VariableId>) -> Self {
        Self { lower, upper }
    }
}

/// Handle returned when adding slack variables to a constraint.
#[derive(Debug, Clone, PartialEq)]
pub struct SlackHandle {
    pub var_ids: SlackVariables,
    pub penalty: f64,
    pub constraint_id: ConstraintId,
    pub bound: SlackBound,
    pub name: Option<String>,
}

/// Summary of slacks created via an elastic constraint helper.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElasticHandle {
    pub lower: Option<SlackHandle>,
    pub upper: Option<SlackHandle>,
}

#[cfg(test)]
mod tests {
    use super::{ElasticHandle, SlackBound, SlackVariables};
    use crate::ids::VariableId;

    #[test]
    fn slack_bound_variants_report_consistent_strings_and_flags() {
        let cases = [
            (SlackBound::Lower, "lower", true, false),
            (SlackBound::Upper, "upper", false, true),
            (SlackBound::Both, "both", true, true),
        ];

        for (bound, name, has_lower, has_upper) in cases {
            assert_eq!(bound.as_str(), name);
            assert_eq!(bound.has_lower(), has_lower);
            assert_eq!(bound.has_upper(), has_upper);
        }
    }

    #[test]
    fn slack_variables_constructor_preserves_ids() {
        let lower = Some(VariableId::new(1));
        let upper = Some(VariableId::new(2));

        let vars = SlackVariables::new(lower, upper);

        assert_eq!(vars.lower, lower);
        assert_eq!(vars.upper, upper);
    }

    #[test]
    fn default_handles_start_without_any_slack_variables() {
        let vars = SlackVariables::default();
        let elastic = ElasticHandle::default();

        assert!(vars.lower.is_none());
        assert!(vars.upper.is_none());
        assert!(elastic.lower.is_none());
        assert!(elastic.upper.is_none());
    }
}
