use arco_tools::StageMeasurement;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub(crate) const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub(crate) enum OutputFormat {
    Table,
    Json,
    Ndjson,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub(crate) enum Scenario {
    ModelBuild,
    Fac25,
    KdlCompile,
}

impl Scenario {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Scenario::ModelBuild => "model-build",
            Scenario::Fac25 => "fac25",
            Scenario::KdlCompile => "kdl-compile",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CaseConfig {
    pub(crate) name: String,
    pub(crate) variables: usize,
    pub(crate) constraints: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct CscMatrix {
    pub(crate) col_ptrs: Vec<u64>,
    pub(crate) row_indices: Vec<u32>,
    pub(crate) values: Vec<f64>,
}

#[derive(Debug, Clone)]
pub(crate) struct CaseExecution {
    pub(crate) variables: usize,
    pub(crate) constraints: usize,
    pub(crate) stage_measurements: Vec<StageMeasurement>,
    pub(crate) csc: Option<CscMatrix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BenchRecord {
    pub(crate) schema_version: u32,
    pub(crate) run_id: String,
    pub(crate) scenario: String,
    pub(crate) case_name: String,
    pub(crate) repetition: u32,
    pub(crate) variables: usize,
    pub(crate) constraints: usize,
    pub(crate) stage: String,
    pub(crate) duration_ms: f64,
    pub(crate) rss_before_bytes: Option<u64>,
    pub(crate) rss_after_bytes: Option<u64>,
    pub(crate) rss_delta_bytes: Option<i64>,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SummaryKey {
    pub(crate) scenario: String,
    pub(crate) case_name: String,
    pub(crate) stage: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SummaryRow {
    pub(crate) scenario: String,
    pub(crate) case_name: String,
    pub(crate) stage: String,
    pub(crate) samples: usize,
    pub(crate) mean_duration_ms: f64,
    pub(crate) max_duration_ms: f64,
    pub(crate) mean_rss_delta_bytes: Option<f64>,
    pub(crate) max_rss_after_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompareRow {
    pub(crate) scenario: String,
    pub(crate) case_name: String,
    pub(crate) stage: String,
    pub(crate) baseline_mean_duration_ms: f64,
    pub(crate) candidate_mean_duration_ms: f64,
    pub(crate) duration_change_pct: Option<f64>,
    pub(crate) baseline_mean_rss_delta_bytes: Option<f64>,
    pub(crate) candidate_mean_rss_delta_bytes: Option<f64>,
    pub(crate) rss_change_pct: Option<f64>,
}
