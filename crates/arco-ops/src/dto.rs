//! Stable data-transfer objects exposed by `arco-ops`.
//!
//! These types intentionally copy the small, user-facing algebraic problem
//! vocabulary instead of re-exporting compile-internal target structs.

/// Variable domain kind in an exported algebraic problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpsVariableKind {
    Continuous,
    Integer,
    Binary,
}

/// Linear constraint comparison sense.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpsConstraintSense {
    GreaterEqual,
    LessEqual,
    Equal,
}

/// Objective optimization direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpsObjectiveSense {
    Minimize,
    Maximize,
}

/// Variable instance DTO.
#[derive(Debug, Clone, PartialEq)]
pub struct OpsVariableInstance {
    pub name: String,
    pub family: String,
    pub lower: f64,
    pub upper: Option<f64>,
    pub kind: OpsVariableKind,
}

/// Linear coefficient DTO.
#[derive(Debug, Clone, PartialEq)]
pub struct OpsLinearTerm {
    pub variable_name: String,
    pub coefficient: f64,
}

/// Linear constraint DTO.
#[derive(Debug, Clone, PartialEq)]
pub struct OpsLinearConstraint {
    pub name: String,
    pub sense: OpsConstraintSense,
    pub rhs: f64,
    pub terms: Vec<OpsLinearTerm>,
}

/// Linear objective DTO.
#[derive(Debug, Clone, PartialEq)]
pub struct OpsLinearObjective {
    pub name: String,
    pub sense: OpsObjectiveSense,
    pub constant: f64,
    pub terms: Vec<OpsLinearTerm>,
}

/// Linear report DTO.
#[derive(Debug, Clone, PartialEq)]
pub struct OpsLinearReport {
    pub(crate) name: String,
    pub(crate) constant: f64,
    pub(crate) terms: Vec<OpsLinearTerm>,
}

/// Algebraic problem DTO used at interaction-surface boundaries.
#[derive(Debug, Clone, PartialEq)]
pub struct OpsAlgebraicProblem {
    pub variable_instances: Vec<OpsVariableInstance>,
    pub constraints: Vec<OpsLinearConstraint>,
    pub objective: OpsLinearObjective,
    pub reports: Vec<OpsLinearReport>,
}
