#!/usr/bin/env bash
set -euo pipefail

readonly apt_deps_value="${APT_DEPS:?APT_DEPS is required}"

IFS=' ' read -r -a apt_deps <<<"${apt_deps_value}"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends "${apt_deps[@]}"
