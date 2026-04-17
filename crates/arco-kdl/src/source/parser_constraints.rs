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
        let mut generation_bindings = Vec::new();
        let mut generation_filters = Vec::new();
        let mut expression = optional_property_string(child, "expression", context)?;

        for grandchild in child.iter_children() {
            match grandchild.name().value() {
                "index" => {
                    generation_bindings.push(parse_constraint_index_binding(grandchild, context)?);
                }
                "if" => {
                    generation_filters.push(property_string(grandchild, "expression", context)?);
                }
                "expression" => {
                    expression = Some(algebra_text_from_node(grandchild, context)?);
                }
                "slack" => {}
                other => {
                    return Err(unsupported_declaration_error(grandchild, other, context));
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
            parsed_expression: parse_constraint_formula_decl(&expression, child, context)?,
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
