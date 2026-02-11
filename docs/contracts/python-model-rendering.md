# Python Model Rendering Contract

This file encodes rendering guarantees for the Python API as executable doctests.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x1 = model.add_variable(bounds=arco.Binary, name="x[1]")
>>> x2 = model.add_variable(bounds=arco.Binary, name="x[2]")
>>> x3 = model.add_variable(bounds=arco.Binary, name="x[3]")
>>> x4 = model.add_variable(bounds=arco.Binary, name="x[4]")
>>> x5 = model.add_variable(bounds=arco.Binary, name="x[5]")
>>> _ = model.add_constraint(expr=2.0 * x1 + 8.0 * x2 + 4.0 * x3 + 2.0 * x4 + 5.0 * x5 <= 10.0)
>>> _ = model.add_constraint(expr=x1 + x3 - x5 == 0.0)
>>> model.maximize(expr=5.0 * x1 + 3.0 * x2 + 2.0 * x3 + 7.0 * x4 + 4.0 * x5)
>>> rendered = str(model)
>>> rendered.startswith("Max 5 x[1] + 3 x[2] + 2 x[3] + 7 x[4] + 4 x[5]")
True
>>> "\ns.t.\n" in rendered
True
>>> "Subject to" in rendered
False
>>> "Binary: x[1], x[2], x[3], x[4], x[5]" in rendered
True
>>> " <= 10" in rendered
True
>>> operator_columns = [
...     line.index(" <=")
...     for line in rendered.splitlines()
...     if " <=" in line
... ] + [
...     line.index(" = ")
...     for line in rendered.splitlines()
...     if " = " in line
... ]
>>> len(operator_columns) >= 2
True
>>> max(operator_columns) - min(operator_columns) <= 1
True
```

```python doctest
>>> import contextlib
>>> import io
>>> import arco
>>> model = arco.Model()
>>> vars_ = [model.add_variable(bounds=arco.Binary, name=f"x[{i + 1}]") for i in range(35)]
>>> model.minimize(expr=sum(vars_))
>>> for idx in range(22):
...     _ = model.add_constraint(expr=sum(vars_) <= float(idx), name=f"c[{idx + 1}]")
>>> preview = str(model)
>>> "... (5 more terms)" in preview
True
>>> "... (2 more constraints)" in preview
True
>>> buf = io.StringIO()
>>> with contextlib.redirect_stdout(buf):
...     model.pprint()
>>> full = buf.getvalue()
>>> "... (2 more constraints)" in full
False
>>> "c[22]:" in full
True
```

```python doctest
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
>>> rendered = str(model)
>>> "Index sets:" in rendered
True
>>> "T = [0, 1]" in rendered
True
>>> "G = [solar, wind, gas]" in rendered
True
>>> "gen[0,solar]" in rendered
True
>>> "gen[1,gas]" in rendered
True
>>> "Bounds:" in rendered
True
>>> "0 <= gen[t,g] <= 100  for t in T, g in G" in rendered
True
```
