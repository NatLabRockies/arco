use crate::algebra::{ConstraintBody, Expr, parse_constraint_formula, parse_value_formula};
use crate::normalize::normalize_surface_syntax;
use kdl::{KdlDocument, KdlNode, KdlValue};
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode, SourceSpan};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceProgram {
    pub models: Vec<ModelDecl>,
    pub sets: Vec<SetDecl>,
    pub technologies: Vec<TechnologyDecl>,
    pub operations: Vec<OperationDecl>,
    pub assets: Vec<AssetDecl>,
    pub instances: Vec<InstancesDecl>,
    pub rules: Vec<RuleDecl>,
    pub expressions: Vec<ExpressionDecl>,
    pub objectives: Vec<ObjectiveDecl>,
    pub scenarios: Vec<ScenarioDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDecl {
    pub name: String,
    pub sets: Vec<SetDecl>,
    pub parameters: Vec<ParamDecl>,
    pub controls: Vec<ControlDecl>,
    pub constraints: Vec<ConstraintDecl>,
    pub optimize: ObjectiveDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetDecl {
    pub name: String,
    pub alias: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    pub name: String,
    pub indices: Vec<String>,
    pub value: Option<LiteralValue>,
}

/// A single index dimension in a control declaration, optionally bound to a
/// named set domain via `in="set_name"`.
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

/// A named variable declaration with optional kind and bounds, used for both
/// `invest` and `control` entries inside a `technology` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedVariableDecl {
    pub name: String,
    pub kind: Option<VariableKindDecl>,
    pub lower: Option<BoundExpr>,
    pub upper: Option<BoundExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnologyDecl {
    pub name: String,
    /// Optional set alias: `technology "CoupledStorage" as="storage_techs"`.
    /// When present, the technology's asset set is registered under this name
    /// instead of the technology name, making set references in constraints
    /// explicit and greppable.
    pub set_name: Option<String>,
    pub investments: Vec<NamedVariableDecl>,
    pub controls: Vec<NamedVariableDecl>,
    pub states: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDecl {
    pub name: String,
    pub constraints: Vec<ConstraintDecl>,
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
pub struct AssetDecl {
    pub name: String,
    pub technology: String,
    pub operation: Option<String>,
    pub parameters: BTreeMap<String, LiteralValue>,
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
pub struct InstancesDecl {
    pub name: String,
    pub source: String,
    pub technology: String,
    pub operation: Option<String>,
    pub columns: Vec<ColumnMappingDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnMappingDecl {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDecl {
    pub name: String,
    pub constraints: Vec<ConstraintDecl>,
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
    pub sense: String,
    pub expression: String,
    pub parsed_expression: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    /// Scalar expression evaluated at the primal solution.
    Scalar,
    /// Constraint shadow prices (dual values).
    Dual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportDecl {
    pub kind: ReportKind,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioDecl {
    pub name: String,
    pub horizon: HorizonDecl,
    pub data: Vec<DataBindingDecl>,
    pub set_bindings: Vec<SetBindingDecl>,
    pub assets: Vec<String>,
    pub instances: Vec<String>,
    pub technologies: Vec<String>,
    pub operations: Vec<String>,
    pub rules: Vec<String>,
    pub model_use: Option<String>,
    pub objective: Option<String>,
    pub reports: Vec<ReportDecl>,
    /// User-declared sets whose members are listed inline in the scenario.
    /// E.g. `generators { "gen1"; "gen2" }` produces `("generators", ["gen1", "gen2"])`.
    pub custom_sets: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HorizonDecl {
    pub steps: usize,
    pub resolution: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataBindingDecl {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetBindingDecl {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSource {
    pub program: SourceProgram,
    pub source_text: NamedSource<String>,
}

struct ParseContext<'a> {
    path: &'a Path,
    source_text: &'a NamedSource<String>,
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse kdl {path}: {source}")]
    Kdl {
        path: PathBuf,
        #[source]
        source: kdl::KdlError,
    },
    #[error("missing required node `{name}` in {path}")]
    MissingNode {
        name: &'static str,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("missing required argument {index} on node `{node}` in {path}")]
    MissingArgument {
        node: String,
        index: usize,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("missing required property `{property}` on node `{node}` in {path}")]
    MissingProperty {
        node: String,
        property: &'static str,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("unexpected value for `{field}` on node `{node}` in {path}")]
    InvalidValue {
        node: String,
        field: String,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("unsupported declaration `{name}` in {path}")]
    UnsupportedDeclaration {
        name: String,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
    #[error("invalid algebra in `{node}` in {path}: {reason}")]
    InvalidAlgebra {
        node: String,
        reason: String,
        path: PathBuf,
        source_text: Box<NamedSource<String>>,
        span: SourceSpan,
    },
}

impl SourceProgram {
    pub fn first_scenario(&self) -> Option<&ScenarioDecl> {
        self.scenarios.first()
    }

    pub fn technology(&self, name: &str) -> Option<&TechnologyDecl> {
        self.technologies.iter().find(|decl| decl.name == name)
    }

    pub fn model(&self, name: &str) -> Option<&ModelDecl> {
        self.models.iter().find(|decl| decl.name == name)
    }

    pub fn operation(&self, name: &str) -> Option<&OperationDecl> {
        self.operations.iter().find(|decl| decl.name == name)
    }

    pub fn asset(&self, name: &str) -> Option<&AssetDecl> {
        self.assets.iter().find(|decl| decl.name == name)
    }

    pub fn instances(&self, name: &str) -> Option<&InstancesDecl> {
        self.instances.iter().find(|decl| decl.name == name)
    }

    pub fn rule(&self, name: &str) -> Option<&RuleDecl> {
        self.rules.iter().find(|decl| decl.name == name)
    }

    pub fn expression(&self, name: &str) -> Option<&ExpressionDecl> {
        self.expressions.iter().find(|decl| decl.name == name)
    }

    pub fn objective(&self, name: &str) -> Option<&ObjectiveDecl> {
        self.objectives.iter().find(|decl| decl.name == name)
    }

    pub fn scenario(&self, name: &str) -> Option<&ScenarioDecl> {
        self.scenarios.iter().find(|decl| decl.name == name)
    }
}

impl Diagnostic for SourceError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        let code = match self {
            Self::Io { .. } => "arco::source::io",
            Self::Kdl { .. } => "arco::source::kdl",
            Self::MissingNode { .. } => "arco::source::missing_node",
            Self::MissingArgument { .. } => "arco::source::missing_argument",
            Self::MissingProperty { .. } => "arco::source::missing_property",
            Self::InvalidValue { .. } => "arco::source::invalid_value",
            Self::UnsupportedDeclaration { .. } => "arco::source::unsupported_declaration",
            Self::InvalidAlgebra { .. } => "arco::source::invalid_algebra",
        };
        Some(Box::new(code))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match self {
            Self::MissingNode { name, .. } => Some(Box::new(format!(
                "add a `{name}` child declaration to this block"
            ))),
            Self::MissingArgument { index, .. } => Some(Box::new(format!(
                "add argument {index} to this declaration"
            ))),
            Self::MissingProperty { property, .. } => Some(Box::new(format!(
                "add a `{property}` property to this declaration"
            ))),
            Self::InvalidValue { field, .. } => Some(Box::new(format!(
                "replace `{field}` with a value of the expected type"
            ))),
            Self::UnsupportedDeclaration { .. } => Some(Box::new(
                "remove the declaration or add parser support for it",
            )),
            Self::InvalidAlgebra { .. } => Some(Box::new(
                "fix the algebra syntax so the expression can be parsed into the DSL AST",
            )),
            Self::Io { .. } | Self::Kdl { .. } => None,
        }
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        match self {
            Self::MissingNode { source_text, .. }
            | Self::MissingArgument { source_text, .. }
            | Self::MissingProperty { source_text, .. }
            | Self::InvalidValue { source_text, .. }
            | Self::UnsupportedDeclaration { source_text, .. }
            | Self::InvalidAlgebra { source_text, .. } => Some(source_text.as_ref()),
            Self::Io { .. } | Self::Kdl { .. } => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let labeled = match self {
            Self::MissingNode { span, .. }
            | Self::MissingArgument { span, .. }
            | Self::MissingProperty { span, .. }
            | Self::InvalidValue { span, .. }
            | Self::UnsupportedDeclaration { span, .. }
            | Self::InvalidAlgebra { span, .. } => Some(LabeledSpan::new_with_span(
                Some("this declaration".to_string()),
                *span,
            )),
            Self::Io { .. } | Self::Kdl { .. } => None,
        }?;
        Some(Box::new(std::iter::once(labeled)))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::Kdl { source, .. } => Some(source),
            Self::Io { .. }
            | Self::MissingNode { .. }
            | Self::MissingArgument { .. }
            | Self::MissingProperty { .. }
            | Self::InvalidValue { .. }
            | Self::UnsupportedDeclaration { .. }
            | Self::InvalidAlgebra { .. } => None,
        }
    }
}

pub fn parse_program_file(path: &Path) -> Result<ParsedSource, SourceError> {
    info!("parsing {}", path.display());
    let text = fs::read_to_string(path).map_err(|source| SourceError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_program_text(&text, path)
}

pub fn parse_program_text(text: &str, path: &Path) -> Result<ParsedSource, SourceError> {
    let normalized = normalize_surface_syntax(text);
    let source_text =
        NamedSource::new(path.display().to_string(), normalized.clone()).with_language("kdl");
    let document: KdlDocument = normalized.parse().map_err(|source| SourceError::Kdl {
        path: path.to_path_buf(),
        source,
    })?;
    let context = ParseContext {
        path,
        source_text: &source_text,
    };
    let program = parse_document(&document, &context)?;

    Ok(ParsedSource {
        program,
        source_text,
    })
}

fn parse_document(
    document: &KdlDocument,
    context: &ParseContext<'_>,
) -> Result<SourceProgram, SourceError> {
    let mut program = SourceProgram::default();

    for node in document.nodes() {
        match node.name().value() {
            "model" => program.models.push(parse_model(node, context)?),
            "set" => program.sets.push(parse_set(node, context)?),
            "technology" => program.technologies.push(parse_technology(node, context)?),
            "operation" => program.operations.push(parse_operation(node, context)?),
            "asset" => program.assets.push(parse_asset(node, context)?),
            "instances" => program.instances.push(parse_instances(node, context)?),
            "rule" => program.rules.push(parse_rule(node, context)?),
            "expression" => program.expressions.push(parse_expression(node, context)?),
            "minimize" => program
                .objectives
                .push(parse_optimize(node, "minimize", context)?),
            "maximize" => program
                .objectives
                .push(parse_optimize(node, "maximize", context)?),
            "scenario" => program.scenarios.push(parse_scenario(node, context)?),
            other => {
                return Err(SourceError::UnsupportedDeclaration {
                    name: other.to_string(),
                    path: context.path.to_path_buf(),
                    source_text: Box::new(context.source_text.clone()),
                    span: node.span(),
                });
            }
        }
    }

    Ok(program)
}

fn parse_model(node: &KdlNode, context: &ParseContext<'_>) -> Result<ModelDecl, SourceError> {
    let mut sets = Vec::new();
    let mut parameters = Vec::new();
    let mut controls = Vec::new();
    let constraints = parse_constraints(node, context)?;
    let mut optimize = None;

    for child in node.iter_children() {
        match child.name().value() {
            "set" => sets.push(parse_set(child, context)?),
            "param" => parameters.push(ParamDecl {
                name: first_arg_string(child, 0, context)?,
                indices: declaration_indices(child)
                    .into_iter()
                    .map(|idx| idx.name)
                    .collect(),
                value: positional_value(child, context)?,
            }),
            "control" => controls.push(parse_control(child, context)?),
            "minimize" => optimize = Some(parse_optimize(child, "minimize", context)?),
            "maximize" => optimize = Some(parse_optimize(child, "maximize", context)?),
            _ => {}
        }
    }

    Ok(ModelDecl {
        name: first_arg_string(node, 0, context)?,
        sets,
        parameters,
        controls,
        constraints,
        optimize: optimize
            .ok_or_else(|| missing_node_error("minimize_or_maximize", node, context))?,
    })
}

fn parse_set(node: &KdlNode, context: &ParseContext<'_>) -> Result<SetDecl, SourceError> {
    Ok(SetDecl {
        name: first_arg_string(node, 0, context)?,
        alias: node
            .get("alias")
            .and_then(KdlValue::as_string)
            .map(ToString::to_string),
        source: optional_property_string(node, "from", context)?,
    })
}

fn parse_control(node: &KdlNode, context: &ParseContext<'_>) -> Result<ControlDecl, SourceError> {
    let kind = optional_property_string(node, "kind", context)?
        .map(|value| parse_variable_kind_decl(node, &value, context))
        .transpose()?;

    let mut lower = optional_property_literal(node, "lower", context)?.map(BoundExpr::Literal);
    let mut upper = optional_property_literal(node, "upper", context)?.map(BoundExpr::Literal);

    // Child nodes named "lower" or "upper" carry algebra formulas and override
    // literal properties when present.
    for child in node.iter_children() {
        match child.name().value() {
            "lower" => {
                let formula_text = property_string(child, "expression", context)?;
                lower = Some(BoundExpr::Formula(
                    parse_value_formula(&formula_text)
                        .map_err(|e| algebra_error(child, e.to_string(), context))?,
                ));
            }
            "upper" => {
                let formula_text = property_string(child, "expression", context)?;
                upper = Some(BoundExpr::Formula(
                    parse_value_formula(&formula_text)
                        .map_err(|e| algebra_error(child, e.to_string(), context))?,
                ));
            }
            _ => {}
        }
    }

    Ok(ControlDecl {
        name: first_arg_string(node, 0, context)?,
        indices: declaration_indices(node),
        lower,
        upper,
        kind,
    })
}

fn parse_variable_kind_decl(
    node: &KdlNode,
    value: &str,
    context: &ParseContext<'_>,
) -> Result<VariableKindDecl, SourceError> {
    match value {
        "continuous" => Ok(VariableKindDecl::Continuous),
        "integer" => Ok(VariableKindDecl::Integer),
        "binary" => Ok(VariableKindDecl::Binary),
        _ => Err(invalid_value_error(node, "kind".to_string(), context)),
    }
}

fn declaration_indices(node: &KdlNode) -> Vec<IndexDecl> {
    let mut indices = Vec::new();
    for child in node.iter_children() {
        match child.name().value() {
            "lower" | "upper" => {}
            name => {
                let domain = child
                    .get("in")
                    .and_then(KdlValue::as_string)
                    .map(ToString::to_string);
                indices.push(IndexDecl {
                    name: name.to_string(),
                    domain,
                });
            }
        }
    }
    indices
}

fn positional_value(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Option<LiteralValue>, SourceError> {
    if declaration_indices(node).is_empty() {
        return node
            .get(1)
            .map(|value| literal_from_arg(node, value, context))
            .transpose();
    }

    Ok(None)
}

fn literal_from_arg(
    node: &KdlNode,
    value: &KdlValue,
    context: &ParseContext<'_>,
) -> Result<LiteralValue, SourceError> {
    if let Some(string) = value.as_string() {
        return Ok(LiteralValue::String(string.to_string()));
    }
    if let Some(integer) = value.as_integer() {
        return Ok(LiteralValue::Integer(integer));
    }
    if let Some(boolean) = value.as_bool() {
        return Ok(LiteralValue::Boolean(boolean));
    }
    if let Some(decimal) = value.as_float() {
        return Ok(LiteralValue::Decimal(decimal.to_string()));
    }

    Err(invalid_value_error(
        node,
        "argument value".to_string(),
        context,
    ))
}

fn parse_optimize(
    node: &KdlNode,
    sense: &str,
    context: &ParseContext<'_>,
) -> Result<ObjectiveDecl, SourceError> {
    let expression = property_string(node, "expression", context)?;
    Ok(ObjectiveDecl {
        name: first_arg_string(node, 0, context)?,
        sense: sense.to_string(),
        parsed_expression: parse_value_formula(&expression)
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        expression,
    })
}

fn parse_technology(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<TechnologyDecl, SourceError> {
    let mut investments = Vec::new();
    let mut controls = Vec::new();
    let mut states = Vec::new();
    for child in node.iter_children() {
        match child.name().value() {
            "invest" => investments.push(parse_named_variable_decl(child, context)?),
            "control" => controls.push(parse_named_variable_decl(child, context)?),
            "state" => states.push(first_arg_string(child, 0, context)?),
            _ => {}
        }
    }

    let set_name = node
        .get("as")
        .and_then(KdlValue::as_string)
        .map(ToString::to_string);

    Ok(TechnologyDecl {
        name: first_arg_string(node, 0, context)?,
        set_name,
        investments,
        controls,
        states,
    })
}

/// Parse a named variable declaration with optional `kind`, `lower`, and
/// `upper` literal properties. Used for both `invest` and `control` children
/// inside a `technology` block.
fn parse_named_variable_decl(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<NamedVariableDecl, SourceError> {
    let kind = optional_property_string(node, "kind", context)?
        .map(|value| parse_variable_kind_decl(node, &value, context))
        .transpose()?;
    Ok(NamedVariableDecl {
        name: first_arg_string(node, 0, context)?,
        kind,
        lower: optional_property_literal(node, "lower", context)?.map(BoundExpr::Literal),
        upper: optional_property_literal(node, "upper", context)?.map(BoundExpr::Literal),
    })
}

fn parse_operation(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<OperationDecl, SourceError> {
    Ok(OperationDecl {
        name: first_arg_string(node, 0, context)?,
        constraints: parse_constraints(node, context)?,
    })
}

fn parse_asset(node: &KdlNode, context: &ParseContext<'_>) -> Result<AssetDecl, SourceError> {
    let mut technology = None;
    let mut operation = None;
    let mut parameters = BTreeMap::new();

    for child in node.iter_children() {
        match child.name().value() {
            "technology" => technology = Some(first_arg_string(child, 0, context)?),
            "operation" => operation = Some(first_arg_string(child, 0, context)?),
            name => {
                parameters.insert(name.to_string(), first_arg_literal(child, context)?);
            }
        }
    }

    Ok(AssetDecl {
        name: first_arg_string(node, 0, context)?,
        technology: technology.ok_or_else(|| missing_node_error("technology", node, context))?,
        operation,
        parameters,
    })
}

fn parse_instances(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<InstancesDecl, SourceError> {
    let mut technology = None;
    let mut operation = None;
    let mut columns = Vec::new();

    for child in node.iter_children() {
        match child.name().value() {
            "technology" => technology = Some(first_arg_string(child, 0, context)?),
            "operation" => operation = Some(first_arg_string(child, 0, context)?),
            "map" => columns.push(ColumnMappingDecl {
                source: property_string(child, "from", context)?,
                target: first_arg_string(child, 0, context)?,
            }),
            _ => {}
        }
    }

    Ok(InstancesDecl {
        name: first_arg_string(node, 0, context)?,
        source: property_string(node, "from", context)?,
        technology: technology.ok_or_else(|| missing_node_error("technology", node, context))?,
        operation,
        columns,
    })
}

fn parse_rule(node: &KdlNode, context: &ParseContext<'_>) -> Result<RuleDecl, SourceError> {
    Ok(RuleDecl {
        name: first_arg_string(node, 0, context)?,
        constraints: parse_constraints(node, context)?,
    })
}

fn parse_expression(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<ExpressionDecl, SourceError> {
    let formula = child_arg_string(node, "formula", 0, context)?;
    Ok(ExpressionDecl {
        name: first_arg_string(node, 0, context)?,
        parsed_formula: parse_value_formula(&formula)
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        formula,
    })
}

/// Keywords recognized as built-in scenario children. Anything else with
/// child string arguments is treated as a user-declared custom set.
const SCENARIO_KEYWORDS: &[&str] = &[
    "data",
    "set",
    "asset",
    "instances",
    "technology",
    "operation",
    "rule",
    "use",
    "minimize",
    "maximize",
    "report",
    "horizon",
    "solver",
];

fn parse_scenario(node: &KdlNode, context: &ParseContext<'_>) -> Result<ScenarioDecl, SourceError> {
    let horizon_node = child_node(node, "horizon", context)?;
    let mut data = Vec::new();
    let mut set_bindings = Vec::new();
    let mut assets = Vec::new();
    let mut instances = Vec::new();
    let mut technologies = Vec::new();
    let mut operations = Vec::new();
    let mut rules = Vec::new();
    let mut model_use = None;
    let mut objective = None;
    let mut reports = Vec::new();
    let mut custom_sets: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for child in node.iter_children() {
        match child.name().value() {
            "data" => data.push(DataBindingDecl {
                name: first_arg_string(child, 0, context)?,
                source: property_string(child, "from", context)?,
            }),
            "set" => set_bindings.push(SetBindingDecl {
                name: first_arg_string(child, 0, context)?,
                source: property_string(child, "from", context)?,
            }),
            "asset" => assets.push(first_arg_string(child, 0, context)?),
            "instances" => instances.push(first_arg_string(child, 0, context)?),
            "technology" => technologies.push(first_arg_string(child, 0, context)?),
            "operation" => operations.push(first_arg_string(child, 0, context)?),
            "rule" => rules.push(first_arg_string(child, 0, context)?),
            "use" => model_use = Some(first_arg_string(child, 0, context)?),
            "minimize" | "maximize" => {
                objective = Some(first_arg_string(child, 0, context)?);
            }
            "report" => {
                let first = first_arg_string(child, 0, context)?;
                match first.as_str() {
                    "dual" => {
                        let target = first_arg_string(child, 1, context)?;
                        reports.push(ReportDecl {
                            kind: ReportKind::Dual,
                            target,
                        });
                    }
                    _ => {
                        reports.push(ReportDecl {
                            kind: ReportKind::Scalar,
                            target: first,
                        });
                    }
                }
            }
            name if !SCENARIO_KEYWORDS.contains(&name) => {
                let members = parse_custom_set_members(child);
                if !members.is_empty() {
                    custom_sets
                        .entry(name.to_string())
                        .or_default()
                        .extend(members);
                }
            }
            _ => {}
        }
    }

    Ok(ScenarioDecl {
        name: first_arg_string(node, 0, context)?,
        horizon: HorizonDecl {
            steps: property_usize(horizon_node, "steps", context)?,
            resolution: property_string(horizon_node, "resolution", context)?,
        },
        data,
        set_bindings,
        assets,
        instances,
        technologies,
        operations,
        rules,
        model_use,
        objective,
        reports,
        custom_sets,
    })
}

/// Extract string members from a custom set node. Members are child nodes
/// whose names are string literals, e.g.:
/// ```kdl
/// generators {
///     "gen1"
///     "gen2"
/// }
/// ```
fn parse_custom_set_members(node: &KdlNode) -> Vec<String> {
    let mut members = Vec::new();
    // Members as positional arguments on child nodes (each child is a string literal node name)
    for child in node.iter_children() {
        members.push(child.name().value().to_string());
    }
    // Also check for inline string arguments on the node itself
    for entry in node.entries() {
        if entry.name().is_none() {
            if let Some(s) = entry.value().as_string() {
                members.push(s.to_string());
            }
        }
    }
    members
}

fn parse_constraints(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Vec<ConstraintDecl>, SourceError> {
    let mut constraints = Vec::new();
    for (index, child) in node
        .iter_children()
        .filter(|child| child.name().value() == "constraint")
        .enumerate()
    {
        let has_generation_children = child
            .iter_children()
            .any(|grandchild| matches!(grandchild.name().value(), "over" | "when" | "expr"));

        if has_generation_children {
            constraints.push(parse_constraint_with_generation(child, index, context)?);
        } else {
            let expression = property_string(child, "expression", context)?;
            let generation_filter = optional_property_string(child, "if", context)?;
            constraints.push(ConstraintDecl {
                name: constraint_name(child, index),
                parsed_expression: parse_constraint_formula(&expression)
                    .map_err(|error| algebra_error(child, error.to_string(), context))?,
                generation_bindings: Vec::new(),
                parsed_generation_filter: generation_filter
                    .as_deref()
                    .map(parse_value_formula)
                    .transpose()
                    .map_err(|error| algebra_error(child, error.to_string(), context))?,
                generation_filter,
                expression,
            });
        }
    }
    Ok(constraints)
}

fn parse_constraint_with_generation(
    node: &KdlNode,
    index: usize,
    context: &ParseContext<'_>,
) -> Result<ConstraintDecl, SourceError> {
    let mut generation_bindings = Vec::new();
    let mut generation_filter = None;
    let mut expression = None;

    for child in node.iter_children() {
        match child.name().value() {
            "over" => {
                generation_bindings.push(GenerationBinding {
                    variable: first_arg_string(child, 0, context)?,
                    domain: property_string(child, "in", context)?,
                });
            }
            "when" => {
                generation_filter = Some(first_arg_string(child, 0, context)?);
            }
            "expr" => {
                expression = Some(property_string(child, "expression", context)?);
            }
            _ => {}
        }
    }

    let expression = expression.ok_or_else(|| missing_node_error("expr", node, context))?;
    let parsed_generation_filter = generation_filter
        .as_deref()
        .map(parse_value_formula)
        .transpose()
        .map_err(|error| algebra_error(node, error.to_string(), context))?;

    Ok(ConstraintDecl {
        name: constraint_name(node, index),
        parsed_expression: parse_constraint_formula(&expression)
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        generation_bindings,
        parsed_generation_filter,
        generation_filter,
        expression,
    })
}

fn constraint_name(node: &KdlNode, index: usize) -> String {
    node.get("name")
        .and_then(KdlValue::as_string)
        .map(ToString::to_string)
        .or_else(|| {
            node.get(0)
                .and_then(KdlValue::as_string)
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| format!("constraint_{}", index + 1))
}

fn child_node<'a>(
    node: &'a KdlNode,
    name: &'static str,
    context: &ParseContext<'_>,
) -> Result<&'a KdlNode, SourceError> {
    node.children()
        .and_then(|children| children.get(name))
        .ok_or_else(|| missing_node_error(name, node, context))
}

fn child_arg_string(
    node: &KdlNode,
    child_name: &'static str,
    index: usize,
    context: &ParseContext<'_>,
) -> Result<String, SourceError> {
    let child = child_node(node, child_name, context)?;
    first_arg_string(child, index, context)
}

fn first_arg_string(
    node: &KdlNode,
    index: usize,
    context: &ParseContext<'_>,
) -> Result<String, SourceError> {
    node.get(index)
        .and_then(KdlValue::as_string)
        .map(ToString::to_string)
        .or_else(|| {
            node.get("name")
                .and_then(KdlValue::as_string)
                .map(ToString::to_string)
        })
        .ok_or_else(|| missing_argument_error(node, index, context))
}

fn first_arg_literal(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<LiteralValue, SourceError> {
    let value = node
        .get(0)
        .ok_or_else(|| missing_argument_error(node, 0, context))?;

    if let Some(string) = value.as_string() {
        return Ok(LiteralValue::String(string.to_string()));
    }
    if let Some(integer) = value.as_integer() {
        return Ok(LiteralValue::Integer(integer));
    }
    if let Some(boolean) = value.as_bool() {
        return Ok(LiteralValue::Boolean(boolean));
    }
    if let Some(decimal) = value.as_float() {
        return Ok(LiteralValue::Decimal(decimal.to_string()));
    }

    Err(invalid_value_error(node, "argument 0".to_string(), context))
}

fn property_string(
    node: &KdlNode,
    property: &'static str,
    context: &ParseContext<'_>,
) -> Result<String, SourceError> {
    node.get(property)
        .and_then(KdlValue::as_string)
        .map(ToString::to_string)
        .ok_or_else(|| missing_property_error(node, property, context))
}

fn optional_property_string(
    node: &KdlNode,
    property: &'static str,
    context: &ParseContext<'_>,
) -> Result<Option<String>, SourceError> {
    let Some(value) = node.get(property) else {
        return Ok(None);
    };
    value
        .as_string()
        .map(|text| Some(text.to_string()))
        .ok_or_else(|| invalid_value_error(node, property.to_string(), context))
}

fn optional_property_literal(
    node: &KdlNode,
    property: &'static str,
    context: &ParseContext<'_>,
) -> Result<Option<LiteralValue>, SourceError> {
    let Some(value) = node.get(property) else {
        return Ok(None);
    };

    if let Some(string) = value.as_string() {
        return Ok(Some(LiteralValue::String(string.to_string())));
    }
    if let Some(integer) = value.as_integer() {
        return Ok(Some(LiteralValue::Integer(integer)));
    }
    if let Some(boolean) = value.as_bool() {
        return Ok(Some(LiteralValue::Boolean(boolean)));
    }
    if let Some(decimal) = value.as_float() {
        return Ok(Some(LiteralValue::Decimal(decimal.to_string())));
    }

    Err(invalid_value_error(node, property.to_string(), context))
}

fn property_usize(
    node: &KdlNode,
    property: &'static str,
    context: &ParseContext<'_>,
) -> Result<usize, SourceError> {
    let integer = node
        .get(property)
        .and_then(KdlValue::as_integer)
        .ok_or_else(|| missing_property_error(node, property, context))?;

    usize::try_from(integer).map_err(|_| invalid_value_error(node, property.to_string(), context))
}

fn missing_node_error(
    name: &'static str,
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> SourceError {
    SourceError::MissingNode {
        name,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

fn missing_argument_error(node: &KdlNode, index: usize, context: &ParseContext<'_>) -> SourceError {
    SourceError::MissingArgument {
        node: node.name().value().to_string(),
        index,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

fn missing_property_error(
    node: &KdlNode,
    property: &'static str,
    context: &ParseContext<'_>,
) -> SourceError {
    SourceError::MissingProperty {
        node: node.name().value().to_string(),
        property,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

fn invalid_value_error(node: &KdlNode, field: String, context: &ParseContext<'_>) -> SourceError {
    SourceError::InvalidValue {
        node: node.name().value().to_string(),
        field,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

fn algebra_error(node: &KdlNode, reason: String, context: &ParseContext<'_>) -> SourceError {
    SourceError::InvalidAlgebra {
        node: node.name().value().to_string(),
        reason,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}
