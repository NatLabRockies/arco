use crate::ObjectiveSense;
use crate::algebra::{parse_constraint_formula, parse_value_formula};
use crate::source::ast::{GenerationBinding, IndexDecl, LiteralValue, ObjectiveDecl};
use crate::source::error::SourceError;
use kdl::{KdlNode, KdlValue};
use miette::NamedSource;
use std::path::Path;

pub(super) struct ParseContext<'a> {
    pub(super) path: &'a Path,
    pub(super) source_text: &'a NamedSource<String>,
}

pub(super) fn parse_constraint_index_binding(
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

pub(super) fn parse_optimize(
    node: &KdlNode,
    sense: ObjectiveSense,
    context: &ParseContext<'_>,
) -> Result<ObjectiveDecl, SourceError> {
    let expression = property_string(node, "expression", context)?;
    Ok(ObjectiveDecl {
        name: first_arg_string(node, 0, context)?,
        sense,
        parsed_expression: parse_value_formula(&expression)
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        expression,
    })
}

pub(super) fn parse_optional_filter_expression(
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

pub(super) fn parse_reduce(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Option<String>, SourceError> {
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

pub(super) fn declaration_indices(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Vec<IndexDecl>, SourceError> {
    let mut indices = Vec::new();

    if let Some(property_index) = optional_property_string(node, "index", context)? {
        indices.push(IndexDecl {
            name: property_index.clone(),
            domain: Some(property_index),
        });
    }

    for child in node.iter_children() {
        match child.name().value() {
            "filter" | "bounds" => {}
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

pub(super) fn algebra_text_from_node(
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

pub(super) fn positional_value(
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

pub(super) fn literal_from_arg(
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

pub(super) fn child_node<'a>(
    node: &'a KdlNode,
    name: &'static str,
    context: &ParseContext<'_>,
) -> Result<&'a KdlNode, SourceError> {
    node.children()
        .and_then(|children| children.get(name))
        .ok_or_else(|| missing_node_error(name, node, context))
}

pub(super) fn child_arg_string(
    node: &KdlNode,
    child_name: &'static str,
    index: usize,
    context: &ParseContext<'_>,
) -> Result<String, SourceError> {
    let child = child_node(node, child_name, context)?;
    first_arg_string(child, index, context)
}

pub(super) fn first_arg_string(
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

pub(super) fn property_string(
    node: &KdlNode,
    property: &'static str,
    context: &ParseContext<'_>,
) -> Result<String, SourceError> {
    node.get(property)
        .and_then(KdlValue::as_string)
        .map(ToString::to_string)
        .ok_or_else(|| missing_property_error(node, property, context))
}

pub(super) fn optional_property_string(
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

pub(super) fn optional_property_literal(
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

pub(super) fn missing_node_error(
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

pub(super) fn missing_argument_error(
    node: &KdlNode,
    index: usize,
    context: &ParseContext<'_>,
) -> SourceError {
    SourceError::MissingArgument {
        node: node.name().value().to_string(),
        index,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

pub(super) fn missing_property_error(
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

pub(super) fn invalid_value_error(
    node: &KdlNode,
    field: String,
    context: &ParseContext<'_>,
) -> SourceError {
    SourceError::InvalidValue {
        node: node.name().value().to_string(),
        field,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

pub(super) fn algebra_error(
    node: &KdlNode,
    reason: String,
    context: &ParseContext<'_>,
) -> SourceError {
    SourceError::InvalidAlgebra {
        node: node.name().value().to_string(),
        reason,
        path: context.path.to_path_buf(),
        source_text: Box::new(context.source_text.clone()),
        span: node.span(),
    }
}

pub(super) fn parse_constraint_formula_decl(
    expression: &str,
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<crate::algebra::ConstraintBody, SourceError> {
    parse_constraint_formula(expression)
        .map_err(|error| algebra_error(node, error.to_string(), context))
}

#[cfg(test)]
mod tests {
    use super::{ParseContext, parse_reduce};
    use miette::NamedSource;
    use std::path::Path;

    #[test]
    fn parse_reduce_accepts_property_or_child() {
        let source = NamedSource::new("test.kdl", "param p reduce=\"sum\"".to_string());
        let context = ParseContext {
            path: Path::new("test.kdl"),
            source_text: &source,
        };
        let node: kdl::KdlNode = "param p reduce=\"sum\"".parse().expect("kdl node");
        let reduce = parse_reduce(&node, &context).expect("reduce parsed");
        assert_eq!(reduce.as_deref(), Some("sum"));
    }
}
