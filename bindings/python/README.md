# Arco Python bindings

The Python package remains in `bindings/python` for path compatibility with existing build, release, and editable-install workflows. Its Rust crate is named `arco-python` and is the Python interaction surface; public Python imports stay under `arco`.

Python-facing solve orchestration is routed through the shared `arco-ops` facade where it overlaps with other interaction surfaces. The public Python API is unchanged.

Build and install locally with uv:

```bash
cd bindings/python
uv sync --group dev
uv run --with maturin maturin develop
```

To enable the IPOPT nonlinear backend, build with the `ipopt` feature (requires
a system IPOPT install):

```bash
cd bindings/python
uv run --with maturin maturin develop --features ipopt
```

Run linting:

```bash
uv run ruff check .
uv run ty check .
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
