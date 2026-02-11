# Arco Python bindings

Build and install locally with uv:

```bash
cd bindings/python
uv sync --group dev
uv run maturin develop
```

Run linting:

```bash
uv run ruff check .
uv run ty check .
```

Run examples:

```bash
uv run python examples/simple_lp.py
```
