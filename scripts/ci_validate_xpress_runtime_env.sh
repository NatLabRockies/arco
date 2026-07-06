#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${XPRESSDIR:-}" ]]; then
	printf 'XPRESSDIR is unset. Configure XPRESS_SDK_*_URL repository variables for this runner OS.\n' >&2
	exit 1
fi
