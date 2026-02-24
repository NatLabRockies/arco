---
last_run: 2026-02-23
---

# Arco Build-Phase Comparison

## Build time

From `results/benchmark_points.csv`:

|   N | variables | arco (s) | linopy (s) | jump (s) | pyoptinterface (s) | pyomo (s) | pulp (s) |
| --: | --------: | -------: | ---------: | -------: | -----------------: | --------: | -------: |
| 200 |    80,000 |    4.003 |      4.003 |    4.003 |              4.003 |     6.005 |    6.006 |
| 400 |   320,000 |    4.003 |      4.003 |    4.004 |              6.004 |     8.007 |    8.006 |
| 600 |   720,000 |    4.003 |      4.002 |    4.002 |              8.006 |    12.009 |   12.010 |
| 800 | 1,280,000 |    4.002 |      4.002 |    6.004 |             10.009 |    16.015 |   18.016 |

## Peak memory

|   N | variables | arco (GB) | linopy (GB) | jump (GB) | pyoptinterface (GB) | pyomo (GB) | pulp (GB) |
| --: | --------: | --------: | ----------: | --------: | ------------------: | ---------: | --------: |
| 200 |    80,000 |     0.195 |       0.197 |     0.543 |               0.214 |      0.324 |     0.370 |
| 400 |   320,000 |     0.198 |       0.198 |     0.549 |               0.386 |      0.508 |     0.675 |
| 600 |   720,000 |     0.197 |       0.198 |     0.553 |               0.611 |      0.780 |     1.289 |
| 800 | 1,280,000 |     0.196 |       0.208 |     1.054 |               0.824 |      1.235 |     1.959 |

## Benchmark target

- Problem: same LP structure as `PyPSA/linopy` benchmark branch
- Phase: model build only (no solve)
- Tools compared: arco, linopy, jump, pyoptinterface, pyomo, pulp
- Range: `N = [10, 20, 50, 100, 200, 300, 400, 500, 600, 800]`
- Variable count: `2 * N^2`

## Running the benchmarks

Prerequisites: [uv](https://docs.astral.sh/uv/), [just](https://github.com/casey/just),
and optionally [Julia](https://julialang.org/) for JuMP benchmarks.

```bash
cd benchmarks/

# Install Torc (workflow orchestrator with resource monitoring)
just install-torc

# Install Python dependencies and Julia packages
just bootstrap

# Run the full suite end-to-end
just smoke

# Or step by step:
just start-torc
just run-benchmark
just export-results
just plot
just stop-torc
```

Recipe reference:

| Recipe | Description |
| --- | --- |
| `just install-torc` | Download Torc binary to `~/.local/bin/` |
| `just bootstrap` | `uv sync` + install Julia JuMP/HiGHS packages |
| `just start-torc` | Start the Torc server (job scheduling + resource monitoring) |
| `just run-benchmark` | Submit all jobs from `workflows/benchmark.yaml` |
| `just export-results` | Extract timing/memory data to `results/benchmark_points.csv` |
| `just export-results-id <id>` | Export results for a specific workflow ID |
| `just plot` | Generate plots in `plots/` |
| `just stop-torc` | Stop the Torc server |
| `just smoke` | Run the full pipeline (start, benchmark, export, plot) |
