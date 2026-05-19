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

- A build function decorated with `@block` from `arco`.
- An extract function that reads the solved result and returns the output schema.
- An optional `ctx` object to hold handles the block needs during extraction.

```python doctest
>>> from dataclasses import dataclass
>>> import arco
>>> from arco import block
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
... def build_supply(model: arco.Model, data: SupplyIn, ctx) -> None:
...     x = model.add_variable(
...         bounds=arco.Bounds(lower=0.0, upper=data.capacity),
...         name="supply",
...     )
...     ctx["level"] = x
...     model.minimize(x)
>>>
>>> def extract_supply(result, data: SupplyIn, ctx) -> SupplyOut:
...     return SupplyOut(level=result.value(ctx["level"]))
>>>
>>> @block
... def build_demand(model: arco.Model, data: DemandIn, ctx) -> None:
...     y = model.add_variable(
...         bounds=arco.Bounds(lower=data.supply_level, upper=100.0),
...         name="demand",
...     )
...     ctx["level"] = y
...     model.minimize(y)
>>>
>>> def extract_demand(result, data: DemandIn, ctx) -> DemandOut:
...     return DemandOut(level=result.value(ctx["level"]))
```

## Compose and link blocks

Register blocks with `model.add_block()` and link fields with `.out` and `.in_`.

```python doctest
>>> from dataclasses import dataclass
>>> import arco
>>> from arco import block
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
... def build_supply(model: arco.Model, data: SupplyIn, ctx) -> None:
...     x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=data.capacity), name="supply")
...     ctx["level"] = x
...     model.minimize(x)
>>>
>>> def extract_supply(result, data: SupplyIn, ctx) -> SupplyOut:
...     return SupplyOut(level=result.value(ctx["level"]))
>>>
>>> @block
... def build_demand(model: arco.Model, data: DemandIn, ctx) -> None:
...     y = model.add_variable(bounds=arco.Bounds(lower=data.supply_level, upper=100.0), name="demand")
...     ctx["level"] = y
...     model.minimize(y)
>>>
>>> def extract_demand(result, data: DemandIn, ctx) -> DemandOut:
...     return DemandOut(level=result.value(ctx["level"]))
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
>>> model.has_blocks
True
```

For composed models, inspect `result.blocks[...]` for block-level objective
values and vectors. Use `result.blocks.statuses()` for a compact per-stage
health check, or `result.blocks.report()` for an ordered stage report with
status, objective value, and result vector sizes. Use
`result.blocks.report_json()` when the same report needs to be written to an
operations log or artifact store. The top-level `result.status` is an aggregate
over all blocks.

`result.blocks` also supports `len(result.blocks)` and
`"stage_name" in result.blocks` for lightweight orchestration checks, and
iterates over stage names in execution order.
Use `result.blocks.get("stage_name")` when an optional stage may be absent.
Missing stage lookups such as `result.blocks["missing"]` raise
`arco.BlockResultError` with diagnostic code `arco::block::result`.

Use `result.blocks.diagnostics()` when operations tooling needs one stable
row per stage with status, objective value, result-vector counts, model counts,
and pre-solve memory estimates. `diagnostics_json()` returns the same payload
for logs or workflow artifacts.

Use `result.blocks.artifact_manifest()` to plan retained per-stage artifacts
without writing files or reaching into block internals. The default
`policy="summary"` retains stage diagnostics and compact solution summaries;
`policy="model"` also marks model snapshots for retention, and `policy="none"`
records that no per-stage artifacts should be kept. Unknown policy values
raise `arco.BlockContractError`. `artifact_manifest_json()` returns the same
manifest as JSON for operations logs or artifact stores.
Cycles in block links use the same `BlockContractError` path because the block
graph cannot be scheduled until the cycle is removed.

Use `result.blocks.write_artifacts(path, policy="summary")` when the composed
solve should leave an on-disk artifact bundle. The writer creates a
`manifest.json` file plus one ordered directory per block. The default policy
writes `stage_diagnostics.json` and compact `solution_summary.json` files.
`policy="model"` also writes a compact `model_snapshot.json` for each block,
including counts and memory estimates without exposing internal storage.
Filesystem or encoding failures raise `arco.BlockArtifactError` with diagnostic
code `arco::block::artifact_io`.

## Validation guarantees

Arco validates the typed block contract at registration and link time:

- Build function must be decorated with `@block` from `arco.blocks`.
- Build function signature must be `(model, data)` or `(model, data, ctx)`.
- Build functions must mutate the provided model and return `None`.
- `data` type must be a `dataclass` or `pydantic.BaseModel`.
- Extract function signature must be `(solution, data)` or `(solution, data, ctx)`.
- Extract return type must be a `dataclass` or `pydantic.BaseModel`.
- Link source and target field types must match.
- Unknown block ports accessed through `block.input("...")`,
  `block.output("...")`, `block.in_`, or `block.out` are contract errors.

These failures raise `arco.BlockContractError` with diagnostic code
`arco::block::contract`, so applications can catch one stable Arco error for
block authoring and linking mistakes.

## Next steps

- Use [How-to Guides](../how-to/) for task-focused recipes.
- Use this pattern to break large models into independently testable stages.

---

&larr; [Indexed Models](indexed-models.md) | [Tutorials](./)
