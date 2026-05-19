# API UX Ladder Changes

These changes align the Python API with Arco's API UX ladder. They are
intentional pre-1.0 breaking changes: prefer the new forms rather than adding
compatibility shims.

## Indexed parameters

Use keyword-only `name=` when creating index sets.

| Old                                     | New                                          |
| --------------------------------------- | -------------------------------------------- |
| `arco.IndexSet("asset", members=[...])` | `arco.IndexSet(name="asset", members=[...])` |

Use `axes=` to make the dimension contract explicit.

| Old                               | New                                      |
| --------------------------------- | ---------------------------------------- |
| `arco.param(values, asset, time)` | `arco.param(values, axes=(asset, time))` |
| `arco.param(values, i)`           | `arco.param(values, axes=(i,))`          |

The values object stays the primary positional subject. Axes and names are
configuration.

## Scalar variables

Use keyword-only `bounds=` when creating scalar variables.

| Old                                                                | New                                                                       |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| `model.add_variable(arco.NonNegativeFloat, name="x")`              | `model.add_variable(bounds=arco.NonNegativeFloat, name="x")`              |
| `model.add_variable(arco.Bounds(lower=0.0, upper=10.0), name="x")` | `model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), name="x")` |

The scalar variable API now matches indexed variables: bounds, integrality, and
names are explicit construction fields. `arco.Bounds(...)` also requires
keyword-only `lower=` and `upper=` fields.

## Variable arrays

Use keyword-only `axes=` when creating indexed variables.

| Old                                            | New                                                   |
| ---------------------------------------------- | ----------------------------------------------------- |
| `model.add_variables(asset, time, bounds=...)` | `model.add_variables(axes=(asset, time), bounds=...)` |
| `model.add_variables(i, bounds=...)`           | `model.add_variables(axes=(i,), bounds=...)`          |

For sparse arrays, inspect the dense and active counts directly:

```python
x = model.add_variables(axes=(i, r, h), bounds=arco.NonNegativeFloat, active=mask)
x.shape
x.dense_count
x.active_count
x.memory_estimate()
```

`shape` and `dense_count` describe the indexed domain. `active_count` describes
how many variables were actually created after applying `active=`.
`memory_estimate()` adds the storage kind, expression term counts, and
estimated term bytes for pre-solve memory planning.

When reading sparse array results, `solution.value(x)` preserves the dense
indexed shape and marks inactive coordinates as `nan`. This keeps axis
coordinates stable while making inactive entries explicit.

## Solution access

Use the ladder-facing accessors for ordinary result reads.

| Old                                   | New                               |
| ------------------------------------- | --------------------------------- |
| `solution.get_value(x)`               | `solution.value(x)`               |
| `solution.get_dual(constraint)`       | `solution.dual(constraint)`       |
| `solution.get_slack(constraint)`      | `solution.slack(constraint)`      |
| `solution.get_reduced_cost(variable)` | `solution.reduced_cost(variable)` |
| `solution.get_primal(index=int(x))`   | `solution.value(x)`               |

Raw index accessors remain available for advanced workflows that only have raw
variable or constraint indices. They should not be the first form shown in
beginner docs.

## Name and metadata helpers

Use object-facing lookups and inspection instead of raw-ID name and metadata
shortcuts.

| Old                                      | New                                                                                  |
| ---------------------------------------- | ------------------------------------------------------------------------------------ |
| `model.get_variable_by_name("x")`        | `model.get_variable(name="x")`; missing names raise `VariableNotFoundError`          |
| `model.get_constraint_by_name("demand")` | `model.get_constraint(name="demand")`; missing names raise `ConstraintNotFoundError` |
| `model.get_variable_name(var_id)`        | `model.inspect().variables[...]`                                                     |
| `model.get_constraint_name(con_id)`      | `model.inspect().constraints[...]`                                                   |
| `model.set_objective_name(name="cost")`  | `model.minimize(expr, name="cost")` or `model.maximize(expr, name="profit")`         |

Raw-ID metadata mutators were removed from the Python public surface. Keep
metadata in your domain objects or use model snapshots for inspection.

## Sparse exports

Use the implemented sparse export formats for interchange. Placeholder exports
that only failed at runtime are not part of the public ladder.

| Old                    | New                                                                 |
| ---------------------- | ------------------------------------------------------------------- |
| `model.get_columns()`  | `model.export_csc()`, `model.export_crs()`, or `model.export_coo()` |
| `model.export_arrow()` | `model.export_csc()`, `model.export_crs()`, or `model.export_coo()` |

## Why these changes

- One primary positional subject per function.
- Keyword-only configuration for axes, names, masks, and settings.
- Names and axes are explicit enough to inspect.
- Sparse modeling exposes dense size and active size before solve.
- Result reads use model objects instead of raw IDs where possible.

---

[Migration Notes](./) | [Docs home](../)
