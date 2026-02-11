use arco_expr::ids::{ConstraintId, VariableId};

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

    pub fn has_lower(self) -> bool {
        matches!(self, SlackBound::Lower | SlackBound::Both)
    }

    pub fn has_upper(self) -> bool {
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
    pub fn new(lower: Option<VariableId>, upper: Option<VariableId>) -> Self {
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
