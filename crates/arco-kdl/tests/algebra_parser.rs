use arco_kdl::algebra::{
    BinaryOp, BindingPattern, ConstraintBody, Expr, parse_constraint_formula, parse_value_formula,
};

#[test]
fn parses_filtered_tuple_reduction_expressions() -> Result<(), Box<dyn std::error::Error>> {
    let expression = parse_value_formula(
        "sum(flow[i,j,t] for (i,j) in arcs for t in time if region[i] == \"north\")",
    )?;

    let Expr::Reduction(ref reduction) = expression else {
        return Err("expected reduction expression".into());
    };

    assert_eq!(reduction.bindings.len(), 2);
    assert_eq!(
        reduction.bindings[0].pattern,
        BindingPattern::Tuple(vec!["i".to_string(), "j".to_string()])
    );
    assert_eq!(reduction.bindings[0].domain, "arcs");
    assert_eq!(
        reduction.bindings[1].pattern,
        BindingPattern::Name("t".to_string())
    );
    assert_eq!(reduction.bindings[1].domain, "time");
    assert_eq!(reduction.filters.len(), 1);
    assert_eq!(
        expression.to_string(),
        "sum(flow[i,j,t] for (i, j) in arcs for t in time if region[i] == \"north\")"
    );

    Ok(())
}

#[test]
fn parses_inline_selector_reduction_domain() -> Result<(), Box<dyn std::error::Error>> {
    let expression = parse_value_formula(
        "sum(dispatch[g,t] for g in generator_data[class=solar area=north] for t in time)",
    )?;

    let Expr::Reduction(reduction) = expression else {
        return Err("expected reduction expression".into());
    };

    assert_eq!(reduction.bindings.len(), 2);
    assert_eq!(
        reduction.bindings[0].domain,
        "generator_data[class=solar area=north]"
    );
    assert_eq!(reduction.bindings[1].domain, "time");

    Ok(())
}

#[test]
fn parses_chained_constraint_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let constraint = parse_constraint_formula("-power_mw[a] <= dispatch[a,t] <= power_mw[a]")?;

    let ConstraintBody::Range {
        ref lower,
        ref middle,
        ref upper,
        ..
    } = constraint
    else {
        return Err("expected range constraint".into());
    };

    assert_eq!(lower.to_string(), "-power_mw[a]");
    assert_eq!(middle.to_string(), "dispatch[a,t]");
    assert_eq!(upper.to_string(), "power_mw[a]");
    assert_eq!(
        constraint.to_string(),
        "-power_mw[a] <= dispatch[a,t] <= power_mw[a]"
    );

    Ok(())
}

#[test]
fn parses_sqrt_function_call_with_indexed_argument() -> Result<(), Box<dyn std::error::Error>> {
    let expression = parse_value_formula("sqrt(eta[j]) * charge[j,t]")?;

    let Expr::Binary {
        op: BinaryOp::Multiply,
        ref left,
        ref right,
    } = expression
    else {
        return Err("expected binary multiply expression".into());
    };

    let Expr::FunctionCall { ref name, ref args } = **left else {
        return Err("expected FunctionCall on lhs".into());
    };
    assert_eq!(name, "sqrt");
    assert_eq!(args.len(), 1);
    assert!(
        matches!(&args[0], Expr::Indexed { target, indices } if target == "eta" && indices.len() == 1)
    );

    assert!(matches!(
        right.as_ref(),
        Expr::Indexed { target, indices } if target == "charge" && indices.len() == 2
    ));

    assert_eq!(expression.to_string(), "sqrt(eta[j]) * charge[j,t]");

    Ok(())
}

#[test]
fn parses_pow_function_call_with_two_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let expression = parse_value_formula("pow(1 + r, lifetime)")?;

    let Expr::FunctionCall { ref name, ref args } = expression else {
        return Err("expected FunctionCall".into());
    };
    assert_eq!(name, "pow");
    assert_eq!(args.len(), 2);

    // First arg is (1 + r).
    assert!(
        matches!(&args[0], Expr::Binary { op: BinaryOp::Add, left, right }
            if matches!(left.as_ref(), Expr::Number(n) if n == "1")
            && matches!(right.as_ref(), Expr::Identifier(id) if id == "r")
        )
    );

    // Second arg is the identifier "lifetime".
    assert!(matches!(&args[1], Expr::Identifier(id) if id == "lifetime"));

    assert_eq!(expression.to_string(), "pow(1 + r, lifetime)");

    Ok(())
}

#[test]
fn parses_abs_function_call_with_indexed_argument() -> Result<(), Box<dyn std::error::Error>> {
    let expression = parse_value_formula("abs(x[a,t])")?;

    let Expr::FunctionCall { ref name, ref args } = expression else {
        return Err("expected FunctionCall".into());
    };
    assert_eq!(name, "abs");
    assert_eq!(args.len(), 1);
    assert!(
        matches!(&args[0], Expr::Indexed { target, indices } if target == "x" && indices.len() == 2)
    );

    assert_eq!(expression.to_string(), "abs(x[a,t])");

    Ok(())
}

#[test]
fn parses_nested_trig_function_calls() -> Result<(), Box<dyn std::error::Error>> {
    let expression = parse_value_formula("cos(theta[i,t] + atan(x[l] / r[l]))")?;

    let Expr::FunctionCall { ref name, ref args } = expression else {
        return Err("expected FunctionCall".into());
    };
    assert_eq!(name, "cos");
    assert_eq!(args.len(), 1);

    let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
    } = &args[0]
    else {
        return Err("expected additive argument".into());
    };

    assert!(matches!(
        left.as_ref(),
        Expr::Indexed { target, indices } if target == "theta" && indices.len() == 2
    ));
    assert!(matches!(
        right.as_ref(),
        Expr::FunctionCall { name, args } if name == "atan" && args.len() == 1
    ));

    assert_eq!(
        expression.to_string(),
        "cos(theta[i,t] + atan(x[l] / r[l]))"
    );

    Ok(())
}
