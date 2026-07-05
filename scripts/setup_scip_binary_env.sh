#!/usr/bin/env bash
set -euo pipefail

readonly SCIP_DEPLOY_VERSION="${ARCO_SCIP_DEPLOY_VERSION:-0.12.0}"

usage() {
	printf 'usage: %s <github-env-file>\n' "$0" >&2
}

log() {
	printf 'setup-scip: %s\n' "$*" >&2
}

find_python() {
	if command -v python3 >/dev/null 2>&1; then
		printf 'python3\n'
	elif command -v python >/dev/null 2>&1; then
		printf 'python\n'
	else
		return 1
	fi
}

host_target() {
	rustc -vV | awk '/^host:/ { print $2; found = 1 } END { exit found ? 0 : 1 }'
}

asset_for_target() {
	local target="$1"
	case "$target" in
		x86_64-unknown-linux-gnu)
			printf 'libscip-linux.zip\n'
			;;
		aarch64-unknown-linux-gnu)
			printf 'libscip-linux-arm.zip\n'
			;;
		x86_64-apple-darwin)
			printf 'libscip-macos-intel.zip\n'
			;;
		aarch64-apple-darwin)
			printf 'libscip-macos-arm.zip\n'
			;;
		x86_64-pc-windows-msvc | x86_64-pc-windows-gnu)
			printf 'libscip-windows.zip\n'
			;;
		*)
			return 1
			;;
	esac
}

extract_archive() {
	local archive="$1"
	local install_dir="$2"
	local python_bin
	python_bin="$(find_python)"

	"$python_bin" - "$archive" "$install_dir" <<'PY'
from pathlib import Path
import shutil
import sys
import zipfile

archive = Path(sys.argv[1])
install = Path(sys.argv[2])
staging = install.with_name("scip_extract")

def unpack(zip_path: Path, dest: Path) -> None:
    shutil.rmtree(dest, ignore_errors=True)
    dest.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(zip_path) as zip_file:
        zip_file.extractall(dest)

def entries(path: Path):
    return [entry for entry in path.iterdir() if entry.name != "__MACOSX"]

def normalize(source: Path, dest: Path) -> None:
    shutil.rmtree(dest, ignore_errors=True)
    current_entries = entries(source)
    if len(current_entries) == 1 and current_entries[0].is_dir():
        shutil.move(str(current_entries[0]), dest)
        return

    dest.mkdir(parents=True, exist_ok=True)
    for entry in current_entries:
        shutil.move(str(entry), dest / entry.name)

unpack(archive, staging)
top_entries = entries(staging)
if len(top_entries) == 1 and top_entries[0].suffix.lower() == ".zip":
    nested = staging.with_name(staging.name + "-nested")
    unpack(top_entries[0], nested)
    normalize(nested, install)
else:
    normalize(staging, install)

if not (install / "lib").exists():
    raise SystemExit(f"{install / 'lib'} does not exist after extraction")
if not (install / "include").exists():
    raise SystemExit(f"{install / 'include'} does not exist after extraction")
PY
}

download_and_extract() {
	local target="$1"
	local asset="$2"
	local install_dir="$3"
	local marker="$install_dir/.arco-scip-complete"
	local url="https://github.com/scipopt/scipoptsuite-deploy/releases/download/v${SCIP_DEPLOY_VERSION}/${asset}"
	local archive="${install_dir%/scip_install}/${asset}"

	if [[ -f "$marker" && -d "$install_dir/lib" && -d "$install_dir/include" ]]; then
		log "using cached SCIP deploy $SCIP_DEPLOY_VERSION for $target at $install_dir"
		return
	fi

	rm -rf "$install_dir"
	mkdir -p "${install_dir%/scip_install}"
	log "downloading SCIP deploy $SCIP_DEPLOY_VERSION for $target"
	curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$archive"
	extract_archive "$archive" "$install_dir"
	touch "$marker"
}

append_env() {
	local env_file="$1"
	local name="$2"
	local value="$3"
	printf '%s=%s\n' "$name" "$value" >>"$env_file"
}

main() {
	if [[ $# -ne 1 ]]; then
		usage
		exit 2
	fi

	local env_file="$1"
	local raw_targets="${ARCO_SCIP_TARGETS:-}"
	if [[ -z "$raw_targets" ]]; then
		raw_targets="$(host_target)"
	fi

	local cache_root="${ARCO_SCIP_CACHE_DIR:-${RUNNER_TEMP:-/tmp}/arco-scip}"
	local configured=0
	IFS=',' read -r -a targets <<<"$raw_targets"

	for target in "${targets[@]}"; do
		target="${target//[[:space:]]/}"
		[[ -n "$target" ]] || continue

		local asset
		if ! asset="$(asset_for_target "$target")"; then
			log "no bundled SCIP deploy asset configured for $target"
			continue
		fi

		local install_dir="$cache_root/$SCIP_DEPLOY_VERSION/$target/scip_install"
		download_and_extract "$target" "$asset" "$install_dir"

		local suffix="${target//-/_}"
		append_env "$env_file" "SCIP_SYS_BUNDLED_DIR_${suffix}" "$install_dir"
		if [[ "$configured" -eq 0 ]]; then
			append_env "$env_file" "SCIP_SYS_BUNDLED_DIR" "$install_dir"
		fi
		configured=1
	done

	if [[ "$configured" -eq 0 ]]; then
		log "no supported SCIP prebuilt target found in '$raw_targets'"
	fi
}

main "$@"
