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
