fn expand_generation_bindings(
    bindings: &[GenerationBinding],
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<LinearizationBindings>, LoweringError> {
    let mut scopes = vec![LinearizationBindings::default()];
    for binding in bindings {
        let values = reduction_domain_values(&binding.domain, inputs, program, entrypoint)?;
        let mut next = Vec::new();
        for scope in &scopes {
            for value in &values {
                let mut scope = scope.clone();
                scope.values.insert(binding.variable.clone(), value.clone());
                next.push(scope);
            }
        }
        scopes = next;
    }
    Ok(scopes)
}

fn reduction_domain_values(
    domain: &str,
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<FilterValue>, LoweringError> {
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
        "time" => Ok((1..=program.sets.time.steps)
            .map(|time| FilterValue::Number(time as f64))
            .collect()),
        _ => {
            if let Some(set) = program.set_registry.get(domain) {
                return Ok(set
                    .values
                    .iter()
                    .map(|v| FilterValue::String(v.clone()))
                    .collect());
            }
            if let Some(canonical) = program.set_aliases.get(domain) {
                if let Some(set) = program.set_registry.get(canonical.as_str()) {
                    return Ok(set
                        .values
                        .iter()
                        .map(|v| FilterValue::String(v.clone()))
                        .collect());
                }
            }
            Err(LoweringError::InvalidFormulation {
                message: format!("unsupported reduction domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            })
        }
    }
}

type InlineSelector = (String, String);
type InlineSelectorDomain = (String, Vec<InlineSelector>);

fn parse_inline_selector_domain(
    domain: &str,
    entrypoint: &Path,
) -> Result<Option<InlineSelectorDomain>, LoweringError> {
    let Some(start) = domain.find('[') else {
        return Ok(None);
    };
    let Some(end) = domain.rfind(']') else {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    };
    if end <= start || !domain[end + 1..].trim().is_empty() {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    }

    let base = domain[..start].trim().to_string();
    if base.is_empty() {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    }

    let body = &domain[start + 1..end];
    let bytes = body.as_bytes();
    let mut index = 0;
    let mut selectors = Vec::new();

    while index < bytes.len() {
        while index < bytes.len() && matches!(bytes[index] as char, ' ' | '\t' | ',') {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }

        let key_start = index;
        while index < bytes.len() {
            let character = bytes[index] as char;
            if character == '=' || character == ' ' || character == '\t' || character == ',' {
                break;
            }
            index += 1;
        }
        let key = body[key_start..index].trim();
        if key.is_empty() {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }

        while index < bytes.len() && matches!(bytes[index] as char, ' ' | '\t') {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] as char != '=' {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }
        index += 1;
        while index < bytes.len() && matches!(bytes[index] as char, ' ' | '\t') {
            index += 1;
        }

        if index >= bytes.len() {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }

        let value = if bytes[index] as char == '"' {
            index += 1;
            let mut escaped = false;
            let mut terminated = false;
            let mut literal = String::new();
            while index < bytes.len() {
                let character = bytes[index] as char;
                index += 1;
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
                return Err(LoweringError::InvalidFormulation {
                    message: format!("invalid inline selector domain `{domain}`"),
                    path: entrypoint.to_path_buf(),
                });
            }
            literal
        } else {
            let value_start = index;
            while index < bytes.len() {
                let character = bytes[index] as char;
                if matches!(character, ' ' | '\t' | ',') {
                    break;
                }
                index += 1;
            }
            body[value_start..index].trim().to_string()
        };

        if value.is_empty() {
            return Err(LoweringError::InvalidFormulation {
                message: format!("invalid inline selector domain `{domain}`"),
                path: entrypoint.to_path_buf(),
            });
        }

        selectors.push((key.to_string(), value));
    }

    if selectors.is_empty() {
        return Err(LoweringError::InvalidFormulation {
            message: format!("invalid inline selector domain `{domain}`"),
            path: entrypoint.to_path_buf(),
        });
    }

    Ok(Some((base, selectors)))
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
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, LoweringError> {
    let resolved = indices
        .iter()
        .map(|index| resolve_index_expr(index, bindings, entrypoint))
        .collect::<Result<Vec<_>, _>>()?;

    // Compute the candidate instance name using the same conventions as
    // instantiate_variable_instances / resolve_custom_index_domains.
    let candidate = candidate_instance_name(target, &resolved, entrypoint)?;

    if instantiated_names.contains(&candidate) {
        return Ok(AffineExpr::variable(candidate, 1.0));
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
        if !(1..=program.sets.time.steps as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            if let Some(value) =
                chronology_boundary_value(target, &asset_name, time, program, inputs, entrypoint)?
            {
                return Ok(AffineExpr::constant(value));
            }
            return Err(LoweringError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    if let [FilterValue::Number(_)] = resolved.as_slice() {
        let time = integer_time_index(&resolved[0], entrypoint)?;
        if !(1..=program.sets.time.steps as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            return Err(LoweringError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    parameter_reference_expr(target, &resolved, inputs, entrypoint)
}
