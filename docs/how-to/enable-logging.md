# Enable Logging

Arco uses Rust's `tracing` framework internally and exposes a Python helper.
Use `enable_logging()` to see structured diagnostics from the Rust core and the
solver.

## Enable Python-side logging

```python doctest
>>> import arco
>>> _ = arco.enable_logging(level="arco=debug,highs=info")
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=5.0))
>>> model.minimize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
```

If you omit the `level` argument, `enable_logging()` reads the `RUST_LOG`
environment variable when set and falls back to `"error"` when it is not.

## Solver console output

To see the solver's own progress output, set `log_to_console=True` when calling
`solve()`. This is useful for long-running problems where you want to monitor
convergence. In doctests and automated pipelines, keep it set to `False`.

## Solver info

Use `solver_info()` to check which solver backends are available and their
versions.

```python doctest
>>> import arco
>>> info = arco.solver_info()
>>> "highs" in str(info).lower() or len(info) > 0
True
```

---

[How-to Guides](./) | [Docs home](../)
