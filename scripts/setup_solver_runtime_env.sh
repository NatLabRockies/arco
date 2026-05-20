#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
	echo "usage: $0 <solver-family> <github-env-file>" >&2
	exit 2
fi

solver_family="$1"
github_env_file="$2"
os_name="$(uname -s)"

append_env() {
	local key="$1"
	local value="$2"
	printf '%s=%s\n' "$key" "$value" >>"$github_env_file"
}

setup_xpress_from_python() {
	local env_file
	env_file="$(mktemp)"
	trap 'rm -f "$env_file"' RETURN

	uv run -p 3.12 --with xpress python - <<'PY' >"$env_file"
import importlib.util
import pathlib

import xpresslibs

root = pathlib.Path(xpresslibs.__file__).resolve().parent
runtime_candidates = [
    root / "lib" / "libxprs.so",
    root / "lib" / "libxprs.dylib",
    root / "bin" / "xprs.dll",
]
runtime_candidates.extend(sorted((root / "lib").glob("libxprs.so.*")))
if not any(candidate.is_file() for candidate in runtime_candidates):
    checked = ", ".join(str(candidate) for candidate in runtime_candidates)
    raise SystemExit(f"missing Xpress runtime library; checked: {checked}")

xpress_spec = importlib.util.find_spec("xpress")
if xpress_spec is None or xpress_spec.origin is None:
    raise SystemExit("missing xpress Python package")
print(f"XPRESSDIR={root}")

xpress_pkg_root = pathlib.Path(xpress_spec.origin).resolve().parent
license_candidates = (
    xpress_pkg_root / "license" / "community-xpauth.xpr",
    root / "license" / "community-xpauth.xpr",
    root / "bin" / "community-xpauth.xpr",
)

for candidate in license_candidates:
    if candidate.is_file():
        print(f"XPAUTH_PATH={candidate}")
        break
PY

	cat "$env_file" >>"$github_env_file"
}

setup_xpress_from_sdk_url() {
	case "$os_name" in
	Linux)
		[[ -n "${XPRESS_SDK_LINUX_URL:-}" ]] || return 1
		curl -fsSL "$XPRESS_SDK_LINUX_URL" -o /tmp/xpress-sdk-linux.tar.gz
		sudo mkdir -p /opt/xpressmp
		sudo tar -xzf /tmp/xpress-sdk-linux.tar.gz -C /opt/xpressmp --strip-components=1
		append_env "XPRESSDIR" "/opt/xpressmp"
		;;
	Darwin)
		[[ -n "${XPRESS_SDK_MACOS_URL:-}" ]] || return 1
		curl -fsSL "$XPRESS_SDK_MACOS_URL" -o /tmp/xpress-sdk-macos.tar.gz
		mkdir -p "$HOME/opt/xpressmp"
		tar -xzf /tmp/xpress-sdk-macos.tar.gz -C "$HOME/opt/xpressmp" --strip-components=1
		append_env "XPRESSDIR" "$HOME/opt/xpressmp"
		;;
	MINGW* | MSYS* | CYGWIN*)
		[[ -n "${XPRESS_SDK_WINDOWS_URL:-}" ]] || return 1
		powershell -NoLogo -NoProfile -Command '
        $ErrorActionPreference = "Stop"
        $archive = "$env:RUNNER_TEMP\\xpress-sdk-windows.zip"
        Invoke-WebRequest -Uri $env:XPRESS_SDK_WINDOWS_URL -OutFile $archive
        $dest = "C:\\xpressmp"
        if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
        New-Item -ItemType Directory -Path $dest | Out-Null
        Expand-Archive -Path $archive -DestinationPath $dest -Force
      '
		append_env "XPRESSDIR" "C:\\xpressmp"
		;;
	*)
		return 1
		;;
	esac
}

case "$solver_family" in
xpress)
	if [[ -n "${XPRESSDIR:-}" ]]; then
		append_env "XPRESSDIR" "$XPRESSDIR"
		exit 0
	fi

	if setup_xpress_from_sdk_url; then
		exit 0
	fi

	setup_xpress_from_python
	;;
highs)
	;;
ipopt)
	;;
*)
	echo "unsupported solver family: $solver_family" >&2
	exit 2
	;;
esac
