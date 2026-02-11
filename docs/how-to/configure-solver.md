# Configure Solver

Arco separates model construction from solver configuration. You can build a
model once and solve it with different solver settings by swapping the solver
object passed to `model.solve()`. This guide shows how to create, customize,
and reuse solver configurations.

## Create a solver object

Use `arco.HiGHS(...)` to create a solver configuration with explicit settings.
Pass the object to `model.solve(solver=...)` to control how the solver behaves
during optimization.

```python doctest
>>> import arco
>>> solver = arco.HiGHS(time_limit=60.0, log_to_console=False)
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.minimize(expr=x)
>>> solution = model.solve(solver=solver)
>>> solution.status
SolutionStatus.OPTIMAL
```

When you do not need to customize anything beyond the defaults, `arco.Solver()`
creates a generic solver backed by HiGHS with default settings.

```python doctest
>>> import arco
>>> solver = arco.Solver(log_to_console=False)
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.minimize(expr=x)
>>> solution = model.solve(solver=solver)
>>> solution.status
SolutionStatus.OPTIMAL
```

## Adjust settings with copy

Use `solver.copy(update={...})` to create a new solver configuration that
inherits all settings from the original and overrides only the keys you
specify. The original solver is left unchanged.

```python doctest
>>> import arco
>>> solver = arco.HiGHS(time_limit=60.0, log_to_console=False)
>>> solver.time_limit
60.0
>>> fast = solver.copy(update={"time_limit": 10.0})
>>> fast.time_limit
10.0
>>> solver.time_limit
60.0
```

This is useful when you have a base configuration for production runs and want
a tighter variant for quick validation without duplicating every setting.

```python doctest
>>> import arco
>>> base = arco.HiGHS(time_limit=120.0, mip_gap=0.01, log_to_console=False)
>>> debug = base.copy(update={"time_limit": 5.0, "mip_gap": 0.05})
>>> debug.time_limit
5.0
>>> debug.mip_gap
0.05
>>> base.mip_gap
0.01
```

## Solver settings reference

All settings return `None` when not explicitly set, in which case the solver
backend uses its own default.

| Setting          | Type    | Description                                                        |
|------------------|---------|--------------------------------------------------------------------|
| `presolve`       | `bool`  | Enable or disable the presolve phase.                              |
| `threads`        | `int`   | Number of threads the solver may use.                              |
| `tolerance`      | `float` | Feasibility tolerance for primal and dual values.                  |
| `time_limit`     | `float` | Maximum wall-clock seconds the solver may run.                     |
| `mip_gap`        | `float` | Relative MIP optimality gap at which the solver stops.             |
| `verbosity`      | `int`   | Solver output verbosity level (backend-specific scale).            |
| `log_to_console` | `bool`  | Whether the solver prints progress to the console during a solve.  |

> [!NOTE]
> Not every backend interprets every setting. If a setting does not apply to the
> chosen backend it is silently ignored.
>
> `time_limit`, `mip_gap`, and `tolerance` must be finite and non-negative.
> `threads` must be at least `1`.

## Pass settings directly to solve

When you only need solver settings for a single call and do not plan to reuse
them, pass the settings as keyword arguments directly to `model.solve()`. This
avoids creating a separate solver object.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> _ = model.add_constraint(expr=x + y >= 5.0)
>>> model.minimize(expr=x + y)
>>> solution = model.solve(log_to_console=False, time_limit=60.0, mip_gap=0.01)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
5.0
```

This is equivalent to constructing an `arco.HiGHS(...)` and passing it via the
`solver` keyword, but more concise for one-off solves.

> [!NOTE]
> Arco also supports `arco.Xpress(...)` for the FICO Xpress solver backend.
> Xpress requires a commercial license. The API mirrors `arco.HiGHS(...)` --
> pass the same settings and use `solver=arco.Xpress(threads=4)` in the solve
> call.

---

[How-to Guides](./) | [Docs home](../)
