#!/usr/bin/env bash
set -euo pipefail

just ci
cargo check -p arco-python --no-default-features
