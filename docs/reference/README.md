# API Reference

The Python API surface is defined in the type stub file
[`arco.pyi`](../../bindings/python/arco/arco.pyi). This file provides type
signatures for all public classes, methods, and functions.

Treat the stub as the authoritative reference for call shapes, keyword-only
arguments, return types, and which APIs are public. Narrative examples and
workflow guidance live in the tutorials and how-to guides; this reference page
keeps the public contract anchored to the typed surface that editors and static
checks consume.

The public surface intentionally favors beginner-friendly modeling methods and
standard interchange formats. Expert APIs such as sparse-matrix import/export
and raw coefficient editing remain available, but are documented as escape
hatches rather than the primary modeling path.

## Quick links

- `arco.Model` — the central class for building and solving optimization
  problems.
- `arco.Bounds` — variable and constraint bound specifications.
- `arco.SolveResult` — solution data returned by `Model.solve()`.
- `arco.IndexSet` — named index sets for multi-dimensional variable arrays.
- `arco.VariableArray` — multi-dimensional arrays of decision variables.

---

[Back to docs home](../README.md)
