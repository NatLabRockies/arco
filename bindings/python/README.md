# Arco Python bindings

Build and install locally with uv:

```bash
cd bindings/python
uv sync --group dev
uv run --with maturin maturin develop
```

Run linting:

```bash
uv run ruff check .
uv run ty check .
```

Run Python example formulations from the repository root:

```bash
cd ../..
uv run examples/dense-lp/formulation.py --solve --json
uv run examples/sdom/formulation.py --solve --json
```

For interactive exploration of dense-lp (no extra script boilerplate):

```bash
cd ../..
uv run --with ipython --with-editable ./bindings/python ipython -i examples/dense-lp/formulation.py
```

Inside IPython, use `model` to inspect the formulation and call `solve()` when ready.

## Declarations In The Python API

The Python bindings expose declaration-style helpers for decision variables and
execution entrypoints:

- `model.control(...)`: declare a decision variable (scalar) or variable family
  (indexed).
- `model.scenario(...)`: declare a named execution configuration.
- `model.run_scenario(...)`: execute a previously declared scenario.
- `solver_params={...}`: pass backend-specific raw solver options.

Example:

```python
import numpy as np
import arco

N = 10
model = arco.Model()
rows = arco.IndexSet(name="row", size=N)
cols = arco.IndexSet(name="col", size=N)

# control declaration (decision-variable family)
q = model.control("queen", rows, cols, bounds=arco.Binary)

model.add_constraints(q.sum(over=cols) == 1.0, name="row")
model.add_constraints(q.sum(over=rows) == 1.0, name="col")
for k in range(-(N - 1), N):
    model.add_constraint(np.diag(q, k).sum() <= 1.0, name=f"diag_down_{k}")
    model.add_constraint(np.diag(np.fliplr(q), k).sum() <= 1.0, name=f"diag_up_{k}")

model.minimize(0.0)

# scenario declaration (execution entrypoint)
model.scenario(
    "baseline",
    log_to_console=False,
    solver_params={"random_seed": 7},
)
solution = model.run_scenario("baseline")
assert solution.is_optimal()
```

## Result Export To pandas/polars DataFrames

Solve results can be exported directly to DataFrames for analysis workflows.

- `solution.to_pandas(table="variables")`
- `solution.to_polars(table="variables")`

Supported `table` values are:

- `"variables"`: one row per variable with primal value and reduced cost.
- `"constraints"`: one row per constraint with dual value.
- `"summary"`: one-row solve summary (status, objective, solve time).

Example:

```python
solution = model.run_scenario("baseline")

variables_df = solution.to_pandas(table="variables")
constraints_df = solution.to_pandas(table="constraints")
summary_df = solution.to_pandas(table="summary")

variables_pl = solution.to_polars(table="variables")
```

## Progress Callbacks In Python

You can pass a callable to `solve(progress=...)` to receive progress events
from Python.

```python
import arco

events: list[dict[str, object]] = []

def on_progress(event: dict[str, object]) -> None:
    events.append(event)
    stage = event.get("stage")
    if stage == "done":
        print(
            "status=",
            event.get("status"),
            "objective=",
            event.get("objective_value"),
        )

solution = model.solve(log_to_console=False, progress=on_progress)
```

## Backend-Specific Solver Parameters

You can pass backend options directly with `solver_params` on either
`Solver(...)`, `solve(...)`, or scenario declarations.

```python
solver = arco.HiGHS(
    solver_params={
        "random_seed": 7,
        "simplex_strategy": 4,
        "mip_detect_symmetry": True,
    }
)

solution = model.solve(solver=solver)

# Or as per-call overrides
solution = model.solve(solver_params={"random_seed": 11})
```

`solver_params` values must be `bool`, `int`, `float`, or `str`.

### Backend Support

- `HiGHS`: generic `solver_params` pass-through is supported. Any valid HiGHS
  option name can be provided.
- `Ipopt`: generic `solver_params` pass-through is supported. Any valid IPOPT
  option name can be provided.
- `Xpress`: generic `solver_params` pass-through is not yet implemented.
  Use typed settings (`time_limit`, `mip_gap`, `presolve`, `threads`,
  `tolerance`, `verbosity`, `log_to_console`) for now.

### Common HiGHS Options

These are examples; you are not limited to these names:

```python
solution = model.solve(
        solver=arco.HiGHS(
                solver_params={
                        "random_seed": 7,
                        "simplex_strategy": 4,
                        "mip_detect_symmetry": True,
                        "presolve": "on",
                        "threads": 8,
                }
        )
)
```

### Common IPOPT Options

```python
solution = model.solve(
        solver=arco.Ipopt(
                solver_params={
                        "print_level": 5,
                        "max_iter": 2000,
                        "linear_solver": "mumps",
                        "warm_start_init_point": "yes",
                        "tol": 1e-8,
                }
        )
)
```

### How To Find All Option Names

- HiGHS options reference:
  https://ergo-code.github.io/HiGHS/dev/options/definitions
- IPOPT options reference:
  https://coin-or.github.io/Ipopt/OPTIONS.html

If an option name/value is invalid for the selected backend, the backend solve
will fail with a solver-specific error.

Emitted event payloads include:

- `{"stage": "start", "num_variables": ..., "num_constraints": ...}`
- `{"stage": "done", "status": ..., "objective_value": ..., "solve_time_seconds": ...}`
- `{"stage": "error", "error_type": ..., "error": ...}`

## Exception Handling

The Python bindings expose a structured exception hierarchy rooted at
`arco.ArcoError`, and Rust errors are mapped into these subclasses.

Core Rust-to-Python mappings include:

- `arco_core::model::ModelError::EmptyModel` -> `arco.ModelEmptyError`
- `arco_core::model::ModelError::InvalidVariableId` -> `arco.VariableInvalidIdError`
- `arco_core::model::ModelError::InvalidVariableBounds` -> `arco.VariableInvalidBoundsError`
- `arco_core::model::ModelError::InvalidConstraintId` -> `arco.ConstraintInvalidIdError`
- `arco_core::model::ModelError::InvalidConstraintBounds` -> `arco.ConstraintInvalidBoundsError`
- `arco_core::model::ModelError::NoObjective` -> `arco.ObjectiveMissingError`
- `arco_core::model::ModelError::MultipleObjectives` -> `arco.ObjectiveAlreadySetError`
- `arco_solver::SolverError::SolveFailure(Infeasible)` -> `arco.SolverInfeasibleError`
- `arco_solver::SolverError::SolveFailure(Unbounded)` -> `arco.SolverUnboundedError`
- `arco_solver::SolverError::SolveFailure(TimeLimit)` -> `arco.SolverTimeLimitError`

Wrapper-level helper APIs also raise standard Python exceptions when appropriate:

- `ValueError`: invalid declaration arguments (for example, empty control or
  scenario names).
- `KeyError`: unknown scenario name passed to `run_scenario(...)`.
- `ModuleNotFoundError`: optional dependency missing for DataFrame export
  (`pandas` or `polars`).

Recommended handling pattern:

```python
import arco

try:
    model.control("", rows, cols, bounds=arco.Binary)
except ValueError as exc:
    print(f"Invalid control declaration: {exc}")

model.scenario("baseline", log_to_console=False)
try:
    solution = model.run_scenario("baseline")
except KeyError as exc:
    print(f"Unknown scenario: {exc}")
    raise

try:
    # Catch all Rust-mapped errors first.
    solution = model.solve()
except arco.ArcoError as exc:
    print(f"Arco engine error: {type(exc).__name__}: {exc}")
    raise

try:
    variables_df = solution.to_pandas(table="variables")
except ModuleNotFoundError:
    print("Install pandas first: uv add pandas")

try:
    variables_pl = solution.to_polars(table="variables")
except ModuleNotFoundError:
    print("Install polars first: uv add polars")
```

Install optional dependencies as needed:

```bash
cd bindings/python
uv pip install pandas
uv pip install polars
```

or

```bash
cd bindings/python
uv add pandas
uv add polars
```
