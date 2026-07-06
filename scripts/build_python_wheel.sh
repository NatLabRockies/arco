#!/usr/bin/env bash
set -euo pipefail

main() {
	local interpreter="${PYTHON_WHEEL_INTERPRETER:-python3}"
	local features="${PYTHON_WHEEL_FEATURES:-}"
	local build_args=(
		uv run --no-project --with maturin maturin build
		--release
		--manifest-path bindings/python/Cargo.toml
		-i "$interpreter"
		--compatibility pypi
		--out dist
	)

	if [[ -n "$features" ]]; then
		build_args+=(--features "$features")
	fi

	"${build_args[@]}"
}

main "$@"
