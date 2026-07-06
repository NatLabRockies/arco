#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "${HOME:?HOME is required}/.cargo/bin" >>"${GITHUB_PATH:?GITHUB_PATH is required}"
