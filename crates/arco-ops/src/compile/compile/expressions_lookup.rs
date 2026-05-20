fn bind_generated_expression_indices(
    target: &str,
    generation_bindings: &[arco_kdl::source::GenerationBinding],
    resolved: &[FilterValue],
    scoped_bindings: &mut LinearizationBindings,
    entrypoint: &Path,
) -> Result<(), CompileError> {
    if generation_bindings.is_empty() {
        return Ok(());
    }

    if generation_bindings.len() != resolved.len() {
        return Err(CompileError::InvalidFormulation {
            message: format!(
                "indexed expression `{target}` expects {} index value(s), received {}",
                generation_bindings.len(),
                resolved.len()
            ),
            path: entrypoint.to_path_buf(),
        });
    }

    for (binding, value) in generation_bindings.iter().zip(resolved.iter()) {
        scoped_bindings.insert(binding.variable.clone(), value.clone());
    }

    Ok(())
}

fn candidate_instance_name(
    target: &str,
    resolved: &[FilterValue],
    entrypoint: &Path,
) -> Result<String, CompileError> {
    match resolved {
        [FilterValue::String(a), FilterValue::Number(_)] => {
            let time = integer_time_index(&resolved[1], entrypoint)?;
            Ok(indexed_name(target, a, time as usize))
        }
        [FilterValue::String(a)] => Ok(asset_indexed_name(target, a)),
        [FilterValue::Number(_)] => {
            let time = integer_time_index(&resolved[0], entrypoint)?;
            Ok(time_name(target, time as usize))
        }
        _ => {
            // General case for custom index domains: join all values as
            // strings, matching the format in resolve_custom_index_domains.
            let parts: Vec<String> = resolved
                .iter()
                .map(|v| match v {
                    FilterValue::String(s) => Ok(s.clone()),
                    FilterValue::Number(n) => {
                        if n.fract() == 0.0 {
                            Ok((*n as i64).to_string())
                        } else {
                            Ok(n.to_string())
                        }
                    }
                    FilterValue::Boolean(_) => Err(CompileError::InvalidFormulation {
                        message: format!("unsupported boolean index in reference to `{target}`"),
                        path: entrypoint.to_path_buf(),
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("{}[{}]", target, parts.join(",")))
        }
    }
}

/// Find the variable family key in the signatures map by matching target
/// name and index arity. Used for traceability and to detect whether a
/// target is a known variable family.
fn find_variable_family<'a>(
    target: &str,
    arity: usize,
    variable_signatures: &'a BTreeMap<String, FamilySignature>,
) -> Option<&'a str> {
    variable_signatures
        .iter()
        .find(|(_key, sig)| sig.target == target && sig.indices.len() == arity)
        .map(|(key, _)| key.as_str())
}

fn parameter_name_known(target: &str, program: &SemanticProgram, inputs: &ScenarioInputs) -> bool {
    if inputs.series.contains_key(target)
        || inputs.indexed.contains_key(target)
        || inputs.asset_data.contains_key(target)
        || inputs.generic_data.contains_key(target)
    {
        return true;
    }

    program.parameters.series.iter().any(|name| name == target)
        || program.parameters.indexed.iter().any(|name| name == target)
        || program.parameters.asset.iter().any(|name| name == target)
        || program
            .set_params
            .values()
            .any(|parameters| parameters.contains_key(target))
}

fn parameter_reference_expr(
    target: &str,
    resolved: &[FilterValue],
    inputs: &ScenarioInputs,
    entrypoint: &Path,
) -> Result<AffineExpr, CompileError> {
    let references_generic_table = inputs.generic_data.contains_key(target);
    if let Some(value) = generic_data_value(&inputs.generic_data, target, resolved, entrypoint)? {
        return Ok(AffineExpr::constant(value));
    }
    if references_generic_table {
        return Err(CompileError::MissingDataPoint {
            name: target.to_string(),
            key: format_filter_lookup_key(resolved, entrypoint)?,
            path: entrypoint.to_path_buf(),
        });
    }

    let value = match resolved {
        [index] => {
            if let FilterValue::String(name) = index {
                if find_asset(inputs, name).is_some() {
                    asset_parameter_value(inputs, target, name)
                        .or_else(|| {
                            inputs
                                .asset_data
                                .get(target)
                                .and_then(|values| values.get(name))
                                .copied()
                        })
                        .unwrap_or(0.0)
                } else if let Some(member_params) = inputs.set_params.get(name) {
                    member_params.get(target).copied().unwrap_or(0.0)
                } else {
                    return Err(CompileError::MissingAsset {
                        name: name.clone(),
                        path: entrypoint.to_path_buf(),
                    });
                }
            } else {
                let time = integer_time_index(index, entrypoint)? as usize;
                series_value(&inputs.series, target, time, entrypoint)?
            }
        }
        [FilterValue::String(asset_name), FilterValue::Number(time)] => {
            if time.fract() != 0.0 || *time < 0.0 {
                return Err(CompileError::InvalidFormulation {
                    message: format!("time index `{time}` must be a non-negative integer"),
                    path: entrypoint.to_path_buf(),
                });
            }
            indexed_value(
                &inputs.indexed,
                target,
                asset_name,
                *time as usize,
                entrypoint,
            )?
        }
        _ => {
            return Err(CompileError::InvalidFormulation {
                message: format!("unsupported parameter reference `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    };
    Ok(AffineExpr::constant(value))
}

fn chronology_boundary_value(
    target: &str,
    asset_name: &str,
    time: i64,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    entrypoint: &Path,
) -> Result<Option<f64>, CompileError> {
    if time == 0 && target == "soc" && program.chronology.initial_boundary.is_some() {
        let asset = find_asset(inputs, asset_name).ok_or_else(|| CompileError::MissingAsset {
            name: asset_name.to_string(),
            path: entrypoint.to_path_buf(),
        })?;
        return asset_parameter(asset, "initial_soc_mwh", entrypoint).map(Some);
    }
    if time == 0 && target == "commit" && program.chronology.initial_commitment_boundary.is_some() {
        return asset_data_value(
            &inputs.asset_data,
            "initial_commitment",
            asset_name,
            entrypoint,
        )
        .map(Some);
    }
    if time == 0
        && target == "generation"
        && program.chronology.initial_commitment_boundary.is_some()
    {
        let asset = find_asset(inputs, asset_name).ok_or_else(|| CompileError::MissingAsset {
            name: asset_name.to_string(),
            path: entrypoint.to_path_buf(),
        })?;
        let p_min = asset_parameter(asset, "p_min", entrypoint)?;
        let initial_commitment = asset_data_value(
            &inputs.asset_data,
            "initial_commitment",
            asset_name,
            entrypoint,
        )?;
        return Ok(Some(p_min * initial_commitment));
    }
    Ok(None)
}
