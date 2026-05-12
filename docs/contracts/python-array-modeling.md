# Python Array Modeling Contracts

These contracts lock Python array modeling semantics that are not tied to a
single canonical optimization problem.

## Axis Label Alignment (position-independent semantics)

Array arithmetic aligns by axis labels, not axis position.

```python doctest
>>> import numpy as np
>>> import arco
>>> model = arco.Model()
>>> i = arco.IndexSet("i", members=["a", "b"])
>>> t = arco.IndexSet("t", members=[2020, 2025])
>>> x = model.add_variables(t, i, bounds=arco.NonNegativeFloat, name="X")
>>> weight = arco.param(np.array([[1.0, 2.0], [10.0, 20.0]]), i, t)
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
>>> i = arco.IndexSet("i", members=["a", "b"])
>>> h = arco.IndexSet("h", members=[0, 1, 2])
>>> t = arco.IndexSet("t", members=[2020, 2025])
>>> gen = model.add_variables(i, h, t, bounds=arco.NonNegativeFloat, name="GEN")
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
```

## Active Masks + Directed Alias Axes

```python doctest
>>> import numpy as np
>>> import arco
>>> model = arco.Model()
>>> r = arco.IndexSet("r", members=["north", "south"])
>>> r_from = r.alias("from")
>>> r_to = r.alias("to")
>>> h = arco.IndexSet("h", members=[0, 1])
>>> route_active = arco.param(np.array([[False, True], [True, False]]), r_from, r_to)
>>> transcap = arco.param(np.array([[0.0, 12.0], [8.0, 0.0]]), r_from, r_to)
>>> flow = model.add_variables(r_from, r_to, h, bounds=arco.Bounds(0.0, transcap), active=route_active, name="FLOW")
>>> model.num_variables
4
>>> export_cons = model.add_constraints((flow @ r_to) >= 0.0, name="export_nonneg")
>>> len(export_cons)
4
>>> i = arco.IndexSet("i", members=[0, 1, 2])
>>> y = model.add_variables(i, bounds=arco.NonNegativeFloat, name="Y")
>>> masked = model.add_constraints(y, sense="ge", rhs=0.0, active=[True, False, True], name="masked")
>>> len(masked)
2
```

## Integer Variable + Explicit Range Constraint

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(0.0, 10.0), is_integer=True, name="x")
>>> _ = model.add_constraint(x, bounds=arco.Bounds(3.0, 7.0), name="range_x")
>>> model.minimize(x)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.get_value(x), 6)
3.0
>>> round(solution.objective_value, 6)
3.0
```
