#!/usr/bin/env bash
set -euo pipefail

readonly torc_version="${TORC_VERSION:?TORC_VERSION is required}"
readonly cargo_bin="${HOME:?HOME is required}/.cargo/bin"

if [[ ! -x "${cargo_bin}/torc" || ! -x "${cargo_bin}/torc-server" ]]; then
	mkdir -p "${cargo_bin}"
	curl -fsSL "https://github.com/NatLabRockies/torc/releases/download/v${torc_version}/torc-x86_64-unknown-linux-musl.tar.gz" \
		| tar xz -C "${cargo_bin}" torc torc-server
	chmod +x "${cargo_bin}/torc" "${cargo_bin}/torc-server"
fi

"${cargo_bin}/torc" --version
"${cargo_bin}/torc-server" run --help >/dev/null
