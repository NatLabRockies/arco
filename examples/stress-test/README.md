# Arco Stress Test Guide

This directory contains stress tests for benchmarking arco performance at scale.

## Quick Start

```bash
# Run stress test
python scripts/bench.py --arco-binary $(which arco) --workflows compile,solve --repetitions 3

# Run specific cases
python scripts/bench.py --cases generator-allocation,price-taker-battery --workflows solve
```

## Available Test Cases

### Tier 1: Smoke Tests (Fast)

- `generator-allocation` - Simple LP (~40 vars, ~24 constraints)
- `price-taker-battery` - LP with storage dynamics (~72 vars, ~120 constraints)

### Tier 2: Standard Tests

- `capacity-expansion` - MILP with investment decisions (~50 vars, ~40 constraints)

### Tier 3: Stress Tests

- `unit-commitment` - Complex MILP with binaries, ramping (~500+ vars, ~1000+ constraints)

## CI Integration

The benchmark workflow uses `github-action-benchmark` to track performance over time:

```yaml
- name: Run benchmarks
  run: python scripts/bench.py --output benchmark-results.json

- name: Store benchmark results
  uses: benchmark-action/github-action-benchmark@v1
  with:
    tool: customSmallerIsBetter
    output-file-path: benchmark-results.json
    alert-threshold: "115%"
```

## Regression Thresholds

- **Duration**: 115% alert, 150% fail
- **Peak RSS**: 120% alert, 200% fail
