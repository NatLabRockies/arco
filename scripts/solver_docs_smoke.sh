#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
	echo "usage: $0 <solver-family> [cargo-feature]" >&2
	exit 2
fi

solver_family="$1"
cargo_feature="${2:-}"
model_path="examples/dense-lp/input.kdl"

cargo_args=(cargo +"${RUST_TOOLCHAIN_VERSION:-1.85}" run -p arco-cli)
if [[ -n "$cargo_feature" ]]; then
	cargo_args+=(--features "$cargo_feature")
fi
cargo_args+=(--)

config_dir="$(mktemp -d)"
trap 'rm -rf "$config_dir"' EXIT
export ARCO_CONFIG_DIR="$config_dir"

"${cargo_args[@]}" solver set "$solver_family"
output="$("${cargo_args[@]}" run "$model_path" --compact)"

printf '%s' "$output" | python -m json.tool >/dev/null

if [[ "$solver_family" == "ipopt" ]]; then
	python -c 'import json,sys; p=json.loads(sys.argv[1]); assert p["solve_status"] in {"optimal","feasible"}, p' "$output"
else
	python -c 'import json,sys; p=json.loads(sys.argv[1]); assert p["solve_status"] == "optimal", p' "$output"
fi
