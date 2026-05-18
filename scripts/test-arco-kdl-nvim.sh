#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
plugin_dir="$repo_root/tools/arco-kdl-nvim"
test_file="$plugin_dir/test/smoke.lua"

if ! command -v nvim >/dev/null 2>&1; then
  echo "nvim is required to test tools/arco-kdl-nvim" >&2
  exit 1
fi

ARCO_KDL_NVIM_PLUGIN_DIR="$plugin_dir" nvim --headless -u NONE -i NONE \
  +"set runtimepath^=$plugin_dir" \
  +"luafile $test_file" \
  +qa
