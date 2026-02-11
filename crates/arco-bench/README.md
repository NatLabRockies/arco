# arco-bench


`arco-bench` provides a consistent interface for:

- running benchmark scenarios
- writing structured JSONL artifacts
- rendering summaries
- comparing baseline vs candidate runs with optional regression thresholds

## Quick Start

From repo root:

```bash
# Run default model-build scenarios
cargo run -p arco-bench -- run

# Render a saved artifact
cargo run -p arco-bench -- report --input artifacts/bench/<run-id>.jsonl

# Compare two artifacts
cargo run -p arco-bench -- compare \
  --baseline artifacts/bench/baseline.jsonl \
  --candidate artifacts/bench/candidate.jsonl
```

Equivalent `just` shortcuts:

```bash
just bench-run
just bench-report artifacts/bench/<run-id>.jsonl
just bench-compare artifacts/bench/baseline.jsonl artifacts/bench/candidate.jsonl
```

## Commands

### `run`

Execute one or more scenarios and emit benchmark records.

```bash
cargo run -p arco-bench -- run [OPTIONS]
```

Common options:

- `--scenario model-build,fac25`
- `--cases 1000,10000,100000`
- `--variables 67651 --constraints 676`
- `--constraint-ratio 0.01`
- `--repetitions 3`
- `--output artifacts/bench/my-run.jsonl`
- `--format table|json|ndjson`
- `--write-csc /tmp/csc`

Notes:

- default scenario is `model-build`
- default output path is `artifacts/bench/<run-id>.jsonl`

### `report`

Summarize an existing artifact.

```bash
cargo run -p arco-bench -- report --input artifacts/bench/<run-id>.jsonl --format table
```

### `compare`

Compare baseline and candidate artifacts (default stage filter: `total`).

```bash
cargo run -p arco-bench -- compare \
  --baseline artifacts/bench/baseline.jsonl \
  --candidate artifacts/bench/candidate.jsonl \
  --stage total \
  --duration-threshold-pct 5 \
  --memory-threshold-pct 5
```

If thresholds are provided and exceeded, `compare` exits non-zero.

## Artifact Format

`run` writes newline-delimited JSON records (`.jsonl`), one record per stage and repetition.

Each record includes:

- `schema_version`
- `run_id`
- `scenario`
- `case_name`
- `repetition`
- `variables`
- `constraints`
- `stage`
- `duration_ms`
- `rss_before_bytes`
- `rss_after_bytes`
- `rss_delta_bytes`

Example record:

```json
{"schema_version":1,"run_id":"bench_1770608719570","scenario":"model-build","case_name":"vars_10","repetition":1,"variables":10,"constraints":2,"stage":"total","duration_ms":14.928255,"rss_before_bytes":6320128,"rss_after_bytes":6918144,"rss_delta_bytes":598016}
```
