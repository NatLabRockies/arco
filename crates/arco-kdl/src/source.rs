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
    pub data: Vec<DataDecl>,
    pub subsets: Vec<SubsetDecl>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubsetDecl {
    pub name: String,
    pub source: String,
    pub field_filters: BTreeMap<String, LiteralValue>,
    pub filter_by: Option<String>,
    pub comparators: FilterComparators,
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
    /// Legacy source-backed set resolution (pre-spec migration).
    pub source: Option<String>,
    /// Parent-set relationship from `in <parent>` (or legacy `subset_of=`).
    pub subset_of: Option<String>,
    /// Explicit members for top-level `set` declarations.
    pub members: Vec<LiteralValue>,
    /// Canonical predicate filter from `filter { ... }`.
    pub filter_expression: Option<String>,
    pub parsed_filter_expression: Option<Expr>,
    /// Legacy comparator filter surface.
    pub filter_by: Option<String>,
    pub comparators: FilterComparators,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    pub name: String,
    pub indices: Vec<String>,
    pub value: Option<LiteralValue>,
    pub from: Option<String>,
    pub index_by: Option<String>,
    pub uses_index_children: bool,
    pub reduce: Option<String>,
    /// Canonical predicate filter from `filter { ... }`.
    pub filter_expression: Option<String>,
    pub parsed_filter_expression: Option<Expr>,
    /// Legacy comparator filter surface.
    pub filter_by: Option<String>,
    pub comparators: FilterComparators,
    pub units: Option<String>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    pub fn data(&self, name: &str) -> Option<&DataDecl> {
        self.data.iter().find(|decl| decl.name == name)
    }

    pub fn subset(&self, name: &str) -> Option<&SubsetDecl> {
        self.subsets.iter().find(|decl| decl.name == name)
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
            "set" => program.sets.push(parse_set(node, context)?),
            "data" => program.data.push(parse_data(node, context)?),
            "subset" => program.subsets.push(parse_subset(node, context)?),
            "model" => program.models.push(parse_model(node, context)?),
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
    let mut expressions = Vec::new();
    let constraints = parse_constraints(node, context)?;
    let mut optimize = None;

    for child in node.iter_children() {
        match child.name().value() {
            "set" => sets.push(parse_set(child, context)?),
            "param" => parameters.push(parse_param(child, context)?),
            "control" => controls.push(parse_control(child, context)?),
            "expression" => expressions.push(parse_expression(child, context)?),
            "constraint" => {}
            "minimize" => optimize = Some(parse_optimize(child, "minimize", context)?),
            "maximize" => optimize = Some(parse_optimize(child, "maximize", context)?),
            other => {
                return Err(SourceError::UnsupportedDeclaration {
                    name: other.to_string(),
                    path: context.path.to_path_buf(),
                    source_text: Box::new(context.source_text.clone()),
                    span: child.span(),
                });
            }
        }
    }

    Ok(ModelDecl {
        name: first_arg_string(node, 0, context)?,
        sets,
        parameters,
        controls,
        expressions,
        constraints,
        optimize: optimize
            .ok_or_else(|| missing_node_error("minimize_or_maximize", node, context))?,
    })
}

fn parse_data(node: &KdlNode, context: &ParseContext<'_>) -> Result<DataDecl, SourceError> {
    let mut maps = Vec::new();
    let mut sets = Vec::new();
    let mut indices = Vec::new();
    let mut parameters = Vec::new();

    for child in node.iter_children() {
        match child.name().value() {
            "map" => maps.push(MapDecl {
                name: first_arg_string(child, 0, context)?,
                source: optional_property_string(child, "from", context)?,
            }),
            "set" => sets.push(parse_set(child, context)?),
            "index" => {
                let mut columns = Vec::new();
                let mut position = 0;
                while let Some(value) = child.get(position) {
                    let Some(name) = value.as_string() else {
                        return Err(invalid_value_error(
                            child,
                            format!("argument {position}"),
                            context,
                        ));
                    };
                    columns.push(name.to_string());
                    position += 1;
                }
                indices.push(DataIndexDecl { columns });
            }
            "param" => parameters.push(parse_param(child, context)?),
            other => {
                return Err(SourceError::UnsupportedDeclaration {
                    name: other.to_string(),
                    path: context.path.to_path_buf(),
                    source_text: Box::new(context.source_text.clone()),
                    span: child.span(),
                });
            }
        }
    }

    Ok(DataDecl {
        name: first_arg_string(node, 0, context)?,
        source: property_string(node, "from", context)?,
        maps,
        sets,
        indices,
        parameters,
    })
}

fn parse_subset(node: &KdlNode, context: &ParseContext<'_>) -> Result<SubsetDecl, SourceError> {
    let mut field_filters = BTreeMap::new();
    for entry in node.entries() {
        let Some(entry_name) = entry.name() else {
            continue;
        };
        let key = entry_name.value();
        if matches!(
            key,
            "from" | "filter_by" | "eq" | "ge" | "geq" | "le" | "leq"
        ) {
            continue;
        }
        let value = literal_from_arg(node, entry.value(), context)?;
        field_filters.insert(key.to_string(), value);
    }

    Ok(SubsetDecl {
        name: first_arg_string(node, 0, context)?,
        source: property_string(node, "from", context)?,
        field_filters,
        filter_by: optional_property_string(node, "filter_by", context)?,
        comparators: parse_filter_comparators(node, context)?,
    })
}

fn parse_param(node: &KdlNode, context: &ParseContext<'_>) -> Result<ParamDecl, SourceError> {
    let declaration_indices = declaration_indices(node, context)?;
    let indices = declaration_indices
        .iter()
        .map(|index| index.name.clone())
        .collect::<Vec<_>>();
    let index_by = optional_property_string(node, "index_by", context)?;
    let uses_index_children = node
        .iter_children()
        .any(|child| child.name().value() == "index");

    let filter_expression = parse_optional_filter_expression(node, context)?;
    let parsed_filter_expression = filter_expression
        .as_deref()
        .map(parse_value_formula)
        .transpose()
        .map_err(|error| algebra_error(node, error.to_string(), context))?;

    Ok(ParamDecl {
        name: first_arg_string(node, 0, context)?,
        value: positional_value(node, &indices, context)?,
        indices,
        from: optional_property_string(node, "from", context)?,
        index_by,
        uses_index_children,
        reduce: parse_reduce(node, context)?,
        filter_expression,
        parsed_filter_expression,
        filter_by: optional_property_string(node, "filter_by", context)?,
        comparators: parse_filter_comparators(node, context)?,
        units: optional_property_string(node, "units", context)?,
    })
}

fn parse_set(node: &KdlNode, context: &ParseContext<'_>) -> Result<SetDecl, SourceError> {
    let mut subset_of = optional_property_string(node, "subset_of", context)?;
    let mut filter_expression = parse_optional_filter_expression(node, context)?;
    let mut members = Vec::new();

    for child in node.iter_children() {
        match child.name().value() {
            "in" => {
                subset_of = Some(first_arg_string(child, 0, context)?);
            }
            "filter" => {
                filter_expression = Some(property_string(child, "expression", context)?);
            }
            // Top-level explicit set members are represented as child nodes.
            member => {
                if !child.entries().is_empty() {
                    return Err(SourceError::UnsupportedDeclaration {
                        name: member.to_string(),
                        path: context.path.to_path_buf(),
                        source_text: Box::new(context.source_text.clone()),
                        span: child.span(),
                    });
                }
                members.push(LiteralValue::String(member.to_string()));
            }
        }
    }

    let parsed_filter_expression = filter_expression
        .as_deref()
        .map(parse_value_formula)
        .transpose()
        .map_err(|error| algebra_error(node, error.to_string(), context))?;

    Ok(SetDecl {
        name: first_arg_string(node, 0, context)?,
        alias: node
            .get("alias")
            .and_then(KdlValue::as_string)
            .map(ToString::to_string),
        source: optional_property_string(node, "from", context)?,
        subset_of,
        members,
        filter_expression,
        parsed_filter_expression,
        filter_by: optional_property_string(node, "filter_by", context)?,
        comparators: parse_filter_comparators(node, context)?,
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
        indices: declaration_indices(node, context)?,
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

fn declaration_indices(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Vec<IndexDecl>, SourceError> {
    let mut indices = Vec::new();

    if let Some(property_index) = optional_property_string(node, "index_by", context)? {
        indices.push(IndexDecl {
            name: property_index.clone(),
            domain: Some(property_index),
        });
    }

    for child in node.iter_children() {
        match child.name().value() {
            "lower" | "upper" | "filter" => {}
            "index" => {
                let index_name = first_arg_string(child, 0, context)?;
                let domain = child
                    .iter_children()
                    .find(|grandchild| grandchild.name().value() == "in")
                    .map(|in_node| first_arg_string(in_node, 0, context))
                    .transpose()?
                    .or_else(|| {
                        child
                            .get("in")
                            .and_then(KdlValue::as_string)
                            .map(ToString::to_string)
                    })
                    .unwrap_or_else(|| index_name.clone());
                indices.push(IndexDecl {
                    name: index_name,
                    domain: Some(domain),
                });
            }
            _ => {}
        }
    }

    Ok(indices)
}

fn parse_reduce(node: &KdlNode, context: &ParseContext<'_>) -> Result<Option<String>, SourceError> {
    if let Some(reducer) = optional_property_string(node, "reduce", context)? {
        return Ok(Some(reducer));
    }

    for child in node.iter_children() {
        if child.name().value() == "reduce" {
            return Ok(Some(first_arg_string(child, 0, context)?));
        }
    }

    Ok(None)
}

fn parse_filter_comparators(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<FilterComparators, SourceError> {
    Ok(FilterComparators {
        eq: parse_optional_literal_property(node, "eq", context)?,
        ge: parse_optional_literal_property(node, "ge", context)?,
        geq: parse_optional_literal_property(node, "geq", context)?,
        le: parse_optional_literal_property(node, "le", context)?,
        leq: parse_optional_literal_property(node, "leq", context)?,
    })
}

fn parse_optional_filter_expression(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Option<String>, SourceError> {
    for child in node.iter_children() {
        if child.name().value() == "filter" {
            return Ok(Some(algebra_text_from_node(child, context)?));
        }
    }

    Ok(None)
}

fn algebra_text_from_node(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<String, SourceError> {
    if let Ok(expression) = property_string(node, "expression", context) {
        return Ok(expression);
    }
    if let Ok(formula) = child_arg_string(node, "formula", 0, context) {
        return Ok(formula);
    }

    Err(missing_property_error(node, "expression", context))
}

fn parse_optional_literal_property(
    node: &KdlNode,
    property: &'static str,
    context: &ParseContext<'_>,
) -> Result<Option<LiteralValue>, SourceError> {
    let Some(value) = node.get(property) else {
        return Ok(None);
    };
    literal_from_arg(node, value, context).map(Some)
}

fn positional_value(
    node: &KdlNode,
    indices: &[String],
    context: &ParseContext<'_>,
) -> Result<Option<LiteralValue>, SourceError> {
    if indices.is_empty() {
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

fn parse_scenario(node: &KdlNode, context: &ParseContext<'_>) -> Result<ScenarioDecl, SourceError> {
    let mut horizon = HorizonDecl::default();
    let mut data = Vec::new();
    let mut model_use = None;
    let mut reports = Vec::new();

    for child in node.iter_children() {
        match child.name().value() {
            "data" => {
                if child.children().is_some() {
                    return Err(SourceError::InvalidValue {
                        node: child.name().value().to_string(),
                        field: "scenario data must not have child blocks".to_string(),
                        path: context.path.to_path_buf(),
                        source_text: Box::new(context.source_text.clone()),
                        span: child.span(),
                    });
                }
                data.push(DataBindingDecl {
                    name: first_arg_string(child, 0, context)?,
                    source: property_string(child, "from", context)?,
                });
            }
            "use" => model_use = Some(first_arg_string(child, 0, context)?),
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
            // Legacy horizon support, optional in the canonical spec.
            "horizon" => {
                horizon.steps = property_usize(child, "steps", context)?;
                horizon.resolution = property_string(child, "resolution", context)?;
            }
            other => {
                return Err(SourceError::UnsupportedDeclaration {
                    name: other.to_string(),
                    path: context.path.to_path_buf(),
                    source_text: Box::new(context.source_text.clone()),
                    span: child.span(),
                });
            }
        }
    }

    Ok(ScenarioDecl {
        name: first_arg_string(node, 0, context)?,
        horizon,
        data,
        set_bindings: Vec::new(),
        assets: Vec::new(),
        instances: Vec::new(),
        technologies: Vec::new(),
        operations: Vec::new(),
        rules: Vec::new(),
        model_use,
        objective: None,
        reports,
        custom_sets: BTreeMap::new(),
    })
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
        let mut generation_bindings = Vec::new();
        let mut generation_filters = Vec::new();
        let mut expression = optional_property_string(child, "expression", context)?;

        for grandchild in child.iter_children() {
            match grandchild.name().value() {
                "index" => {
                    generation_bindings.push(parse_constraint_index_binding(grandchild, context)?);
                }
                "over" => {
                    generation_bindings.push(GenerationBinding {
                        variable: first_arg_string(grandchild, 0, context)?,
                        domain: property_string(grandchild, "in", context)?,
                    });
                }
                "if" => {
                    generation_filters.push(property_string(grandchild, "expression", context)?);
                }
                "when" => generation_filters.push(first_arg_string(grandchild, 0, context)?),
                "expression" | "expr" => {
                    expression = Some(algebra_text_from_node(grandchild, context)?);
                }
                // Parsed at semantic/lowering stage, but accepted at source parse stage.
                "slack" => {}
                other => {
                    return Err(SourceError::UnsupportedDeclaration {
                        name: other.to_string(),
                        path: context.path.to_path_buf(),
                        source_text: Box::new(context.source_text.clone()),
                        span: grandchild.span(),
                    });
                }
            }
        }

        let expression =
            expression.ok_or_else(|| missing_node_error("expression", child, context))?;
        let generation_filter = if generation_filters.is_empty() {
            None
        } else {
            Some(generation_filters.join(" and "))
        };

        constraints.push(ConstraintDecl {
            name: constraint_name(child, index),
            parsed_expression: parse_constraint_formula(&expression)
                .map_err(|error| algebra_error(child, error.to_string(), context))?,
            generation_bindings,
            parsed_generation_filter: generation_filter
                .as_deref()
                .map(parse_value_formula)
                .transpose()
                .map_err(|error| algebra_error(child, error.to_string(), context))?,
            generation_filter,
            expression,
        });
    }
    Ok(constraints)
}

fn parse_constraint_index_binding(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<GenerationBinding, SourceError> {
    let variable = first_arg_string(node, 0, context)?;
    let domain = node
        .iter_children()
        .find(|child| child.name().value() == "in")
        .map(|child| first_arg_string(child, 0, context))
        .transpose()?
        .or_else(|| {
            node.get("in")
                .and_then(KdlValue::as_string)
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| variable.clone());

    Ok(GenerationBinding { variable, domain })
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
