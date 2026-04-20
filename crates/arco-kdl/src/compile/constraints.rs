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
                constraints.extend(linearize_constraint_body(
                    constraint,
                    &bindings,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                )?);
            }
        } else {
            let generation_scopes = expand_generation_bindings(
                &constraint.generation_bindings,
                inputs,
                program,
                entrypoint,
                &constraint.name,
            )?;
            for scope in generation_scopes {
                if let Some(filter) = &constraint.generation_filter {
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
                        continue;
                    }
                }
                constraints.extend(linearize_constraint_body(
                    constraint,
                    &scope,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                )?);
            }
        }
    }
    Ok(constraints)
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
