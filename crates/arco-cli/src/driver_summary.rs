use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub struct RunSummary {
    pub entrypoint: String,
    pub backend: &'static str,
    pub solve_status: &'static str,
    pub active_scenario: String,
    pub objective: ObjectiveSummary,
    pub reports: Vec<ReportSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dual_reports: Vec<DualReportSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<VariableSummary>,
    pub counts: ProblemCounts,
    pub timing: TimingSummary,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ObjectiveSummary {
    pub name: String,
    pub sense: String,
    pub value: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ReportSummary {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub index: Vec<String>,
    pub values: Vec<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct VariableSummary {
    pub name: String,
    pub representative_value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<VariableValueSummary>>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct VariableValueSummary {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DualReportSummary {
    pub name: String,
    pub values: Vec<DualReportValueSummary>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DualReportValueSummary {
    pub instance: String,
    pub value: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ProblemCounts {
    pub parameters: usize,
    pub variables: usize,
    pub constraints: usize,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TimingSummary {
    pub parse_ms: f64,
    pub validate_ms: f64,
    pub compile_ms: f64,
    pub solve_ms: f64,
    pub total_ms: f64,
    pub peak_memory_bytes: Option<u64>,
}

pub(crate) fn summarize_variables(
    variables: &[arco_ops::execution::MappedVariableResult],
    options: &crate::driver::RunOptions,
) -> Vec<VariableSummary> {
    if options.filter_variable.is_none() {
        return Vec::new();
    }
    variables
        .iter()
        .filter(|variable| {
            options
                .filter_variable
                .as_deref()
                .is_none_or(|pattern| wildcard_match(pattern, &variable.dsl_name))
        })
        .filter_map(|variable| {
            let filtered_values = variable
                .values
                .iter()
                .filter(|value| {
                    options.filter_asset.as_deref().is_none_or(|pattern| {
                        extract_asset_name(&value.compiled_name)
                            .is_some_and(|asset| wildcard_match(pattern, asset))
                    })
                })
                .map(|value| VariableValueSummary {
                    name: trim_family_prefix(&variable.dsl_name, &value.compiled_name),
                    value: value.value,
                })
                .collect::<Vec<_>>();

            if options.filter_asset.is_some() && filtered_values.is_empty() {
                return None;
            }

            let representative_value = filtered_values
                .first()
                .map_or(variable.representative_value, |value| value.value);
            let values = if options.compact || values_are_redundant(&filtered_values) {
                None
            } else {
                Some(filtered_values)
            };

            Some(VariableSummary {
                name: variable.dsl_name.clone(),
                representative_value,
                values,
            })
        })
        .collect()
}

fn values_are_redundant(values: &[VariableValueSummary]) -> bool {
    match values.first() {
        Some(first) => values
            .iter()
            .all(|value| (value.value - first.value).abs() < f64::EPSILON),
        None => true,
    }
}

pub(crate) fn trim_family_prefix(family_name: &str, value_name: &str) -> String {
    let prefix = family_name.split('[').next().unwrap_or(family_name);
    value_name
        .strip_prefix(prefix)
        .map_or_else(|| value_name.to_string(), ToString::to_string)
}

fn extract_asset_name(value_name: &str) -> Option<&str> {
    let start = value_name.find('[')? + 1;
    let remainder = &value_name[start..];
    let end = remainder.find([',', ']'])?;
    let asset = &remainder[..end];
    if asset.is_empty() || asset.chars().all(|character| character.is_ascii_digit()) {
        None
    } else {
        Some(asset)
    }
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut pattern_index = 0usize;
    let mut value_index = 0usize;
    let mut star_index = None;
    let mut match_index = 0usize;

    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star_index = Some(pattern_index);
            match_index = value_index;
            pattern_index += 1;
        } else if let Some(star) = star_index {
            pattern_index = star + 1;
            match_index += 1;
            value_index = match_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}
