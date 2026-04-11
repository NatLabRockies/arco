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

## API comparison

All tools build the same LP: two N&times;N variable matrices, two constraint
families, and a linear objective. The snippets below show each tool's syntax.

<table>
<tr>
<th>Arco</th>
<th>Linopy</th>
<th>JuMP (Julia)</th>
</tr>
<tr>
<td>

```python
model = arco.Model()
i = IndexSet("i", members=range(n))
j = IndexSet("j", members=range(n))
bnd = Bounds(lower=-1e20, upper=1e20)

x = model.add_variables(i, j, bounds=bnd, name="x")
y = model.add_variables(i, j, bounds=bnd, name="y")

model.add_constraints(x - y >= i)
model.add_constraints(x + y >= 0.0)
model.minimize((2 * x + y).sum())
```

</td>
<td>

```python
model = linopy.Model()
idx = np.arange(n)

x = model.add_variables(
    coords=[idx, idx], name="x",
)
y = model.add_variables(
    coords=[idx, idx], name="y",
)

model.add_constraints(x - y >= idx)
model.add_constraints(x + y >= 0)
model.add_objective(
    (2 * x).sum() + y.sum(),
)
```

</td>
<td>

```julia
m = Model(HiGHS.Optimizer)

@variable(m, x[1:n, 1:n])
@variable(m, y[1:n, 1:n])

@constraint(m, [i in 1:n, j in 1:n],
    x[i, j] - y[i, j] >= i - 1)
@constraint(m, [i in 1:n, j in 1:n],
    x[i, j] + y[i, j] >= 0)
@objective(m, Min,
    sum(2x[i,j] + y[i,j]
        for i in 1:n, j in 1:n))
```

</td>
</tr>
</table>

<table>
<tr>
<th>Pyomo</th>
<th>PuLP</th>
<th>PyOptInterface</th>
</tr>
<tr>
<td>

```python
model = ConcreteModel()
model.i = Set(initialize=range(n))
model.j = Set(initialize=range(n))
model.x = Var(model.i, model.j)
model.y = Var(model.i, model.j)

def con1(m, i, j):
    return m.x[i,j] - m.y[i,j] >= i

def con2(m, i, j):
    return m.x[i,j] + m.y[i,j] >= 0

def obj(m):
    return sum(
        2*m.x[i,j] + m.y[i,j]
        for i in m.i for j in m.j
    )

model.con1 = Constraint(
    model.i, model.j, rule=con1,
)
model.con2 = Constraint(
    model.i, model.j, rule=con2,
)
model.obj = Objective(rule=obj)
```

</td>
<td>

```python
model = LpProblem("bench", LpMinimize)
x = {
    (i, j): LpVariable(f"x_{i}_{j}")
    for i in range(n)
    for j in range(n)
}
y = {
    (i, j): LpVariable(f"y_{i}_{j}")
    for i in range(n)
    for j in range(n)
}

for i in range(n):
    for j in range(n):
        model += x[i,j] - y[i,j] >= i
        model += x[i,j] + y[i,j] >= 0

model += lpSum(
    2*x[i,j] + y[i,j]
    for i in range(n)
    for j in range(n)
)
```

</td>
<td>

```python
model = highs.Model()
idx = range(n)
x = model.add_variables(
    idx, idx, name="x",
)
y = model.add_variables(
    idx, idx, name="y",
)

for i in idx:
    for j in idx:
        model.add_linear_constraint(
            x[i,j] - y[i,j], Geq, i)
        model.add_linear_constraint(
            x[i,j] + y[i,j], Geq, 0.0)

obj = quicksum(
    (x[i,j] for i in idx for j in idx),
    lambda v: 2 * v,
) + quicksum(y.values())
model.set_objective(
    obj, ObjectiveSense.Minimize,
)
```

</td>
</tr>
</table>

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

| Recipe                        | Description                                                  |
| ----------------------------- | ------------------------------------------------------------ |
| `just install-torc`           | Download Torc binary to `~/.local/bin/`                      |
| `just bootstrap`              | `uv sync` + install Julia JuMP/HiGHS packages                |
| `just start-torc`             | Start the Torc server (job scheduling + resource monitoring) |
| `just run-benchmark`          | Submit all jobs from `workflows/benchmark.yaml`              |
| `just export-results`         | Extract timing/memory data to `results/benchmark_points.csv` |
| `just export-results-id <id>` | Export results for a specific workflow ID                    |
| `just plot`                   | Generate plots in `plots/`                                   |
| `just stop-torc`              | Stop the Torc server                                         |
| `just smoke`                  | Run the full pipeline (start, benchmark, export, plot)       |
