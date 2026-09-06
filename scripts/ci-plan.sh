#!/usr/bin/env bash
set -euo pipefail

summary_file="${RUNNER_TEMP}/ci-plan-summary.md"

bucket_enabled() {
  local bucket_name="$1"
  local value="${!bucket_name:-false}"
  [[ "$value" == "true" ]]
}

append_bucket_name() {
  local list="$1"
  local label="$2"
  if [[ -n "$list" ]]; then
    printf '%s, %s' "$list" "$label"
  else
    printf '%s' "$label"
  fi
}

enabled_buckets=""
skipped_buckets=""
for bucket in RUST PYTHON PYTHON_COMPAT DOCS SOLVER KDL BENCHMARKS VSCODE_EXTENSION; do
  label="$bucket"
  if [[ "$bucket" == "VSCODE_EXTENSION" ]]; then
    label="VS_CODE_EXTENSION"
  fi

  if bucket_enabled "$bucket"; then
    enabled_buckets="$(append_bucket_name "$enabled_buckets" "$label")"
  else
    skipped_buckets="$(append_bucket_name "$skipped_buckets" "$label")"
  fi
done

enabled_buckets="${enabled_buckets:-none}"
skipped_buckets="${skipped_buckets:-none}"

job_decision() {
  local job="$1"
  local should_run="$2"
  local run_reason="$3"
  local skip_reason="$4"

  if [[ "$should_run" == "true" ]]; then
    printf '| %s | :white_check_mark: run | %s |\n' "$job" "$run_reason"
  else
    printf '| %s | :x: skip | %s |\n' "$job" "$skip_reason"
  fi
}

rust_enabled="false"
bucket_enabled RUST && rust_enabled="true"
python_or_docs_enabled="false"
if bucket_enabled PYTHON || bucket_enabled PYTHON_COMPAT || bucket_enabled DOCS; then
  python_or_docs_enabled="true"
fi
python_compat_enabled="false"
bucket_enabled PYTHON_COMPAT && python_compat_enabled="true"
docs_enabled="false"
bucket_enabled DOCS && docs_enabled="true"
kdl_enabled="false"
bucket_enabled KDL && kdl_enabled="true"
benchmarks_or_rust_enabled="false"
if bucket_enabled BENCHMARKS || bucket_enabled RUST; then
  benchmarks_or_rust_enabled="true"
fi
vscode_extension_enabled="false"
bucket_enabled VSCODE_EXTENSION && vscode_extension_enabled="true"
cli_build_enabled="false"
if bucket_enabled RUST || bucket_enabled SOLVER || bucket_enabled KDL || bucket_enabled BENCHMARKS; then
  cli_build_enabled="true"
fi
solver_enabled="false"
if bucket_enabled RUST || bucket_enabled SOLVER; then
  solver_enabled="true"
fi

{
  echo '## CI Plan'
  echo ''
  echo "Enabled buckets: ${enabled_buckets}"
  echo "Skipped buckets: ${skipped_buckets}"
  echo ''
  echo '| Job | Decision | Reason |'
  echo '|---|---|---|'
  job_decision 'VS Code extension' "$vscode_extension_enabled" 'VS Code extension inputs changed' 'VS Code extension inputs unchanged'
  job_decision 'Rust format check' "$rust_enabled" 'Rust bucket enabled' 'Rust bucket disabled'
  job_decision 'Rust clippy (all-features)' "$rust_enabled" 'Rust bucket enabled' 'Rust bucket disabled'
  job_decision 'Rust test (all-features)' "$rust_enabled" 'Rust bucket enabled' 'Rust bucket disabled'
  job_decision 'Arco CLI build' "$cli_build_enabled" 'Rust, solver, KDL, or benchmarks bucket enabled' 'Rust, solver, KDL, and benchmarks buckets disabled'
  job_decision 'Solver smoke' "$solver_enabled" 'Rust or solver bucket enabled' 'Rust and solver buckets disabled'
  job_decision 'Python validation' "$python_or_docs_enabled" 'Python, compatibility, or docs bucket enabled' 'Python, compatibility, and docs buckets disabled'
  job_decision 'Python supported-version smoke' "$python_compat_enabled" 'Python packaging or release inputs changed' 'Python packaging and release inputs unchanged'
  job_decision 'Docs doctests' "$docs_enabled" 'Docs bucket enabled inside Python validation' 'Docs bucket disabled'
  job_decision 'KDL examples e2e' "$kdl_enabled" 'KDL bucket enabled after CLI build artifact' 'KDL bucket disabled'
  job_decision 'Benchmarks' "$benchmarks_or_rust_enabled" 'benchmarks or Rust bucket enabled' 'benchmarks and Rust buckets disabled'
  echo ''
  if [[ "$enabled_buckets" == "none" ]]; then
    echo ':warning: No CI buckets were activated. All expensive jobs will skip.'
  fi
} > "$summary_file"

cat "$summary_file" >> "$GITHUB_STEP_SUMMARY"
printf 'CI plan: enabled=%s; skipped=%s\n' "$enabled_buckets" "$skipped_buckets"
