//! Slack variable and elastic constraint support.

use crate::slack::{ElasticHandle, SlackBound, SlackHandle, SlackVariables};
use crate::types::{Bounds, Sense, Variable};
use arco_expr::ids::ConstraintId;

use crate::model::error::ModelError;
use crate::model::{Model, slack_variable_name};

impl Model {
    /// Attach slack variables to a constraint bound.
    pub fn add_slack(
        &mut self,
        constraint_id: ConstraintId,
        bound: SlackBound,
        penalty: f64,
        name: Option<String>,
    ) -> Result<SlackHandle, ModelError> {
        self.ensure_constraint_exists(constraint_id)?;
        let sense = self.objective.sense.ok_or(ModelError::NoObjective)?;
        if !penalty.is_finite() || penalty < 0.0 {
            return Err(ModelError::InvalidSlackPenalty { penalty });
        }

        let objective_coeff = match sense {
            Sense::Minimize => penalty,
            Sense::Maximize => -penalty,
        };

        let constraint_name = self
            .get_constraint_name(constraint_id)
            .map(|value| value.to_string());
        let base_name = name.clone().or_else(|| constraint_name.clone());
        let force_suffix = matches!(bound, SlackBound::Both);

        let mut var_ids = SlackVariables::default();
        let mut objective_terms = Vec::new();

        if bound.has_lower() {
            let slack_var =
                self.add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))?;
            self.set_coefficient(slack_var, constraint_id, 1.0)?;
            if let Some(slack_name) = slack_variable_name(
                name.as_deref(),
                constraint_name.as_deref(),
                "slack_lower",
                force_suffix,
            ) {
                self.set_variable_name(slack_var, slack_name)?;
            }
            var_ids.lower = Some(slack_var);
            objective_terms.push((slack_var, objective_coeff));
        }

        if bound.has_upper() {
            let slack_var =
                self.add_variable(Variable::continuous(Bounds::new(0.0, f64::INFINITY)))?;
            self.set_coefficient(slack_var, constraint_id, -1.0)?;
            if let Some(slack_name) = slack_variable_name(
                name.as_deref(),
                constraint_name.as_deref(),
                "slack_upper",
                force_suffix,
            ) {
                self.set_variable_name(slack_var, slack_name)?;
            }
            var_ids.upper = Some(slack_var);
            objective_terms.push((slack_var, objective_coeff));
        }

        self.add_objective_terms(objective_terms);

        tracing::debug!(
            component = "slack",
            operation = "add",
            status = "ok",
            constraint_id = constraint_id.inner(),
            bound = bound.as_str(),
            penalty,
            "Added slack variables"
        );

        let handle = SlackHandle {
            var_ids,
            penalty,
            constraint_id,
            bound,
            name: base_name,
        };
        self.slack_handles.push(handle.clone());

        Ok(handle)
    }

    /// Convenience helper to attach asymmetric slacks to a constraint.
    pub fn make_elastic(
        &mut self,
        constraint_id: ConstraintId,
        upper_penalty: Option<f64>,
        lower_penalty: Option<f64>,
        name: Option<String>,
    ) -> Result<ElasticHandle, ModelError> {
        self.ensure_constraint_exists(constraint_id)?;

        let base_name = name.as_deref();
        let mut handle = ElasticHandle::default();

        if let Some(penalty) = lower_penalty {
            let lower_name = base_name.map(|base| format!("{base}:slack_lower"));
            handle.lower =
                Some(self.add_slack(constraint_id, SlackBound::Lower, penalty, lower_name)?);
        }

        if let Some(penalty) = upper_penalty {
            let upper_name = base_name.map(|base| format!("{base}:slack_upper"));
            handle.upper =
                Some(self.add_slack(constraint_id, SlackBound::Upper, penalty, upper_name)?);
        }

        Ok(handle)
    }
}
