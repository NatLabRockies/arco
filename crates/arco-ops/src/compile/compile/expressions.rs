fn linearize_value_expr(
    expr: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, CompileError> {
    match expr {
        Expr::Number(value) => value.parse::<f64>().map(AffineExpr::constant).map_err(|_| {
            CompileError::InvalidFormulation {
                message: format!("invalid numeric literal `{value}`"),
                path: entrypoint.to_path_buf(),
            }
        }),
        Expr::Identifier(name) => {
            if let Some(binding) = bindings.values.get(name) {
                return Ok(AffineExpr::constant(numeric_filter_value(
                    binding,
                    &synthetic_constraint(name),
                    entrypoint,
                )?));
            }
            if let Some(expression) = named_expressions.get(name) {
                if let Some(generation_bindings) = expression_generation_bindings(name, program) {
                    if !generation_bindings.is_empty() {
                        return Err(CompileError::InvalidFormulation {
                            message: format!(
                                "indexed expression `{name}` expects {} index value(s), received 0",
                                generation_bindings.len()
                            ),
                            path: entrypoint.to_path_buf(),
                        });
                    }
                }
                return linearize_value_expr(
                    expression,
                    bindings,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                );
            }
            if find_variable_family(name, 0, variable_signatures).is_some()
                && instantiated_names.contains(name)
            {
                return Ok(AffineExpr::variable(name.clone(), 1.0));
            }
            Err(CompileError::InvalidFormulation {
                message: format!("unresolved symbol `{name}` in linear expression"),
                path: entrypoint.to_path_buf(),
            })
        }
        Expr::Indexed { target, indices } => linearize_indexed_expr(
            target,
            indices,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::Unary { op, expr } => {
            let value = linearize_value_expr(
                expr,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            match op {
                UnaryOp::Negate => Ok(value.scale(-1.0)),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = linearize_value_expr(
                left,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            let right = linearize_value_expr(
                right,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            match op {
                BinaryOp::Add => {
                    let mut value = left;
                    value.add_assign(right);
                    Ok(value)
                }
                BinaryOp::Subtract => Ok(left.subtract(right)),
                BinaryOp::Multiply => {
                    if left.terms.is_empty() {
                        Ok(right.scale(left.constant))
                    } else if right.terms.is_empty() {
                        Ok(left.scale(right.constant))
                    } else {
                        Err(CompileError::InvalidFormulation {
                            message: "non-linear multiplication is not supported".to_string(),
                            path: entrypoint.to_path_buf(),
                        })
                    }
                }
                BinaryOp::Divide => {
                    let denominator = right.as_scalar(entrypoint, "division denominator")?;
                    Ok(left.scale(1.0 / denominator))
                }
            }
        }
        Expr::FunctionCall { name, args } => {
            let evaluated_args = args
                .iter()
                .map(|arg| {
                    let result = linearize_value_expr(
                        arg,
                        bindings,
                        program,
                        inputs,
                        named_expressions,
                        variable_signatures,
                        instantiated_names,
                        entrypoint,
                    )?;
                    result.as_scalar(entrypoint, &format!("{name}() argument"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let value = evaluate_builtin_function(name, &evaluated_args, entrypoint)?;
            Ok(AffineExpr::constant(value))
        }
        Expr::Reduction(reduction) => linearize_reduction(
            reduction,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::String(_) | Expr::Boolean(_) | Expr::Comparison { .. } => {
            Err(CompileError::InvalidFormulation {
                message: "boolean and string expressions cannot appear in linear algebra"
                    .to_string(),
                path: entrypoint.to_path_buf(),
            })
        }
    }
}

fn evaluate_builtin_function(
    name: &str,
    args: &[f64],
    entrypoint: &Path,
) -> Result<f64, CompileError> {
    match (name, args.len()) {
        ("sqrt", 1) => Ok(args[0].sqrt()),
        ("abs", 1) => Ok(args[0].abs()),
        ("exp", 1) => Ok(args[0].exp()),
        ("ln", 1) => Ok(args[0].ln()),
        ("sin", 1) => Ok(args[0].sin()),
        ("cos", 1) => Ok(args[0].cos()),
        ("atan", 1) => Ok(args[0].atan()),
        ("pow", 2) => Ok(args[0].powf(args[1])),
        (name, n) => Err(CompileError::InvalidFormulation {
            message: format!(
                "{name}() received {n} argument(s), expected {}",
                if name == "pow" { 2 } else { 1 }
            ),
            path: entrypoint.to_path_buf(),
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_reduction(
    reduction: &arco_kdl::algebra::ReductionExpr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, CompileError> {
    let tuple_reduction_domain = tuple_reduction_domain_name(&reduction.bindings, program);
    let expanded =
        expand_reduction_bindings(&reduction.bindings, bindings, inputs, program, entrypoint)?;
    let mut total = AffineExpr::default();
    let mut matched_scope_count = 0usize;
    'outer: for scope in expanded {
        for filter in &reduction.filters {
            if !evaluate_reduction_filter(
                filter,
                &scope,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )? {
                continue 'outer;
            }
        }
        matched_scope_count += 1;
        total.add_assign(linearize_value_expr(
            &reduction.body,
            &scope,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?);
    }
    if matched_scope_count == 0 {
        if let Some(domain) = tuple_reduction_domain {
            return Err(CompileError::EmptyTupleReduction {
                domain,
                path: entrypoint.to_path_buf(),
            });
        }
    }
    Ok(total)
}

fn tuple_reduction_domain_name(
    bindings: &[arco_kdl::algebra::Binding],
    program: &SemanticProgram,
) -> Option<String> {
    let reverse_aliases = build_reverse_alias_lookup(program);
    let tuple_domains = bindings
        .iter()
        .filter_map(|binding| {
            let key = resolve_set_registry_key(binding.domain.as_str(), program, &reverse_aliases)?;
            let set = resolve_set_struct_by_name(key, program, &reverse_aliases)?;
            if set.tuple_rows.is_some() && set.tuple_components.is_some() {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    if tuple_domains.is_empty() {
        None
    } else {
        Some(tuple_domains.into_iter().collect::<Vec<_>>().join(","))
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_reduction_filter(
    filter: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<bool, CompileError> {
    if let Expr::Comparison { op, left, right } = filter {
        let left_affine = linearize_value_expr(
            left,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?;
        let right_affine = linearize_value_expr(
            right,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?;
        let left_value = left_affine.as_scalar(entrypoint, "reduction filter operand")?;
        let right_value = right_affine.as_scalar(entrypoint, "reduction filter operand")?;
        Ok(match op {
            ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
                (left_value - right_value).abs() < 1e-12
            }
            ComparisonOp::NotEqual => (left_value - right_value).abs() >= 1e-12,
            ComparisonOp::Less => left_value < right_value,
            ComparisonOp::LessEqual => left_value <= right_value,
            ComparisonOp::Greater => left_value > right_value,
            ComparisonOp::GreaterEqual => left_value >= right_value,
        })
    } else {
        let value = linearize_value_expr(
            filter,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?
        .as_scalar(entrypoint, "reduction filter expression")?;
        Ok(value.abs() >= 1e-12)
    }
}

fn expand_reduction_bindings(
    bindings: &[arco_kdl::algebra::Binding],
    current: &LinearizationBindings,
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<LinearizationBindings>, CompileError> {
    let reverse_aliases = build_reverse_alias_lookup(program);

    if let Some(first) = bindings.first() {
        let same_domain = bindings
            .iter()
            .all(|binding| binding.domain == first.domain);
        let name_bindings = bindings
            .iter()
            .map(|binding| match &binding.pattern {
                arco_kdl::algebra::BindingPattern::Name(name) => Some(name.clone()),
                arco_kdl::algebra::BindingPattern::Tuple(_) => None,
            })
            .collect::<Option<Vec<_>>>();

        if same_domain {
            if let (Some(names), Some(set)) =
                (name_bindings, program.set_registry.get(&first.domain))
            {
                if let (Some(tuple_components), Some(tuple_rows)) =
                    (set.tuple_components.as_ref(), set.tuple_rows.as_ref())
                {
                    let mut component_to_binding = BTreeMap::new();
                    for name in &names {
                        let component =
                            name.strip_suffix("_r").unwrap_or(name.as_str()).to_string();
                        component_to_binding.insert(component, name.clone());
                    }

                    let mut scopes = Vec::new();
                    for row in tuple_rows {
                        if row.len() != tuple_components.len() {
                            return Err(CompileError::InvalidFormulation {
                                message: format!(
                                    "tuple row arity mismatch in reduction over `{}`: expected `{}`, received `{}`",
                                    first.domain,
                                    tuple_components.len(),
                                    row.len()
                                ),
                                path: entrypoint.to_path_buf(),
                            });
                        }

                        let mut scope = current.clone();
                        let mut matches_anchor = true;
                        for (component, value) in tuple_components.iter().zip(row.iter()) {
                            if let Some(binding_name) = component_to_binding.get(component) {
                                scope.insert(
                                    binding_name.clone(),
                                    FilterValue::String(value.clone()),
                                );
                                continue;
                            }

                            if let Some(existing) = current.values.get(component) {
                                let tuple_value = FilterValue::String(value.clone());
                                if existing != &tuple_value {
                                    matches_anchor = false;
                                    break;
                                }
                            }
                        }

                        if matches_anchor {
                            scopes.push(scope);
                        }
                    }

                    if !scopes.is_empty() {
                        return Ok(scopes);
                    }
                }
            }
        }
    }

    let mut scopes = vec![current.clone()];

    for binding in bindings {
        let mut next = Vec::new();
        for scope in &scopes {
            match &binding.pattern {
                arco_kdl::algebra::BindingPattern::Name(name) => {
                    let values = reduction_values_for_binding_scope(
                        binding.domain.as_str(),
                        name,
                        scope,
                        inputs,
                        program,
                        &reverse_aliases,
                        entrypoint,
                    )?;
                    for value in &values {
                        let mut scoped = scope.clone();
                        scoped.insert(name.clone(), value.clone());
                        next.push(scoped);
                    }
                }
                arco_kdl::algebra::BindingPattern::Tuple(_) => {
                    return Err(CompileError::InvalidFormulation {
                        message: "tuple reduction bindings are not lowered yet".to_string(),
                        path: entrypoint.to_path_buf(),
                    });
                }
            }
        }
        scopes = next;
    }

    Ok(scopes)
}

fn reduction_values_for_binding_scope(
    domain: &str,
    binding_name: &str,
    scope: &LinearizationBindings,
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    reverse_aliases: &BTreeMap<&str, &str>,
    entrypoint: &Path,
) -> Result<Vec<FilterValue>, CompileError> {
    let Some(domain_key) = resolve_set_registry_key(domain, program, reverse_aliases) else {
        return reduction_domain_values(domain, inputs, program, entrypoint);
    };

    let Some(set) = resolve_set_struct_by_name(domain_key, program, reverse_aliases) else {
        return reduction_domain_values(domain, inputs, program, entrypoint);
    };

    let (Some(tuple_components), Some(tuple_rows)) =
        (set.tuple_components.as_ref(), set.tuple_rows.as_ref())
    else {
        return reduction_domain_values(domain, inputs, program, entrypoint);
    };

    tuple_reduction_binding_values(
        domain_key,
        binding_name,
        tuple_components,
        tuple_rows,
        scope,
        entrypoint,
    )
}

fn tuple_reduction_binding_values(
    domain_key: &str,
    binding_name: &str,
    tuple_components: &[String],
    tuple_rows: &[Vec<String>],
    scope: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<Vec<FilterValue>, CompileError> {
    let Some(binding_component_index) =
        tuple_reduction_component_index(binding_name, tuple_components)
    else {
        return Err(CompileError::InvalidFormulation {
            message: format!(
                "reduction binding `{binding_name}` does not match tuple domain `{domain_key}` components `{}`",
                tuple_components.join(",")
            ),
            path: entrypoint.to_path_buf(),
        });
    };

    let scoped_component_values = scope
        .values
        .iter()
        .filter_map(|(name, value)| {
            tuple_reduction_component_index(name, tuple_components).map(|index| (index, value))
        })
        .map(|(index, value)| {
            filter_value_to_key_component(value, entrypoint).map(|key_value| (index, key_value))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut projected_values = BTreeSet::new();

    'rows: for row in tuple_rows {
        if row.len() != tuple_components.len() {
            return Err(CompileError::InvalidFormulation {
                message: format!(
                    "tuple row arity mismatch for tuple reduction domain `{domain_key}`: expected `{}`, received `{}`",
                    tuple_components.len(),
                    row.len()
                ),
                path: entrypoint.to_path_buf(),
            });
        }

        for (component_index, scoped_value) in &scoped_component_values {
            if row[*component_index] != *scoped_value {
                continue 'rows;
            }
        }

        projected_values.insert(row[binding_component_index].clone());
    }

    Ok(projected_values
        .into_iter()
        .map(FilterValue::String)
        .collect())
}

fn tuple_reduction_component_index(name: &str, tuple_components: &[String]) -> Option<usize> {
    tuple_components
        .iter()
        .position(|component| component == name)
        .or_else(|| {
            name.strip_suffix("_r").and_then(|candidate| {
                tuple_components
                    .iter()
                    .position(|component| component == candidate)
            })
        })
}
