use arco_kdl::ObjectiveSense;
use arco_kdl::algebra::{ConstraintBody, Expr};
use arco_kdl::source::{BoundExpr, GenerationBinding, VariableKindDecl};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

    pub fn from_index_decls(
        target: impl Into<String>,
        decls: &[arco_kdl::source::IndexDecl],
    ) -> Self {
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

impl SemanticProgram {
    pub fn time_steps(&self) -> usize {
        self.time_set().map_or(0, |set| set.values.len())
    }

    pub fn is_time_set_name(&self, name: &str) -> bool {
        self.resolve_set(name)
            .zip(self.time_set())
            .is_some_and(|(candidate, time_set)| std::ptr::eq(candidate, time_set))
    }

    pub fn resolve_set(&self, name: &str) -> Option<&ResolvedSet> {
        if let Some(set) = self.set_registry.get(name) {
            return Some(set);
        }
        if let Some(set) = self
            .set_aliases
            .get(name)
            .and_then(|canonical| self.set_registry.get(canonical.as_str()))
        {
            return Some(set);
        }
        for (alias, canonical) in &self.set_aliases {
            if canonical == name {
                if let Some(set) = self.set_registry.get(alias.as_str()) {
                    return Some(set);
                }
            }
        }
        None
    }

    fn time_set(&self) -> Option<&ResolvedSet> {
        self.resolve_set("time").or_else(|| self.resolve_set("t"))
    }
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
#[allow(clippy::struct_field_names)]
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
    pub diagnostic_id: String,
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
    pub compiled_family: String,
    pub filter: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDualReport {
    pub constraint_name: String,
}
