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
model.scenario("baseline", log_to_console=False)
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
