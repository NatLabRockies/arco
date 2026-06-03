#!/usr/bin/env bash
set -euo pipefail

summary_file="$RUNNER_TEMP/ci-plan-summary.md"
any_enabled=false
enabled_buckets=""
skipped_buckets=""

{
  echo '## CI Plan'
  echo ''
  echo '| Bucket | Status |'
  echo '|---|---|'
  for bucket in RUST PYTHON DOCS SOLVER KDL BENCHMARKS RELEASE ACTIONS_CONFIG; do
    label="$bucket"
    if [ "$bucket" = "ACTIONS_CONFIG" ]; then
      label="GITHUB_ACTIONS"
    fi
    val="${!bucket:-false}"
    if [ "$val" = "true" ]; then
      echo "| ${label} | :white_check_mark: enabled |"
      any_enabled=true
      enabled_buckets="${enabled_buckets}${label}, "
    else
      echo "| ${label} | :x: skipped |"
      skipped_buckets="${skipped_buckets}${label}, "
    fi
  done
  echo ''
  echo '| Job | Decision | Reason |'
  echo '|---|---|---|'
  if [ "$RELEASE" = "true" ] || [ "$ACTIONS_CONFIG" = "true" ]; then
    echo '| cargo-dist workflow | :white_check_mark: run | release or GitHub Actions inputs changed |'
  else
    echo '| cargo-dist workflow | :x: skip | release and GitHub Actions inputs unchanged |'
  fi
  if [ "$RUST" = "true" ]; then
    echo '| Rust format check | :white_check_mark: run | Rust bucket enabled |'
    echo '| Rust clippy (all-features) | :white_check_mark: run | Rust bucket enabled |'
    echo '| Rust test (all-features) | :white_check_mark: run | Rust bucket enabled |'
    echo '| Arco CLI build | :white_check_mark: run | Rust bucket enabled |'
  else
    echo '| Rust format check | :x: skip | Rust bucket disabled |'
    echo '| Rust clippy (all-features) | :x: skip | Rust bucket disabled |'
    echo '| Rust test (all-features) | :x: skip | Rust bucket disabled |'
    echo '| Arco CLI build | :x: skip | Rust bucket disabled |'
  fi
  if [ "$PYTHON" = "true" ] || [ "$DOCS" = "true" ]; then
    reasons=""
    [ "$PYTHON" = "true" ] && reasons="Python bucket"
    [ "$DOCS" = "true" ] && reasons="${reasons:+${reasons}, }Docs bucket"
    echo "| Python validation | :white_check_mark: run | ${reasons} enabled |"
  else
    echo '| Python validation | :x: skip | Python bucket disabled |'
  fi
  if [ "$DOCS" = "true" ]; then
    echo '| Docs doctests | :white_check_mark: run | Inside python-validation |'
  else
    echo '| Docs doctests | :x: skip | Docs bucket disabled |'
  fi
  if [ "$KDL" = "true" ]; then
    echo '| KDL examples e2e | :white_check_mark: run | KDL bucket enabled |'
  else
    echo '| KDL examples e2e | :x: skip | KDL bucket disabled |'
  fi
  if [ "$BENCHMARKS" = "true" ] || [ "$RUST" = "true" ]; then
    echo '| Benchmarks | :white_check_mark: run | benchmarks or Rust bucket enabled |'
  else
    echo '| Benchmarks | :x: skip | benchmarks and Rust buckets disabled |'
  fi
  echo ''
  if [ "$any_enabled" != "true" ]; then
    echo ':warning: No CI buckets were activated - all expensive jobs will skip.'
  fi
} > "$summary_file"

cat "$summary_file" >> "$GITHUB_STEP_SUMMARY"
echo '::group::CI plan'
cat "$summary_file"
echo '::endgroup::'

enabled_buckets="${enabled_buckets%, }"
skipped_buckets="${skipped_buckets%, }"
[ -n "$enabled_buckets" ] || enabled_buckets="none"
[ -n "$skipped_buckets" ] || skipped_buckets="none"
echo "::notice title=CI plan::enabled=${enabled_buckets}; skipped=${skipped_buckets}"
