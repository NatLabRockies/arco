use crate::types::{BenchRecord, CompareRow, SummaryKey, SummaryRow};
use std::collections::BTreeMap;

pub(crate) fn summarize_records(records: &[BenchRecord]) -> Vec<SummaryRow> {
    #[derive(Default)]
    struct Acc {
        samples: usize,
        duration_sum: f64,
        duration_max: f64,
        rss_delta_sum: f64,
        rss_delta_count: usize,
        rss_after_max: Option<u64>,
    }

    let mut groups: BTreeMap<SummaryKey, Acc> = BTreeMap::new();
    for record in records {
        let key = SummaryKey {
            scenario: record.scenario.clone(),
            case_name: record.case_name.clone(),
            stage: record.stage.clone(),
        };
        let entry = groups.entry(key).or_default();
        entry.samples += 1;
        entry.duration_sum += record.duration_ms;
        if record.duration_ms > entry.duration_max {
            entry.duration_max = record.duration_ms;
        }
        if let Some(delta) = record.rss_delta_bytes {
            entry.rss_delta_sum += delta as f64;
            entry.rss_delta_count += 1;
        }
        entry.rss_after_max = match (entry.rss_after_max, record.rss_after_bytes) {
            (Some(current), Some(next)) => Some(current.max(next)),
            (None, Some(next)) => Some(next),
            (current, None) => current,
        };
    }

    groups
        .into_iter()
        .map(|(key, acc)| SummaryRow {
            scenario: key.scenario,
            case_name: key.case_name,
            stage: key.stage,
            samples: acc.samples,
            mean_duration_ms: if acc.samples == 0 {
                0.0
            } else {
                acc.duration_sum / acc.samples as f64
            },
            max_duration_ms: acc.duration_max,
            mean_rss_delta_bytes: if acc.rss_delta_count == 0 {
                None
            } else {
                Some(acc.rss_delta_sum / acc.rss_delta_count as f64)
            },
            max_rss_after_bytes: acc.rss_after_max,
        })
        .collect()
}

pub(crate) fn build_comparison_rows(
    baseline_summary: &[SummaryRow],
    candidate_summary: &[SummaryRow],
    stage_filter: &str,
) -> Vec<CompareRow> {
    let mut baseline_map: BTreeMap<SummaryKey, &SummaryRow> = BTreeMap::new();
    for row in baseline_summary {
        if row.stage == stage_filter {
            let key = SummaryKey {
                scenario: row.scenario.clone(),
                case_name: row.case_name.clone(),
                stage: row.stage.clone(),
            };
            baseline_map.insert(key, row);
        }
    }

    let mut rows = Vec::new();
    for candidate in candidate_summary {
        if candidate.stage != stage_filter {
            continue;
        }
        let key = SummaryKey {
            scenario: candidate.scenario.clone(),
            case_name: candidate.case_name.clone(),
            stage: candidate.stage.clone(),
        };
        let Some(baseline) = baseline_map.get(&key) else {
            continue;
        };
        rows.push(CompareRow {
            scenario: key.scenario,
            case_name: key.case_name,
            stage: key.stage,
            baseline_mean_duration_ms: baseline.mean_duration_ms,
            candidate_mean_duration_ms: candidate.mean_duration_ms,
            duration_change_pct: percent_change(
                baseline.mean_duration_ms,
                candidate.mean_duration_ms,
            ),
            baseline_mean_rss_delta_bytes: baseline.mean_rss_delta_bytes,
            candidate_mean_rss_delta_bytes: candidate.mean_rss_delta_bytes,
            rss_change_pct: match (
                baseline.mean_rss_delta_bytes,
                candidate.mean_rss_delta_bytes,
            ) {
                (Some(base), Some(next)) => percent_change(base, next),
                _ => None,
            },
        });
    }

    rows
}

pub(crate) fn has_regressions(
    rows: &[CompareRow],
    duration_threshold_pct: Option<f64>,
    memory_threshold_pct: Option<f64>,
) -> bool {
    rows.iter().any(|row| {
        let duration_failed = duration_threshold_pct
            .is_some_and(|threshold| row.duration_change_pct.is_some_and(|pct| pct > threshold));
        let memory_failed = memory_threshold_pct
            .is_some_and(|threshold| row.rss_change_pct.is_some_and(|pct| pct > threshold));
        duration_failed || memory_failed
    })
}

fn percent_change(baseline: f64, candidate: f64) -> Option<f64> {
    if baseline.abs() <= f64::EPSILON {
        return None;
    }
    Some(((candidate - baseline) / baseline.abs()) * 100.0)
}
