#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
	echo "usage: $0 <github-env-file>" >&2
	exit 2
fi

github_env_file="$1"

if command -v dnf >/dev/null 2>&1; then
	package_manager=dnf
elif command -v yum >/dev/null 2>&1; then
	package_manager=yum
else
	echo "Could not find dnf or yum to install libclang" >&2
	exit 1
fi

install_cmd=("$package_manager" install -y clang-devel cmake)
if [[ "$(id -u)" -ne 0 ]]; then
	install_cmd=(sudo "${install_cmd[@]}")
fi
"${install_cmd[@]}"

libclang_path="$(find /usr /opt -name 'libclang.so*' -type f 2>/dev/null | sort -V | tail -n 1)"
if [[ -z "$libclang_path" ]]; then
	echo "Could not find libclang after installing clang-devel" >&2
	exit 1
fi

scratch_dir="${RUNNER_TEMP:-/tmp}"
toolchain_file="$scratch_dir/arco-manylinux-toolchain.cmake"
cat >"$toolchain_file" <<'EOF'
set(CMAKE_INSTALL_LIBDIR lib CACHE PATH "Install libraries under lib for Rust sys crates" FORCE)
EOF

{
	echo "LIBCLANG_PATH=$(dirname "$libclang_path")"
	echo "CMAKE_TOOLCHAIN_FILE_x86_64_unknown_linux_gnu=$toolchain_file"
} >>"$github_env_file"
