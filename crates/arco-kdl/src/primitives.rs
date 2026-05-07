use crate::ObjectiveSense;
use crate::algebra::{BinaryOp, ComparisonOp, ConstraintBody, Expr, UnaryOp};
use crate::source::{BoundExpr, LiteralValue, ParsedSource, SetDecl, VariableKindDecl};
use arco_model::document::{ArcoDocument, IndexedDataDocument, ModelDocument};
use arco_model::indexed::{
    DuplicateReducer, IndexKey, IndexValue, IndexedData, ParameterTable, Set,
};
use arco_model::{
    Bounds, Constraint, ConstraintId, Model, ModelBuilder, ModelView, Objective, Sense, Variable,
    VariableId,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum PrimitiveBuildError {
    #[error("no model declarations were found")]
    MissingModel,
    #[error("unsupported expression in {context}: `{expr}`")]
    UnsupportedExpression { context: String, expr: String },
    #[error("unsupported numeric literal in {context}: `{literal}`")]
    UnsupportedNumericLiteral { context: String, literal: String },
    #[error("unknown variable `{name}` referenced in {context}")]
    UnknownVariable { context: String, name: String },
    #[error("only <=, >=, and == constraints are supported in {context}")]
    UnsupportedConstraintComparator { context: String },
}

#[derive(Debug, Default)]
struct LinearExpr {
    constant: f64,
    terms: BTreeMap<VariableId, f64>,
}

impl LinearExpr {
    fn with_constant(constant: f64) -> Self {
        Self {
            constant,
            terms: BTreeMap::new(),
        }
    }

    fn add_assign(&mut self, other: Self) {
        self.constant += other.constant;
        for (variable_id, coefficient) in other.terms {
            *self.terms.entry(variable_id).or_insert(0.0) += coefficient;
        }
    }

    fn sub_assign(&mut self, other: Self) {
        self.constant -= other.constant;
        for (variable_id, coefficient) in other.terms {
            *self.terms.entry(variable_id).or_insert(0.0) -= coefficient;
        }
    }

    fn scale(self, factor: f64) -> Self {
        let terms = self
            .terms
            .into_iter()
            .map(|(variable_id, coefficient)| (variable_id, coefficient * factor))
            .collect();
        Self {
            constant: self.constant * factor,
            terms,
        }
    }
}

pub fn build_model(parsed: &ParsedSource) -> Result<Model, PrimitiveBuildError> {
    let model_decl = parsed
        .program
        .models
        .first()
        .ok_or(PrimitiveBuildError::MissingModel)?;

    let mut builder = ModelBuilder::<f64>::new();
    let mut variable_ids = BTreeMap::new();

    for control in &model_decl.controls {
        let lower = parse_bound_expr(control.lower.as_ref(), &control.name, f64::NEG_INFINITY)?;
        let upper = parse_bound_expr(control.upper.as_ref(), &control.name, f64::INFINITY)?;
        let variable = match control.kind {
            Some(VariableKindDecl::Binary) => Variable::binary(),
            Some(VariableKindDecl::Integer) => Variable::integer(Bounds::new(lower, upper)),
            Some(VariableKindDecl::Continuous) | None => {
                Variable::continuous(Bounds::new(lower, upper))
            }
        };

        let variable_id = builder.add_variable(variable).map_err(|error| {
            PrimitiveBuildError::UnsupportedExpression {
                context: format!("control `{}`", control.name),
                expr: error.to_string(),
            }
        })?;
        variable_ids.insert(control.name.clone(), variable_id);
    }

    for constraint in &model_decl.constraints {
        let context = format!("constraint `{}`", constraint.name);
        let (linear_expr, bounds) =
            build_constraint_linear_form(&constraint.parsed_expression, &variable_ids, &context)?;

        let constraint_id = builder
            .add_constraint(Constraint { bounds })
            .map_err(|error| PrimitiveBuildError::UnsupportedExpression {
                context: context.clone(),
                expr: error.to_string(),
            })?;

        for (variable_id, coefficient) in linear_expr.terms {
            if coefficient != 0.0 {
                builder
                    .set_coefficient(variable_id, constraint_id, coefficient)
                    .map_err(|error| PrimitiveBuildError::UnsupportedExpression {
                        context: context.clone(),
                        expr: error.to_string(),
                    })?;
            }
        }
    }

    let mut model = builder.finish();

    for (control_name, variable_id) in &variable_ids {
        model
            .set_variable_name(*variable_id, control_name.clone())
            .map_err(|error| PrimitiveBuildError::UnsupportedExpression {
                context: format!("control `{control_name}`"),
                expr: error.to_string(),
            })?;
    }

    for (index, constraint) in model_decl.constraints.iter().enumerate() {
        let constraint_id = ConstraintId::new(index as u32);
        model
            .set_constraint_name(constraint_id, constraint.name.clone())
            .map_err(|error| PrimitiveBuildError::UnsupportedExpression {
                context: format!("constraint `{}`", constraint.name),
                expr: error.to_string(),
            })?;
    }

    let objective_linear = lower_linear_expression(
        &model_decl.optimize.parsed_expression,
        &variable_ids,
        &format!("objective `{}`", model_decl.optimize.name),
    )?;

    let sense = match model_decl.optimize.sense {
        ObjectiveSense::Minimize => Sense::Minimize,
        ObjectiveSense::Maximize => Sense::Maximize,
    };

    let objective_terms: Vec<(VariableId, f64)> = objective_linear
        .terms
        .into_iter()
        .filter(|(_, coefficient)| *coefficient != 0.0)
        .collect();

    model
        .set_objective(Objective {
            sense: Some(sense),
            terms: objective_terms,
        })
        .map_err(|error| PrimitiveBuildError::UnsupportedExpression {
            context: format!("objective `{}`", model_decl.optimize.name),
            expr: error.to_string(),
        })?;

    model
        .set_objective_name(Some(model_decl.optimize.name.clone()))
        .map_err(|error| PrimitiveBuildError::UnsupportedExpression {
            context: format!("objective `{}`", model_decl.optimize.name),
            expr: error.to_string(),
        })?;

    Ok(model)
}

pub fn build_indexed_data(parsed: &ParsedSource) -> Result<IndexedData<f64>, PrimitiveBuildError> {
    let mut indexed = IndexedData::<f64>::default();

    for set_decl in &parsed.program.sets {
        insert_set_decl(&mut indexed, set_decl);
    }

    for model in &parsed.program.models {
        for set_decl in &model.sets {
            insert_set_decl(&mut indexed, set_decl);
        }
    }

    for param in &parsed.program.params {
        if let Some(value) = param.value.as_ref() {
            let scalar_value = parse_numeric_literal(value, &format!("param `{}`", param.name))?;
            let table = ParameterTable::from_rows(
                &param.name,
                [(IndexKey(Vec::new()), scalar_value)],
                DuplicateReducer::Sum,
            );
            indexed.parameters.insert(param.name.clone(), table);
        }
    }

    Ok(indexed)
}

pub fn build_model_document(parsed: &ParsedSource) -> Result<ModelDocument, PrimitiveBuildError> {
    let model = build_model(parsed)?;
    let mut document = ModelDocument::new_f64();
    document.fingerprint = Some(model.fingerprint().0);
    Ok(document)
}

pub fn build_arco_document(parsed: &ParsedSource) -> Result<ArcoDocument, PrimitiveBuildError> {
    let model_document = if parsed.program.models.is_empty() {
        None
    } else {
        Some(build_model_document(parsed)?)
    };

    let indexed_data_document =
        if parsed.program.sets.is_empty() && parsed.program.params.is_empty() {
            None
        } else {
            let _ = build_indexed_data(parsed)?;
            Some(IndexedDataDocument::new_f64())
        };

    let mut document = ArcoDocument::new_f64();
    document.model = model_document;
    document.indexed_data = indexed_data_document;
    Ok(document)
}

fn insert_set_decl(indexed: &mut IndexedData<f64>, set_decl: &SetDecl) {
    let mut set = Set::new(&set_decl.name);
    for member in &set_decl.members {
        set.insert(literal_to_index_value(member));
    }
    indexed.sets.insert(set_decl.name.clone(), set);
}

fn parse_bound_expr(
    expr: Option<&BoundExpr>,
    control: &str,
    default: f64,
) -> Result<f64, PrimitiveBuildError> {
    match expr {
        None => Ok(default),
        Some(BoundExpr::Literal(value)) => {
            parse_numeric_literal(value, &format!("control `{control}` bound"))
        }
        Some(BoundExpr::Formula(formula)) => Err(PrimitiveBuildError::UnsupportedExpression {
            context: format!("control `{control}` bound"),
            expr: formula.to_string(),
        }),
    }
}

fn parse_numeric_literal(
    literal: &LiteralValue,
    context: &str,
) -> Result<f64, PrimitiveBuildError> {
    match literal {
        LiteralValue::Integer(value) => Ok(*value as f64),
        LiteralValue::Decimal(value) => {
            value
                .parse::<f64>()
                .map_err(|_| PrimitiveBuildError::UnsupportedNumericLiteral {
                    context: context.to_string(),
                    literal: value.clone(),
                })
        }
        LiteralValue::String(value) => Err(PrimitiveBuildError::UnsupportedNumericLiteral {
            context: context.to_string(),
            literal: value.clone(),
        }),
        LiteralValue::Boolean(value) => Err(PrimitiveBuildError::UnsupportedNumericLiteral {
            context: context.to_string(),
            literal: value.to_string(),
        }),
    }
}

fn literal_to_index_value(literal: &LiteralValue) -> IndexValue {
    match literal {
        LiteralValue::String(value) => IndexValue::String(value.clone()),
        LiteralValue::Integer(value) => IndexValue::Integer(*value as i64),
        LiteralValue::Decimal(value) => IndexValue::Decimal(value.clone()),
        LiteralValue::Boolean(value) => IndexValue::Bool(*value),
    }
}

fn parse_number(expr: &Expr, context: &str) -> Result<f64, PrimitiveBuildError> {
    match expr {
        Expr::Number(value) => {
            value
                .parse::<f64>()
                .map_err(|_| PrimitiveBuildError::UnsupportedExpression {
                    context: context.to_string(),
                    expr: expr.to_string(),
                })
        }
        _ => Err(PrimitiveBuildError::UnsupportedExpression {
            context: context.to_string(),
            expr: expr.to_string(),
        }),
    }
}

fn lower_linear_expression(
    expr: &Expr,
    variable_ids: &BTreeMap<String, VariableId>,
    context: &str,
) -> Result<LinearExpr, PrimitiveBuildError> {
    match expr {
        Expr::Number(value) => value
            .parse::<f64>()
            .map(LinearExpr::with_constant)
            .map_err(|_| PrimitiveBuildError::UnsupportedExpression {
                context: context.to_string(),
                expr: expr.to_string(),
            }),
        Expr::Identifier(name) => {
            let variable_id = variable_ids.get(name).copied().ok_or_else(|| {
                PrimitiveBuildError::UnknownVariable {
                    context: context.to_string(),
                    name: name.clone(),
                }
            })?;
            let mut linear = LinearExpr::default();
            linear.terms.insert(variable_id, 1.0);
            Ok(linear)
        }
        Expr::Indexed { target, .. } => {
            let variable_id = variable_ids.get(target).copied().ok_or_else(|| {
                PrimitiveBuildError::UnknownVariable {
                    context: context.to_string(),
                    name: target.clone(),
                }
            })?;
            let mut linear = LinearExpr::default();
            linear.terms.insert(variable_id, 1.0);
            Ok(linear)
        }
        Expr::Unary {
            op: UnaryOp::Negate,
            expr,
        } => Ok(lower_linear_expression(expr, variable_ids, context)?.scale(-1.0)),
        Expr::Binary { op, left, right } => match op {
            BinaryOp::Add => {
                let mut linear = lower_linear_expression(left, variable_ids, context)?;
                linear.add_assign(lower_linear_expression(right, variable_ids, context)?);
                Ok(linear)
            }
            BinaryOp::Subtract => {
                let mut linear = lower_linear_expression(left, variable_ids, context)?;
                linear.sub_assign(lower_linear_expression(right, variable_ids, context)?);
                Ok(linear)
            }
            BinaryOp::Multiply => {
                if let Expr::Number(_) = left.as_ref() {
                    let factor = parse_number(left, context)?;
                    Ok(lower_linear_expression(right, variable_ids, context)?.scale(factor))
                } else if let Expr::Number(_) = right.as_ref() {
                    let factor = parse_number(right, context)?;
                    Ok(lower_linear_expression(left, variable_ids, context)?.scale(factor))
                } else {
                    Err(PrimitiveBuildError::UnsupportedExpression {
                        context: context.to_string(),
                        expr: expr.to_string(),
                    })
                }
            }
            BinaryOp::Divide => {
                let divisor = parse_number(right, context)?;
                if divisor == 0.0 {
                    return Err(PrimitiveBuildError::UnsupportedExpression {
                        context: context.to_string(),
                        expr: expr.to_string(),
                    });
                }
                Ok(lower_linear_expression(left, variable_ids, context)?.scale(1.0 / divisor))
            }
        },
        Expr::String(_)
        | Expr::Boolean(_)
        | Expr::Comparison { .. }
        | Expr::FunctionCall { .. }
        | Expr::Reduction(_) => Err(PrimitiveBuildError::UnsupportedExpression {
            context: context.to_string(),
            expr: expr.to_string(),
        }),
    }
}

fn build_constraint_linear_form(
    body: &ConstraintBody,
    variable_ids: &BTreeMap<String, VariableId>,
    context: &str,
) -> Result<(LinearExpr, Bounds), PrimitiveBuildError> {
    match body {
        ConstraintBody::Comparison { op, left, right } => {
            let mut diff = lower_linear_expression(left, variable_ids, context)?;
            diff.sub_assign(lower_linear_expression(right, variable_ids, context)?);
            let constant = diff.constant;
            diff.constant = 0.0;

            let bounds = match op {
                ComparisonOp::LessEqual => Bounds::new(f64::NEG_INFINITY, -constant),
                ComparisonOp::GreaterEqual => Bounds::new(-constant, f64::INFINITY),
                ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
                    Bounds::new(-constant, -constant)
                }
                ComparisonOp::Less | ComparisonOp::Greater | ComparisonOp::NotEqual => {
                    return Err(PrimitiveBuildError::UnsupportedConstraintComparator {
                        context: context.to_string(),
                    });
                }
            };
            Ok((diff, bounds))
        }
        ConstraintBody::Range {
            lower,
            lower_op,
            middle,
            upper_op,
            upper,
        } => {
            if !matches!(lower_op, ComparisonOp::LessEqual | ComparisonOp::Less)
                || !matches!(upper_op, ComparisonOp::LessEqual | ComparisonOp::Less)
            {
                return Err(PrimitiveBuildError::UnsupportedConstraintComparator {
                    context: context.to_string(),
                });
            }

            let mut middle_linear = lower_linear_expression(middle, variable_ids, context)?;
            let lower_value = parse_number(lower, context)?;
            let upper_value = parse_number(upper, context)?;
            let constant = middle_linear.constant;
            middle_linear.constant = 0.0;
            Ok((
                middle_linear,
                Bounds::new(lower_value - constant, upper_value - constant),
            ))
        }
    }
}
