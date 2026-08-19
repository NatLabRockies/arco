# Arco Python bindings

The Python package remains in `bindings/python` for path compatibility with existing build, release, and editable-install workflows. Its Rust crate is named `arco-python` and is the Python interaction surface; public Python imports stay under `arco`.

Python-facing solve orchestration is routed through the shared `arco-ops` facade where it overlaps with other interaction surfaces. The public Python API remains under `arco`.

Write a Python-built model to LP format with the shared exporter:

```python
from pathlib import Path

import arco

model = arco.Model()
x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
model.add_constraint(x >= 1.0, name="minimum")
model.minimize(x, name="cost")

output_path = Path("model.lp")
model.write_lp(output_path)
```

`write_lp` accepts a string or any `os.PathLike[str]`, overwrites the target,
and returns `None`.

Build and install locally with the repository solver setup:

```bash
just py-dev
```

Default Python builds include HiGHS, SCIP, and the runtime-loaded Xpress
backend. Solving with `arco.Xpress(...)` still requires the FICO Xpress runtime
and a valid license on the target machine.

For direct `maturin` workflows, run through the solver build environment so the
bindings link against the cached HiGHS and SCIP distributions:

```bash
cd ../..
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 ARCO_HIGHS_ENABLE_APPLE_STATIC=1 ./scripts/with_solver_build_env.sh \
  bash -lc 'cd bindings/python && uv sync --group dev && ./.venv/bin/maturin develop'
```

`CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=0` are intentional for cold
editable builds: they avoid incremental bookkeeping and debug-info generation
that slow a clean `maturin develop` build. Leave them unset for normal Rust
edit-compile-test loops where incremental rebuilds and debug info help.

To enable the IPOPT nonlinear backend, build with the `ipopt` feature (requires
a system IPOPT install):

```bash
cd bindings/python
uv run --with maturin maturin develop --features ipopt
```

To disable optional commercial or native solver features for a source build,
turn off default Cargo features and opt back into the backends you need:

```bash
cd bindings/python
uv run --with maturin maturin develop --no-default-features --features pyo3/extension-module
```

Without a solver feature, that solver's Python class remains importable but
solve will fail fast with a rebuild hint.

Run linting from the repository root so the project-pinned tool versions are used:

```bash
just py-lint-check
just py-type
```

Run Python example formulations from the repository root:

```bash
cd ../..
uv run examples/dense-lp/formulation.py --solve --json
uv run examples/sdom/formulation.py --solve --json
```

For interactive exploration of dense-lp (no extra script boilerplate):

```bash
cd ../..
uv run --with ipython --with-editable ./bindings/python ipython -i examples/dense-lp/formulation.py
```

Inside IPython, use `model` to inspect the formulation and call `solve()` when ready.

## Running example problems

The `examples/` tree contains standalone Python scripts that build models
directly through the bindings, covering different problem classes:

- LP — linear programs (default HiGHS backend).
- MILP — mixed-integer linear programs (HiGHS).
- NLP — nonlinear programs (requires bindings built with `--features ipopt`).
- QP / QCP — (quadratically constrained) quadratic programs, solved through
  the appropriate backend for the problem class.

Run from `bindings/python` so the locally built extension is on the import path:

```bash
cd bindings/python

# LP — Multi-period DC-OPF (HiGHS)
uv run python ../../examples/multi-period-optimal-power-flow/dc-opf-24bus-wind-load-shedding/problem.py

# NLP — Multi-period AC-OPF (IPOPT)
uv run python ../../examples/multi-period-optimal-power-flow/ac-opf-24bus-wind-load-shedding/problem.py
```

Each script prints the solver status and final objective value alongside the
reference value from the original formulation. Substitute the path to any other
`problem.py` (or `formulation.py`) under `examples/` to run a different model.
