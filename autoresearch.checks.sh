#!/usr/bin/env bash
set -euo pipefail

just fmt-check clippy-all test-core docs-test
cargo check -p arco-python --no-default-features
