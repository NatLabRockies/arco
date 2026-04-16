# Arco Python bindings

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
