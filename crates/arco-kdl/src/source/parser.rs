use crate::algebra::parse_value_formula;
use crate::source::ast::{
    BoundExpr, DataBindingDecl, DataDecl, DataIndexDecl, ExpressionDecl, HorizonDecl, ModelDecl,
    ParsedSource, ReportDecl, ReportKind, ScenarioDecl, SetDecl, SourceProgram, SubsetDecl,
    VariableKindDecl,
};
use crate::source::error::SourceError;
use crate::source::parser_constraints::parse_constraints;
use crate::source::parser_helpers::{
    ParseContext, algebra_error, child_arg_string, declaration_indices, first_arg_string,
    invalid_value_error, missing_node_error, optional_property_literal, optional_property_string,
    parse_filter_comparators, parse_optimize, parse_optional_filter_expression, parse_reduce,
    positional_value, property_string, property_usize,
};
use crate::source::surface::normalize_surface_syntax;
use kdl::{KdlDocument, KdlNode, KdlValue};
use miette::NamedSource;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::info;

pub fn parse_program_file(path: &Path) -> Result<ParsedSource, SourceError> {
    info!(path = %path.display(), status = "ok", "parsing source file");
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
            "map" => maps.push(crate::source::MapDecl {
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
        let value = crate::source::parser_helpers::literal_from_arg(node, entry.value(), context)?;
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

fn parse_param(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<crate::source::ParamDecl, SourceError> {
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

    Ok(crate::source::ParamDecl {
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
            member => {
                if !child.entries().is_empty() {
                    return Err(SourceError::UnsupportedDeclaration {
                        name: member.to_string(),
                        path: context.path.to_path_buf(),
                        source_text: Box::new(context.source_text.clone()),
                        span: child.span(),
                    });
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

fn parse_control(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<crate::source::ControlDecl, SourceError> {
    let kind = optional_property_string(node, "kind", context)?
        .map(|value| parse_variable_kind_decl(node, &value, context))
        .transpose()?;

    let mut lower = optional_property_literal(node, "lower", context)?.map(BoundExpr::Literal);
    let mut upper = optional_property_literal(node, "upper", context)?.map(BoundExpr::Literal);

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
        model_use,
        reports,
        custom_sets: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_program_text;
    use std::path::PathBuf;

    #[test]
    fn parses_low_level_model_and_scenario() {
        let path = PathBuf::from("test.kdl");
        let text = r#"
model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }
  constraint "balance" {
    x[a,t] <= 1
  }
  minimize "Obj" {
    x[a,t]
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

        let parsed = parse_program_text(text, &path).expect("program parses");
        assert_eq!(parsed.program.models.len(), 1);
        assert_eq!(parsed.program.scenarios.len(), 1);
    }
}
