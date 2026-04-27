fn emit_terminal_boundary_constraints(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    entrypoint: &Path,
) -> Result<Vec<LinearConstraint>, CompileError> {
    if program.chronology.terminal_boundary.is_none()
        || !variable_signatures.contains_key("soc[a,t]")
    {
        return Ok(Vec::new());
    }

    let mut constraints = Vec::new();
    for asset in &inputs.assets {
        if !asset.families.contains("soc") {
            continue;
        }
        constraints.push(LinearConstraint {
            name: format!("terminal_soc[{}]", asset.name),
            sense: ConstraintSense::Equal,
            rhs: asset_parameter(asset, "terminal_soc_mwh", entrypoint)?,
            terms: vec![term(
                &indexed_name("soc", &asset.name, program.sets.time.steps),
                1.0,
            )],
        });
    }
    Ok(constraints)
}

fn constraint_instance_bindings(
    constraint: &ResolvedConstraint,
    inputs: &ScenarioInputs,
    steps: usize,
) -> Vec<LinearizationBindings> {
    let binds_asset = constraint_uses_free_index(&constraint.expression, "a");
    let binds_time = constraint_uses_free_index(&constraint.expression, "t");
    let assets = if binds_asset {
        relevant_constraint_assets(constraint, inputs)
            .into_iter()
            .map(|asset| asset.name.clone())
            .collect::<Vec<_>>()
    } else {
        vec![String::new()]
    };
    let times = if binds_time {
        (1..=steps).collect::<Vec<_>>()
    } else {
        vec![0]
    };

    let mut bindings = Vec::new();
    for asset in &assets {
        for time in &times {
            let mut scope = LinearizationBindings::default();
            if binds_asset {
                scope
                    .values
                    .insert("a".to_string(), FilterValue::String(asset.clone()));
            }
            if binds_time {
                scope
                    .values
                    .insert("t".to_string(), FilterValue::Number(*time as f64));
            }
            bindings.push(scope);
        }
    }
    bindings
}

fn relevant_constraint_assets<'a>(
    _constraint: &ResolvedConstraint,
    inputs: &'a ScenarioInputs,
) -> Vec<&'a AssetInputs> {
    inputs.assets.iter().collect()
}

fn constraint_uses_free_index(body: &ConstraintBody, name: &str) -> bool {
    match body {
        ConstraintBody::Comparison { left, right, .. } => {
            expr_uses_free_index(left, name, &mut BTreeSet::new())
                || expr_uses_free_index(right, name, &mut BTreeSet::new())
        }
        ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            expr_uses_free_index(lower, name, &mut BTreeSet::new())
                || expr_uses_free_index(middle, name, &mut BTreeSet::new())
                || expr_uses_free_index(upper, name, &mut BTreeSet::new())
        }
    }
}

fn expr_uses_free_index(expr: &Expr, name: &str, bound: &mut BTreeSet<String>) -> bool {
    match expr {
        Expr::Identifier(identifier) => identifier == name && !bound.contains(identifier),
        Expr::Indexed { indices, .. } => indices
            .iter()
            .any(|index| expr_uses_free_index(index, name, bound)),
        Expr::Unary { expr, .. } => expr_uses_free_index(expr, name, bound),
        Expr::Binary { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Comparison { left, right, .. } => {
            expr_uses_free_index(left, name, bound) || expr_uses_free_index(right, name, bound)
        }
        Expr::Reduction(reduction) => {
            let mut local_bound = bound.clone();
            for binding in &reduction.bindings {
                match &binding.pattern {
                    crate::algebra::BindingPattern::Name(identifier) => {
                        local_bound.insert(identifier.clone());
                    }
                    crate::algebra::BindingPattern::Tuple(identifiers) => {
                        local_bound.extend(identifiers.iter().cloned());
                    }
                }
            }
            expr_uses_free_index(&reduction.body, name, &mut local_bound)
                || reduction
                    .filters
                    .iter()
                    .any(|filter| expr_uses_free_index(filter, name, &mut local_bound))
        }
        Expr::FunctionCall { args, .. } => args
            .iter()
            .any(|arg| expr_uses_free_index(arg, name, bound)),
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => false,
    }
}

fn bindings_asset<'a>(
    bindings: &LinearizationBindings,
    inputs: &'a ScenarioInputs,
) -> Option<&'a AssetInputs> {
    bindings.values.get("a").and_then(|value| match value {
        FilterValue::String(name) => find_asset(inputs, name),
        _ => None,
    })
}

fn bindings_time(
    bindings: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<Option<usize>, CompileError> {
    bindings
        .values
        .get("t")
        .map(|value| integer_time_index(value, entrypoint).map(|time| time as usize))
        .transpose()
}

fn constraint_binding_suffix(
    bindings: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<String, CompileError> {
    let mut indices = Vec::new();
    if let Some(value) = bindings.values.get("a") {
        indices.push(string_filter_value(
            value,
            &synthetic_constraint("constraint"),
            entrypoint,
        )?);
    }
    if let Some(value) = bindings.values.get("t") {
        indices.push(integer_time_index(value, entrypoint)?.to_string());
    }
    // Include any custom binding variables (e.g. "b" from `over "b" in="periods"`)
    for (name, value) in &bindings.values {
        if name == "a" || name == "t" {
            continue;
        }
        match value {
            FilterValue::String(s) => indices.push(s.clone()),
            FilterValue::Number(n) => {
                if n.fract() == 0.0 {
                    indices.push((*n as i64).to_string());
                } else {
                    indices.push(n.to_string());
                }
            }
            FilterValue::Boolean(b) => indices.push(b.to_string()),
        }
    }
    if indices.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("[{}]", indices.join(",")))
    }
}
