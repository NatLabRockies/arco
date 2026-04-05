use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Variable};
use arco_expr::{ConstraintId, VariableId};
use arco_highs::AsyncCrsBuilder;
use arco_tools::memory::{capture_rss_bytes, rss_delta};
use std::collections::BTreeMap;
use std::env;
use std::time::Instant;

type SparseEntries = BTreeMap<ConstraintId, (Vec<usize>, Vec<f64>)>;
type DenseEntries = Vec<Option<(Vec<usize>, Vec<f64>)>>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mode {
    Baseline,
    Dense,
    Async,
}

impl Mode {
    fn parse(value: &str) -> Self {
        match value {
            "baseline" => Self::Baseline,
            "dense" => Self::Dense,
            "async" => Self::Async,
            _ => Self::Baseline,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Dense => "dense",
            Self::Async => "async",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Config {
    variables: usize,
    constraints: usize,
    nonzeros_per_column: usize,
    repetitions: usize,
    use_parallel: bool,
    chunk_count: usize,
    mode: Mode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            variables: 250_000,
            constraints: 100_000,
            nonzeros_per_column: 3,
            repetitions: 8,
            use_parallel: false,
            chunk_count: num_cpus::get(),
            mode: Mode::Baseline,
        }
    }
}

fn parse_config() -> Config {
    let mut cfg = Config::default();
    for arg in env::args().skip(1) {
        if let Some(v) = arg.strip_prefix("--variables=") {
            cfg.variables = v.parse().unwrap_or(cfg.variables);
        } else if let Some(v) = arg.strip_prefix("--constraints=") {
            cfg.constraints = v.parse().unwrap_or(cfg.constraints);
        } else if let Some(v) = arg.strip_prefix("--nnz-per-col=") {
            cfg.nonzeros_per_column = v.parse().unwrap_or(cfg.nonzeros_per_column);
        } else if let Some(v) = arg.strip_prefix("--repetitions=") {
            cfg.repetitions = v.parse().unwrap_or(cfg.repetitions);
        } else if let Some(v) = arg.strip_prefix("--chunk-count=") {
            cfg.chunk_count = v.parse().unwrap_or(cfg.chunk_count);
        } else if let Some(v) = arg.strip_prefix("--parallel=") {
            cfg.use_parallel = matches!(v, "1" | "true" | "yes" | "on");
        } else if let Some(v) = arg.strip_prefix("--mode=") {
            cfg.mode = Mode::parse(v);
        }
    }
    cfg.nonzeros_per_column = cfg.nonzeros_per_column.max(1);
    cfg.repetitions = cfg.repetitions.max(1);
    cfg.chunk_count = cfg.chunk_count.max(1);
    cfg
}

fn build_model(cfg: Config) -> (Model, BTreeMap<VariableId, usize>) {
    let mut model = Model::with_capacities(cfg.variables, cfg.constraints);
    let mut var_id_to_col = BTreeMap::new();

    for col in 0..cfg.variables {
        let var_id = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 1.0)))
            .unwrap_or_else(|err| panic!("failed to add variable: {err}"));
        var_id_to_col.insert(var_id, col);
    }

    for _ in 0..cfg.constraints {
        model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 1.0),
            })
            .unwrap_or_else(|err| panic!("failed to add constraint: {err}"));
    }

    for col in 0..cfg.variables {
        let var_id = VariableId::new(col as u32);
        for offset in 0..cfg.nonzeros_per_column {
            let row = (col + offset * 131) % cfg.constraints;
            model
                .set_coefficient(var_id, ConstraintId::new(row as u32), 1.0 + offset as f64)
                .unwrap_or_else(|err| panic!("failed to set coefficient: {err}"));
        }
    }

    (model, var_id_to_col)
}

fn build_rows_baseline(
    model: &Model,
    var_id_to_col: &BTreeMap<VariableId, usize>,
) -> SparseEntries {
    let mut constraint_entries: SparseEntries = BTreeMap::new();

    for (var_id, column) in model.columns() {
        let Ok(var) = model.get_variable(var_id) else {
            continue;
        };
        if !var.is_active {
            continue;
        }

        let Some(&col_idx) = var_id_to_col.get(&var_id) else {
            continue;
        };

        for (constraint_id, coeff) in column {
            let entry = constraint_entries
                .entry(*constraint_id)
                .or_insert_with(|| (Vec::new(), Vec::new()));
            entry.0.push(col_idx);
            entry.1.push(*coeff);
        }
    }

    constraint_entries
}

fn build_rows_dense(model: &Model, var_id_to_col: &BTreeMap<VariableId, usize>) -> DenseEntries {
    let mut row_entries: DenseEntries = vec![None; model.num_constraints()];

    for (var_id, column) in model.columns() {
        let Ok(var) = model.get_variable(var_id) else {
            continue;
        };
        if !var.is_active {
            continue;
        }

        let Some(&col_idx) = var_id_to_col.get(&var_id) else {
            continue;
        };

        for (constraint_id, coeff) in column {
            let row_idx = constraint_id.inner() as usize;
            let Some(slot) = row_entries.get_mut(row_idx) else {
                continue;
            };
            let entry = slot.get_or_insert_with(|| (Vec::new(), Vec::new()));
            entry.0.push(col_idx);
            entry.1.push(*coeff);
        }
    }

    row_entries
}

fn non_empty_dense(entries: &DenseEntries) -> usize {
    entries.iter().filter(|entry| entry.is_some()).count()
}

fn main() {
    let cfg = parse_config();

    println!(
        "profile_crs_build: mode={} variables={} constraints={} nnz_per_col={} repetitions={} parallel={} chunk_count={}",
        cfg.mode.as_str(),
        cfg.variables,
        cfg.constraints,
        cfg.nonzeros_per_column,
        cfg.repetitions,
        cfg.use_parallel,
        cfg.chunk_count
    );

    let model_build_start = Instant::now();
    let (model, var_id_to_col) = build_model(cfg);
    let model_build_ms = model_build_start.elapsed().as_secs_f64() * 1000.0;
    println!("model_build_ms={model_build_ms:.3}");

    let mut durations_ms = Vec::with_capacity(cfg.repetitions);
    let rss_before = capture_rss_bytes("profile_crs_build_before");

    match cfg.mode {
        Mode::Baseline => {
            let warmup = build_rows_baseline(&model, &var_id_to_col);
            println!("warmup_row_entries={}", warmup.len());
            for _ in 0..cfg.repetitions {
                let started = Instant::now();
                let entries = build_rows_baseline(&model, &var_id_to_col);
                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                assert!(!entries.is_empty(), "unexpected empty constraint entries");
                durations_ms.push(elapsed_ms);
            }
        }
        Mode::Dense => {
            let warmup = build_rows_dense(&model, &var_id_to_col);
            println!("warmup_row_entries={}", non_empty_dense(&warmup));
            for _ in 0..cfg.repetitions {
                let started = Instant::now();
                let entries = build_rows_dense(&model, &var_id_to_col);
                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                assert!(!entries.is_empty(), "unexpected empty dense entries");
                durations_ms.push(elapsed_ms);
            }
        }
        Mode::Async => {
            let builder = AsyncCrsBuilder::new()
                .with_parallel(cfg.use_parallel)
                .with_chunk_count(cfg.chunk_count);
            let warmup = builder.build_blocking(&model, &var_id_to_col);
            println!("warmup_row_entries={}", warmup.constraint_entries.len());
            for _ in 0..cfg.repetitions {
                let started = Instant::now();
                let entries = builder.build_blocking(&model, &var_id_to_col);
                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                assert!(
                    !entries.constraint_entries.is_empty(),
                    "unexpected empty async entries"
                );
                durations_ms.push(elapsed_ms);
            }
        }
    }

    let rss_after = capture_rss_bytes("profile_crs_build_after");

    let sum: f64 = durations_ms.iter().sum();
    let mean_ms = sum / durations_ms.len() as f64;
    let min_ms = durations_ms
        .iter()
        .copied()
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max_ms = durations_ms
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));

    println!("durations_ms={durations_ms:?}");
    println!("mean_ms={mean_ms:.3}");
    println!("min_ms={min_ms:.3}");
    println!("max_ms={max_ms:.3}");
    println!("rss_before_bytes={rss_before:?}");
    println!("rss_after_bytes={rss_after:?}");
    println!("rss_delta_bytes={:?}", rss_delta(rss_before, rss_after));
}
