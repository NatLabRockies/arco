#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
	echo "usage: $0 <github-env-file>" >&2
	exit 2
fi

bash "$(dirname "$0")/setup_solver_runtime_env.sh" xpress "$1"
