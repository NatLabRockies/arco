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
>>> model.add_constraint(expr=1.5 * x + 2.0 * y == 5.0, name="balance")
Constraint('balance', Bounds(5, 5))
>>> model.add_constraint(expr=x + y >= 3.0, name="floor")
Constraint('floor', Bounds(3, inf))
>>> model.minimize(expr=x + y)
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
>>> model.minimize(expr=x + y)
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
>>> gen = model.add_variables(
...     index_sets=[t, g],
...     bounds=arco.Bounds(lower=0.0, upper=100.0),
...     name="gen",
... )
>>> capacity = {"solar": 50.0, "wind": 80.0, "gas": 100.0}
>>> caps = [capacity[name] for name in g.members] * t.size
>>> _ = model.add_constraints(expr=gen <= caps)
>>> _ = model.add_constraints(expr=gen.sum(over=g) >= [120.0, 90.0])
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
>>> gen = model.add_variables(
...     index_sets=[T, G],
...     bounds=arco.Bounds(lower=0.0, upper=100.0),
... )
>>> caps = [capacity[g] for g in G.members] * T.size
>>> cap_cons = model.add_constraints(expr=gen <= caps)
>>> demand_cons = model.add_constraints(expr=gen.sum(over=G) >= demand)
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
>>> con = model.add_constraint(expr=1.5 * x + 2.0 * y == 5.0, name="balance")
>>> model.minimize(expr=x + y)
>>>
>>> snapshot = model.inspect(include_coeffs=True)
>>> snapshot.metadata.variables
2
>>> snapshot.metadata.constraints
1
>>> [v.name for v in snapshot.variables]
['x', 'y']
>>> snapshot.constraints[0].name
'balance'
```

## Solution summary

After solving, call `arco.solution_summary()` to print a diagnostic overview
of the result to the console. Pass `verbose=True` to include variable values,
dual prices, and timing information.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="x")
>>> model.add_constraint(expr=x >= 3.0, name="floor")
Constraint('floor', Bounds(3, inf))
>>> model.minimize(expr=x)
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

## Export to CSC format

Use `export_csc()` to get the constraint matrix in compressed sparse column
format. This is useful for interoperability with scipy, numpy, or custom
analysis tools.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=5.0))
>>> _ = model.add_constraint(expr=x >= 1.0)
>>> model.minimize(expr=x)
>>> csc = model.export_csc()
>>> "col_ptrs" in csc
True
>>> "row_indices" in csc
True
>>> "values" in csc
True
```

---

[How-to Guides](./) | [Docs home](../)
