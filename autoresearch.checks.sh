#!/usr/bin/env bash
set -euo pipefail

just ci
cargo check -p arco-python --no-default-features
uv run --project bindings/python --with pytest --with numpy pytest bindings/python/tests/test_arco_stub_operators.py -q
