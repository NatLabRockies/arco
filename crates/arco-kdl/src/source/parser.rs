use crate::ObjectiveSense;
use crate::algebra::parse_value_formula;
use crate::source::ast::{
    BoundExpr, DataBindingDecl, DataDecl, DataIndexDecl, ExpressionDecl, IncludeDecl, IndexDecl,
    ModelDecl, ParsedSource, ProjectionDecl, ReportDecl, ReportKind, ScenarioDecl, SetDecl,
    SourceProgram, VariableKindDecl,
};
use crate::source::error::SourceError;
use crate::source::parser_constraints::{parse_constraint, parse_constraints};
use crate::source::parser_helpers::{
    ParseContext, algebra_error, algebra_text_from_node, declaration_indices, first_arg_string,
    invalid_value_error, missing_node_error, optional_property_literal, optional_property_string,
    parse_constraint_index_binding, parse_optimize, parse_optional_filter_expression, parse_reduce,
    positional_value, property_string, unsupported_declaration_error,
};
use crate::source::surface::{format_surface_document, normalize_surface_syntax};
use kdl::{KdlDocument, KdlError, KdlNode, KdlValue};
use miette::NamedSource;
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Declaration {
    Include,
    Set,
    Data,
    Param,
    Model,
    Projection,
    Scenario,
    Map,
    Alias,
    Index,
    In,
    Filter,
    Reduce,
    If,
    Formula,
    Bounds,
    Lower,
    Upper,
    From,
    To,
    Control,
    Var,
    Expression,
    Constraint,
    Minimize,
    Maximize,
    Use,
    Report,
}

impl Declaration {
    fn from_node(node: &KdlNode) -> Option<Self> {
        match node.name().value() {
            "include" => Some(Self::Include),
            "set" => Some(Self::Set),
            "data" => Some(Self::Data),
            "param" => Some(Self::Param),
            "model" => Some(Self::Model),
            "projection" => Some(Self::Projection),
            "scenario" => Some(Self::Scenario),
            "map" => Some(Self::Map),
            "alias" => Some(Self::Alias),
            "index" => Some(Self::Index),
            "in" => Some(Self::In),
            "filter" => Some(Self::Filter),
            "reduce" => Some(Self::Reduce),
            "if" => Some(Self::If),
            "formula" => Some(Self::Formula),
            "bounds" => Some(Self::Bounds),
            "lower" => Some(Self::Lower),
            "upper" => Some(Self::Upper),
            "from" => Some(Self::From),
            "to" => Some(Self::To),
            "control" => Some(Self::Control),
            "var" => Some(Self::Var),
            "expression" => Some(Self::Expression),
            "constraint" => Some(Self::Constraint),
            "minimize" => Some(Self::Minimize),
            "maximize" => Some(Self::Maximize),
            "use" => Some(Self::Use),
            "report" => Some(Self::Report),
            _ => None,
        }
    }
}

pub fn parse_program_file(path: &Path) -> Result<ParsedSource, SourceError> {
    info!(path = %path.display(), status = "ok", "parsing source file");
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    parse_program_file_with_base(path, base_dir, true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdlFormatMode {
    ArcoSurface,
    KdlCompatible,
}

pub fn format_program_text(text: &str) -> Result<String, KdlError> {
    format_program_text_with_mode(text, KdlFormatMode::ArcoSurface)
}

pub fn format_program_text_with_mode(text: &str, mode: KdlFormatMode) -> Result<String, KdlError> {
    let normalized = normalize_surface_syntax(text);
    let mut document: KdlDocument = normalized.parse()?;
    document.autoformat();
    match mode {
        KdlFormatMode::ArcoSurface => Ok(format_surface_document(&document)),
        KdlFormatMode::KdlCompatible => Ok(document.to_string()),
    }
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

fn parse_program_file_with_base(
    path: &Path,
    base_dir: &Path,
    allow_includes: bool,
) -> Result<ParsedSource, SourceError> {
    let text = fs::read_to_string(path).map_err(|source| SourceError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let normalized = normalize_surface_syntax(&text);
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
    let program = if allow_includes {
        parse_entrypoint_document(&document, &context, base_dir)?
    } else {
        parse_document(&document, &context)?
    };

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
        match Declaration::from_node(node) {
            Some(Declaration::Include) => program.includes.push(parse_include(node, context)?),
            Some(Declaration::Set) => program.sets.push(parse_set(node, context)?),
            Some(Declaration::Data) => program.data.push(parse_data(node, context)?),
            Some(Declaration::Param) => program.params.push(parse_param(node, context)?),
            Some(Declaration::Model) => program.models.push(parse_model(node, context)?),
            Some(Declaration::Projection) => {
                program.projections.push(parse_projection(node, context)?);
            }
            Some(Declaration::Scenario) => program.scenarios.push(parse_scenario(node, context)?),
            _ => {
                return Err(unsupported_declaration_error(
                    node,
                    node.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(program)
}

fn parse_entrypoint_document(
    document: &KdlDocument,
    context: &ParseContext<'_>,
    base_dir: &Path,
) -> Result<SourceProgram, SourceError> {
    let mut program = SourceProgram::default();

    for node in document.nodes() {
        match Declaration::from_node(node) {
            Some(Declaration::Include) => {
                let include = parse_include(node, context)?;
                merge_top_level_include(&mut program, &include, node, context, base_dir)?;
            }
            Some(Declaration::Set) => program.sets.push(parse_set(node, context)?),
            Some(Declaration::Data) => program.data.push(parse_data(node, context)?),
            Some(Declaration::Param) => program.params.push(parse_param(node, context)?),
            Some(Declaration::Model) => program
                .models
                .push(parse_model_with_includes(node, context, base_dir)?),
            Some(Declaration::Projection) => {
                program.projections.push(parse_projection(node, context)?);
            }
            Some(Declaration::Scenario) => program.scenarios.push(parse_scenario(node, context)?),
            _ => {
                return Err(unsupported_declaration_error(
                    node,
                    node.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(program)
}

fn merge_top_level_include(
    target: &mut SourceProgram,
    include: &IncludeDecl,
    include_node: &KdlNode,
    context: &ParseContext<'_>,
    base_dir: &Path,
) -> Result<(), SourceError> {
    let included_path = base_dir.join(&include.path);
    let included = parse_program_file_with_base(&included_path, base_dir, false)?;
    reject_nested_includes(&included.program, include_node, context)?;
    if !included.program.scenarios.is_empty() {
        return Err(include_error(
            include_node,
            "included files must not define `scenario` declarations".to_string(),
            context,
        ));
    }

    target.params.extend(included.program.params);
    target.data.extend(included.program.data);
    target.models.extend(included.program.models);
    target.sets.extend(included.program.sets);
    target.projections.extend(included.program.projections);

    Ok(())
}

fn reject_nested_includes(
    program: &SourceProgram,
    include_node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<(), SourceError> {
    if !program.includes.is_empty()
        || program
            .models
            .iter()
            .any(|model| !model.includes.is_empty())
    {
        return Err(include_error(
            include_node,
            "included files must not contain `include` declarations".to_string(),
            context,
        ));
    }
    Ok(())
}

fn parse_include(node: &KdlNode, context: &ParseContext<'_>) -> Result<IncludeDecl, SourceError> {
    if node.children().is_some() {
        return Err(include_error(
            node,
            "include declarations must not have child blocks".to_string(),
            context,
        ));
    }

    let path = first_arg_string(node, 0, context)?;
    let include_path = Path::new(&path);
    if include_path.is_absolute() || include_path.has_root() {
        return Err(include_error(
            node,
            "include paths must be relative to the entrypoint file".to_string(),
            context,
        ));
    }

    Ok(IncludeDecl { path })
}

fn include_error(node: &KdlNode, reason: String, context: &ParseContext<'_>) -> SourceError {
    SourceError::InvalidInclude {
        reason,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

fn parse_model(node: &KdlNode, context: &ParseContext<'_>) -> Result<ModelDecl, SourceError> {
    let mut sets = Vec::new();
    let mut parameters = Vec::new();
    let mut controls = Vec::new();
    let mut expressions = Vec::new();
    let mut includes = Vec::new();
    let constraints = parse_constraints(node, context)?;
    let mut optimize = None;

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::Include) => includes.push(parse_include(child, context)?),
            Some(Declaration::Set) => sets.push(parse_set(child, context)?),
            Some(Declaration::Param) => parameters.push(parse_param(child, context)?),
            Some(Declaration::Control | Declaration::Var) => {
                controls.push(parse_control(child, context)?);
            }
            Some(Declaration::Expression) => expressions.push(parse_expression(child, context)?),
            Some(Declaration::Constraint) => {}
            Some(Declaration::Minimize) => {
                if optimize.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "multiple objectives are not allowed".to_string(),
                        context,
                    ));
                }
                optimize = Some(parse_optimize(child, ObjectiveSense::Minimize, context)?);
            }
            Some(Declaration::Maximize) => {
                if optimize.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "multiple objectives are not allowed".to_string(),
                        context,
                    ));
                }
                optimize = Some(parse_optimize(child, ObjectiveSense::Maximize, context)?);
            }
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(ModelDecl {
        name: first_arg_string(node, 0, context)?,
        includes,
        sets,
        parameters,
        controls,
        expressions,
        constraints,
        optimize: optimize
            .ok_or_else(|| missing_node_error("minimize_or_maximize", node, context))?,
    })
}

fn parse_model_with_includes(
    node: &KdlNode,
    context: &ParseContext<'_>,
    base_dir: &Path,
) -> Result<ModelDecl, SourceError> {
    let mut sets = Vec::new();
    let mut parameters = Vec::new();
    let mut controls = Vec::new();
    let mut expressions = Vec::new();
    let mut constraints = Vec::new();
    let mut optimize = None;

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::Include) => {
                let include = parse_include(child, context)?;
                merge_model_include(
                    &mut sets,
                    &mut parameters,
                    &mut controls,
                    &mut expressions,
                    &mut constraints,
                    &mut optimize,
                    &include,
                    child,
                    context,
                    base_dir,
                )?;
            }
            Some(Declaration::Set) => sets.push(parse_set(child, context)?),
            Some(Declaration::Param) => parameters.push(parse_param(child, context)?),
            Some(Declaration::Control | Declaration::Var) => {
                controls.push(parse_control(child, context)?);
            }
            Some(Declaration::Expression) => expressions.push(parse_expression(child, context)?),
            Some(Declaration::Constraint) => {
                let index = constraints.len();
                constraints.push(parse_constraint(child, index, context)?);
            }
            Some(Declaration::Minimize) => {
                if optimize.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "multiple objectives are not allowed".to_string(),
                        context,
                    ));
                }
                optimize = Some(parse_optimize(child, ObjectiveSense::Minimize, context)?);
            }
            Some(Declaration::Maximize) => {
                if optimize.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "multiple objectives are not allowed".to_string(),
                        context,
                    ));
                }
                optimize = Some(parse_optimize(child, ObjectiveSense::Maximize, context)?);
            }
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(ModelDecl {
        name: first_arg_string(node, 0, context)?,
        includes: Vec::new(),
        sets,
        parameters,
        controls,
        expressions,
        constraints,
        optimize: optimize
            .ok_or_else(|| missing_node_error("minimize_or_maximize", node, context))?,
    })
}

#[allow(clippy::too_many_arguments)]
fn merge_model_include(
    sets: &mut Vec<SetDecl>,
    parameters: &mut Vec<crate::source::ParamDecl>,
    controls: &mut Vec<crate::source::ControlDecl>,
    expressions: &mut Vec<ExpressionDecl>,
    constraints: &mut Vec<crate::source::ConstraintDecl>,
    optimize: &mut Option<crate::source::ObjectiveDecl>,
    include: &IncludeDecl,
    include_node: &KdlNode,
    context: &ParseContext<'_>,
    base_dir: &Path,
) -> Result<(), SourceError> {
    let included_path = base_dir.join(&include.path);
    let included = parse_model_fragment_file(&included_path)?;
    if !included.includes.is_empty() {
        return Err(include_error(
            include_node,
            "included files must not contain `include` declarations".to_string(),
            context,
        ));
    }

    sets.extend(included.sets);
    parameters.extend(included.parameters);
    controls.extend(included.controls);
    expressions.extend(included.expressions);
    for mut constraint in included.constraints {
        if constraint.name_inferred {
            constraint.name = format!("constraint_{}", constraints.len() + 1);
        }
        constraints.push(constraint);
    }
    if let Some(included_objective) = included.optimize {
        if optimize.is_some() {
            return Err(invalid_value_error(
                include_node,
                "multiple objectives are not allowed".to_string(),
                context,
            ));
        }
        *optimize = Some(included_objective);
    }

    Ok(())
}

#[derive(Default)]
struct ModelFragment {
    includes: Vec<IncludeDecl>,
    sets: Vec<SetDecl>,
    parameters: Vec<crate::source::ParamDecl>,
    controls: Vec<crate::source::ControlDecl>,
    expressions: Vec<ExpressionDecl>,
    constraints: Vec<crate::source::ConstraintDecl>,
    optimize: Option<crate::source::ObjectiveDecl>,
}

fn parse_model_fragment_file(path: &Path) -> Result<ModelFragment, SourceError> {
    let text = fs::read_to_string(path).map_err(|source| SourceError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let normalized = normalize_surface_syntax(&text);
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
    let mut fragment = ModelFragment::default();

    for node in document.nodes() {
        match Declaration::from_node(node) {
            Some(Declaration::Include) => fragment.includes.push(parse_include(node, &context)?),
            Some(Declaration::Set) => fragment.sets.push(parse_set(node, &context)?),
            Some(Declaration::Param) => fragment.parameters.push(parse_param(node, &context)?),
            Some(Declaration::Control | Declaration::Var) => {
                fragment.controls.push(parse_control(node, &context)?);
            }
            Some(Declaration::Expression) => {
                fragment.expressions.push(parse_expression(node, &context)?);
            }
            Some(Declaration::Constraint) => {
                let index = fragment.constraints.len();
                fragment
                    .constraints
                    .push(parse_constraint(node, index, &context)?);
            }
            Some(Declaration::Minimize) => {
                if fragment.optimize.is_some() {
                    return Err(invalid_value_error(
                        node,
                        "multiple objectives are not allowed".to_string(),
                        &context,
                    ));
                }
                fragment.optimize = Some(parse_optimize(node, ObjectiveSense::Minimize, &context)?);
            }
            Some(Declaration::Maximize) => {
                if fragment.optimize.is_some() {
                    return Err(invalid_value_error(
                        node,
                        "multiple objectives are not allowed".to_string(),
                        &context,
                    ));
                }
                fragment.optimize = Some(parse_optimize(node, ObjectiveSense::Maximize, &context)?);
            }
            Some(
                Declaration::Data
                | Declaration::Model
                | Declaration::Projection
                | Declaration::Scenario,
            ) => {
                return Err(include_error(
                    node,
                    format!(
                        "`{}` is not allowed in a model-scope include",
                        node.name().value()
                    ),
                    &context,
                ));
            }
            _ => {
                return Err(unsupported_declaration_error(
                    node,
                    node.name().value(),
                    &context,
                ));
            }
        }
    }

    Ok(fragment)
}

fn parse_data(node: &KdlNode, context: &ParseContext<'_>) -> Result<DataDecl, SourceError> {
    let mut maps = Vec::new();
    let mut sets = Vec::new();
    let mut indices = Vec::new();
    let mut parameters = Vec::new();

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::Map) => maps.push(crate::source::MapDecl {
                name: first_arg_string(child, 0, context)?,
                source: optional_property_string(child, "from", context)?,
            }),
            Some(Declaration::Alias) => maps.push(crate::source::MapDecl {
                name: first_arg_string(child, 0, context)?,
                source: optional_property_string(child, "column", context)?,
            }),
            Some(Declaration::Set) => sets.push(parse_set(child, context)?),
            Some(Declaration::Index) => {
                indices.push(DataIndexDecl {
                    columns: collect_string_args(child, context)?,
                });
            }
            Some(Declaration::Param) => parameters.push(parse_param(child, context)?),
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(DataDecl {
        name: first_arg_string(node, 0, context)?,
        source: optional_property_string(node, "source", context)?.ok_or_else(|| {
            SourceError::MissingProperty {
                node: node.name().value().to_string(),
                property: "source",
                path: context.path.to_path_buf(),
                source_text: Box::new(context.source_text.clone()),
                span: node.span(),
            }
        })?,
        maps,
        sets,
        indices,
        parameters,
    })
}

fn parse_param(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<crate::source::ParamDecl, SourceError> {
    if node.get("index_by").is_some() {
        return Err(invalid_value_error(
            node,
            "`index_by` is not supported; use `index`".to_string(),
            context,
        ));
    }

    let declaration_indices = declaration_indices(node, context)?;
    let indices = declaration_indices
        .iter()
        .map(|index| index.name.clone())
        .collect::<Vec<_>>();
    let index = optional_property_string(node, "index", context)?;
    let uses_index_children = node
        .iter_children()
        .any(|child| Declaration::from_node(child) == Some(Declaration::Index));

    let filter_expression = parse_optional_filter_expression(node, context)?;
    let parsed_filter_expression = filter_expression
        .as_deref()
        .map(parse_value_formula)
        .transpose()
        .map_err(|error| algebra_error(node, error.to_string(), context))?;

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::Index | Declaration::Filter | Declaration::Reduce) => {}
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(crate::source::ParamDecl {
        name: first_arg_string(node, 0, context)?,
        value: positional_value(node, &indices, context)?,
        indices,
        from: optional_property_string(node, "from", context)?,
        index,
        uses_index_children,
        reduce: parse_reduce(node, context)?,
        filter_expression,
        parsed_filter_expression,
        units: optional_property_string(node, "units", context)?,
    })
}

fn parse_set(node: &KdlNode, context: &ParseContext<'_>) -> Result<SetDecl, SourceError> {
    if node.get("from").is_some() {
        return Err(invalid_value_error(
            node,
            "`from` is not supported on `set` declarations".to_string(),
            context,
        ));
    }

    let mut subset_of = None;
    let mut filter_expression = parse_optional_filter_expression(node, context)?;
    let mut members = Vec::new();
    let mut tuple_indices = Vec::new();

    if let Some(parent) = optional_property_string(node, "in", context)? {
        subset_of = Some(parent);
    }

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::In) => subset_of = Some(first_arg_string(child, 0, context)?),
            Some(Declaration::Index) => {
                let index_name = first_arg_string(child, 0, context)?;
                let domain = child
                    .iter_children()
                    .find(|grandchild| Declaration::from_node(grandchild) == Some(Declaration::In))
                    .map(|in_node| first_arg_string(in_node, 0, context))
                    .transpose()?
                    .or_else(|| {
                        child
                            .get("in")
                            .and_then(KdlValue::as_string)
                            .map(ToString::to_string)
                    })
                    .unwrap_or_else(|| index_name.clone());
                tuple_indices.push(IndexDecl {
                    name: index_name,
                    domain: Some(domain),
                });
            }
            Some(Declaration::Filter) => {
                filter_expression = Some(algebra_text_from_node(child, context)?);
            }
            _ => {
                let member = child.name().value();
                if !child.entries().is_empty() {
                    return Err(unsupported_declaration_error(child, member, context));
                }
                members.push(crate::source::LiteralValue::String(member.to_string()));
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
            .get("as")
            .or_else(|| node.get("alias"))
            .and_then(KdlValue::as_string)
            .map(ToString::to_string),
        rule_id: node
            .get("id")
            .or_else(|| node.get("rule_id"))
            .and_then(KdlValue::as_string)
            .map(ToString::to_string),
        subset_of,
        tuple_indices,
        members,
        filter_expression,
        parsed_filter_expression,
    })
}

fn parse_control(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<crate::source::ControlDecl, SourceError> {
    if node.get("index_by").is_some() {
        return Err(invalid_value_error(
            node,
            "`index_by` is not supported; use `index`".to_string(),
            context,
        ));
    }

    let kind = optional_property_string(node, "kind", context)?
        .map(|value| parse_variable_kind_decl(node, &value, context))
        .transpose()?;

    let value = optional_property_literal(node, "value", context)?;
    let mut lower = optional_property_literal(node, "lower", context)?.map(BoundExpr::Literal);
    let mut upper = optional_property_literal(node, "upper", context)?.map(BoundExpr::Literal);

    if let Some(value) = value {
        lower = Some(BoundExpr::Literal(value.clone()));
        upper = Some(BoundExpr::Literal(value));
    }

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::Bounds) => {
                for bound in child.iter_children() {
                    match Declaration::from_node(bound) {
                        Some(Declaration::Lower) => {
                            let formula_text = algebra_text_from_node(bound, context)?;
                            lower = Some(BoundExpr::Formula(
                                parse_value_formula(&formula_text)
                                    .map_err(|e| algebra_error(bound, e.to_string(), context))?,
                            ));
                        }
                        Some(Declaration::Upper) => {
                            let formula_text = algebra_text_from_node(bound, context)?;
                            upper = Some(BoundExpr::Formula(
                                parse_value_formula(&formula_text)
                                    .map_err(|e| algebra_error(bound, e.to_string(), context))?,
                            ));
                        }
                        _ => {
                            return Err(unsupported_declaration_error(
                                bound,
                                bound.name().value(),
                                context,
                            ));
                        }
                    }
                }
            }
            Some(Declaration::Index) => {}
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(crate::source::ControlDecl {
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

fn parse_expression(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<ExpressionDecl, SourceError> {
    let mut generation_bindings = Vec::new();
    let mut generation_filters = Vec::new();
    let formula_from_property = optional_property_string(node, "expression", context)?;
    let mut formula_from_formula_child = None;
    let mut formula_from_expression_child = None;

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::Index) => {
                generation_bindings.push(parse_constraint_index_binding(child, context)?);
            }
            Some(Declaration::If) => {
                generation_filters.push(property_string(child, "expression", context)?);
            }
            Some(Declaration::Expression) => {
                if formula_from_expression_child.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "expression declarations support at most one `expression` child"
                            .to_string(),
                        context,
                    ));
                }
                formula_from_expression_child = Some(algebra_text_from_node(child, context)?);
            }
            Some(Declaration::Formula) => {
                if formula_from_formula_child.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "expression declarations support at most one `formula` child".to_string(),
                        context,
                    ));
                }
                if child.children().is_some() {
                    return Err(invalid_value_error(
                        child,
                        "`formula` children in expression declarations must use positional string form `formula \"...\"`; use `expression { ... }` for block algebra"
                            .to_string(),
                        context,
                    ));
                }
                formula_from_formula_child = Some(first_arg_string(child, 0, context)?);
            }
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    let formula_source_count = [
        formula_from_property.as_ref(),
        formula_from_formula_child.as_ref(),
        formula_from_expression_child.as_ref(),
    ]
    .into_iter()
    .flatten()
    .count();
    if formula_source_count > 1 {
        return Err(invalid_value_error(
            node,
            "expression declarations support only one formula source: `expression=...`, `formula ...`, or `expression { ... }`"
                .to_string(),
            context,
        ));
    }

    if generation_filters.len() > 1 {
        return Err(invalid_value_error(
            node,
            "expression declarations support at most one `if` child".to_string(),
            context,
        ));
    }

    let formula = formula_from_expression_child
        .or(formula_from_property)
        .or(formula_from_formula_child)
        .ok_or_else(|| missing_node_error("expression", node, context))?;
    let generation_filter = generation_filters.into_iter().next();

    if let Some((projection, op, target)) = parse_reduce_projection_formula(&formula) {
        let lowered = format!("__reduce_projection__(\"{projection}\", \"{op}\", \"{target}\")");
        return Ok(ExpressionDecl {
            name: first_arg_string(node, 0, context)?,
            parsed_formula: crate::algebra::Expr::Identifier(lowered.clone()),
            formula: lowered,
            abstraction: Some(crate::source::ExpressionAbstractionDecl::ReduceProjection {
                projection,
                op,
                target,
            }),
            generation_bindings,
            parsed_generation_filter: generation_filter
                .as_deref()
                .map(parse_value_formula)
                .transpose()
                .map_err(|error| algebra_error(node, error.to_string(), context))?,
            generation_filter,
        });
    }

    Ok(ExpressionDecl {
        name: first_arg_string(node, 0, context)?,
        parsed_formula: parse_value_formula(&formula)
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        formula,
        abstraction: None,
        generation_bindings,
        parsed_generation_filter: generation_filter
            .as_deref()
            .map(parse_value_formula)
            .transpose()
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        generation_filter,
    })
}

fn parse_reduce_projection_formula(formula: &str) -> Option<(String, String, String)> {
    let trimmed = formula.trim();
    if !trimmed.starts_with("reduce") {
        return None;
    }
    let after_reduce = trimmed["reduce".len()..].trim_start();

    let open_brace_rel = after_reduce.find('{')?;
    let head = after_reduce[..open_brace_rel].trim();
    let body_and_tail = &after_reduce[open_brace_rel + 1..];
    let close_brace_rel = body_and_tail.find('}')?;
    let body = body_and_tail[..close_brace_rel].trim();
    let tail = body_and_tail[close_brace_rel + 1..].trim();
    if !tail.is_empty() {
        return None;
    }

    let projection = head.trim_matches('"').trim().to_string();
    let mut body_parts = body.split_whitespace();
    let op = body_parts.next()?.trim_matches('"').to_string();
    let target = body_parts.next()?.trim_matches('"').to_string();

    if projection.is_empty() || op.is_empty() || target.is_empty() || body_parts.next().is_some() {
        return None;
    }

    Some((projection, op, target))
}

fn parse_projection(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<ProjectionDecl, SourceError> {
    let mut from_domain = None;
    let mut to_keys = Vec::new();

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::From) => {
                from_domain = Some(parse_projection_from_domain(child, context)?);
            }
            Some(Declaration::To) => {
                to_keys = parse_projection_to_keys(child, context)?;
            }
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    if to_keys.is_empty() {
        return Err(missing_node_error("to", node, context));
    }

    Ok(ProjectionDecl {
        name: first_arg_string(node, 0, context)?,
        from_domain: from_domain.ok_or_else(|| missing_node_error("from", node, context))?,
        to_keys,
    })
}

fn parse_projection_from_domain(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<String, SourceError> {
    if let Some(value) = node.get(0) {
        let Some(value_text) = value.as_string() else {
            return Err(invalid_value_error(node, "argument 0".to_string(), context));
        };
        return Ok(value_text.to_string());
    }

    let mut nested = node.iter_children();
    let Some(domain_node) = nested.next() else {
        return Err(missing_node_error("from", node, context));
    };

    if nested.next().is_some()
        || !domain_node.entries().is_empty()
        || domain_node.children().is_some()
    {
        return Err(invalid_value_error(
            node,
            "from domain block must contain exactly one bare node".to_string(),
            context,
        ));
    }

    Ok(domain_node.name().value().to_string())
}

fn parse_projection_to_keys(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Vec<String>, SourceError> {
    let values = collect_string_args(node, context)?;
    if !values.is_empty() {
        return Ok(values);
    }

    let mut keys = Vec::new();
    for key_node in node.iter_children() {
        if !key_node.entries().is_empty() || key_node.children().is_some() {
            return Err(invalid_value_error(
                node,
                "to key block must contain only bare key nodes".to_string(),
                context,
            ));
        }
        keys.push(key_node.name().value().to_string());
    }

    Ok(keys)
}

fn collect_string_args(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Vec<String>, SourceError> {
    let mut values = Vec::new();
    let mut position = 0;

    while let Some(value) = node.get(position) {
        let Some(value_text) = value.as_string() else {
            return Err(invalid_value_error(
                node,
                format!("argument {position}"),
                context,
            ));
        };
        values.push(value_text.to_string());
        position += 1;
    }

    Ok(values)
}

fn parse_scenario(node: &KdlNode, context: &ParseContext<'_>) -> Result<ScenarioDecl, SourceError> {
    let mut data = Vec::new();
    let mut model_use = None;
    let mut reports = Vec::new();

    for child in node.iter_children() {
        match Declaration::from_node(child) {
            Some(Declaration::Data) => {
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
                    source: property_string(child, "source", context)?,
                });
            }
            Some(Declaration::Use) => model_use = Some(first_arg_string(child, 0, context)?),
            Some(Declaration::Report) => {
                let first = first_arg_string(child, 0, context)?;
                if first.as_str() == "dual" {
                    let target = first_arg_string(child, 1, context)?;
                    reports.push(ReportDecl {
                        kind: ReportKind::Dual,
                        target,
                        filter_expression: None,
                        parsed_filter_expression: None,
                    });
                } else {
                    let filter_expression = parse_optional_filter_expression(child, context)?;
                    let parsed_filter_expression = filter_expression
                        .as_deref()
                        .map(parse_value_formula)
                        .transpose()
                        .map_err(|error| algebra_error(child, error.to_string(), context))?;
                    reports.push(ReportDecl {
                        kind: ReportKind::Scalar,
                        target: first,
                        filter_expression,
                        parsed_filter_expression,
                    });
                }
            }
            _ => {
                return Err(unsupported_declaration_error(
                    child,
                    child.name().value(),
                    context,
                ));
            }
        }
    }

    Ok(ScenarioDecl {
        name: first_arg_string(node, 0, context)?,
        data,
        model_use,
        reports,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_program_text;
    use std::fs;
    use std::path::PathBuf;

    fn fixture_text(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name);
        fs::read_to_string(path).expect("fixture should load")
    }

    #[test]
    fn parses_low_level_model_and_scenario() {
        let path = PathBuf::from("test.kdl");
        let text = fixture_text("parser_unit_parses_low_level_model_and_scenario_text.kdl");

        let parsed = parse_program_text(&text, &path).expect("program parses");
        assert_eq!(parsed.program.models.len(), 1);
        assert_eq!(parsed.program.scenarios.len(), 1);
    }

    #[test]
    fn var_keyword_parses_same_as_control() {
        let path = PathBuf::from("test.kdl");

        let with_var = fixture_text("parser_unit_var_keyword_parses_same_as_control_with_var.kdl");

        let with_control =
            fixture_text("parser_unit_var_keyword_parses_same_as_control_with_control.kdl");

        let var_parsed = parse_program_text(&with_var, &path).expect("var syntax parses");
        let control_parsed =
            parse_program_text(&with_control, &path).expect("control syntax parses");

        // Both should produce identical control declarations
        assert_eq!(var_parsed.program.models.len(), 1);
        assert_eq!(control_parsed.program.models.len(), 1);

        let var_controls = &var_parsed.program.models[0].controls;
        let control_controls = &control_parsed.program.models[0].controls;

        assert_eq!(var_controls.len(), control_controls.len());
        assert_eq!(var_controls[0].name, control_controls[0].name);
        assert_eq!(
            var_controls[0].indices.len(),
            control_controls[0].indices.len()
        );
    }

    #[test]
    fn alias_keyword_parses_same_as_map() {
        let path = PathBuf::from("test.kdl");

        let with_alias =
            fixture_text("parser_unit_alias_keyword_parses_same_as_map_with_alias.kdl");

        let with_map = fixture_text("parser_unit_alias_keyword_parses_same_as_map_with_map.kdl");

        let alias_parsed = parse_program_text(&with_alias, &path).expect("alias syntax parses");
        let map_parsed = parse_program_text(&with_map, &path).expect("map syntax parses");

        assert_eq!(alias_parsed.program.data.len(), 1);
        assert_eq!(map_parsed.program.data.len(), 1);

        let alias_maps = &alias_parsed.program.data[0].maps;
        let map_maps = &map_parsed.program.data[0].maps;

        assert_eq!(alias_maps.len(), map_maps.len());
        assert_eq!(alias_maps[0].name, map_maps[0].name);
        assert_eq!(alias_maps[0].source, map_maps[0].source);
    }

    #[test]
    fn data_from_property_is_rejected() {
        let path = PathBuf::from("test.kdl");

        let with_from = fixture_text("parser_unit_data_from_property_is_rejected_with_from.kdl");

        let error = parse_program_text(&with_from, &path).expect_err("from= should be rejected");
        assert!(error.to_string().contains("source"));
    }

    #[test]
    fn set_as_property_parses_same_as_alias() {
        let path = PathBuf::from("test.kdl");

        let with_as = fixture_text("parser_unit_set_as_property_parses_same_as_alias_with_as.kdl");
        let with_alias =
            fixture_text("parser_unit_set_as_property_parses_same_as_alias_with_alias.kdl");

        let as_parsed = parse_program_text(&with_as, &path).expect("as= syntax parses");
        let alias_parsed = parse_program_text(&with_alias, &path).expect("alias= syntax parses");

        assert_eq!(as_parsed.program.sets.len(), 1);
        assert_eq!(alias_parsed.program.sets.len(), 1);

        assert_eq!(
            as_parsed.program.sets[0].alias,
            alias_parsed.program.sets[0].alias
        );
        assert_eq!(as_parsed.program.sets[0].alias, Some("g".to_string()));
    }

    #[test]
    fn where_keyword_is_rejected() {
        let path = PathBuf::from("test.kdl");

        let with_where = fixture_text("parser_unit_where_keyword_is_rejected_with_where.kdl");

        parse_program_text(&with_where, &path).expect_err("where syntax should fail");
    }

    #[test]
    fn mixed_old_and_new_syntax_parses() {
        let path = PathBuf::from("test.kdl");

        // Mix of legacy-compatible and canonical forms we still support.
        let mixed = fixture_text("parser_unit_mixed_old_and_new_syntax_parses_mixed.kdl");

        let parsed = parse_program_text(&mixed, &path).expect("mixed syntax parses");

        assert_eq!(parsed.program.data.len(), 1);
        assert_eq!(parsed.program.data[0].maps.len(), 2);

        assert_eq!(parsed.program.sets.len(), 2);
        assert_eq!(parsed.program.sets[0].alias, Some("o".to_string()));
        assert_eq!(parsed.program.sets[1].alias, Some("n".to_string()));

        assert_eq!(parsed.program.models.len(), 1);
        assert_eq!(parsed.program.models[0].controls.len(), 2);
    }
}
