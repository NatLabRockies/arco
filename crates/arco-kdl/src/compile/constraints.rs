fn compile_constraint_instances(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<Vec<LinearConstraint>, CompileError> {
    let mut constraints = Vec::new();
    for constraint in &program.active_constraints {
        let binding_order = constraint
            .generation_bindings
            .iter()
            .map(|binding| binding.variable.clone())
            .collect::<Vec<_>>();
        let mut empty_subset_keys = BTreeSet::new();

        if constraint.generation_bindings.is_empty() {
            for bindings in
                constraint_instance_bindings(constraint, inputs, program.sets.time.steps)
            {
                let asset = bindings_asset(&bindings, inputs);
                let time = bindings_time(&bindings, entrypoint)?;
                if let Some(filter) = &constraint.generation_filter {
                    if !evaluate_constraint_filter(
                        filter,
                        constraint,
                        FilterScope { asset, time },
                        inputs,
                        entrypoint,
                    )? {
                        continue;
                    }
                }

                match linearize_constraint_body(
                    constraint,
                    &bindings,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                ) {
                    Ok(linearized) => constraints.extend(linearized),
                    Err(CompileError::EmptyTupleReduction { .. }) => {
                        empty_subset_keys.insert(constraint_scope_key(&bindings, &binding_order));
                    }
                    Err(error) => return Err(error),
                }
            }
        } else {
            let generation_scopes = expand_generation_bindings(
                &constraint.generation_bindings,
                inputs,
                program,
                entrypoint,
                &constraint.diagnostic_id,
            )?;
            for scope in generation_scopes {
                if let Some(filter) = &constraint.generation_filter {
                    match evaluate_reduction_filter(
                        filter,
                        &scope,
                        program,
                        inputs,
                        named_expressions,
                        variable_signatures,
                        instantiated_names,
                        entrypoint,
                    ) {
                        Ok(false) => continue,
                        Ok(true) => {}
                        Err(CompileError::EmptyTupleReduction { .. }) => {
                            empty_subset_keys.insert(constraint_scope_key(&scope, &binding_order));
                            continue;
                        }
                        Err(error) => return Err(error),
                    }
                }

                match linearize_constraint_body(
                    constraint,
                    &scope,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                ) {
                    Ok(linearized) => constraints.extend(linearized),
                    Err(CompileError::EmptyTupleReduction { .. }) => {
                        empty_subset_keys.insert(constraint_scope_key(&scope, &binding_order));
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        if !empty_subset_keys.is_empty() {
            return Err(CompileError::InvalidFormulation {
                message: format!(
                    "empty constraint-relevant tuple subset for `{}` at keys: {}",
                    constraint.diagnostic_id,
                    empty_subset_keys.into_iter().collect::<Vec<_>>().join("; ")
                ),
                path: entrypoint.to_path_buf(),
            });
        }
    }
    Ok(constraints)
}

fn constraint_scope_key(bindings: &LinearizationBindings, binding_order: &[String]) -> String {
    let mut values = Vec::new();
    if binding_order.is_empty() {
        for value in bindings.values.values() {
            values.push(render_filter_value(value));
        }
    } else {
        for name in binding_order {
            if let Some(value) = bindings.values.get(name) {
                values.push(render_filter_value(value));
            }
        }
    }

    if values.is_empty() {
        "<scalar>".to_string()
    } else {
        values.join(",")
    }
}

fn render_filter_value(value: &FilterValue) -> String {
    match value {
        FilterValue::String(value) => value.clone(),
        FilterValue::Number(value) => {
            if value.fract() == 0.0 {
                (*value as i64).to_string()
            } else {
                value.to_string()
            }
        }
        FilterValue::Boolean(value) => value.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_constraint_body(
    constraint: &ResolvedConstraint,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<Vec<LinearConstraint>, CompileError> {
    let suffix = constraint_binding_suffix(bindings, entrypoint)?;
    match &constraint.expression {
        ConstraintBody::Comparison { op, left, right } => Ok(vec![linearize_comparison(
            format!("{}{}", constraint.name, suffix),
            *op,
            left,
            right,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?]),
        ConstraintBody::Range {
            lower,
            lower_op,
            middle,
            upper_op,
            upper,
        } => Ok(vec![
            linearize_comparison(
                format!("{}{}_lower", constraint.name, suffix),
                *lower_op,
                lower,
                middle,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?,
            linearize_comparison(
                format!("{}{}_upper", constraint.name, suffix),
                *upper_op,
                middle,
                upper,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?,
        ]),
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_comparison(
    name: String,
    op: ComparisonOp,
    left: &Expr,
    right: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<LinearConstraint, CompileError> {
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
    let expression = left.subtract(right);
    let sense = comparison_to_constraint_sense(op, entrypoint)?;
    Ok(LinearConstraint {
        name,
        sense,
        rhs: -expression.constant,
        terms: expression.into_terms(),
    })
}

fn comparison_to_constraint_sense(
    op: ComparisonOp,
    path: &Path,
) -> Result<ConstraintSense, CompileError> {
    match op {
        ComparisonOp::Equal | ComparisonOp::DoubleEqual => Ok(ConstraintSense::Equal),
        ComparisonOp::LessEqual => Ok(ConstraintSense::LessEqual),
        ComparisonOp::GreaterEqual => Ok(ConstraintSense::GreaterEqual),
        ComparisonOp::Less | ComparisonOp::Greater | ComparisonOp::NotEqual => {
            Err(CompileError::InvalidFormulation {
                message: format!(
                    "strict or not-equal comparison `{op}` is not supported in linear constraints"
                ),
                path: path.to_path_buf(),
            })
        }
    }
}
