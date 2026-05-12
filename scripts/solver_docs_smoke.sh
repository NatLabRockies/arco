#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
	echo "usage: $0 <solver-family> [cargo-feature]" >&2
	exit 2
fi

solver_family="$1"
cargo_feature="${2:-}"
model_path="examples/dense-lp/input.kdl"
python_bin="${PYTHON:-python3}"

if [[ "$solver_family" == "ipopt" ]]; then
	echo "Skipping IPOPT docs smoke: the IPOPT model-view adapter is intentionally not implemented yet." >&2
	exit 0
fi

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

printf '%s' "$output" | "$python_bin" -m json.tool >/dev/null
"$python_bin" -c 'import json,sys; p=json.loads(sys.argv[1]); assert p["solve_status"] == "optimal", p' "$output"
