use crate::source::{
    ConstraintDecl, ControlDecl, DataBindingDecl, HorizonDecl, IndexDecl, ObjectiveDecl, ParamDecl,
    ScenarioDecl, SetDecl, SourceProgram,
};
use miette::Diagnostic;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalProgram {
    pub models: Vec<CanonicalModel>,
    pub scenarios: Vec<CanonicalScenario>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalModel {
    pub name: String,
    pub sets: Vec<SetDecl>,
    pub parameters: Vec<ParamDecl>,
    pub controls: Vec<ControlDecl>,
    pub constraints: Vec<ConstraintDecl>,
    pub optimize: ObjectiveDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalScenario {
    pub name: String,
    pub horizon: HorizonDecl,
    pub data: Vec<DataBindingDecl>,
    pub model: String,
}

#[derive(Debug, Error)]
pub enum NormalizeError {
    #[error("missing declaration `{kind}` named `{name}` in {path}")]
    MissingDeclaration {
        kind: &'static str,
        name: String,
        path: PathBuf,
    },
    #[error("scenario `{scenario}` does not bind a model or template in {path}")]
    MissingScenarioModel { scenario: String, path: PathBuf },
}

impl Diagnostic for NormalizeError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        let code = match self {
            Self::MissingDeclaration { .. } => "arco::normalize::missing_declaration",
            Self::MissingScenarioModel { .. } => "arco::normalize::missing_scenario_model",
        };
        Some(Box::new(code))
    }
}

pub fn normalize_program(
    program: &SourceProgram,
    path: &Path,
) -> Result<CanonicalProgram, NormalizeError> {
    let mut models = program
        .models
        .iter()
        .map(normalize_model_decl)
        .collect::<Vec<_>>();

    for scenario in &program.scenarios {
        if scenario.model_use.is_none() && has_direct_wiring(scenario) {
            models.push(normalize_direct_wiring_to_model(program, scenario, path)?);
        }
    }

    models.sort_by_key(|model| model.name.clone());

    let mut scenarios = program
        .scenarios
        .iter()
        .map(|scenario| normalize_scenario_decl(scenario, path))
        .collect::<Result<Vec<_>, _>>()?;
    scenarios.sort_by_key(|scenario| scenario.name.clone());

    Ok(CanonicalProgram { models, scenarios })
}

fn normalize_model_decl(model: &crate::source::ModelDecl) -> CanonicalModel {
    CanonicalModel {
        name: model.name.clone(),
        sets: model.sets.clone(),
        parameters: model.parameters.clone(),
        controls: model.controls.clone(),
        constraints: model.constraints.clone(),
        optimize: model.optimize.clone(),
    }
}

fn normalize_direct_wiring_to_model(
    program: &SourceProgram,
    scenario: &ScenarioDecl,
    path: &Path,
) -> Result<CanonicalModel, NormalizeError> {
    let mut controls = Vec::new();
    let mut constraints = Vec::new();

    // Auto-wire: collect technology and operation names from assets and
    // instances when the scenario doesn't list them explicitly.
    let tech_names = infer_technology_names(program, scenario);
    let op_names = infer_operation_names(program, scenario);
    let rule_names = infer_rule_names(program, scenario);

    for tech_name in &tech_names {
        let technology =
            program
                .technology(tech_name)
                .ok_or_else(|| NormalizeError::MissingDeclaration {
                    kind: "technology",
                    name: tech_name.clone(),
                    path: path.to_path_buf(),
                })?;
        for invest in &technology.investments {
            controls.push(ControlDecl {
                name: invest.name.clone(),
                indices: vec![IndexDecl {
                    name: "a".to_string(),
                    domain: None,
                }],
                lower: invest.lower.clone(),
                upper: invest.upper.clone(),
                kind: invest.kind,
            });
        }
        for control in &technology.controls {
            controls.push(ControlDecl {
                name: control.name.clone(),
                indices: vec![
                    IndexDecl {
                        name: "a".to_string(),
                        domain: None,
                    },
                    IndexDecl {
                        name: "t".to_string(),
                        domain: None,
                    },
                ],
                lower: control.lower.clone(),
                upper: control.upper.clone(),
                kind: control.kind,
            });
        }
    }

    for op_name in &op_names {
        let operation =
            program
                .operation(op_name)
                .ok_or_else(|| NormalizeError::MissingDeclaration {
                    kind: "operation",
                    name: op_name.clone(),
                    path: path.to_path_buf(),
                })?;
        constraints.extend(operation.constraints.iter().cloned());
    }

    for rule_name in &rule_names {
        let rule = program
            .rule(rule_name)
            .ok_or_else(|| NormalizeError::MissingDeclaration {
                kind: "rule",
                name: rule_name.clone(),
                path: path.to_path_buf(),
            })?;
        constraints.extend(rule.constraints.iter().cloned());
    }

    let objective_name =
        scenario
            .objective
            .clone()
            .ok_or_else(|| NormalizeError::MissingDeclaration {
                kind: "objective",
                name: direct_wiring_model_name(&scenario.name),
                path: path.to_path_buf(),
            })?;
    let objective = program
        .objective(&objective_name)
        .ok_or_else(|| NormalizeError::MissingDeclaration {
            kind: "objective",
            name: objective_name,
            path: path.to_path_buf(),
        })?
        .clone();

    controls.sort_by_key(render_control_signature);
    constraints.sort_by_key(|constraint| constraint.name.clone());

    Ok(CanonicalModel {
        name: direct_wiring_model_name(&scenario.name),
        sets: Vec::new(),
        parameters: Vec::new(),
        controls,
        constraints,
        optimize: objective,
    })
}

fn normalize_scenario_decl(
    scenario: &ScenarioDecl,
    path: &Path,
) -> Result<CanonicalScenario, NormalizeError> {
    let model = scenario
        .model_use
        .clone()
        .or_else(|| {
            if has_direct_wiring(scenario) {
                Some(direct_wiring_model_name(&scenario.name))
            } else {
                None
            }
        })
        .ok_or_else(|| NormalizeError::MissingScenarioModel {
            scenario: scenario.name.clone(),
            path: path.to_path_buf(),
        })?;

    Ok(CanonicalScenario {
        name: scenario.name.clone(),
        horizon: scenario.horizon.clone(),
        data: scenario.data.clone(),
        model,
    })
}

fn has_direct_wiring(scenario: &ScenarioDecl) -> bool {
    !scenario.technologies.is_empty()
        || !scenario.operations.is_empty()
        || !scenario.rules.is_empty()
        || !scenario.instances.is_empty()
        || !scenario.assets.is_empty()
}

fn direct_wiring_model_name(scenario_name: &str) -> String {
    format!("{scenario_name}Model")
}

fn render_control_signature(control: &ControlDecl) -> String {
    if control.indices.is_empty() {
        return control.name.clone();
    }
    let names = control
        .indices
        .iter()
        .map(|idx| idx.name.as_str())
        .collect::<Vec<_>>()
        .join(",");
    format!("{}[{}]", control.name, names)
}

pub fn normalize_surface_syntax(text: &str) -> String {
    let math_keywords = [
        "constraint",
        "expression",
        "minimize",
        "maximize",
        "expr",
        "lower",
        "upper",
    ];
    let mut normalized = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let rewritten = math_keywords
            .iter()
            .find_map(|keyword| rewrite_math_block(text, index, keyword));
        if let Some((replacement, end)) = rewritten {
            normalized.push_str(&replacement);
            index = end;
            continue;
        }

        normalized.push(bytes[index] as char);
        index += 1;
    }

    normalized
}

fn rewrite_math_block(text: &str, start: usize, keyword: &str) -> Option<(String, usize)> {
    if !matches_keyword_at(text, start, keyword) {
        return None;
    }

    let bytes = text.as_bytes();
    let mut index = start + keyword.len();
    let mut in_string = false;
    let mut escaped = false;
    let mut opening_brace = None;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => {
                opening_brace = Some(index);
                break;
            }
            b'\n' => return None,
            _ => {}
        }
        index += 1;
    }

    let opening_brace = opening_brace?;

    // For constraint blocks, peek at the body content. If it starts with
    // over/when/expr, this is a generation-style constraint with proper KDL
    // children, not bare math. Skip the rewrite so only inner `expr` blocks
    // get their math rewritten.
    if keyword == "constraint" && body_starts_with_generation_keyword(text, opening_brace) {
        return None;
    }

    let mut closing_index = opening_brace + 1;
    let mut brace_depth = 1usize;
    in_string = false;
    escaped = false;

    while closing_index < bytes.len() {
        let byte = bytes[closing_index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            closing_index += 1;
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => brace_depth += 1,
            b'}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        closing_index += 1;
    }

    if brace_depth != 0 {
        return None;
    }

    let header = text[start..opening_brace].trim_end();
    let body = normalize_math_body(&text[opening_brace + 1..closing_index]);
    let encoded_body = encode_kdl_string(&body);

    let replacement = match keyword {
        "constraint" => format!("{header} expression={encoded_body}"),
        "expression" => format!("{header} {{ formula {encoded_body} }}"),
        "minimize" | "maximize" => format!("{header} expression={encoded_body}"),
        "expr" => format!("{header} expression={encoded_body}"),
        "lower" | "upper" => format!("{header} expression={encoded_body}"),
        _ => return None,
    };

    Some((replacement, closing_index + 1))
}

fn body_starts_with_generation_keyword(text: &str, opening_brace: usize) -> bool {
    let trimmed = text[opening_brace + 1..].trim_start();
    for keyword in ["over", "when", "expr"] {
        let Some(rest) = trimmed.strip_prefix(keyword) else {
            continue;
        };
        if rest.starts_with([' ', '\t', '{']) {
            return true;
        }
    }
    false
}

fn matches_keyword_at(text: &str, start: usize, keyword: &str) -> bool {
    let bytes = text.as_bytes();
    let end = start + keyword.len();

    if end > bytes.len() || &bytes[start..end] != keyword.as_bytes() {
        return false;
    }

    let previous_ok = start == 0 || is_keyword_boundary(bytes[start - 1] as char);
    let next_ok = end >= bytes.len() || is_keyword_boundary(bytes[end] as char);
    previous_ok && next_ok
}

fn is_keyword_boundary(character: char) -> bool {
    character.is_ascii_whitespace() || matches!(character, '{' | '}')
}

fn normalize_math_body(body: &str) -> String {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn encode_kdl_string(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() + 2);
    encoded.push('"');
    for character in value.chars() {
        match character {
            '\\' => encoded.push_str("\\\\"),
            '"' => encoded.push_str("\\\""),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            _ => encoded.push(character),
        }
    }
    encoded.push('"');
    encoded
}

/// When the scenario lists technologies explicitly, use those. Otherwise walk
/// the scenario's asset and instances declarations to discover which
/// technologies are in use.
fn infer_technology_names(program: &SourceProgram, scenario: &ScenarioDecl) -> Vec<String> {
    if !scenario.technologies.is_empty() {
        return scenario.technologies.clone();
    }
    let mut names = BTreeSet::new();
    for asset_name in &scenario.assets {
        if let Some(asset) = program.asset(asset_name) {
            names.insert(asset.technology.clone());
        }
    }
    for instances_name in &scenario.instances {
        if let Some(inst) = program.instances(instances_name) {
            names.insert(inst.technology.clone());
        }
    }
    names.into_iter().collect()
}

/// When the scenario lists operations explicitly, use those. Otherwise collect
/// operations from asset and instances declarations.
fn infer_operation_names(program: &SourceProgram, scenario: &ScenarioDecl) -> Vec<String> {
    if !scenario.operations.is_empty() {
        return scenario.operations.clone();
    }
    let mut names = BTreeSet::new();
    for asset_name in &scenario.assets {
        if let Some(op) = program.asset(asset_name).and_then(|a| a.operation.clone()) {
            names.insert(op);
        }
    }
    for instances_name in &scenario.instances {
        if let Some(op) = program
            .instances(instances_name)
            .and_then(|i| i.operation.clone())
        {
            names.insert(op);
        }
    }
    names.into_iter().collect()
}

/// When the scenario lists rules explicitly, use those. Otherwise include
/// every declared rule in the program.
fn infer_rule_names(program: &SourceProgram, scenario: &ScenarioDecl) -> Vec<String> {
    if !scenario.rules.is_empty() {
        return scenario.rules.clone();
    }
    program.rules.iter().map(|r| r.name.clone()).collect()
}
