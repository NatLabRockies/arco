#!/usr/bin/env bash
set -euo pipefail

log() {
	printf 'install-macos-scip-runtime: %s\n' "$*" >&2
}

runtime_dir_for_gcc() {
	local brew_prefix
	brew_prefix="$(brew --prefix gcc 2>/dev/null || true)"

	if [[ -n "$brew_prefix" ]]; then
		printf '%s\n' "$brew_prefix/lib/gcc/current"
	else
		printf '%s\n' "$(brew --prefix)/opt/gcc/lib/gcc/current"
	fi
}

runtime_is_complete() {
	local runtime_dir="$1"
	local library

	[[ -d "$runtime_dir" ]] || return 1
	for library in libgcc_s.1.1.dylib libgfortran.5.dylib libquadmath.0.dylib; do
		[[ -f "$runtime_dir/$library" ]] || return 1
	done
}

main() {
	if [[ "$(uname -s)" != "Darwin" ]]; then
		log "not running on macOS; skipping"
		return
	fi

	if ! command -v brew >/dev/null 2>&1; then
		log "Homebrew is required to install the SCIP GCC runtime"
		return 1
	fi

	local runtime_dir
	runtime_dir="$(runtime_dir_for_gcc)"
	if runtime_is_complete "$runtime_dir"; then
		log "using Homebrew GCC runtime at $runtime_dir"
		return
	fi

	log "installing Homebrew GCC runtime"
	brew install gcc

	runtime_dir="$(runtime_dir_for_gcc)"
	if ! runtime_is_complete "$runtime_dir"; then
		log "Homebrew GCC runtime is incomplete at $runtime_dir"
		return 1
	fi
	log "using Homebrew GCC runtime at $runtime_dir"
}

main "$@"
