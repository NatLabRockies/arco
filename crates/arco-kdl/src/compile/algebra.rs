fn compile_algebra(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    entrypoint: &Path,
) -> Result<AlgebraicProblem, CompileError> {
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
    let mut constraints = compile_constraint_instances(
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
            sense: program.active_objective.sense,
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

    fn as_scalar(&self, path: &Path, context: &str) -> Result<f64, CompileError> {
        if self.terms.is_empty() {
            Ok(self.constant)
        } else {
            Err(CompileError::InvalidFormulation {
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
) -> Result<Vec<VariableInstance>, CompileError> {
    let mut instances = Vec::new();
    for (family, signature) in variable_signatures {
        let overrides = program.variable_overrides.get(&signature.target);
        let resolved = resolve_variable_domains(
            signature, program, inputs, family, overrides, entrypoint,
        )?;
        instances.extend(resolved);
    }
    Ok(instances)
}

/// Expand variable instances by resolving each index domain via the set
/// registry and alias system. Produces the cartesian product of all
/// resolved domains, applying asset-based filtering where applicable.
fn resolve_variable_domains(
    signature: &FamilySignature,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    family: &str,
    overrides: Option<&VariableDeclOverrides>,
    entrypoint: &Path,
) -> Result<Vec<VariableInstance>, CompileError> {
    let asset_names: BTreeSet<&str> = inputs.assets.iter().map(|a| a.name.as_str()).collect();
    let asset_lookup: BTreeMap<&str, &AssetInputs> = inputs
        .assets
        .iter()
        .map(|a| (a.name.as_str(), a))
        .collect();

    // Resolve each index to its domain values and track which are asset domains.
    let mut domain_values: Vec<Vec<String>> = Vec::new();
    let mut asset_index: Option<usize> = None;
    for (i, index_name) in signature.indices.iter().enumerate() {
        let values = resolve_single_index_domain(
            index_name, signature, program, family, entrypoint,
        )?;
        if is_asset_domain(index_name, signature, program, &asset_names) {
            asset_index = Some(i);
        }
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
        let asset = asset_index.and_then(|idx| asset_lookup.get(combo[idx].as_str()).copied());

        if !variable_instance_is_active(&signature.target, asset) {
            continue;
        }

        let name = format!("{}[{}]", signature.target, combo.join(","));
        let (lower, upper, kind) =
            variable_domain_policy(&signature.target, asset, overrides, entrypoint)?;
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

/// Determine whether an index resolves to the assets domain by checking
/// if the resolved set values match the known asset names.
fn is_asset_domain(
    index_name: &str,
    signature: &FamilySignature,
    program: &SemanticProgram,
    asset_names: &BTreeSet<&str>,
) -> bool {
    if asset_names.is_empty() {
        return false;
    }
    let effective_name = signature
        .index_domains
        .get(index_name)
        .map(|s| s.as_str())
        .unwrap_or(index_name);
    resolve_set_by_name(effective_name, program).is_some_and(|vals| {
        let set: BTreeSet<&str> = vals.iter().map(|v| v.as_str()).collect();
        set == *asset_names
    })
}

/// Resolve values for a single index. Checks the signature's explicit domain
/// binding first, then looks up the set registry and alias system.
fn resolve_single_index_domain(
    index_name: &str,
    signature: &FamilySignature,
    program: &SemanticProgram,
    family: &str,
    entrypoint: &Path,
) -> Result<Vec<String>, CompileError> {
    let effective_name = signature
        .index_domains
        .get(index_name)
        .map(|s| s.as_str())
        .unwrap_or(index_name);

    resolve_set_by_name(effective_name, program).ok_or_else(|| CompileError::InvalidFormulation {
        message: format!(
            "index `{index_name}` in `{family}` references unknown set `{effective_name}`"
        ),
        path: entrypoint.to_path_buf(),
    })
}

/// Resolve a set name to its values, checking the registry and alias system.
fn resolve_set_by_name(name: &str, program: &SemanticProgram) -> Option<Vec<String>> {
    // Direct registry lookup.
    if let Some(set) = program.set_registry.get(name) {
        if !set.values.is_empty() {
            return Some(set.values.clone());
        }
    }
    // Alias lookup: name -> canonical -> registry.
    if let Some(canonical) = program.set_aliases.get(name) {
        if let Some(set) = program.set_registry.get(canonical.as_str()) {
            if !set.values.is_empty() {
                return Some(set.values.clone());
            }
        }
    }
    // Reverse alias: check if name is a canonical form whose alias has registry entries.
    for (alias, canonical) in &program.set_aliases {
        if canonical == name {
            if let Some(set) = program.set_registry.get(alias.as_str()) {
                if !set.values.is_empty() {
                    return Some(set.values.clone());
                }
            }
        }
    }
    None
}

fn variable_domain_policy(
    target: &str,
    asset: Option<&AssetInputs>,
    overrides: Option<&VariableDeclOverrides>,
    path: &Path,
) -> Result<(f64, Option<f64>, VariableKind), CompileError> {
    let (mut lower, mut upper, mut kind) = match target {
        "build" => (
            0.0,
            Some(asset_parameter(
                asset.ok_or_else(|| CompileError::InvalidFormulation {
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
    match (target, asset) {
        ("build", Some(asset)) => asset.candidate,
        ("build", None) => true,
        (_, None) => true,
        (_, Some(asset)) => {
            // If the asset has family information, filter by it;
            // otherwise assume the variable is active for all assets.
            asset.families.is_empty() || asset.families.contains(target)
        }
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
