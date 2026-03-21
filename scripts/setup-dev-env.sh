#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

if ! command -v just >/dev/null 2>&1; then
  echo "just is required" >&2
  exit 1
fi

if ! command -v uvx >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -LsSf https://astral.sh/uv/install.sh | sh
fi

if ! command -v uv >/dev/null 2>&1 || ! command -v uvx >/dev/null 2>&1; then
  echo "uv and uvx are required" >&2
  exit 1
fi

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export UV_CACHE_DIR="${UV_CACHE_DIR:-$repo_root/.uv-cache}"
cd "$repo_root"

(cd bindings/python && uv sync)
uv run python scripts/sync_python_licenses.py
(cd bindings/python && uv run --with maturin maturin develop)
uvx --from "${PREK_SPEC:-prek==0.3.6}" prek install --overwrite --prepare-hooks --hook-type pre-commit --hook-type commit-msg --hook-type pre-push
