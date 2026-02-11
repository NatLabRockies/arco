# Define Block Schemas

Use block schemas to make block inputs and outputs explicit and type-safe.

## Pick a schema type

Arco supports:

- `dataclass` (recommended default for low overhead)
- `pydantic.BaseModel` (strict validation and rich error messages)

## Dataclass schema example

```python
from dataclasses import dataclass
import arco
from arco.blocks import block

@dataclass(slots=True)
class CapacityIn:
    capacity: float

@dataclass(slots=True)
class CapacityOut:
    level: float

@block
def build_capacity(model: arco.Model, data: CapacityIn) -> None:
    x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=data.capacity), name="x")
    model.minimize(expr=x)

def extract_capacity(result, data: CapacityIn) -> CapacityOut:
    return CapacityOut(level=result.get_primal(index=0))
```

## Pydantic schema example

```python
from pydantic import BaseModel, Field
import arco
from arco.blocks import block

class DemandIn(BaseModel):
    floor: float = Field(ge=0.0)

class DemandOut(BaseModel):
    value: float

@block
def build_demand(model: arco.Model, data: DemandIn) -> None:
    y = model.add_variable(bounds=arco.Bounds(lower=data.floor, upper=100.0), name="y")
    model.minimize(expr=y)

def extract_demand(result, data: DemandIn) -> DemandOut:
    return DemandOut(value=result.get_primal(index=0))
```

## Link by typed fields

`BlockHandle` exposes typed field ports:

```python
model.link(upstream.out.level, downstream.in_.floor)
```

Arco validates source and target field types before solve.

## Common errors

- Build function not decorated with `@block` from `arco.blocks`.
- Missing type annotation on `data`.
- Extract function return type missing or unsupported.
- Linking fields with different types.

---

[Back to how-to](./)
