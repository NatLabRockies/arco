#!/usr/bin/env bash
set -euo pipefail

readonly rust_toolchain_version="${RUST_TOOLCHAIN_VERSION:?RUST_TOOLCHAIN_VERSION is required}"
readonly rust_components="${RUST_COMPONENTS:-}"

rustup toolchain install "${rust_toolchain_version}" --profile minimal

if [[ "${rust_components}" != "none" && -n "${rust_components}" ]]; then
	IFS=',' read -r -a components <<<"${rust_components}"
	for component in "${components[@]}"; do
		component="${component//[[:space:]]/}"
		[[ -n "${component}" ]] || continue
		rustup component add "${component}" --toolchain "${rust_toolchain_version}"
	done
fi

rustup default "${rust_toolchain_version}"
