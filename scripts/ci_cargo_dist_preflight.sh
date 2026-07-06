#!/usr/bin/env bash
set -euo pipefail

readonly expected_container_image="quay.io/pypa/manylinux_2_28_x86_64"
readonly dist_plan_args="${DIST_PLAN_ARGS:?DIST_PLAN_ARGS is required}"
readonly dist_targets="${DIST_TARGETS:?DIST_TARGETS is required}"
readonly dist_container_image="${DIST_CONTAINER_IMAGE:-}"

validate_container_image() {
	if [[ -z "${dist_container_image}" ]]; then
		return
	fi

	if [[ "${dist_container_image}" != "${expected_container_image}" ]]; then
		printf '::error::Update the pinned cargo-dist preflight container for %s.\n' "${dist_container_image}" >&2
		exit 1
	fi
}

append_target_arg() {
	local target="$1"
	case "${target}" in
		aarch64-apple-darwin | aarch64-unknown-linux-gnu | x86_64-apple-darwin | x86_64-unknown-linux-gnu | x86_64-pc-windows-msvc)
			dist_args+=("--target=${target}")
			;;
		*)
			printf '::error::Unexpected cargo-dist preflight target: %s\n' "${target}" >&2
			exit 1
			;;
	esac
}

validate_container_image

IFS=',' read -r -a targets <<<"${dist_targets}"
dist_args=(--artifacts=local)
for target in "${targets[@]}"; do
	target="${target//[[:space:]]/}"
	[[ -n "${target}" ]] || continue
	append_target_arg "${target}"
done

expected_args="${dist_args[*]}"
if [[ "${dist_plan_args}" != "${expected_args}" ]]; then
	printf '::error::Unexpected cargo-dist args: %s\n' "${dist_plan_args}" >&2
	printf 'Expected: %s\n' "${expected_args}" >&2
	exit 1
fi

dist build --print=linkage --output-format=json "${dist_args[@]}" >dist-manifest.json
dist print-upload-files-from-manifest --manifest dist-manifest.json
