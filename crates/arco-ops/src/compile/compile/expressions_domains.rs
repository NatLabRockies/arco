fn expand_generation_bindings(
    bindings: &[GenerationBinding],
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
    binding_context: &str,
) -> Result<Vec<LinearizationBindings>, CompileError> {
    if let Some(tuple_scopes) =
        expand_tuple_generation_bindings(bindings, program, entrypoint, binding_context)?
    {
        return Ok(tuple_scopes);
    }
    let mut scopes = vec![LinearizationBindings::default()];
    for binding in bindings {
        let values = reduction_domain_values(&binding.domain, inputs, program, entrypoint)?;
        let mut next = Vec::new();
        for scope in &scopes {
            for value in &values {
                let mut scope = scope.clone();
                scope.insert(binding.variable.clone(), value.clone());
                next.push(scope);
            }
        }
        scopes = next;
    }
    Ok(scopes)
}

fn expand_tuple_generation_bindings(
    bindings: &[GenerationBinding],
    program: &SemanticProgram,
    entrypoint: &Path,
    binding_context: &str,
) -> Result<Option<Vec<LinearizationBindings>>, CompileError> {
    if bindings.is_empty() {
        return Ok(None);
    }

    let reverse_aliases = build_reverse_alias_lookup(program);
    let first_domain = bindings[0].domain.as_str();
    let Some(first_key) = resolve_set_registry_key(first_domain, program, &reverse_aliases) else {
        return Ok(None);
    };

    for binding in bindings.iter().skip(1) {
        let Some(key) =
            resolve_set_registry_key(binding.domain.as_str(), program, &reverse_aliases)
        else {
            return Ok(None);
        };
        if key != first_key {
            return Ok(None);
        }
    }

    let Some(set) = resolve_set_struct_by_name(first_key, program, &reverse_aliases) else {
        return Ok(None);
    };
    let (Some(tuple_components), Some(tuple_rows)) =
        (set.tuple_components.as_ref(), set.tuple_rows.as_ref())
    else {
        return Ok(None);
    };

    let received_components = bindings
        .iter()
        .map(|binding| binding.variable.clone())
        .collect::<Vec<_>>();
    let uses_tuple_shorthand = bindings.len() == 1 && {
        let binding = &bindings[0];
        binding.variable == binding.domain
    };
    if !uses_tuple_shorthand && tuple_components != &received_components {
        return Err(CompileError::InvalidFormulation {
            message: tuple_domain_index_order_mismatch_message(
                binding_context,
                first_key,
                tuple_components,
                &received_components,
            ),
            path: entrypoint.to_path_buf(),
        });
    }

    let mut scopes = Vec::with_capacity(tuple_rows.len());
    for row in tuple_rows {
        if row.len() != tuple_components.len() {
            return Err(CompileError::InvalidFormulation {
                message: format!(
                    "tuple row arity mismatch for `{binding_context}`: expected `{}`, received `{}`",
                    tuple_components.len(),
                    row.len()
                ),
                path: entrypoint.to_path_buf(),
            });
        }

        let mut scope = LinearizationBindings::default();
        if uses_tuple_shorthand {
            for (component, value) in tuple_components.iter().zip(row.iter()) {
                scope.insert(component.clone(), FilterValue::String(value.clone()));
            }
        } else {
            for (binding, value) in bindings.iter().zip(row.iter()) {
                scope.insert(binding.variable.clone(), FilterValue::String(value.clone()));
            }
        }
        scopes.push(scope);
    }

    Ok(Some(scopes))
}

fn reduction_domain_values(
    domain: &str,
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<FilterValue>, CompileError> {
    if let Some((base, selectors)) = parse_inline_selector_domain(domain, entrypoint)? {
        let base_values = reduction_domain_values(base.as_str(), inputs, program, entrypoint)?;
        let filtered = base_values
            .into_iter()
            .filter(|value| domain_value_matches_selectors(value, &selectors, inputs))
            .collect();
        return Ok(filtered);
    }

    match domain {
        "assets" => Ok(inputs
            .assets
            .iter()
            .map(|asset| FilterValue::String(asset.name.clone()))
            .collect()),
        "candidate_assets" => Ok(inputs
            .assets
            .iter()
            .filter(|asset| asset.candidate)
            .map(|asset| FilterValue::String(asset.name.clone()))
            .collect()),
        domain if program.is_time_set_name(domain) => Ok((1..=program.time_steps())
            .map(|time| FilterValue::Number(time as f64))
            .collect()),
        _ => program
            .resolve_set(domain)
            .map(|set| {
                set.values
                    .iter()
                    .map(|value| FilterValue::String(value.clone()))
                    .collect()
            })
            .ok_or_else(|| CompileError::InvalidFormulation {
                message: format!("unsupported reduction domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            }),
    }
}

type InlineSelector = (String, String);
type InlineSelectorDomain = (String, Vec<InlineSelector>);

fn parse_inline_selector_domain(
    domain: &str,
    entrypoint: &Path,
) -> Result<Option<InlineSelectorDomain>, CompileError> {
    let Some(start) = domain.find('[') else {
        return Ok(None);
    };
    let Some(end) = domain.rfind(']') else {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    };
    if end <= start || !domain[end + 1..].trim().is_empty() {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }

    let base = domain[..start].trim().to_string();
    if base.is_empty() {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }

    let body = &domain[start + 1..end];
    let bytes = body.as_bytes();
    let mut index = 0;
    let mut selectors = Vec::new();

    while index < bytes.len() {
        skip_selector_delimiters(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }

        let key = read_selector_key(body, bytes, &mut index, domain, entrypoint)?;
        expect_selector_equals(bytes, &mut index, domain, entrypoint)?;
        let value = read_selector_value(body, bytes, &mut index, domain, entrypoint)?;

        selectors.push((key, value));
    }

    if selectors.is_empty() {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }

    Ok(Some((base, selectors)))
}

fn skip_selector_delimiters(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && matches!(bytes[*index] as char, ' ' | '\t' | ',') {
        *index += 1;
    }
}

fn skip_selector_whitespace(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && matches!(bytes[*index] as char, ' ' | '\t') {
        *index += 1;
    }
}

fn invalid_inline_selector_domain(domain: &str, entrypoint: &Path) -> CompileError {
    CompileError::InvalidFormulation {
        message: format!("invalid inline selector domain `{domain}`"),
        path: entrypoint.to_path_buf(),
    }
}

fn read_selector_key(
    body: &str,
    bytes: &[u8],
    index: &mut usize,
    domain: &str,
    entrypoint: &Path,
) -> Result<String, CompileError> {
    let key_start = *index;
    while *index < bytes.len() {
        let character = bytes[*index] as char;
        if character == '=' || character == ' ' || character == '\t' || character == ',' {
            break;
        }
        *index += 1;
    }
    let key = body[key_start..*index].trim();
    if key.is_empty() {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }
    Ok(key.to_string())
}

fn expect_selector_equals(
    bytes: &[u8],
    index: &mut usize,
    domain: &str,
    entrypoint: &Path,
) -> Result<(), CompileError> {
    skip_selector_whitespace(bytes, index);
    if *index >= bytes.len() || bytes[*index] as char != '=' {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }
    *index += 1;
    skip_selector_whitespace(bytes, index);
    if *index >= bytes.len() {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }
    Ok(())
}

fn read_selector_value(
    body: &str,
    bytes: &[u8],
    index: &mut usize,
    domain: &str,
    entrypoint: &Path,
) -> Result<String, CompileError> {
    let value = if bytes[*index] as char == '"' {
        read_quoted_selector_value(bytes, index, domain, entrypoint)?
    } else {
        read_unquoted_selector_value(body, bytes, index)
    };

    if value.is_empty() {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }
    Ok(value)
}

fn read_quoted_selector_value(
    bytes: &[u8],
    index: &mut usize,
    domain: &str,
    entrypoint: &Path,
) -> Result<String, CompileError> {
    *index += 1;
    let mut escaped = false;
    let mut terminated = false;
    let mut literal = String::new();

    while *index < bytes.len() {
        let character = bytes[*index] as char;
        *index += 1;

        if escaped {
            literal.push(character);
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => {
                terminated = true;
                break;
            }
            _ => literal.push(character),
        }
    }

    if escaped || !terminated {
        return Err(invalid_inline_selector_domain(domain, entrypoint));
    }

    Ok(literal)
}

fn read_unquoted_selector_value(body: &str, bytes: &[u8], index: &mut usize) -> String {
    let value_start = *index;
    while *index < bytes.len() {
        let character = bytes[*index] as char;
        if matches!(character, ' ' | '\t' | ',') {
            break;
        }
        *index += 1;
    }
    body[value_start..*index].trim().to_string()
}

fn domain_value_matches_selectors(
    value: &FilterValue,
    selectors: &[(String, String)],
    inputs: &ScenarioInputs,
) -> bool {
    let FilterValue::String(member) = value else {
        return false;
    };
    let key = vec![member.clone()];

    selectors.iter().all(|(field, expected)| {
        if let Some(param_values) = inputs.set_params.get(field) {
            if let Some(actual) = param_values.get(member) {
                return numeric_or_string_match(*actual, expected);
            }
        }

        if let Some(table) = inputs.generic_data.get(field) {
            if let Some(actual) = table.values.get(&key) {
                return numeric_or_string_match(*actual, expected);
            }
        }

        false
    })
}

fn numeric_or_string_match(actual: f64, expected: &str) -> bool {
    if let Ok(expected_number) = expected.parse::<f64>() {
        return (actual - expected_number).abs() < 1e-9;
    }

    if expected.eq_ignore_ascii_case("true") {
        return (actual - 1.0).abs() < 1e-9;
    }
    if expected.eq_ignore_ascii_case("false") {
        return actual.abs() < 1e-9;
    }

    let actual_string = if actual.fract().abs() < 1e-9 {
        (actual as i64).to_string()
    } else {
        actual.to_string()
    };
    actual_string == expected
}

#[allow(clippy::too_many_arguments)]
fn linearize_indexed_expr(
    target: &str,
    indices: &[Expr],
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, CompileError> {
    let resolved = match (
        indices.len(),
        resolve_tuple_key_index(indices, bindings, program),
    ) {
        (1, Some(tuple_key_values)) => tuple_key_values,
        _ => resolve_index_values(indices, bindings, named_expressions, entrypoint)?,
    };

    // Compute the candidate instance name using the same conventions as
    // instantiate_variable_instances / resolve_custom_index_domains.
    let candidate = candidate_instance_name(target, &resolved, entrypoint)?;

    if instantiated_names.contains(&candidate) {
        return Ok(AffineExpr::variable(candidate, 1.0));
    }

    if let Some(expression) = named_expressions.get(target) {
        let mut scoped_bindings = bindings.clone();
        if let Some(generation_bindings) =
            expression_generation_bindings(target, program, expression_generation_index)
        {
            bind_generated_expression_indices(
                target,
                generation_bindings,
                &resolved,
                &mut scoped_bindings,
                entrypoint,
            )?;
        }

        if let Some(filter) =
            expression_generation_filter(target, program, expression_generation_index)
        {
            if !evaluate_reduction_filter(
                filter,
                &scoped_bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )? {
                return Ok(AffineExpr::default());
            }
        }

        return linearize_value_expr(
            expression,
            &scoped_bindings,
            program,
            inputs,
            named_expressions,
            expression_generation_index,
            variable_signatures,
            instantiated_names,
            entrypoint,
        );
    }

    // The candidate was not found in the instantiated set. Before falling
    // through to parameter lookup, handle chronology boundary cases for
    // [String, Number] references where the time index is out of range.
    if let [FilterValue::String(_), FilterValue::Number(_)] = resolved.as_slice() {
        let synthetic = synthetic_constraint(target);
        let asset_name = string_filter_value(&resolved[0], &synthetic, entrypoint)?;
        let time = integer_time_index(&resolved[1], entrypoint)?;

        // Only attempt chronology handling when the time is out of the
        // normal 1..=steps range AND a variable family with matching
        // arity exists (so we know this target is a variable, not a
        // parameter that happens to be missing).
        if !(1..=program.time_steps() as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            if let Some(value) =
                chronology_boundary_value(target, &asset_name, time, program, inputs, entrypoint)?
            {
                return Ok(AffineExpr::constant(value));
            }
            return Err(CompileError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    if let [FilterValue::Number(_)] = resolved.as_slice() {
        let time = integer_time_index(&resolved[0], entrypoint)?;
        if !(1..=program.time_steps() as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            return Err(CompileError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    if !parameter_name_known(target, program, inputs) {
        return Err(CompileError::MissingDeclaration {
            kind: "parameter",
            name: target.to_string(),
            path: entrypoint.to_path_buf(),
        });
    }

    parameter_reference_expr(target, &resolved, inputs, entrypoint)
}

fn resolve_index_values(
    indices: &[Expr],
    bindings: &LinearizationBindings,
    named_expressions: &BTreeMap<String, Expr>,
    entrypoint: &Path,
) -> Result<Vec<FilterValue>, CompileError> {
    indices
        .iter()
        .map(|index| resolve_index_expr(index, bindings, named_expressions, entrypoint))
        .collect::<Result<Vec<_>, _>>()
}

fn resolve_tuple_key_index(
    indices: &[Expr],
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
) -> Option<Vec<FilterValue>> {
    let index = indices.first()?;
    let Expr::Identifier(tuple_domain) = index else {
        return None;
    };

    let reverse_aliases = build_reverse_alias_lookup(program);
    let key = resolve_set_registry_key(tuple_domain, program, &reverse_aliases)?;
    let set = resolve_set_struct_by_name(key, program, &reverse_aliases)?;
    let tuple_components = set.tuple_components.as_ref()?;

    let mut resolved = Vec::with_capacity(tuple_components.len());
    for component in tuple_components {
        let value = bindings.values.get(component)?;
        resolved.push(value.clone());
    }

    Some(resolved)
}

fn build_expression_generation_index(program: &SemanticProgram) -> ExpressionGenerationIndex {
    program
        .active_expressions
        .iter()
        .enumerate()
        .map(|(index, expression)| (expression.name.clone(), index))
        .collect()
}

fn expression_generation_bindings<'a>(
    name: &str,
    program: &'a SemanticProgram,
    expression_generation_index: &ExpressionGenerationIndex,
) -> Option<&'a [arco_kdl::source::GenerationBinding]> {
    let expression_index = *expression_generation_index.get(name)?;
    program
        .active_expressions
        .get(expression_index)
        .map(|expression| expression.generation_bindings.as_slice())
}

fn expression_generation_filter<'a>(
    name: &str,
    program: &'a SemanticProgram,
    expression_generation_index: &ExpressionGenerationIndex,
) -> Option<&'a Expr> {
    let expression_index = *expression_generation_index.get(name)?;
    program
        .active_expressions
        .get(expression_index)
        .and_then(|expression| expression.generation_filter.as_ref())
}
