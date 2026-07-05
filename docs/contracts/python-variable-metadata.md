# Python Variable Metadata Contract

This file encodes handle-based variable metadata guarantees for the Python API as executable doctests.

## Creation-time metadata round-trips through the handle and snapshot

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(
...     bounds=arco.Bounds(lower=0.0, upper=10.0),
...     name="x",
...     metadata={"role": "output", "units": "MW"},
... )
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="y")
>>> model.get_variable_metadata(x)
{'role': 'output', 'units': 'MW'}
>>> model.get_variable_metadata(y) is None
True
>>> snapshot = model.inspect()
>>> snapshot.variables[0].metadata
{'role': 'output', 'units': 'MW'}
>>> snapshot.variables[1].metadata is None
True
```

## Metadata is optional and existing scalar creation still works

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
>>> model.get_variable_metadata(x) is None
True
>>> model.add_variable(bounds=arco.Binary, name="flag")
Variable('flag', Binary)
```

## Contract surface stays handle-based

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.NonNegativeFloat, metadata={"role": "output"})
>>> hasattr(model, "get_variable_metadata")
True
>>> hasattr(model, "set_variable_metadata")
False
>>> hasattr(model, "get_constraint_metadata")
False
>>> hasattr(model, "set_constraint_metadata")
False
>>> model.get_variable_metadata(x)
{'role': 'output'}
```
