# Block Composition

This tutorial shows how to split a model into typed blocks and pass data between
blocks without string-key dictionaries.

## Why typed blocks?

Typed blocks make data flow explicit:

- Inputs are schema objects (`dataclass` or `pydantic.BaseModel`).
- Outputs are schema objects returned by an `extract` function.
- Links connect typed fields (`supply.out.level -> demand.in_.supply_level`).

This gives editor autocomplete, earlier validation errors, and safer refactors.

## Define schemas and block functions

Each block has:

- A build function decorated with `@block` from `arco.blocks`.
- An extract function that reads the solved result and returns the output schema.

```python doctest
>>> from dataclasses import dataclass
>>> import arco
>>> from arco.blocks import block
>>>
>>> @dataclass(slots=True)
... class SupplyIn:
...     capacity: float
>>>
>>> @dataclass(slots=True)
... class SupplyOut:
...     level: float
>>>
>>> @dataclass(slots=True)
... class DemandIn:
...     supply_level: float
>>>
>>> @dataclass(slots=True)
... class DemandOut:
...     level: float
>>>
>>> @block
... def build_supply(model: arco.Model, data: SupplyIn) -> None:
...     x = model.add_variable(
...         bounds=arco.Bounds(lower=0.0, upper=data.capacity),
...         name="supply",
...     )
...     model.minimize(expr=x)
>>>
>>> def extract_supply(result, data: SupplyIn) -> SupplyOut:
...     return SupplyOut(level=result.get_primal(index=0))
>>>
>>> @block
... def build_demand(model: arco.Model, data: DemandIn) -> None:
...     y = model.add_variable(
...         bounds=arco.Bounds(lower=data.supply_level, upper=100.0),
...         name="demand",
...     )
...     model.minimize(expr=y)
>>>
>>> def extract_demand(result, data: DemandIn) -> DemandOut:
...     return DemandOut(level=result.get_primal(index=0))
```

## Compose and link blocks

Register blocks with `model.add_block()` and link fields with `.out` and `.in_`.

```python doctest
>>> from dataclasses import dataclass
>>> import arco
>>> from arco.blocks import block
>>>
>>> @dataclass(slots=True)
... class SupplyIn:
...     capacity: float
>>>
>>> @dataclass(slots=True)
... class SupplyOut:
...     level: float
>>>
>>> @dataclass(slots=True)
... class DemandIn:
...     supply_level: float
>>>
>>> @dataclass(slots=True)
... class DemandOut:
...     level: float
>>>
>>> @block
... def build_supply(model: arco.Model, data: SupplyIn) -> None:
...     x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=data.capacity), name="supply")
...     model.minimize(expr=x)
>>>
>>> def extract_supply(result, data: SupplyIn) -> SupplyOut:
...     return SupplyOut(level=result.get_primal(index=0))
>>>
>>> @block
... def build_demand(model: arco.Model, data: DemandIn) -> None:
...     y = model.add_variable(bounds=arco.Bounds(lower=data.supply_level, upper=100.0), name="demand")
...     model.minimize(expr=y)
>>>
>>> def extract_demand(result, data: DemandIn) -> DemandOut:
...     return DemandOut(level=result.get_primal(index=0))
>>>
>>> model = arco.Model()
>>> supply = model.add_block(
...     build_supply,
...     data=SupplyIn(capacity=50.0),
...     extract=extract_supply,
... )
>>> demand = model.add_block(
...     build_demand,
...     extract=extract_demand,
... )
>>> model.link(supply.out.level, demand.in_.supply_level)
>>> result = model.solve(log_to_console=False)
>>> result.blocks.keys()
['build_supply', 'build_demand']
>>> result.blocks['build_supply'].is_optimal()
True
>>> result.blocks['build_demand'].is_optimal()
True
```

For composed models, inspect `result.blocks[...]` for block-level objective
values and vectors. The top-level `result.status` is an aggregate over all
blocks.

## Validation guarantees

Arco validates the typed block contract at registration and link time:

- Build function must be decorated with `@block` from `arco.blocks`.
- Build function signature must be `(model, data)` or `(model, data, ctx)`.
- `data` type must be a `dataclass` or `pydantic.BaseModel`.
- Extract function signature must be `(solution, data)` or `(solution, data, ctx)`.
- Extract return type must be a `dataclass` or `pydantic.BaseModel`.
- Link source and target field types must match.

## Next steps

- Use [How-to Guides](../how-to/) for task-focused recipes.
- Use this pattern to break large models into independently testable stages.

---

&larr; [Indexed Models](indexed-models.md) | [Tutorials](./)
