#!/usr/bin/env bash
set -euo pipefail

dist generate --mode=ci
python3 scripts/ci_bundle_scip_runtime.py workflow .github/workflows/v-release.yml
git diff --exit-code -- .github/workflows/v-release.yml
