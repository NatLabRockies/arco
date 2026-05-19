# Python Array Modeling Contracts

These contracts lock Python array modeling semantics that are not tied to a
single canonical optimization problem.

## Axis Label Alignment (position-independent semantics)

Array arithmetic aligns by axis labels, not axis position.

```python doctest
>>> import numpy as np
>>> import arco
>>> model = arco.Model()
>>> i = arco.IndexSet(name="i", members=["a", "b"])
>>> t = arco.IndexSet(name="t", members=[2020, 2025])
>>> x = model.add_variables(axes=(t, i), bounds=arco.NonNegativeFloat, name="X")
>>> weight = arco.param(np.array([[1.0, 2.0], [10.0, 20.0]]), axes=(i, t), name="weight")
>>> weight.name
'weight'
>>> model.minimize((weight * x).sum())
>>> snapshot = model.inspect(include_coeffs=True)
>>> coeffs = [float(term[1]) for term in snapshot.objective.terms]
>>> coeffs
[1.0, 10.0, 2.0, 20.0]
```

## Named-axis NumPy Operations (einsum, roll, diff, tuple reductions)

```python doctest
>>> import numpy as np
>>> import arco
>>> model = arco.Model()
>>> i = arco.IndexSet(name="i", members=["a", "b"])
>>> h = arco.IndexSet(name="h", members=[0, 1, 2])
>>> t = arco.IndexSet(name="t", members=[2020, 2025])
>>> gen = model.add_variables(axes=(i, h, t), bounds=arco.NonNegativeFloat, name="GEN")
>>> ramp = np.diff(gen, axis=h)
>>> ramp.shape
(2, 2, 2)
>>> rolled = np.roll(gen, -1, axis=h)
>>> rolled.shape
(2, 3, 2)
>>> tuple(axis.name for axis in rolled.index_sets)
('i', 'h', 't')
>>> via_einsum = np.einsum("t,i,iht->", np.array([0.95, 0.90]), np.array([10.0, 20.0]), gen)
>>> model.minimize(via_einsum)
>>> total_by_tuple_reduction = (gen @ (i, h, t))
>>> type(total_by_tuple_reduction).__name__
'Expr'
>>> try:
...     np.sum(gen, axis=(i, i))
... except arco.ArrayDimensionError as exc:
...     "duplicate axis" in str(exc)
True
```

## Active Masks + Directed Alias Axes

```python doctest
>>> import numpy as np
>>> import arco
>>> model = arco.Model()
>>> r = arco.IndexSet(name="r", members=["north", "south"])
>>> r_from = r.alias("from")
>>> r_to = r.alias("to")
>>> h = arco.IndexSet(name="h", members=[0, 1])
>>> route_active = arco.param(np.array([[False, True], [True, False]]), axes=(r_from, r_to))
>>> transcap = arco.param(np.array([[0.0, 12.0], [8.0, 0.0]]), axes=(r_from, r_to))
>>> flow = model.add_variables(axes=(r_from, r_to, h), bounds=arco.Bounds(lower=0.0, upper=transcap), active=route_active, name="FLOW")
>>> model.num_variables
4
>>> export_cons = model.add_constraints((flow @ r_to) >= 0.0, name="export_nonneg")
>>> len(export_cons)
4
>>> i = arco.IndexSet(name="i", members=[0, 1, 2])
>>> y = model.add_variables(axes=(i,), bounds=arco.NonNegativeFloat, name="Y")
>>> masked = model.add_constraints(y, sense="ge", rhs=0.0, active=[True, False, True], name="masked")
>>> len(masked)
2
```

## Integer Variable + Explicit Range Constraint

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), is_integer=True, name="x")
>>> _ = model.add_constraint(x, bounds=arco.Bounds(lower=3.0, upper=7.0), name="range_x")
>>> model.minimize(x)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.value(x), 6)
3.0
>>> round(solution.objective_value, 6)
3.0
```
