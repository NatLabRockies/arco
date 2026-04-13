use crate::ObjectiveSense;
use crate::algebra::{ConstraintBody, Expr};
use miette::NamedSource;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceProgram {
    pub params: Vec<ParamDecl>,
    pub data: Vec<DataDecl>,
    pub models: Vec<ModelDecl>,
    pub sets: Vec<SetDecl>,
    pub scenarios: Vec<ScenarioDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDecl {
    pub name: String,
    pub sets: Vec<SetDecl>,
    pub parameters: Vec<ParamDecl>,
    pub controls: Vec<ControlDecl>,
    pub expressions: Vec<ExpressionDecl>,
    pub constraints: Vec<ConstraintDecl>,
    pub optimize: ObjectiveDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    pub name: String,
    pub source: String,
    pub maps: Vec<MapDecl>,
    pub sets: Vec<SetDecl>,
    pub indices: Vec<DataIndexDecl>,
    pub parameters: Vec<ParamDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecl {
    pub name: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataIndexDecl {
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterComparators {
    pub eq: Option<LiteralValue>,
    pub ge: Option<LiteralValue>,
    pub geq: Option<LiteralValue>,
    pub le: Option<LiteralValue>,
    pub leq: Option<LiteralValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetDecl {
    pub name: String,
    pub alias: Option<String>,
    pub subset_of: Option<String>,
    pub members: Vec<LiteralValue>,
    pub filter_expression: Option<String>,
    pub parsed_filter_expression: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    pub name: String,
    pub indices: Vec<String>,
    pub value: Option<LiteralValue>,
    pub from: Option<String>,
    pub index: Option<String>,
    pub uses_index_children: bool,
    pub reduce: Option<String>,
    pub filter_expression: Option<String>,
    pub parsed_filter_expression: Option<Expr>,
    pub units: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDecl {
    pub name: String,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlDecl {
    pub name: String,
    pub indices: Vec<IndexDecl>,
    pub lower: Option<BoundExpr>,
    pub upper: Option<BoundExpr>,
    pub kind: Option<VariableKindDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationBinding {
    pub variable: String,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintDecl {
    pub name: String,
    pub expression: String,
    pub parsed_expression: ConstraintBody,
    pub generation_bindings: Vec<GenerationBinding>,
    pub generation_filter: Option<String>,
    pub parsed_generation_filter: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    String(String),
    Integer(i128),
    Decimal(String),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKindDecl {
    Continuous,
    Integer,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundExpr {
    Literal(LiteralValue),
    Formula(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionDecl {
    pub name: String,
    pub formula: String,
    pub parsed_formula: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveDecl {
    pub name: String,
    pub sense: ObjectiveSense,
    pub expression: String,
    pub parsed_expression: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    Scalar,
    Dual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportDecl {
    pub kind: ReportKind,
    pub target: String,
    pub filter_expression: Option<String>,
    pub parsed_filter_expression: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioDecl {
    pub name: String,
    pub data: Vec<DataBindingDecl>,
    pub model_use: Option<String>,
    pub reports: Vec<ReportDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataBindingDecl {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSource {
    pub program: SourceProgram,
    pub source_text: NamedSource<String>,
}

impl SourceProgram {
    pub fn first_scenario(&self) -> Option<&ScenarioDecl> {
        self.scenarios.first()
    }

    pub fn data(&self, name: &str) -> Option<&DataDecl> {
        self.data.iter().find(|decl| decl.name == name)
    }

    pub fn model(&self, name: &str) -> Option<&ModelDecl> {
        self.models.iter().find(|decl| decl.name == name)
    }

    pub fn scenario(&self, name: &str) -> Option<&ScenarioDecl> {
        self.scenarios.iter().find(|decl| decl.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_program_lookup_helpers_find_named_decls() {
        let mut program = SourceProgram::default();
        program.models.push(ModelDecl {
            name: "ModelA".to_string(),
            sets: Vec::new(),
            parameters: Vec::new(),
            controls: Vec::new(),
            expressions: Vec::new(),
            constraints: Vec::new(),
            optimize: ObjectiveDecl {
                name: "Obj".to_string(),
                sense: ObjectiveSense::Minimize,
                expression: "0".to_string(),
                parsed_expression: Expr::Number("0".to_string()),
            },
        });
        program.scenarios.push(ScenarioDecl {
            name: "Base".to_string(),
            data: Vec::new(),
            model_use: Some("ModelA".to_string()),
            reports: Vec::new(),
        });

        assert!(program.model("ModelA").is_some());
        assert!(program.scenario("Base").is_some());
        assert!(program.first_scenario().is_some());
    }
}
