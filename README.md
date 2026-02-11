> [!warning] Arco is built primarily for internal use within our organization.
> You are welcome to try it, but we make no guarantees about API stability or
> robustness at this stage. For battle-tested alternatives, consider
> [Pyomo](https://www.pyomo.org/) (Python) or [JuMP](https://jump.dev/) (Julia).

<div align="center">

<h1>ARCO</h1>

<p><strong>A memory-smart optimization library for solving problems on constrained hardware.</strong></p>

<p>
  <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/rust-1.85-brightgreen"></a>
  <a href="docs/diataxis.md"><img alt="Docs" src="https://img.shields.io/badge/docs-di%C3%A1taxis-green"></a>
  <a href="https://github.com/astral-sh/ruff"><img alt="Ruff" src="https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json"></a>
  <a href="https://github.com/astral-sh/ty"><img alt="ty" src="https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ty/main/assets/badge/v0.json"></a>
</p>

</div>

Arco is an experimental framework that serves a narrow set of linear and
mixed-integer optimization. The primary user-facing API is the Python binding
module `arco`, backed by Rust crates for model construction, solver integration,
and diagnostics.
The name stands for **Assembled Resource-Constrained Optimization** to reflect
both the memory-focused design and the API ideas assembled from multiple
optimization ecosystems.

## Philosophy

Arco is built for harder optimization problems on constrained resources. We are
intentional about every allocation, careful with stack and heap behavior, and
relentless about minimizing memory usage so more systems can run real workloads.
The goal is not to trade speed for frugality, it is to keep performance at the
max while staying memory disciplined.

## At a Glance

<table>
<tr>
<td valign="top" width="33%">

<strong>What Arco Optimizes For</strong>

<ul>
  <li>LP/MIP workloads where memory pressure is a first-class constraint</li>
  <li>Predictable memory behavior across stack and heap allocations</li>
  <li>High throughput in model construction, orchestration, and solver handoff</li>
</ul>

</td>
<td valign="top" width="33%">

<strong>What Arco Is Not</strong>

<ul>
  <li>Not a general-purpose optimization framework</li>
  <li>Not a symbolic math system for arbitrary modeling workflows</li>
  <li>Not yet a production-hardened platform for critical infrastructure</li>
</ul>

</td>
<td valign="top" width="33%">

<strong>Current Maturity</strong>

<ul>
  <li>Experimental project with active iteration and evolving ergonomics</li>
  <li>Best fit today for evaluation, prototyping, and focused production pilots</li>
  <li>Readiness details are tracked in <a href="#features-and-status">Features and Status</a></li>
</ul>

</td>
</tr>
</table>

## Quickstart

Use `uv` to install and run Arco from Python.

```bash
uv add arco
uv run python -c "import arco; print(arco.__name__)"
```

Use `pip` if you prefer a standard Python virtual environment workflow.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install arco
python -c "import arco; print(arco.__name__)"
```

## API comparison

<a id="api-comparison"></a>

<table>
<tr>
<th>With ARCO</th>
<th>With Pyomo</th>
</tr>
<tr>
<td>

```python
import arco

model = arco.Model()

x = model.add_variable(lb=0, name="x")
y = model.add_variable(lb=0, name="y")

model.add_constraint(x + y >= 5.0)
model.minimize(3.0 * x + 2.0 * y)

result = model.solve()
# x=0.0, y=5.0, objective=10.0
```

</td>
<td>

```python
import pyomo.environ as pyo

model = pyo.ConcreteModel()

model.x = pyo.Var(within=pyo.NonNegativeReals)
model.y = pyo.Var(within=pyo.NonNegativeReals)

model.demand = pyo.Constraint(
    expr=model.x + model.y >= 5.0
)
model.cost = pyo.Objective(
    expr=3.0 * model.x + 2.0 * model.y,
    sense=pyo.minimize
)

solver = pyo.SolverFactory('highs')
result = solver.solve(model)
```

</td>
</tr>
</table>

## Features and Roadmap

- **High Performance**: Rust core with zero-copy data structures for maximum
  speed and minimal memory overhead.
- **Python First**: Native Python bindings via PyO3 with intuitive operator
  overloading for model construction.
- **Solver Included**: HiGHS solver embedded out of the box. No external solver
  installation or configuration required.
- **Multi-Solver Support**: Pluggable solver backends including HiGHS (open
  source) and FICO Xpress (commercial).
- **Block Composition**: DAG-based orchestration for multi-stage optimization
  problems with automatic dependency resolution.
- **Memory Instrumentation**: Built-in diagnostics for tracking memory usage and
  identifying bottlenecks.

| Feature                  | Status       | Feature                   | Status           |
| :----------------------- | :----------- | :------------------------ | :--------------- |
| **Model Construction**   | ✅ Available | **Block Orchestration**   | ✅ Available     |
| **LP / MIP Solving**     | ✅ Available | **DAG Execution**         | ✅ Available     |
| **HiGHS Backend**        | ✅ Available | **Warm Starting**         | ✅ Available     |
| **Xpress Backend**       | ✅ Available | **Memory Diagnostics**    | ✅ Available     |
| **Sparse Matrix Export** | ✅ Available | **Schema Validation**     | ✅ Available     |
| **Slack Variables**      | ✅ Available | **Parallel Block Solve**  | 🚧 Under Testing |
| **Dual / Reduced Costs** | ✅ Available | **Distributed Execution** | 📋 Planned       |

## Benchmarking

Use the `arco-bench` CLI to run model benchmarks, inspect artifacts, and compare
runs.

```bash
# Run default model-build scenarios
cargo run -p arco-bench -- run

# Run FAC-25 and model-build with custom cases and repetitions
cargo run -p arco-bench -- run \
  --scenario model-build,fac25 \
  --cases 1000,10000 \
  --repetitions 3

# Summarize an artifact
cargo run -p arco-bench -- report \
  --input artifacts/bench/bench_<timestamp>.jsonl

# Compare candidate against baseline and fail on regressions
cargo run -p arco-bench -- compare \
  --baseline artifacts/bench/baseline.jsonl \
  --candidate artifacts/bench/candidate.jsonl \
  --duration-threshold-pct 5 \
  --memory-threshold-pct 5

# Gate regressions across both total and variables stages
just bench-gate artifacts/bench/baseline.jsonl artifacts/bench/candidate.jsonl 5 5
```

By default, `run` writes newline-delimited JSON records to
`artifacts/bench/<run-id>.jsonl`. Use `--format table|json|ndjson` on `run`,
`report`, and `compare` for terminal output control.

## Contributing

Contributions are welcome. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the
development workflow, testing expectations, and documentation requirements.
Release/versioning behavior is defined in [`RELEASE_POLICY.md`](RELEASE_POLICY.md).
