#!/usr/bin/env bash
set -euo pipefail

readonly github_output="${GITHUB_OUTPUT:?GITHUB_OUTPUT is required}"
readonly manifest_path="plan-dist-manifest.json"

dist plan --output-format=json >"${manifest_path}"
printf 'artifacts_matrix=%s\n' "$(jq -c '.ci.github.artifacts_matrix' "${manifest_path}")" >>"${github_output}"
