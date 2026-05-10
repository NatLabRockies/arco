use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlgebraicProblem {
    #[serde(default = "default_linearized")]
    pub linearized: bool,
    pub variable_instances: Vec<VariableInstance>,
    pub constraints: Vec<LinearConstraint>,
    pub objective: LinearObjective,
    pub reports: Vec<LinearReport>,
    #[serde(default)]
    pub nonlinear: Option<crate::compile::compile::NonlinearProblem>,
}

fn default_linearized() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableInstance {
    pub name: String,
    pub family: String,
    pub lower: f64,
    pub upper: Option<f64>,
    pub kind: VariableKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableKind {
    Continuous,
    Integer,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearConstraint {
    pub name: String,
    pub sense: ConstraintSense,
    pub rhs: f64,
    pub terms: Vec<LinearTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintSense {
    GreaterEqual,
    LessEqual,
    Equal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearObjective {
    pub name: String,
    pub sense: ObjectiveSense,
    pub constant: f64,
    pub terms: Vec<LinearTerm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearReport {
    pub name: String,
    pub constant: f64,
    pub terms: Vec<LinearTerm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearTerm {
    pub variable_name: String,
    pub coefficient: f64,
}
