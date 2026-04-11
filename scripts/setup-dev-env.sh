#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

if ! command -v just >/dev/null 2>&1; then
  echo "just >= 1.43.0 is required; install it with: cargo install just --locked --version 1.43.0" >&2
  exit 1
fi

just_version="$(just --version | awk '{print $2}')"
if ! printf '%s\n%s\n' "1.43.0" "$just_version" | sort -V -C; then
  echo "just >= 1.43.0 is required; found $just_version" >&2
  echo "Upgrade with: cargo install just --locked --version 1.43.0" >&2
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

just py-dev
uvx --from "${PREK_SPEC:-prek==0.3.6}" prek install --overwrite --prepare-hooks --hook-type pre-commit --hook-type commit-msg --hook-type pre-push
