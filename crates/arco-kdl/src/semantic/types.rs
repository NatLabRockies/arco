use crate::ObjectiveSense;
use crate::algebra::{ConstraintBody, Expr};
use crate::source::{BoundExpr, GenerationBinding, VariableKindDecl};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

/// Time resolution for the time horizon, using ISO 8601 duration format.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum TimeResolution {
    /// 15 minutes (PT15M)
    FifteenMinutes,
    /// 30 minutes (PT30M)
    ThirtyMinutes,
    /// 1 hour (PT1H)
    #[default]
    Hourly,
    /// 1 day (P1D)
    Daily,
    /// 1 week (P1W)
    Weekly,
    /// 1 month (P1M)
    Monthly,
    /// 1 year (P1Y)
    Yearly,
}

impl TimeResolution {
    /// Returns the duration in hours for this resolution.
    pub fn as_hours(&self) -> f64 {
        match self {
            TimeResolution::FifteenMinutes => 0.25,
            TimeResolution::ThirtyMinutes => 0.5,
            TimeResolution::Hourly => 1.0,
            TimeResolution::Daily => 24.0,
            TimeResolution::Weekly => 168.0,  // 24 * 7
            TimeResolution::Monthly => 720.0, // Approximate: 30 days
            TimeResolution::Yearly => 8760.0, // Approximate: 365 days
        }
    }

    /// Returns the ISO 8601 duration string representation.
    pub fn as_iso8601(&self) -> &'static str {
        match self {
            TimeResolution::FifteenMinutes => "PT15M",
            TimeResolution::ThirtyMinutes => "PT30M",
            TimeResolution::Hourly => "PT1H",
            TimeResolution::Daily => "P1D",
            TimeResolution::Weekly => "P1W",
            TimeResolution::Monthly => "P1M",
            TimeResolution::Yearly => "P1Y",
        }
    }
}

impl fmt::Display for TimeResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_iso8601())
    }
}

impl FromStr for TimeResolution {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PT15M" | "PT15m" | "15min" | "15MIN" => Ok(TimeResolution::FifteenMinutes),
            "PT30M" | "PT30m" | "30min" | "30MIN" => Ok(TimeResolution::ThirtyMinutes),
            "PT1H" | "PT1h" | "1h" | "1H" | "hourly" | "HOURLY" | "Hourly" => {
                Ok(TimeResolution::Hourly)
            }
            "P1D" | "P1d" | "1d" | "1D" | "daily" | "DAILY" | "Daily" => Ok(TimeResolution::Daily),
            "P1W" | "P1w" | "1w" | "1W" | "weekly" | "WEEKLY" | "Weekly" => {
                Ok(TimeResolution::Weekly)
            }
            "P1M" | "P1m" | "1m" | "1M" | "monthly" | "MONTHLY" | "Monthly" => {
                Ok(TimeResolution::Monthly)
            }
            "P1Y" | "P1y" | "1y" | "1Y" | "yearly" | "YEARLY" | "Yearly" | "annual" | "ANNUAL"
            | "Annual" => Ok(TimeResolution::Yearly),
            _ => Err(format!("Unknown time resolution: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FamilySignature {
    pub target: String,
    pub indices: Vec<String>,
    pub index_domains: BTreeMap<String, String>,
}

impl FamilySignature {
    pub fn new(
        target: impl Into<String>,
        indices: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            target: target.into(),
            indices: indices.into_iter().map(Into::into).collect(),
            index_domains: BTreeMap::new(),
        }
    }

    pub fn from_index_decls(target: impl Into<String>, decls: &[crate::source::IndexDecl]) -> Self {
        let mut index_domains = BTreeMap::new();
        let indices = decls
            .iter()
            .map(|idx| {
                if let Some(domain) = &idx.domain {
                    index_domains.insert(idx.name.clone(), domain.clone());
                }
                idx.name.clone()
            })
            .collect();
        Self {
            target: target.into(),
            indices,
            index_domains,
        }
    }

    pub fn render(&self) -> String {
        if self.indices.is_empty() {
            return self.target.clone();
        }
        format!("{}[{}]", self.target, self.indices.join(","))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableDeclOverrides {
    pub kind: Option<VariableKindDecl>,
    pub lower: Option<BoundExpr>,
    pub upper: Option<BoundExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticProgram {
    pub active_scenario: String,
    pub sets: ResolvedSets,
    pub set_registry: BTreeMap<String, ResolvedSet>,
    pub set_aliases: BTreeMap<String, String>,
    pub set_params: BTreeMap<String, BTreeMap<String, f64>>,
    pub parameters: ResolvedParameters,
    pub variable_families: Vec<FamilySignature>,
    pub variable_overrides: BTreeMap<String, VariableDeclOverrides>,
    pub chronology: ResolvedChronology,
    pub active_constraints: Vec<ResolvedConstraint>,
    pub active_expressions: Vec<ResolvedExpression>,
    pub active_objective: ResolvedObjective,
    pub active_reports: Vec<ResolvedReport>,
    pub active_variable_reports: Vec<ResolvedVariableReport>,
    pub active_dual_reports: Vec<ResolvedDualReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSets {
    pub time: ResolvedTimeSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTimeSet {
    pub steps: usize,
    pub resolution: TimeResolution,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSet {
    pub values: Vec<String>,
    pub tuple_components: Option<Vec<String>>,
    pub tuple_component_domains: Option<Vec<String>>,
    pub tuple_rows: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResolvedParameters {
    pub series: Vec<String>,
    pub indexed: Vec<String>,
    pub asset: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResolvedChronology {
    pub initial_boundary: Option<String>,
    pub terminal_boundary: Option<String>,
    pub initial_commitment_boundary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConstraint {
    pub name: String,
    pub source_kind: String,
    pub source_name: String,
    pub expression_text: String,
    pub expression: ConstraintBody,
    pub generation_bindings: Vec<GenerationBinding>,
    pub generation_filter_text: Option<String>,
    pub generation_filter: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedExpression {
    pub name: String,
    pub formula_text: String,
    pub formula: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedObjective {
    pub name: String,
    pub sense: ObjectiveSense,
    pub expression_text: String,
    pub expression: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedReport {
    pub name: String,
    pub formula_text: String,
    pub formula: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVariableReport {
    pub control_name: String,
    pub indices: Vec<String>,
    pub filter: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDualReport {
    pub constraint_name: String,
}
