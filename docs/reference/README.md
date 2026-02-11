# API Reference

The Python API surface is defined in the type stub file
[`arco.pyi`](../../bindings/python/arco.pyi). This file provides type
signatures for all public classes, methods, and functions.

Generated reference documentation with prose descriptions is planned for a
future release, once docstrings are added to the Rust source via PyO3.

## Quick links

- `arco.Model` — the central class for building and solving optimization
  problems.
- `arco.Bounds` — variable and constraint bound specifications.
- `arco.SolveResult` — solution data returned by `Model.solve()`.
- `arco.IndexSet` — named index sets for multi-dimensional variable arrays.
- `arco.VariableArray` — multi-dimensional arrays of decision variables.

## Known Problems

- [Known Problems](known-problems.md) — canonical markdown catalog of default
  optimization problems used for benchmarking and regression discussions.

---

[Back to docs home](../)
