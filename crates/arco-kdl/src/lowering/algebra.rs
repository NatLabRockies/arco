fn lower_algebra(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    entrypoint: &Path,
) -> Result<AlgebraicProblem, LoweringError> {
    let named_expressions = program
        .active_expressions
        .iter()
        .map(|expression| (expression.name.clone(), expression.formula.clone()))
        .collect::<BTreeMap<_, _>>();
    let variable_signatures = program
        .variable_families
        .iter()
        .map(|family| (family.render(), family.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut variable_instances =
        instantiate_variable_instances(program, inputs, &variable_signatures, entrypoint)?;
    let instantiated_names: BTreeSet<String> =
        variable_instances.iter().map(|i| i.name.clone()).collect();
    let mut constraints = lower_constraint_instances(
        program,
        inputs,
        &named_expressions,
        &variable_signatures,
        &instantiated_names,
        entrypoint,
    )?;
    constraints.extend(emit_terminal_boundary_constraints(
        program,
        inputs,
        &variable_signatures,
        entrypoint,
    )?);

    let objective = linearize_value_expr(
        &program.active_objective.expression,
        &LinearizationBindings::default(),
        program,
        inputs,
        &named_expressions,
        &variable_signatures,
        &instantiated_names,
        entrypoint,
    )?;
    let reports = program
        .active_reports
        .iter()
        .map(|report| {
            linearize_value_expr(
                &report.formula,
                &LinearizationBindings::default(),
                program,
                inputs,
                &named_expressions,
                &variable_signatures,
                &instantiated_names,
                entrypoint,
            )
            .map(|linearized| LinearReport {
                name: report.name.clone(),
                constant: linearized.constant,
                terms: linearized.into_terms(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    variable_instances.sort_by(|a, b| a.name.cmp(&b.name));
    constraints.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(AlgebraicProblem {
        variable_instances,
        constraints,
        objective: LinearObjective {
            name: program.active_objective.name.clone(),
            sense: objective_sense(&program.active_objective.sense),
            constant: objective.constant,
            terms: objective.into_terms(),
        },
        reports,
    })
}

impl AffineExpr {
    fn constant(value: f64) -> Self {
        Self {
            constant: value,
            terms: BTreeMap::new(),
        }
    }

    fn variable(name: String, coefficient: f64) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(name, coefficient);
        Self {
            constant: 0.0,
            terms,
        }
    }

    fn add_assign(&mut self, other: Self) {
        self.constant += other.constant;
        for (name, coefficient) in other.terms {
            let entry = self.terms.entry(name).or_default();
            *entry += coefficient;
        }
        self.terms
            .retain(|_, coefficient| coefficient.abs() >= 1e-12);
    }

    fn subtract(self, other: Self) -> Self {
        let mut value = self;
        value.add_assign(other.scale(-1.0));
        value
    }

    fn scale(mut self, factor: f64) -> Self {
        self.constant *= factor;
        for coefficient in self.terms.values_mut() {
            *coefficient *= factor;
        }
        self.terms
            .retain(|_, coefficient| coefficient.abs() >= 1e-12);
        self
    }

    fn as_scalar(&self, path: &Path, context: &str) -> Result<f64, LoweringError> {
        if self.terms.is_empty() {
            Ok(self.constant)
        } else {
            Err(LoweringError::InvalidFormulation {
                message: format!("{context} must remain scalar"),
                path: path.to_path_buf(),
            })
        }
    }

    fn into_terms(self) -> Vec<LinearTerm> {
        self.terms
            .into_iter()
            .filter(|(_, coefficient)| coefficient.abs() >= 1e-12)
            .map(|(variable_name, coefficient)| LinearTerm {
                variable_name,
                coefficient,
            })
            .collect()
    }
}

fn instantiate_variable_instances(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    entrypoint: &Path,
) -> Result<Vec<VariableInstance>, LoweringError> {
    let mut instances = Vec::new();
    for (family, signature) in variable_signatures {
        let overrides = program.variable_overrides.get(&signature.target);
        match signature.indices.as_slice() {
            [asset_index, time_index] if asset_index == "a" && time_index == "t" => {
                for asset in &inputs.assets {
                    if !variable_instance_is_active(&signature.target, Some(asset)) {
                        continue;
                    }
                    for time in 1..=program.sets.time.steps {
                        instances.push(variable_instance_from_signature(
                            family,
                            signature,
                            Some(asset),
                            Some(time),
                            overrides,
                            entrypoint,
                        )?);
                    }
                }
            }
            [asset_index] if asset_index == "a" => {
                for asset in &inputs.assets {
                    if !variable_instance_is_active(&signature.target, Some(asset)) {
                        continue;
                    }
                    instances.push(variable_instance_from_signature(
                        family,
                        signature,
                        Some(asset),
                        None,
                        overrides,
                        entrypoint,
                    )?);
                }
            }
            [time_index] if time_index == "t" => {
                for time in 1..=program.sets.time.steps {
                    instances.push(variable_instance_from_signature(
                        family,
                        signature,
                        None,
                        Some(time),
                        overrides,
                        entrypoint,
                    )?);
                }
            }
            _ => {
                // Try to resolve custom index domains via the set registry.
                let resolved = resolve_custom_index_domains(
                    signature, program, inputs, family, overrides, entrypoint,
                )?;
                instances.extend(resolved);
            }
        }
    }
    Ok(instances)
}

/// Expand variable instances for families with custom index domains that
/// don't match the built-in "a"/"t" patterns. Each index is resolved by
/// checking its explicit domain binding (from `IndexDecl`) or falling back
/// to the set_registry. Indices bound to "time" produce numeric time steps;
/// all others produce string-named instances.
fn resolve_custom_index_domains(
    signature: &FamilySignature,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    family: &str,
    overrides: Option<&VariableDeclOverrides>,
    entrypoint: &Path,
) -> Result<Vec<VariableInstance>, LoweringError> {
    // Resolve each index to its domain values.
    let mut domain_values: Vec<Vec<String>> = Vec::new();
    for index_name in &signature.indices {
        let values = resolve_single_index_domain(
            index_name, signature, program, inputs, family, entrypoint,
        )?;
        domain_values.push(values);
    }

    // Cartesian product of all domain values.
    let mut combos: Vec<Vec<String>> = vec![vec![]];
    for values in &domain_values {
        let mut next = Vec::new();
        for combo in &combos {
            for value in values {
                let mut extended = combo.clone();
                extended.push(value.clone());
                next.push(extended);
            }
        }
        combos = next;
    }

    let mut instances = Vec::new();
    for combo in &combos {
        let name = format!("{}[{}]", signature.target, combo.join(","));
        let (lower, upper, kind) =
            variable_domain_policy(&signature.target, None, overrides, entrypoint)?;
        instances.push(VariableInstance {
            name,
            family: family.to_string(),
            lower,
            upper,
            kind,
        });
    }
    Ok(instances)
}

/// Resolve values for a single index. Checks the signature's explicit domain
/// binding first, then falls back to known index names ("a" -> assets,
/// "t" -> time), and finally looks up any set matching the index name in
/// the registry.
fn resolve_single_index_domain(
    index_name: &str,
    signature: &FamilySignature,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    family: &str,
    entrypoint: &Path,
) -> Result<Vec<String>, LoweringError> {
    // Check explicit domain binding from IndexDecl.
    if let Some(domain) = signature.index_domains.get(index_name) {
        if domain == "time" {
            return Ok((1..=program.sets.time.steps)
                .map(|t| t.to_string())
                .collect());
        }
        if domain == "assets" {
            return Ok(inputs.assets.iter().map(|a| a.name.clone()).collect());
        }
        if let Some(set) = program.set_registry.get(domain.as_str()) {
            return Ok(set.values.clone());
        }
        if let Some(canonical) = program.set_aliases.get(domain.as_str()) {
            if let Some(set) = program.set_registry.get(canonical.as_str()) {
                return Ok(set.values.clone());
            }
        }
        return Err(LoweringError::InvalidFormulation {
            message: format!(
                "index `{index_name}` in `{family}` references unknown set `{domain}`"
            ),
            path: entrypoint.to_path_buf(),
        });
    }

    // Fallback: infer from conventional index names.
    match index_name {
        "a" => Ok(inputs.assets.iter().map(|a| a.name.clone()).collect()),
        "t" => Ok((1..=program.sets.time.steps)
            .map(|t| t.to_string())
            .collect()),
        _ => {
            // Last resort: check if the index name itself is a set in the registry.
            if let Some(set) = program.set_registry.get(index_name) {
                return Ok(set.values.clone());
            }
            if let Some(canonical) = program.set_aliases.get(index_name) {
                if let Some(set) = program.set_registry.get(canonical.as_str()) {
                    return Ok(set.values.clone());
                }
            }
            Err(LoweringError::InvalidFormulation {
                message: format!("unsupported variable family domain `{family}`"),
                path: entrypoint.to_path_buf(),
            })
        }
    }
}

fn variable_instance_from_signature(
    family: &str,
    signature: &FamilySignature,
    asset: Option<&AssetInputs>,
    time: Option<usize>,
    overrides: Option<&VariableDeclOverrides>,
    entrypoint: &Path,
) -> Result<VariableInstance, LoweringError> {
    let (lower, upper, kind) =
        variable_domain_policy(&signature.target, asset, overrides, entrypoint)?;
    let name = match (asset, time) {
        (Some(asset), Some(time)) => indexed_name(&signature.target, &asset.name, time),
        (Some(asset), None) => asset_indexed_name(&signature.target, &asset.name),
        (None, Some(time)) => time_name(&signature.target, time),
        (None, None) => signature.target.clone(),
    };
    Ok(VariableInstance {
        name,
        family: family.to_string(),
        lower,
        upper,
        kind,
    })
}

fn variable_domain_policy(
    target: &str,
    asset: Option<&AssetInputs>,
    overrides: Option<&VariableDeclOverrides>,
    path: &Path,
) -> Result<(f64, Option<f64>, VariableKind), LoweringError> {
    let (mut lower, mut upper, mut kind) = match target {
        "build" => (
            0.0,
            Some(asset_parameter(
                asset.ok_or_else(|| LoweringError::InvalidFormulation {
                    message: "`build[a]` requires an asset scope".to_string(),
                    path: path.to_path_buf(),
                })?,
                "max_build",
                path,
            )?),
            VariableKind::Continuous,
        ),
        "unserved_energy" => (0.0, None, VariableKind::Continuous),
        "charge" | "discharge" | "generation" => (0.0, None, VariableKind::Continuous),
        "dispatch" => (
            if asset.is_some_and(|asset| has_asset_parameter(asset, "energy_mwh")) {
                f64::NEG_INFINITY
            } else {
                0.0
            },
            None,
            VariableKind::Continuous,
        ),
        "commit" | "start" | "shutdown" => (0.0, Some(1.0), VariableKind::Binary),
        _ => (f64::NEG_INFINITY, None, VariableKind::Continuous),
    };

    if let Some(overrides) = overrides {
        if let Some(decl_kind) = &overrides.kind {
            kind = match decl_kind {
                VariableKindDecl::Continuous => VariableKind::Continuous,
                VariableKindDecl::Integer => VariableKind::Integer,
                VariableKindDecl::Binary => VariableKind::Binary,
            };
        }
        if let Some(bound) = &overrides.lower {
            if let Some(value) = literal_bound_to_f64(bound, path)? {
                lower = value;
            }
        }
        if let Some(bound) = &overrides.upper {
            if let Some(value) = literal_bound_to_f64(bound, path)? {
                upper = Some(value);
            }
        }
    }

    Ok((lower, upper, kind))
}

fn variable_instance_is_active(target: &str, asset: Option<&AssetInputs>) -> bool {
    match target {
        "build" => asset.is_some_and(|asset| asset.candidate),
        "unserved_energy" => true,
        _ => asset.is_some_and(|asset| asset.families.contains(target)),
    }
}

fn objective_sense(value: &str) -> ObjectiveSense {
    match value {
        "maximize" => ObjectiveSense::Maximize,
        _ => ObjectiveSense::Minimize,
    }
}

fn term(variable_name: &str, coefficient: f64) -> LinearTerm {
    LinearTerm {
        variable_name: variable_name.to_string(),
        coefficient,
    }
}

fn indexed_name(family: &str, asset_name: &str, time: usize) -> String {
    format!("{family}[{asset_name},{time}]")
}

fn time_name(family: &str, time: usize) -> String {
    format!("{family}[{time}]")
}

fn asset_indexed_name(target: &str, asset: &str) -> String {
    format!("{target}[{asset}]")
}
