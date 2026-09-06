# Manage Model Memory

Python models normally remain available after `solve()`. If a model is large
and will not be reused, pass the explicit `arco.consume_model` solver parameter
to release its modeling storage during the solve.

```python
import arco

model = arco.Model()
x = model.add_variable(bounds=arco.NonNegativeFloat)
model.add_constraint(x >= 1.0)
model.minimize(x)

solution = model.solve(
    solver=arco.HiGHS(
        log_to_console=False,
        parameters={"arco.consume_model": "true"},
    )
)
assert solution.is_optimal()
assert model.num_variables == 0
```

For HiGHS and Xpress, Arco prepares the native problem first. After preparation
succeeds, Arco clears the canonical model, array and constraint display
metadata, block definitions, and any previous solution before native
optimization starts. The returned solution remains usable, but the model cannot
be inspected or solved again.

If native preparation fails, the model remains unchanged, including any
previous solution. Once preparation has succeeded, a later optimization or
solution extraction error leaves the model consumed because its canonical
storage has already been released. Solver outcomes such as infeasible,
unbounded, or time limit are returned as solution statuses and also leave the
model consumed.

The parameter is an explicit ownership choice at the native preparation
handoff. Omit it when callers need to inspect, solve again, or recover the model
after a backend exception. Models using other solver families retain their
existing consumption timing; models without the parameter keep their normal
reusable behavior.

Incremental objective terms are copied into the model after validation, so the
source expression remains reusable for another `add_objective_terms` call. A
large expression can therefore be kept as a single Python object while it is
added; invalid terms leave the existing objective unchanged.

Sparse variable reductions over one axis retain the source variable array and
the output labels until a consumer needs expression rows. They still represent
every output slot, including groups containing only implicit zeroes, so
comparisons and model insertion keep their dense row membership. Reading values,
inspecting terms, or inserting the reduction materializes those rows and then
uses the output labels, including any labels applied by `relabel_axis`.

[How-to Guides](./) | [Docs home](../)
