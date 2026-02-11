# Handle Errors

Arco raises typed exceptions for model-building mistakes and returns status codes
for solver outcomes. This separation lets you use standard Python exception
handling for programming errors while inspecting the solve result for
optimization-level outcomes like infeasibility or unboundedness.

## Model-building errors

Errors in model construction are raised immediately when the offending call is
made. Each error type corresponds to a specific kind of mistake, so you can
catch exactly the problem you expect.

Passing bounds where the lower exceeds the upper raises `BoundsInvalidError`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> try:
...     model.add_variable(bounds=arco.Bounds(lower=10.0, upper=0.0))
... except arco.BoundsInvalidError:
...     print("lower bound exceeds upper bound")
lower bound exceeds upper bound
```

Calling `solve()` on a model that has no variables raises `ModelEmptyError`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> try:
...     model.solve(log_to_console=False)
... except arco.ModelEmptyError:
...     print("model has no variables")
model has no variables
```

These exceptions are raised before the solver is ever invoked, so you get fast
feedback during development.

## Solver outcomes

Infeasible and unbounded models do not raise exceptions. The solver runs to
completion and reports the outcome on the returned `SolveResult`. Check the
status to decide what to do next.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=1.0))
>>> _ = model.add_constraint(expr=x >= 5.0)
>>> model.minimize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_infeasible()
True
>>> solution.status
SolutionStatus.INFEASIBLE
```

This design keeps the control flow predictable: exceptions mean something went
wrong building the model, while status codes mean the solver finished but the
problem itself has no feasible or bounded solution.

## Catch all arco errors

Every arco exception inherits from `arco.ArcoError`, which itself inherits from
`Exception`. When you want a single handler for any model-building error, catch
the base class.

```python doctest
>>> import arco
>>> try:
...     model = arco.Model()
...     model.add_variable(bounds=arco.Bounds(lower=10.0, upper=0.0))
... except arco.ArcoError as e:
...     print(type(e).__name__)
BoundsInvalidError
```

This is useful at API boundaries or in batch pipelines where you want to log the
error and continue rather than crash.

## Error reference

The table below lists the most common error classes. All inherit from
`arco.ArcoError`.

| Error | Description |
|-------|-------------|
| `ModelEmptyError` | Model has no variables |
| `ObjectiveMissingError` | No objective set before solving |
| `ObjectiveAlreadySetError` | Objective already defined |
| `BoundsInvalidError` | Lower bound exceeds upper bound |
| `VariableInvalidBoundsError` | Variable bounds are invalid |
| `ConstraintInvalidBoundsError` | Constraint bounds are invalid |
| `SlackInvalidPenaltyError` | Slack penalty must be finite and non-negative |
| `ArrayShapeMismatchError` | Array dimensions don't match |

> [!NOTE]
> Solver outcomes such as infeasible, unbounded, and time limit are not
> exceptions. They are returned as status values on `SolveResult`. Use
> `solution.is_infeasible()`, `solution.status`, and related methods
> to inspect the outcome after calling `model.solve()`.

---

[How-to Guides](./) | [Docs home](../)
