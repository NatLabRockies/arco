//! Portable intermediate representation for Arco optimization models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableProblem {
    pub variable_instances: Vec<PortableVariableInstance>,
    pub constraints: Vec<PortableLinearConstraint>,
    pub objective: PortableLinearObjective,
    pub reports: Vec<PortableLinearReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableVariableInstance {
    pub name: String,
    pub family: String,
    pub lower: f64,
    pub upper: Option<f64>,
    pub kind: PortableVariableKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortableVariableKind {
    Continuous,
    Integer,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableLinearConstraint {
    pub name: String,
    pub sense: PortableConstraintSense,
    pub rhs: f64,
    pub terms: Vec<PortableLinearTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortableConstraintSense {
    GreaterEqual,
    LessEqual,
    Equal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableLinearObjective {
    pub name: String,
    pub sense: PortableObjectiveSense,
    pub constant: f64,
    pub terms: Vec<PortableLinearTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortableObjectiveSense {
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableLinearReport {
    pub name: String,
    pub constant: f64,
    pub terms: Vec<PortableLinearTerm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableLinearTerm {
    pub variable_name: String,
    pub coefficient: f64,
}
