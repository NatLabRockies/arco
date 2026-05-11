use crate::algebra::parse_value_formula;
use crate::source::ast::ConstraintDecl;
use crate::source::error::SourceError;
use crate::source::parser_helpers::{
    ParseContext, algebra_error, algebra_text_from_node, missing_node_error,
    optional_property_string, parse_constraint_formula_decl, parse_constraint_index_binding,
    property_string, unsupported_declaration_error,
};
use kdl::{KdlNode, KdlValue};

pub(super) fn parse_constraints(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<Vec<ConstraintDecl>, SourceError> {
    let mut constraints = Vec::new();
    for (index, child) in node
        .iter_children()
        .filter(|child| child.name().value() == "constraint")
        .enumerate()
    {
        constraints.push(parse_constraint(child, index, context)?);
    }
    Ok(constraints)
}

pub(super) fn parse_constraint(
    node: &KdlNode,
    index: usize,
    context: &ParseContext<'_>,
) -> Result<ConstraintDecl, SourceError> {
    let mut generation_bindings = Vec::new();
    let mut generation_filters = Vec::new();
    let mut expression = optional_property_string(node, "expression", context)?;

    for child in node.iter_children() {
        match child.name().value() {
            "index" => {
                generation_bindings.push(parse_constraint_index_binding(child, context)?);
            }
            "if" => {
                generation_filters.push(property_string(child, "expression", context)?);
            }
            "expression" => {
                expression = Some(algebra_text_from_node(child, context)?);
            }
            "slack" => {}
            other => {
                return Err(unsupported_declaration_error(child, other, context));
            }
        }
    }

    let expression = expression.ok_or_else(|| missing_node_error("expression", node, context))?;
    let generation_filter = if generation_filters.is_empty() {
        None
    } else {
        Some(generation_filters.join(" and "))
    };

    let (name, name_inferred) = constraint_name(node, index);
    Ok(ConstraintDecl {
        name,
        name_inferred,
        parsed_expression: parse_constraint_formula_decl(&expression, node, context)?,
        generation_bindings,
        parsed_generation_filter: generation_filter
            .as_deref()
            .map(parse_value_formula)
            .transpose()
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        generation_filter,
        expression,
    })
}

fn constraint_name(node: &KdlNode, index: usize) -> (String, bool) {
    if let Some(name) = node
        .get("name")
        .and_then(KdlValue::as_string)
        .map(ToString::to_string)
        .or_else(|| {
            node.get(0)
                .and_then(KdlValue::as_string)
                .map(ToString::to_string)
        })
    {
        return (name, false);
    }

    (format!("constraint_{}", index + 1), true)
}
