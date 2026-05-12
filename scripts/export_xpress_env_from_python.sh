#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
	echo "usage: $0 <github-env-file>" >&2
	exit 2
fi

github_env_file="$1"

env_file="$(mktemp)"
trap 'rm -f "$env_file"' EXIT

uv run -p 3.12 --with xpress python - <<'PY' >"$env_file"
import pathlib
import xpresslibs

root = pathlib.Path(xpresslibs.__file__).resolve().parent
lib_dir = root / "lib"
if not lib_dir.is_dir():
    raise SystemExit(f"missing Xpress lib dir: {lib_dir}")

print(f"XPRESSDIR={root}")
PY

cat "$env_file" >>"$github_env_file"
