fn emit_terminal_boundary_constraints(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    entrypoint: &Path,
) -> Result<Vec<LinearConstraint>, CompileError> {
    if program.chronology.terminal_boundary.is_none() {
        return Ok(Vec::new());
    }

    let Some(soc_signature) = variable_signatures
        .values()
        .find(|signature| signature.target == "soc" && signature.indices.len() == 2)
    else {
        return Ok(Vec::new());
    };

    let mut constraints = Vec::new();
    for asset in &inputs.assets {
        if !asset.families.contains(&soc_signature.target) {
            continue;
        }
        constraints.push(LinearConstraint {
            name: format!("terminal_{}[{}]", soc_signature.target, asset.name),
            sense: ConstraintSense::Equal,
            rhs: asset_parameter(asset, "terminal_soc_mwh", entrypoint)?,
            terms: vec![term(
                &format!(
                    "{}[{},{}]",
                    soc_signature.target, asset.name, program.time_steps()
                ),
                1.0,
            )],
        });
    }
    Ok(constraints)
}

fn infer_constraint_generation_bindings(
    constraint: &ResolvedConstraint,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    entrypoint: &Path,
) -> Result<(Vec<String>, Vec<LinearizationBindings>), CompileError> {
    let mut binding_domains = BTreeMap::<String, String>::new();
    let mut binding_order = Vec::<String>::new();
    infer_constraint_binding_domains_from_body(
        &constraint.expression,
        variable_signatures,
        &mut BTreeSet::new(),
        &mut binding_domains,
        &mut binding_order,
        entrypoint,
    )?;

    if binding_order.is_empty() {
        return Ok((binding_order, vec![LinearizationBindings::default()]));
    }

    let bindings = binding_order
        .iter()
        .map(|name| GenerationBinding {
            variable: name.clone(),
            domain: binding_domains
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
        })
        .collect::<Vec<_>>();
    let mut scopes = expand_generation_bindings(
        &bindings,
        inputs,
        program,
        entrypoint,
        &constraint.diagnostic_id,
    )?;
    for scope in &mut scopes {
        for binding in &bindings {
            if !domain_is_time_like(program, binding.domain.as_str()) {
                continue;
            }
            let Some(FilterValue::String(value)) = scope.values.get(&binding.variable).cloned()
            else {
                continue;
            };
            let parsed = value
                .parse::<f64>()
                .map_err(|_| CompileError::InvalidFormulation {
                    message: format!("time index `{value}` must be numeric"),
                    path: entrypoint.to_path_buf(),
                })?;
            scope
                .values
                .insert(binding.variable.clone(), FilterValue::Number(parsed));
        }
    }
    Ok((binding_order, scopes))
}

fn domain_is_time_like(program: &SemanticProgram, domain: &str) -> bool {
    program.is_time_set_name(domain)
}

fn infer_constraint_binding_domains_from_body(
    body: &ConstraintBody,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    bound: &mut BTreeSet<String>,
    binding_domains: &mut BTreeMap<String, String>,
    binding_order: &mut Vec<String>,
    entrypoint: &Path,
) -> Result<(), CompileError> {
    match body {
        ConstraintBody::Comparison { left, right, .. } => {
            infer_constraint_binding_domains_from_expr(
                left,
                variable_signatures,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )?;
            infer_constraint_binding_domains_from_expr(
                right,
                variable_signatures,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )
        }
        ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            infer_constraint_binding_domains_from_expr(
                lower,
                variable_signatures,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )?;
            infer_constraint_binding_domains_from_expr(
                middle,
                variable_signatures,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )?;
            infer_constraint_binding_domains_from_expr(
                upper,
                variable_signatures,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )
        }
    }
}

fn infer_constraint_binding_domains_from_expr(
    expr: &Expr,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    bound: &mut BTreeSet<String>,
    binding_domains: &mut BTreeMap<String, String>,
    binding_order: &mut Vec<String>,
    entrypoint: &Path,
) -> Result<(), CompileError> {
    match expr {
        Expr::Indexed { target, indices } => {
            if let Some(signature) = variable_signatures.values().find(|signature| {
                signature.target == *target && signature.indices.len() == indices.len()
            }) {
                for (position, index_expr) in indices.iter().enumerate() {
                    let index_name = &signature.indices[position];
                    let domain = signature
                        .index_domains
                        .get(index_name)
                        .map_or(index_name.as_str(), |domain| domain.as_str());
                    infer_constraint_binding_domains_from_index_expr(
                        index_expr,
                        domain,
                        bound,
                        binding_domains,
                        binding_order,
                        entrypoint,
                    )?;
                }
            }
            for index_expr in indices {
                infer_constraint_binding_domains_from_expr(
                    index_expr,
                    variable_signatures,
                    bound,
                    binding_domains,
                    binding_order,
                    entrypoint,
                )?;
            }
            Ok(())
        }
        Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => Ok(()),
        Expr::Unary { expr, .. } => infer_constraint_binding_domains_from_expr(
            expr,
            variable_signatures,
            bound,
            binding_domains,
            binding_order,
            entrypoint,
        ),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            infer_constraint_binding_domains_from_expr(
                left,
                variable_signatures,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )?;
            infer_constraint_binding_domains_from_expr(
                right,
                variable_signatures,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )
        }
        Expr::Reduction(reduction) => {
            let mut local_bound = bound.clone();
            for binding in &reduction.bindings {
                match &binding.pattern {
                    arco_kdl::algebra::BindingPattern::Name(identifier) => {
                        local_bound.insert(identifier.clone());
                    }
                    arco_kdl::algebra::BindingPattern::Tuple(identifiers) => {
                        local_bound.extend(identifiers.iter().cloned());
                    }
                }
            }
            infer_constraint_binding_domains_from_expr(
                &reduction.body,
                variable_signatures,
                &mut local_bound,
                binding_domains,
                binding_order,
                entrypoint,
            )?;
            for filter in &reduction.filters {
                infer_constraint_binding_domains_from_expr(
                    filter,
                    variable_signatures,
                    &mut local_bound,
                    binding_domains,
                    binding_order,
                    entrypoint,
                )?;
            }
            Ok(())
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                infer_constraint_binding_domains_from_expr(
                    arg,
                    variable_signatures,
                    bound,
                    binding_domains,
                    binding_order,
                    entrypoint,
                )?;
            }
            Ok(())
        }
    }
}

fn infer_constraint_binding_domains_from_index_expr(
    expr: &Expr,
    domain: &str,
    bound: &BTreeSet<String>,
    binding_domains: &mut BTreeMap<String, String>,
    binding_order: &mut Vec<String>,
    entrypoint: &Path,
) -> Result<(), CompileError> {
    match expr {
        Expr::Identifier(identifier) if !bound.contains(identifier) => {
            if let Some(existing) = binding_domains.get(identifier) {
                if existing != domain {
                    return Err(CompileError::InvalidFormulation {
                        message: format!(
                            "free index `{identifier}` resolves to conflicting domains `{existing}` and `{domain}`"
                        ),
                        path: entrypoint.to_path_buf(),
                    });
                }
            } else {
                binding_domains.insert(identifier.clone(), domain.to_string());
                binding_order.push(identifier.clone());
            }
            Ok(())
        }
        Expr::Unary { expr, .. } => infer_constraint_binding_domains_from_index_expr(
            expr,
            domain,
            bound,
            binding_domains,
            binding_order,
            entrypoint,
        ),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            infer_constraint_binding_domains_from_index_expr(
                left,
                domain,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )?;
            infer_constraint_binding_domains_from_index_expr(
                right,
                domain,
                bound,
                binding_domains,
                binding_order,
                entrypoint,
            )
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                infer_constraint_binding_domains_from_index_expr(
                    index,
                    domain,
                    bound,
                    binding_domains,
                    binding_order,
                    entrypoint,
                )?;
            }
            Ok(())
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                infer_constraint_binding_domains_from_index_expr(
                    arg,
                    domain,
                    bound,
                    binding_domains,
                    binding_order,
                    entrypoint,
                )?;
            }
            Ok(())
        }
        Expr::Reduction(_)
        | Expr::Identifier(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::Boolean(_) => Ok(()),
    }
}

fn bindings_asset<'a>(
    bindings: &LinearizationBindings,
    inputs: &'a ScenarioInputs,
) -> Option<&'a AssetInputs> {
    for preferred in ["a", "asset"] {
        if let Some(FilterValue::String(name)) = bindings.values.get(preferred) {
            if let Some(asset) = find_asset(inputs, name) {
                return Some(asset);
            }
        }
    }

    let mut matches = bindings.values.values().filter_map(|value| match value {
        FilterValue::String(name) => find_asset(inputs, name),
        FilterValue::Number(_) | FilterValue::Boolean(_) => None,
    });
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first)
}

fn bindings_time(
    bindings: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<Option<usize>, CompileError> {
    for preferred in ["t", "time"] {
        if let Some(value) = bindings.values.get(preferred) {
            return integer_time_index(value, entrypoint).map(|time| Some(time as usize));
        }
    }

    let mut times = bindings
        .values
        .values()
        .filter_map(|value| integer_time_index(value, entrypoint).ok())
        .collect::<Vec<_>>();
    times.sort_unstable();
    times.dedup();
    Ok(match times.as_slice() {
        [time] => Some(*time as usize),
        _ => None,
    })
}

fn constraint_binding_suffix(bindings: &LinearizationBindings, binding_order: &[String]) -> String {
    let mut indices = Vec::new();
    let mut emitted = BTreeSet::new();

    for name in binding_order {
        if let Some(value) = bindings.values.get(name) {
            indices.push(render_filter_value(value));
            emitted.insert(name.as_str());
        }
    }

    let remaining_names = if emitted.is_empty() {
        bindings
            .order
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    } else {
        bindings
            .order
            .iter()
            .map(String::as_str)
            .filter(|name| !emitted.contains(*name))
            .collect::<Vec<_>>()
    };
    for name in remaining_names {
        if let Some(value) = bindings.values.get(name) {
            indices.push(render_filter_value(value));
        }
    }

    if indices.is_empty() {
        String::new()
    } else {
        format!("[{}]", indices.join(","))
    }
}
