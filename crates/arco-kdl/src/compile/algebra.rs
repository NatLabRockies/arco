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
    let reverse_aliases = build_reverse_alias_lookup(program);
    let asset_names: BTreeSet<&str> = inputs.assets.iter().map(|a| a.name.as_str()).collect();
    let asset_lookup: BTreeMap<&str, &AssetInputs> = inputs
        .assets
        .iter()
        .map(|a| (a.name.as_str(), a))
        .collect();

    let asset_index = signature.indices.iter().position(|index_name| {
        is_asset_domain(
            index_name,
            signature,
            program,
            &reverse_aliases,
            &asset_names,
        )
    });

    let mut instances = Vec::new();
    if let Some(tuple_rows) = resolve_tuple_domain_rows(
        signature,
        program,
        &reverse_aliases,
        family,
        entrypoint,
    )? {
        for combo in tuple_rows {
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

        return Ok(instances);
    }

    let mut domain_values: Vec<Vec<String>> = Vec::new();
    for index_name in &signature.indices {
        let values = resolve_single_index_domain(
            index_name,
            signature,
            program,
            &reverse_aliases,
            family,
            entrypoint,
        )?;
        domain_values.push(values);
    }

    let mut current = Vec::with_capacity(domain_values.len());
    let mut expansion = VariableExpansionContext {
        signature,
        family,
        overrides,
        entrypoint,
        asset_lookup: &asset_lookup,
        asset_index,
        instances: &mut instances,
    };
    expansion.expand(&domain_values, 0, &mut current)?;

    Ok(instances)
}

struct VariableExpansionContext<'a> {
    signature: &'a FamilySignature,
    family: &'a str,
    overrides: Option<&'a VariableDeclOverrides>,
    entrypoint: &'a Path,
    asset_lookup: &'a BTreeMap<&'a str, &'a AssetInputs>,
    asset_index: Option<usize>,
    instances: &'a mut Vec<VariableInstance>,
}

impl VariableExpansionContext<'_> {
    fn expand(
        &mut self,
        domain_values: &[Vec<String>],
        depth: usize,
        current: &mut Vec<String>,
    ) -> Result<(), CompileError> {
        if depth == domain_values.len() {
            let asset = self
                .asset_index
                .and_then(|idx| self.asset_lookup.get(current[idx].as_str()).copied());

            if variable_instance_is_active(&self.signature.target, asset) {
                let name = format!("{}[{}]", self.signature.target, current.join(","));
                let (lower, upper, kind) =
                    variable_domain_policy(&self.signature.target, asset, self.overrides, self.entrypoint)?;
                self.instances.push(VariableInstance {
                    name,
                    family: self.family.to_string(),
                    lower,
                    upper,
                    kind,
                });
            }

            return Ok(());
        }

        for value in &domain_values[depth] {
            current.push(value.clone());
            self.expand(domain_values, depth + 1, current)?;
            current.pop();
        }

        Ok(())
    }
}

fn resolve_tuple_domain_rows<'a>(
    signature: &FamilySignature,
    program: &'a SemanticProgram,
    reverse_aliases: &BTreeMap<&str, &str>,
    family: &str,
    entrypoint: &Path,
) -> Result<Option<&'a [Vec<String>]>, CompileError> {
    if signature.indices.len() < 2 {
        return Ok(None);
    }

    let first_index = &signature.indices[0];
    let first_domain_name = signature
        .index_domains
        .get(first_index)
        .map_or(first_index.as_str(), |domain| domain.as_str());
    let Some(first_key) = resolve_set_registry_key(first_domain_name, program, reverse_aliases)
    else {
        return Ok(None);
    };

    for index_name in signature.indices.iter().skip(1) {
        let domain_name = signature
            .index_domains
            .get(index_name)
            .map_or(index_name.as_str(), |domain| domain.as_str());
        let Some(key) = resolve_set_registry_key(domain_name, program, reverse_aliases) else {
            return Ok(None);
        };
        if key != first_key {
            return Ok(None);
        }
    }

    let Some(set) = resolve_set_struct_by_name(first_key, program, reverse_aliases) else {
        return Ok(None);
    };
    let (Some(tuple_components), Some(tuple_rows)) =
        (set.tuple_components.as_ref(), set.tuple_rows.as_ref())
    else {
        return Ok(None);
    };

    if tuple_components != &signature.indices {
        return Err(CompileError::InvalidFormulation {
            message: format!(
                "index order mismatch for `{family}`: expected `{}`, received `{}`",
                tuple_components.join(","),
                signature.indices.join(",")
            ),
            path: entrypoint.to_path_buf(),
        });
    }

    Ok(Some(tuple_rows.as_slice()))
}

/// Determine whether an index resolves to the assets domain by checking
/// if the resolved set values match the known asset names.
fn is_asset_domain(
    index_name: &str,
    signature: &FamilySignature,
    program: &SemanticProgram,
    reverse_aliases: &BTreeMap<&str, &str>,
    asset_names: &BTreeSet<&str>,
) -> bool {
    if asset_names.is_empty() {
        return false;
    }

    let effective_name = signature
        .index_domains
        .get(index_name)
        .map_or(index_name, |s| s.as_str());
    let Some(set) = resolve_set_struct_by_name(effective_name, program, reverse_aliases) else {
        return false;
    };
    if set.values.len() != asset_names.len() {
        return false;
    }

    set.values
        .iter()
        .all(|value| asset_names.contains(value.as_str()))
}

/// Resolve values for a single index. Checks the signature's explicit domain
/// binding first, then looks up the set registry and alias system.
fn resolve_single_index_domain(
    index_name: &str,
    signature: &FamilySignature,
    program: &SemanticProgram,
    reverse_aliases: &BTreeMap<&str, &str>,
    family: &str,
    entrypoint: &Path,
) -> Result<Vec<String>, CompileError> {
    let effective_name = signature
        .index_domains
        .get(index_name)
        .map_or(index_name, |s| s.as_str());

    resolve_set_by_name(effective_name, program, reverse_aliases).ok_or_else(|| {
        CompileError::InvalidFormulation {
            message: format!(
                "index `{index_name}` in `{family}` references unknown set `{effective_name}`"
            ),
            path: entrypoint.to_path_buf(),
        }
    })
}

fn build_reverse_alias_lookup(program: &SemanticProgram) -> BTreeMap<&str, &str> {
    let mut reverse_aliases = BTreeMap::new();
    for (alias, canonical) in &program.set_aliases {
        reverse_aliases
            .entry(canonical.as_str())
            .or_insert(alias.as_str());
    }
    reverse_aliases
}

fn resolve_set_registry_key<'a>(
    name: &str,
    program: &'a SemanticProgram,
    reverse_aliases: &BTreeMap<&str, &str>,
) -> Option<&'a str> {
    if let Some((key, _)) = program.set_registry.get_key_value(name) {
        return Some(key.as_str());
    }

    if let Some(canonical) = program.set_aliases.get(name)
        && let Some((key, _)) = program.set_registry.get_key_value(canonical.as_str())
    {
        return Some(key.as_str());
    }

    if let Some(alias) = reverse_aliases.get(name)
        && let Some((key, _)) = program.set_registry.get_key_value(*alias)
    {
        return Some(key.as_str());
    }

    None
}

fn resolve_set_struct_by_name<'a>(
    name: &str,
    program: &'a SemanticProgram,
    reverse_aliases: &BTreeMap<&str, &str>,
) -> Option<&'a crate::semantic::ResolvedSet> {
    let key = resolve_set_registry_key(name, program, reverse_aliases)?;
    program.set_registry.get(key)
}

/// Resolve a set name to its values, checking the registry and alias system.
fn resolve_set_by_name(
    name: &str,
    program: &SemanticProgram,
    reverse_aliases: &BTreeMap<&str, &str>,
) -> Option<Vec<String>> {
    let set = resolve_set_struct_by_name(name, program, reverse_aliases)?;
    if set.values.is_empty() {
        return None;
    }
    Some(set.values.clone())
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
