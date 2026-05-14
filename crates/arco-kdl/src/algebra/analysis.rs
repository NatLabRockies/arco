use crate::algebra::types::{BinaryOp, BindingPattern, ConstraintBody, Expr};
use std::collections::BTreeSet;

pub fn collect_named_expression_dependencies(expr: &Expr) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_named_dependencies(expr, &mut BTreeSet::new(), &mut names);
    names
}

pub fn constraint_mentions_previous_time(constraint: &ConstraintBody) -> bool {
    match constraint {
        ConstraintBody::Comparison { left, right, .. } => {
            expr_mentions_previous_time(left) || expr_mentions_previous_time(right)
        }
        ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            expr_mentions_previous_time(lower)
                || expr_mentions_previous_time(middle)
                || expr_mentions_previous_time(upper)
        }
    }
}

fn collect_named_dependencies(
    expr: &Expr,
    bound: &mut BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Identifier(name) | Expr::String(name) => {
            if !bound.contains(name) {
                names.insert(name.clone());
            }
        }
        Expr::Indexed { target, indices } => {
            if !bound.contains(target) {
                names.insert(target.clone());
            }
            for index in indices {
                collect_named_dependencies(index, bound, names);
            }
        }
        Expr::Unary { expr, .. } => collect_named_dependencies(expr, bound, names),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            collect_named_dependencies(left, bound, names);
            collect_named_dependencies(right, bound, names);
        }
        Expr::Reduction(reduction) => {
            let mut local_bound = bound.clone();
            for binding in &reduction.bindings {
                match &binding.pattern {
                    BindingPattern::Name(name) => {
                        local_bound.insert(name.clone());
                    }
                    BindingPattern::Tuple(names) => {
                        local_bound.extend(names.iter().cloned());
                    }
                }
            }
            collect_named_dependencies(&reduction.body, &mut local_bound, names);
            for filter in &reduction.filters {
                collect_named_dependencies(filter, &mut local_bound, names);
            }
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_named_dependencies(arg, bound, names);
            }
        }
        Expr::Number(_) | Expr::Boolean(_) => {}
    }
}

fn expr_mentions_previous_time(expr: &Expr) -> bool {
    match expr {
        Expr::Indexed { indices, .. } => indices.iter().any(index_mentions_previous_time),
        Expr::Unary { expr, .. } => expr_mentions_previous_time(expr),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            expr_mentions_previous_time(left) || expr_mentions_previous_time(right)
        }
        Expr::FunctionCall { args, .. } => args.iter().any(expr_mentions_previous_time),
        Expr::Reduction(reduction) => {
            expr_mentions_previous_time(&reduction.body)
                || reduction.filters.iter().any(expr_mentions_previous_time)
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) | Expr::Identifier(_) => false,
    }
}

fn index_mentions_previous_time(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Binary {
            op: BinaryOp::Subtract,
            left,
            right,
        } if matches!(left.as_ref(), Expr::Identifier(name) if name == "t")
            && matches!(right.as_ref(), Expr::Number(value) if value == "1")
    ) || expr_mentions_previous_time(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_named_expression_dependencies_includes_quoted_expression_refs() {
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::String("dispatch_new_gen_per_bus[b]".to_string())),
            right: Box::new(Expr::Identifier("mw_load_per_existing_bus".to_string())),
        };

        let dependencies = collect_named_expression_dependencies(&expr);

        assert!(dependencies.contains("dispatch_new_gen_per_bus[b]"));
        assert!(dependencies.contains("mw_load_per_existing_bus"));
    }
}
