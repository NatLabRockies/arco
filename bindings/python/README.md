# Arco Python bindings

The Python package remains in `bindings/python` for path compatibility with existing build, release, and editable-install workflows. Its Rust crate is named `arco-python` and is the Python interaction surface; public Python imports stay under `arco`.

Python-facing solve orchestration is routed through the shared `arco-ops` facade where it overlaps with other interaction surfaces. The public Python API is unchanged.

Build and install locally with uv:

```bash
cd bindings/python
uv sync --group dev
uv run --with maturin maturin develop
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
