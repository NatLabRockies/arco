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

default_cache_root() {
	if [[ -n "${ARCO_SCIP_CACHE_DIR:-}" ]]; then
		printf '%s\n' "$ARCO_SCIP_CACHE_DIR"
	elif [[ -n "${XDG_CACHE_HOME:-}" ]]; then
		printf '%s\n' "$XDG_CACHE_HOME/arco-scip"
	elif [[ -n "${HOME:-}" ]]; then
		printf '%s\n' "$HOME/.cache/arco-scip"
	else
		printf '%s\n' "${RUNNER_TEMP:-/tmp}/arco-scip"
	fi
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

path_for_target() {
	local path="$1"
	local target="$2"

	if [[ "$target" == *-pc-windows-* ]] && command -v cygpath >/dev/null 2>&1; then
		cygpath -m "$path"
	else
		printf '%s\n' "$path"
	fi
}

prepend_runtime_paths() {
	local env_file="$1"
	local paths="$2"
	local github_paths="${3:-}"

	[[ -n "$paths" ]] || return
	append_env "$env_file" "LD_LIBRARY_PATH" "$paths${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
	append_env "$env_file" "DYLD_LIBRARY_PATH" "$paths${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
	append_env "$env_file" "LIBRARY_PATH" "$paths${LIBRARY_PATH:+:$LIBRARY_PATH}"
	if [[ -n "${GITHUB_PATH:-}" ]]; then
		if [[ -n "$github_paths" ]]; then
			printf '%s\n' "$github_paths" >>"$GITHUB_PATH"
		else
			printf '%s\n' "${paths//:/$'\n'}" >>"$GITHUB_PATH"
		fi
	fi
}

fortran_runtime_dir_for_target() {
	local target="$1"
	local candidate=""

	case "$target" in
		*-unknown-linux-gnu) ;;
		*) return 1 ;;
	esac

	if command -v ldconfig >/dev/null 2>&1; then
		candidate="$(ldconfig -p 2>/dev/null | awk '/libgfortran\.so\.5/ { print $NF; exit }')"
		if [[ -n "$candidate" && -f "$candidate" ]]; then
			dirname -- "$candidate"
			return 0
		fi
	fi

	for candidate in \
		/usr/lib/*/libgfortran.so.5 \
		/usr/lib/libgfortran.so.5 \
		/lib/*/libgfortran.so.5 \
		/lib/libgfortran.so.5; do
		if [[ -f "$candidate" ]]; then
			dirname -- "$candidate"
			return 0
		fi
	done

	log "could not find libgfortran.so.5 for $target; install libgfortran5 or gfortran for SCIP-enabled product builds"
	return 1
}

gcc_runtime_dir_for_target() {
	local target="$1"
	local brew_prefix=""
	local candidate
	local library
	local runtime_complete

	case "$target" in
		*-apple-darwin) ;;
		*) return 1 ;;
	esac

	if command -v brew >/dev/null 2>&1; then
		brew_prefix="$(brew --prefix gcc 2>/dev/null || true)"
	fi

	for candidate in \
		"${brew_prefix:+$brew_prefix/lib/gcc/current}" \
		/opt/homebrew/opt/gcc/lib/gcc/current \
		/usr/local/opt/gcc/lib/gcc/current; do
		[[ -n "$candidate" ]] || continue
		runtime_complete=1
		for library in libgcc_s.1.1.dylib libgfortran.5.dylib libquadmath.0.dylib; do
			if [[ ! -f "$candidate/$library" ]]; then
				runtime_complete=0
				break
			fi
		done
		if [[ "$runtime_complete" -eq 1 ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done

	log "could not find Homebrew GCC runtime for $target; SCIP wheel repair requires libgcc_s.1.1.dylib, libgfortran.5.dylib, and libquadmath.0.dylib"
	return 1
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

	local cache_root
	cache_root="$(default_cache_root)"
	local configured=0
	local runtime_paths=""
	local github_runtime_paths=""
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
		local env_install_dir
		env_install_dir="$(path_for_target "$install_dir" "$target")"
		local library_path="$env_install_dir/lib"
		local runtime_library_path="$install_dir/lib"
		append_env "$env_file" "SCIP_SYS_BUNDLED_DIR_${suffix}" "$env_install_dir"
		append_env "$env_file" "ARCO_SCIP_LIBRARY_PATH_${suffix}" "$library_path"
		if [[ "$configured" -eq 0 ]]; then
			append_env "$env_file" "SCIP_SYS_BUNDLED_DIR" "$env_install_dir"
			append_env "$env_file" "ARCO_SCIP_LIBRARY_PATH" "$library_path"
		fi
		runtime_paths="${runtime_paths:+$runtime_paths:}$runtime_library_path"
		github_runtime_paths="${github_runtime_paths:+$github_runtime_paths$'\n'}$library_path"
		if [[ "$target" == *-pc-windows-* && -d "$install_dir/bin" ]]; then
			local runtime_bin_path="$install_dir/bin"
			local bin_path="$env_install_dir/bin"
			runtime_paths="$runtime_paths:$runtime_bin_path"
			github_runtime_paths="$github_runtime_paths"$'\n'"$bin_path"
		fi
		if fortran_runtime_dir="$(fortran_runtime_dir_for_target "$target")"; then
			append_env "$env_file" "ARCO_SCIP_FORTRAN_RUNTIME_PATH_${suffix}" "$fortran_runtime_dir"
			if [[ "$configured" -eq 0 ]]; then
				append_env "$env_file" "ARCO_SCIP_FORTRAN_RUNTIME_PATH" "$fortran_runtime_dir"
			fi
			runtime_paths="${runtime_paths:+$runtime_paths:}$fortran_runtime_dir"
		fi
		if gcc_runtime_dir="$(gcc_runtime_dir_for_target "$target")"; then
			append_env "$env_file" "ARCO_SCIP_GCC_RUNTIME_PATH_${suffix}" "$gcc_runtime_dir"
			if [[ "$configured" -eq 0 ]]; then
				append_env "$env_file" "ARCO_SCIP_GCC_RUNTIME_PATH" "$gcc_runtime_dir"
			fi
			runtime_paths="${runtime_paths:+$runtime_paths:}$gcc_runtime_dir"
		fi
		configured=1
	done

	if [[ "$configured" -eq 0 ]]; then
		log "no supported SCIP prebuilt target found in '$raw_targets'"
	else
		prepend_runtime_paths "$env_file" "$runtime_paths" "$github_runtime_paths"
	fi
}

main "$@"
