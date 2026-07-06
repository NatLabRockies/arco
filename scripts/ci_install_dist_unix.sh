#!/usr/bin/env bash
set -euo pipefail

readonly cargo_dist_version="${CARGO_DIST_VERSION:?CARGO_DIST_VERSION is required}"

curl --proto '=https' --tlsv1.2 -LsSf "https://github.com/axodotdev/cargo-dist/releases/download/v${cargo_dist_version}/cargo-dist-installer.sh" | sh
