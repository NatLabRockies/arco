#!/usr/bin/env bash
set -euo pipefail

dist generate --mode=ci
git diff --exit-code -- .github/workflows/v-release.yml
