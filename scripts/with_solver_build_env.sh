#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s <command> [args...]\n' "$0" >&2
}

if [[ $# -eq 0 ]]; then
	usage
	exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
env_file="$(mktemp "${TMPDIR:-/tmp}/arco-solver-build-env.XXXXXX")"

cleanup() {
	rm -f "$env_file"
}
trap cleanup EXIT

"$script_dir/setup_highs_binary_env.sh" "$env_file"
"$script_dir/setup_scip_binary_env.sh" "$env_file"

while IFS= read -r line || [[ -n "$line" ]]; do
	[[ -n "$line" ]] || continue
	case "$line" in
		*=*) ;;
		*) continue ;;
	esac

	name="${line%%=*}"
	value="${line#*=}"
	if [[ "$name" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
		export "$name=$value"
	fi
done <"$env_file"

exec "$@"
