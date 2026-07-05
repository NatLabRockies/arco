# Inspect a Model

Use iterators and snapshots to examine model structure before solving. This is
useful for debugging coefficient errors, verifying constraint counts, and
understanding what the solver will see.

## Listing variables and constraints

Use `model.list_variables()` and `model.list_constraints()` to iterate over
everything registered on the model. Each item carries its name and bounds.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="y")
>>> model.add_constraint(1.5 * x + 2.0 * y == 5.0, name="balance")
Constraint('balance', Bounds(5, 5))
>>> model.add_constraint(x + y >= 3.0, name="floor")
Constraint('floor', Bounds(3, inf))
>>> model.minimize(x + y)
>>> model.num_variables
2
>>> model.num_constraints
2
>>> [v.name for v in model.list_variables()]
['x', 'y']
>>> [c.name for c in model.list_constraints()]
['balance', 'floor']
```

You can also access the full object to inspect bounds and integrality.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="x")
>>> y = model.add_variable(bounds=arco.Binary, name="y")
>>> model.minimize(x + y)
>>> for v in model.list_variables():
...     print(v.name, v.bounds, v.is_integer)
x Bounds(lower=0, upper=10) False
y Bounds(lower=0, upper=1) True
```

## Pretty-printing the model

Use `print(model)` for a concise ASCII preview of the algebraic model. For a
full dump, call `model.pprint()`.

```python doctest
>>> import contextlib
>>> import io
>>> import arco
>>> model = arco.Model()
>>> t = arco.IndexSet(name="T", size=2)
>>> g = arco.IndexSet(name="G", members=["solar", "wind", "gas"])
>>> gen = model.add_variables(axes=(t, g), bounds=arco.Bounds(lower=0.0, upper=100.0), name="gen")
>>> capacity = {"solar": 50.0, "wind": 80.0, "gas": 100.0}
>>> caps = [capacity[name] for name in g.members] * t.size
>>> _ = model.add_constraints(gen <= caps)
>>> _ = model.add_constraints(gen.sum(over=g) >= [120.0, 90.0])
>>> preview = str(model)
>>> "s.t." in preview
True
>>> "Subject to" in preview
False
>>> "gen[0,solar]" in preview
True
>>> "gen[0,solar] + gen[0,wind] + gen[0,gas] >= 120" in preview
True
>>> "Bounds:" in preview
True
>>> "0 <= gen[t,g] <= 100  for t in T, g in G" in preview
True
>>> buffer = io.StringIO()
>>> with contextlib.redirect_stdout(buffer):
...     model.pprint()
>>> full = buffer.getvalue()
>>> "gen[1,solar] + gen[1,wind] + gen[1,gas] >= 90" in full
True
```

The output uses `s.t.` and ASCII operators (`<=`, `>=`, `=`), aligns relation
operators for readability, and groups variable domains (for example,
`Binary: ...`) near the bottom.

## Verifying array constraints

When you build constraints from variable arrays, `add_constraints` returns a
list of `Constraint` objects. Use the list length and the constraint bounds to
confirm the model matches your data.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> T = arco.IndexSet(name="T", size=2)
>>> G = arco.IndexSet(name="G", members=["solar", "wind", "gas"])
>>> capacity = {"solar": 50.0, "wind": 80.0, "gas": 100.0}
>>> demand = [120.0, 90.0]
>>>
>>> gen = model.add_variables(axes=(T, G), bounds=arco.Bounds(lower=0.0, upper=100.0))
>>> caps = [capacity[g] for g in G.members] * T.size
>>> cap_cons = model.add_constraints(gen <= caps)
>>> demand_cons = model.add_constraints(gen.sum(over=G) >= demand)
>>>
>>> len(cap_cons) == T.size * G.size
True
>>> len(demand_cons) == T.size
True
>>> model.num_constraints
8
>>> [c.bounds.lower for c in demand_cons]
[120.0, 90.0]
```

The demand constraints each carry a lower bound that matches the input data,
confirming the right-hand side was wired correctly. The capacity constraints
total 6 (2 periods times 3 generators), and the demand constraints total 2
(one per period), giving 8 constraints overall.

## Model snapshot

After building a model, call `inspect(include_coeffs=True)` to obtain a
snapshot object that describes every variable, constraint, and coefficient the
solver would receive. The snapshot is a plain data structure you can query
programmatically.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="y")
>>> con = model.add_constraint(1.5 * x + 2.0 * y == 5.0, name="balance")
>>> model.minimize(x + y)
>>>
>>> snapshot = model.inspect(include_coeffs=True)
>>> snapshot.metadata.variables
2
>>> snapshot.metadata.constraints
1
>>> snapshot.metadata.coefficients
2
>>> snapshot.metadata.memory.coefficient_value_bytes
16
>>> snapshot.metadata.memory.sparse_matrix_bytes >= 16
True
>>> [v.name for v in snapshot.variables]
['x', 'y']
>>> snapshot.constraints[0].name
'balance'
```

### Read variable metadata from a snapshot

If you attach metadata when creating a variable, the snapshot preserves it on
the matching `VariableView`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> flow = model.add_variable(
...     bounds=arco.Bounds(lower=0.0, upper=10.0),
...     name="flow",
...     metadata={"role": "output", "units": "MW"},
... )
>>> snapshot = model.inspect()
>>> snapshot.variables[0].metadata
{'role': 'output', 'units': 'MW'}
```

For sparse arrays created with an `active=` mask, inspect the array before
solving to compare the dense shape with the variables actually created.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> i = arco.IndexSet(name="i", members=[0, 1])
>>> r = arco.IndexSet(name="r", members=[0, 1])
>>> h = arco.IndexSet(name="h", members=[0, 1])
>>> active = np.array([[True, False], [False, True]])
>>> dispatch = model.add_variables(axes=(i, r, h), bounds=arco.NonNegativeFloat, active=active)
>>> dispatch.shape
(2, 2, 2)
>>> dispatch.dense_count
8
>>> dispatch.active_count
4
>>> dispatch.memory_estimate()["inactive_slots"]
4
>>> dispatch.memory_estimate()["active_density"]
0.5
>>> dispatch.memory_estimate()["linear_terms"]
4
>>> dispatch.memory_estimate()["estimated_dense_linear_term_bytes"] > dispatch.memory_estimate()["estimated_term_bytes"]
True
>>> dispatch.memory_estimate()["estimated_solver_sparse_matrix_bytes"] < dispatch.memory_estimate()["estimated_dense_linear_term_bytes"]
True
>>> model.inspect().metadata.variables
4
>>> model.inspect().metadata.memory.sparse_matrix_bytes > 0
True
```

`memory_estimate()` also works on expression arrays. It reports the current
array storage kind, dense slots, active and inactive slots, active density,
expression term counts, estimated in-memory term bytes, and a solver-oriented
sparse matrix byte estimate before the expression is lowered into constraints
or an objective. The solver-oriented fields use the same
value/index/column-pointer accounting as `model.inspect().metadata.memory`,
with `estimated_solver_sparse_matrix_bytes` as the total planning estimate.

When solved, `result.value(dispatch)` preserves the dense indexed shape and
uses `nan` for inactive entries. This keeps array coordinates stable while
making inactive points visible in downstream NumPy code.

## CLI pre-solve size checks

For KDL models, use `arco inspect --json` before solving to check declaration
counts, expanded instance counts, coefficient estimates, and conservative
sparse-matrix memory estimates.

```sh
arco inspect examples/capacity-expansion/input.kdl --json
```

The `meta.counts` object distinguishes declarations from expanded instances:
`variable` and `constraint` count KDL declarations, while
`variable_instances`, `constraint_instances`, and `coefficient_instances`
estimate the lowered model size. The `meta.memory.sparse_matrix_bytes` field
estimates value, index, and column-pointer bytes for a sparse matrix view of
the constraints. Treat this as a no-solve planning signal; concrete solver
backends may allocate additional internal structures.

For tuple-domain variables, `variable_instances` and each variable record's
`instances` field count tuple rows, not the Cartesian product of tuple
component domains. The component sizes remain visible in the variable's `set`
bindings so you can review the domain shape without inflating memory estimates.

## Solution summary

After solving, call `arco.solution_summary()` to print a diagnostic overview
of the result to the console. Pass `verbose=True` to include variable values,
dual prices, and timing information.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="x")
>>> model.add_constraint(x >= 3.0, name="floor")
Constraint('floor', Bounds(3, inf))
>>> model.minimize(x)
>>> solution = model.solve(log_to_console=False)
>>> arco.solution_summary(solution)  # doctest: +SKIP
>>> solution.solve_time_seconds() >= 0.0
True
```

The compact output looks like this:

```
Solution Summary
├ solver          : HiGHS
└ Termination
  ├ status        : OPTIMAL
  └ objective     : 3.00000e+00
```

With `verbose=True`, the output includes variable values, dual prices, and
solver work statistics.

## Expert sparse exports

When you need raw solver-order sparse matrix exports (`export_csc`,
`export_crs`, `export_coo`) for integration code, use the dedicated expert
guide: [Use Expert APIs](./use-expert-apis.md). This inspect guide keeps the
default debug workflow centered on named objects and `inspect(...)` output.

---

[How-to Guides](./) | [Docs home](../)
