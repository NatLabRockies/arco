#!/usr/bin/env bash
set -euo pipefail

if command -v cargo >/dev/null 2>&1; then
	exit 0
fi

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
printf '%s\n' "${HOME:?HOME is required}/.cargo/bin" >>"${GITHUB_PATH:?GITHUB_PATH is required}"
