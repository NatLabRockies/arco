use crate::ObjectiveSense;
use crate::algebra::parse_value_formula;
use crate::source::ast::{
    BoundExpr, DataBindingDecl, DataDecl, DataIndexDecl, ExpressionDecl, IndexDecl, ModelDecl,
    ParsedSource, ProjectionDecl, ReportDecl, ReportKind, ScenarioDecl, SetDecl, SourceProgram,
    VariableKindDecl,
};
use crate::source::error::SourceError;
use crate::source::parser_constraints::parse_constraints;
use crate::source::parser_helpers::{
    ParseContext, algebra_error, algebra_text_from_node, declaration_indices, first_arg_string,
    invalid_value_error, missing_node_error, optional_property_literal, optional_property_string,
    parse_optimize, parse_optional_filter_expression, parse_reduce, positional_value,
    property_string, unsupported_declaration_error,
};
use crate::source::surface::normalize_surface_syntax;
use kdl::{KdlDocument, KdlNode, KdlValue};
use miette::NamedSource;
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
            "param" => program.params.push(parse_param(node, context)?),
            "model" => program.models.push(parse_model(node, context)?),
            "projection" => program.projections.push(parse_projection(node, context)?),
            "scenario" => program.scenarios.push(parse_scenario(node, context)?),
            other => {
                return Err(unsupported_declaration_error(node, other, context));
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
            "control" | "var" => controls.push(parse_control(child, context)?),
            "expression" => expressions.push(parse_expression(child, context)?),
            "constraint" => {}
            "minimize" => {
                if optimize.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "multiple objectives are not allowed".to_string(),
                        context,
                    ));
                }
                optimize = Some(parse_optimize(child, ObjectiveSense::Minimize, context)?);
            }
            "maximize" => {
                if optimize.is_some() {
                    return Err(invalid_value_error(
                        child,
                        "multiple objectives are not allowed".to_string(),
                        context,
                    ));
                }
                optimize = Some(parse_optimize(child, ObjectiveSense::Maximize, context)?);
            }
            other => {
                return Err(unsupported_declaration_error(child, other, context));
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
            "alias" => maps.push(crate::source::MapDecl {
                name: first_arg_string(child, 0, context)?,
                source: optional_property_string(child, "column", context)?,
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
                return Err(unsupported_declaration_error(child, other, context));
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
        .any(|child| child.name().value() == "index");

    let filter_expression = parse_optional_filter_expression(node, context)?;
    let parsed_filter_expression = filter_expression
        .as_deref()
        .map(parse_value_formula)
        .transpose()
        .map_err(|error| algebra_error(node, error.to_string(), context))?;

    for child in node.iter_children() {
        match child.name().value() {
            "index" | "filter" | "reduce" => {}
            other => return Err(unsupported_declaration_error(child, other, context)),
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
        match child.name().value() {
            "in" => subset_of = Some(first_arg_string(child, 0, context)?),
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
                tuple_indices.push(IndexDecl {
                    name: index_name,
                    domain: Some(domain),
                });
            }
            "filter" => {
                filter_expression = Some(algebra_text_from_node(child, context)?);
            }
            member => {
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
        match child.name().value() {
            "bounds" => {
                for bound in child.iter_children() {
                    match bound.name().value() {
                        "lower" => {
                            let formula_text = algebra_text_from_node(bound, context)?;
                            lower = Some(BoundExpr::Formula(
                                parse_value_formula(&formula_text)
                                    .map_err(|e| algebra_error(bound, e.to_string(), context))?,
                            ));
                        }
                        "upper" => {
                            let formula_text = algebra_text_from_node(bound, context)?;
                            upper = Some(BoundExpr::Formula(
                                parse_value_formula(&formula_text)
                                    .map_err(|e| algebra_error(bound, e.to_string(), context))?,
                            ));
                        }
                        other => {
                            return Err(unsupported_declaration_error(bound, other, context));
                        }
                    }
                }
            }
            "index" => {}
            other => {
                return Err(unsupported_declaration_error(child, other, context));
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
    let formula = algebra_text_from_node(node, context)?;
    Ok(ExpressionDecl {
        name: first_arg_string(node, 0, context)?,
        parsed_formula: parse_value_formula(&formula)
            .map_err(|error| algebra_error(node, error.to_string(), context))?,
        formula,
    })
}

fn parse_projection(
    node: &KdlNode,
    context: &ParseContext<'_>,
) -> Result<ProjectionDecl, SourceError> {
    let mut from_domain = None;
    let mut to_keys = Vec::new();

    for child in node.iter_children() {
        match child.name().value() {
            "from" => {
                from_domain = Some(first_arg_string(child, 0, context)?);
            }
            "to" => {
                let mut position = 0;
                while let Some(value) = child.get(position) {
                    let Some(key) = value.as_string() else {
                        return Err(invalid_value_error(
                            child,
                            format!("argument {position}"),
                            context,
                        ));
                    };
                    to_keys.push(key.to_string());
                    position += 1;
                }
            }
            other => return Err(unsupported_declaration_error(child, other, context)),
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

fn parse_scenario(node: &KdlNode, context: &ParseContext<'_>) -> Result<ScenarioDecl, SourceError> {
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
                    source: property_string(child, "source", context)?,
                });
            }
            "use" => model_use = Some(first_arg_string(child, 0, context)?),
            "report" => {
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
            other => {
                return Err(unsupported_declaration_error(child, other, context));
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

    #[test]
    fn var_keyword_parses_same_as_control() {
        let path = PathBuf::from("test.kdl");

        let with_var = r#"
model "M" {
  var "x" lower=0 {
    index "a"
  }
  minimize "Obj" { x[a] }
}
scenario "S" { use "M" }
"#;

        let with_control = r#"
model "M" {
  control "x" lower=0 {
    index "a"
  }
  minimize "Obj" { x[a] }
}
scenario "S" { use "M" }
"#;

        let var_parsed = parse_program_text(with_var, &path).expect("var syntax parses");
        let control_parsed =
            parse_program_text(with_control, &path).expect("control syntax parses");

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

        let with_alias = r#"
data "D" source="file.csv" {
  alias "X" column="name"
}
"#;

        let with_map = r#"
data "D" source="file.csv" {
  map "X" from="name"
}
"#;

        let alias_parsed = parse_program_text(with_alias, &path).expect("alias syntax parses");
        let map_parsed = parse_program_text(with_map, &path).expect("map syntax parses");

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

        let with_from = r#"
data "D" from="file.csv" {
  map "X" from="name"
}
"#;

        let error = parse_program_text(with_from, &path).expect_err("from= should be rejected");
        assert!(error.to_string().contains("source"));
    }

    #[test]
    fn set_as_property_parses_same_as_alias() {
        let path = PathBuf::from("test.kdl");

        let with_as = r#"set "X" as="g""#;
        let with_alias = r#"set "X" alias="g""#;

        let as_parsed = parse_program_text(with_as, &path).expect("as= syntax parses");
        let alias_parsed = parse_program_text(with_alias, &path).expect("alias= syntax parses");

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

        let with_where = r#"set "thermal" { in "gen"; where { type == "thermal" } }"#;

        parse_program_text(with_where, &path).expect_err("where syntax should fail");
    }

    #[test]
    fn mixed_old_and_new_syntax_parses() {
        let path = PathBuf::from("test.kdl");

        // Mix of legacy-compatible and canonical forms we still support.
        let mixed = r#"
data "D" source="file.csv" {
  map "old_col" from="x"
  alias "new_col" column="y"
}

set "old_set" alias="o"
set "new_set" as="n"

model "M" {
  control "old_var" { index "a" }
  var "new_var" { index "b" }
  minimize "Obj" { old_var[a] + new_var[b] }
}

scenario "S" { use "M" }
"#;

        let parsed = parse_program_text(mixed, &path).expect("mixed syntax parses");

        assert_eq!(parsed.program.data.len(), 1);
        assert_eq!(parsed.program.data[0].maps.len(), 2);

        assert_eq!(parsed.program.sets.len(), 2);
        assert_eq!(parsed.program.sets[0].alias, Some("o".to_string()));
        assert_eq!(parsed.program.sets[1].alias, Some("n".to_string()));

        assert_eq!(parsed.program.models.len(), 1);
        assert_eq!(parsed.program.models[0].controls.len(), 2);
    }
}
