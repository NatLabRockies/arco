#!/usr/bin/env bash
set -euo pipefail

readonly HIGHS_VERSION="${ARCO_HIGHS_VERSION:-1.15.0}"
readonly HIGHS_LINUX_GLIBC_MIN="${ARCO_HIGHS_LINUX_GLIBC_MIN:-2.38}"
readonly HIGHS_SOURCE_SHA256="${ARCO_HIGHS_SOURCE_SHA256:-c3fc3e9ee43e6d562361f8647b4c69f958c95356a1af8bc5a3647f5882230d44}"

usage() {
	printf 'usage: %s <github-env-file>\n' "$0" >&2
}

log() {
	printf 'setup-highs: %s\n' "$*" >&2
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
	if [[ -n "${ARCO_HIGHS_CACHE_DIR:-}" ]]; then
		printf '%s\n' "$ARCO_HIGHS_CACHE_DIR"
	elif [[ -n "${XDG_CACHE_HOME:-}" ]]; then
		printf '%s\n' "$XDG_CACHE_HOME/arco-highs"
	elif [[ -n "${HOME:-}" ]]; then
		printf '%s\n' "$HOME/.cache/arco-highs"
	else
		printf '%s\n' "${RUNNER_TEMP:-/tmp}/arco-highs"
	fi
}

asset_for_target() {
	local target="$1"
	case "$target" in
		aarch64-apple-darwin)
			if [[ "${ARCO_HIGHS_ENABLE_APPLE_STATIC:-0}" != "1" ]]; then
				log "official HiGHS macOS static archive discovery is opt-in; set ARCO_HIGHS_ENABLE_APPLE_STATIC=1 to use it"
				return 1
			fi
			printf 'highs-%s-arm-apple-static-mit.tar.gz\n' "$HIGHS_VERSION"
			;;
		x86_64-unknown-linux-gnu)
			printf 'highs-%s-x86_64-linux-gnu-static-mit.tar.gz\n' "$HIGHS_VERSION"
			;;
		aarch64-unknown-linux-gnu)
			printf 'highs-%s-aarch64-linux-gnu-static-mit.tar.gz\n' "$HIGHS_VERSION"
			;;
		x86_64-pc-windows-msvc)
			if [[ "${ARCO_HIGHS_ENABLE_WINDOWS_STATIC:-0}" != "1" ]]; then
				log "official HiGHS Windows static archive discovery is opt-in; set ARCO_HIGHS_ENABLE_WINDOWS_STATIC=1 only for a compatible MSVC toolchain"
				return 1
			fi
			printf 'highs-%s-x86_64-windows-static-mit.zip\n' "$HIGHS_VERSION"
			;;
		*)
			return 1
			;;
	esac
}

host_jobs() {
	if [[ -n "${ARCO_HIGHS_BUILD_JOBS:-}" ]]; then
		printf '%s\n' "$ARCO_HIGHS_BUILD_JOBS"
	elif command -v nproc >/dev/null 2>&1; then
		nproc
	elif command -v sysctl >/dev/null 2>&1; then
		sysctl -n hw.ncpu 2>/dev/null || printf '4\n'
	else
		printf '4\n'
	fi
}

host_glibc_version() {
	if command -v getconf >/dev/null 2>&1; then
		getconf GNU_LIBC_VERSION 2>/dev/null | awk '$1 == "glibc" { print $2 }'
	elif command -v ldd >/dev/null 2>&1; then
		ldd --version 2>&1 | awk 'NR == 1 { version = $NF } END { print version }'
	fi
}

version_at_least() {
	local current="$1"
	local minimum="$2"
	awk -v current="$current" -v minimum="$minimum" '
		BEGIN {
			split(current, current_parts, ".")
			split(minimum, minimum_parts, ".")
			for (i = 1; i <= 3; i++) {
				current_part = current_parts[i] + 0
				minimum_part = minimum_parts[i] + 0
				if (current_part > minimum_part) {
					exit 0
				}
				if (current_part < minimum_part) {
					exit 1
				}
			}
			exit 0
		}
	'
}

host_can_link_zlib() {
	if ! command -v cc >/dev/null 2>&1; then
		log "could not find cc for zlib link check; using source-build fallback"
		return 1
	fi

	local source_file
	source_file="$(mktemp "${RUNNER_TEMP:-/tmp}/arco-highs-zlib.XXXXXX.c")"
	local output_file="${source_file%.c}"
	printf 'int main(void) { return 0; }\n' >"$source_file"
	if cc "$source_file" -lz -o "$output_file" >/dev/null 2>&1; then
		rm -f "$source_file" "$output_file"
		return 0
	fi

	rm -f "$source_file" "$output_file"
	log "could not link zlib required by official HiGHS $HIGHS_VERSION static archives; using source-build fallback"
	return 1
}

host_can_link_archive() {
	local target="$1"
	local host
	local glibc_version

	case "$target" in
		*-apple-darwin)
			host="$(host_target 2>/dev/null || true)"
			if [[ "$target" != "$host" ]]; then
				log "official HiGHS static archive discovery is only enabled for native macOS targets; host is '${host:-unknown}', target is $target, using source-build fallback"
				return 1
			fi
			;;
		*-unknown-linux-gnu)
			host="$(host_target 2>/dev/null || true)"
			if [[ "$target" != "$host" ]]; then
				log "official HiGHS static archive discovery is only enabled for native Linux targets; host is '${host:-unknown}', target is $target, using source-build fallback"
				return 1
			fi
			glibc_version="$(host_glibc_version)"
			if [[ -z "$glibc_version" ]]; then
				log "could not detect host glibc version for $target; using source-build fallback"
				return 1
			fi
			if ! version_at_least "$glibc_version" "$HIGHS_LINUX_GLIBC_MIN"; then
				log "official HiGHS $HIGHS_VERSION static archives require glibc >= $HIGHS_LINUX_GLIBC_MIN; host has $glibc_version for $target, using source-build fallback"
				return 1
			fi
			if ! host_can_link_zlib; then
				return 1
			fi
			;;
		*-pc-windows-msvc)
			host="$(host_target 2>/dev/null || true)"
			if [[ "$target" != "$host" ]]; then
				log "official HiGHS static archive discovery is only enabled for native Windows targets; host is '${host:-unknown}', target is $target, using source-build fallback"
				return 1
			fi
			;;
	esac
}

host_can_build_source_cache() {
	local target="$1"
	local host

	if [[ "${ARCO_HIGHS_ENABLE_SOURCE_CACHE:-1}" != "1" ]]; then
		log "HiGHS source cache disabled; set ARCO_HIGHS_ENABLE_SOURCE_CACHE=1 to use it"
		return 1
	fi

	host="$(host_target 2>/dev/null || true)"
	if [[ "$target" != "$host" ]]; then
		log "HiGHS source cache is only enabled for native targets; host is '${host:-unknown}', target is $target"
		return 1
	fi

	if ! command -v cmake >/dev/null 2>&1; then
		log "could not find cmake for HiGHS source cache; using highs-sys source-build fallback"
		return 1
	fi

	case "$target" in
		*-pc-windows-msvc)
			return 0
			;;
		*-apple-darwin | *-unknown-linux-gnu)
			for program in cc c++; do
				if ! command -v "$program" >/dev/null 2>&1; then
					log "could not find $program for HiGHS source cache; using highs-sys source-build fallback"
					return 1
				fi
			done
			;;
		*)
			return 1
			;;
	esac
}

rewrite_pkg_config() {
	local root="$1"
	local target="$2"
	local link_zlib="${3:-1}"
	local link_extras="${4:-0}"
	local pc_file="$root/lib/pkgconfig/highs.pc"
	local rewrite_root
	local python_bin
	rewrite_root="$(path_for_target "$root" "$target")"
	python_bin="$(find_python)"

	"$python_bin" - "$pc_file" "$rewrite_root" "$target" "$link_zlib" "$link_extras" <<'PY'
from pathlib import Path
import sys

pc_file = Path(sys.argv[1])
root = sys.argv[2]
target = sys.argv[3]
link_zlib = sys.argv[4] == "1"
link_extras = sys.argv[5] == "1"

if "apple-darwin" in target:
    libs = "-L${libdir} -lhighs"
    if link_extras:
        libs += " -lhighs_extras"
    if link_zlib:
        libs += " -lz"
    libs += " -lc++"
elif "pc-windows-msvc" in target:
    libs = "-L${libdir} -lhighs"
    if link_extras:
        libs += " -lhighs_extras"
else:
    libs = "-L${libdir} -lhighs"
    if link_extras:
        libs += " -lhighs_extras"
    if link_zlib:
        libs += " -lz"
    libs += " -lstdc++"

rewrites = {
    "prefix": root,
    "libdir": "${prefix}/lib",
    "includedir": "${prefix}/include/highs",
    "Libs": libs,
}

lines = []
for line in pc_file.read_text(encoding="utf-8").splitlines():
    key = line.split("=", 1)[0].split(":", 1)[0]
    if key in rewrites:
        line = f"{key}: {rewrites[key]}" if key == "Libs" else f"{key}={rewrites[key]}"
    lines.append(line)

pc_file.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
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

extract_archive() {
	local archive="$1"
	local root="$2"
	local python_bin

	case "$archive" in
		*.tar.gz)
			tar -xzf "$archive" -C "$root"
			;;
		*.zip)
			python_bin="$(find_python)"
			"$python_bin" - "$archive" "$root" <<'PY'
from pathlib import Path
import sys
import zipfile

archive = Path(sys.argv[1])
root = Path(sys.argv[2])
with zipfile.ZipFile(archive) as zip_file:
    zip_file.extractall(root)
PY
			;;
		*)
			log "unsupported HiGHS archive format: $archive"
			return 1
			;;
	esac
}

link_zlib_for_target() {
	local target="$1"

	case "$target" in
		*-pc-windows-msvc)
			printf '0\n'
			;;
		*)
			printf '1\n'
			;;
	esac
}

sanitize_cache_component() {
	local value="$1"

	printf '%s' "$value" | tr -c 'A-Za-z0-9._-' '-'
}

msvc_toolset_cache_component() {
	local program
	local output
	local version

	if [[ -n "${VCToolsVersion:-}" ]]; then
		sanitize_cache_component "$VCToolsVersion"
		return
	fi

	for program in link.exe cl.exe; do
		if ! command -v "$program" >/dev/null 2>&1; then
			continue
		fi

		output="$("$program" 2>&1 || true)"
		version="$(
			printf '%s\n' "$output" |
				awk '/Version/ {
					for (i = 1; i <= NF; i++) {
						if ($i ~ /^[0-9]+([.][0-9]+)+$/) {
							print $i
							exit
						}
					}
				}'
		)"
		if [[ -n "$version" ]]; then
			sanitize_cache_component "$version"
			return
		fi
	done

	printf 'unknown\n'
}

source_cache_dirname() {
	local target="$1"

	case "$target" in
		*-pc-windows-msvc)
			printf '%s-source-release-msvc-%s\n' "$target" "$(msvc_toolset_cache_component)"
			;;
		*)
			printf '%s-source\n' "$target"
			;;
	esac
}

download_and_extract() {
	local target="$1"
	local asset="$2"
	local root="$3"
	local marker="$root/.arco-highs-complete"
	local url="https://github.com/ERGO-Code/HiGHS/releases/download/v${HIGHS_VERSION}/${asset}"
	local archive_dir
	local archive
	local link_zlib
	archive_dir="$(dirname -- "$root")"
	archive="$archive_dir/$asset"

	if [[ -f "$marker" ]]; then
		log "using cached HiGHS $HIGHS_VERSION for $target at $root"
		link_zlib="$(link_zlib_for_target "$target")"
		rewrite_pkg_config "$root" "$target" "$link_zlib" 0
		return
	fi

	rm -rf "$root"
	mkdir -p "$root" "$archive_dir"
	log "downloading HiGHS $HIGHS_VERSION for $target"
	curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$archive"
	extract_archive "$archive" "$root"
	link_zlib="$(link_zlib_for_target "$target")"
	rewrite_pkg_config "$root" "$target" "$link_zlib" 0
	touch "$marker"
}

sha256_file() {
	local path="$1"

	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$path" | awk '{ print $1 }'
	elif command -v shasum >/dev/null 2>&1; then
		shasum -a 256 "$path" | awk '{ print $1 }'
	else
		log "could not find sha256sum or shasum to verify HiGHS source"
		return 1
	fi
}

download_source_archive() {
	local archive="$1"
	local actual_sha

	if [[ ! -f "$archive" ]]; then
		log "downloading HiGHS $HIGHS_VERSION source"
		curl --proto '=https' --tlsv1.2 -fsSL \
			"https://github.com/ERGO-Code/HiGHS/archive/refs/tags/v${HIGHS_VERSION}.tar.gz" \
			-o "$archive"
	fi

	actual_sha="$(sha256_file "$archive")"
	if [[ "$actual_sha" != "$HIGHS_SOURCE_SHA256" ]]; then
		rm -f "$archive"
		log "HiGHS source checksum mismatch: expected $HIGHS_SOURCE_SHA256, got $actual_sha"
		return 1
	fi
}

build_source_cache() {
	local target="$1"
	local root="$2"
	local marker="$root/.arco-highs-source-complete"
	local parent
	local archive
	local source_dir
	local build_dir
	local install_dir
	local jobs
	local cmake_args

	if [[ -f "$marker" && -f "$root/lib/pkgconfig/highs.pc" && ( -f "$root/lib/libhighs.a" || -f "$root/lib/highs.lib" ) ]]; then
		log "using cached source-built HiGHS $HIGHS_VERSION for $target at $root"
		rewrite_pkg_config "$root" "$target" 0 1
		return
	fi

	parent="$(dirname -- "$root")"
	mkdir -p "$parent"
	archive="$parent/highs-${HIGHS_VERSION}-source.tar.gz"
	source_dir="$(mktemp -d "$parent/source.XXXXXX")"
	build_dir="$(mktemp -d "$parent/build.XXXXXX")"
	install_dir="$(mktemp -d "$parent/install.XXXXXX")"
	jobs="$(host_jobs)"

	download_source_archive "$archive"
	tar -xzf "$archive" -C "$source_dir" --strip-components 1

	cmake_args=(
		-S "$source_dir"
		-B "$build_dir"
		-DCMAKE_BUILD_TYPE=Release
		-DCMAKE_INSTALL_PREFIX="$install_dir"
		-DBUILD_CXX_EXE=OFF
		-DBUILD_EXAMPLES=OFF
		-DBUILD_SHARED_EXTRAS_LIB=OFF
		-DFAST_BUILD=ON
		-DBUILD_SHARED_LIBS=OFF
		-DCMAKE_INTERPROCEDURAL_OPTIMIZATION=FALSE
		-DCMAKE_INSTALL_LIBDIR=lib
		-DZLIB=OFF
		-DCMAKE_INSTALL_DOCDIR=
	)
	if [[ "$target" == *-apple-darwin ]]; then
		cmake_args+=(
			-DCMAKE_OSX_DEPLOYMENT_TARGET="${ARCO_HIGHS_MACOS_DEPLOYMENT_TARGET:-${MACOSX_DEPLOYMENT_TARGET:-11.0}}"
		)
	fi

	log "building source HiGHS $HIGHS_VERSION for $target"
	cmake "${cmake_args[@]}" >/dev/null
	cmake --build "$build_dir" --config Release --target install --parallel "$jobs" >/dev/null

	rm -rf "$root"
	mv "$install_dir" "$root"
	rewrite_pkg_config "$root" "$target" 0 1
	touch "$marker"
	rm -rf "$source_dir" "$build_dir"
}

append_env() {
	local env_file="$1"
	local name="$2"
	local value="$3"
	printf '%s=%s\n' "$name" "$value" >>"$env_file"
}

append_pkg_config_env() {
	local env_file="$1"
	local target="$2"
	local root="$3"
	local env_root
	local pkg_config_dir
	local suffix="${target//-/_}"
	env_root="$(path_for_target "$root" "$target")"
	pkg_config_dir="$env_root/lib/pkgconfig"

	append_env "$env_file" "PKG_CONFIG_PATH_${suffix}" "$pkg_config_dir"
	if [[ "$configured" -eq 0 ]]; then
		local current_path="${PKG_CONFIG_PATH:-}"
		if [[ -n "$current_path" ]]; then
			append_env "$env_file" "PKG_CONFIG_PATH" "$pkg_config_dir:$current_path"
		else
			append_env "$env_file" "PKG_CONFIG_PATH" "$pkg_config_dir"
		fi
		append_env "$env_file" "ARCO_HIGHS_ROOT" "$env_root"
	fi
	configured=1
}

main() {
	if [[ $# -ne 1 ]]; then
		usage
		exit 2
	fi

	local env_file="$1"
	local raw_targets="${ARCO_HIGHS_TARGETS:-}"
	if [[ -z "$raw_targets" ]]; then
		raw_targets="$(host_target)"
	fi

	local cache_root
	local configured=0
	cache_root="$(default_cache_root)"
	IFS=',' read -r -a targets <<<"$raw_targets"

	for target in "${targets[@]}"; do
		target="${target//[[:space:]]/}"
		[[ -n "$target" ]] || continue

		local asset
		if asset="$(asset_for_target "$target")" && host_can_link_archive "$target"; then
			local root="$cache_root/$HIGHS_VERSION/$target"
			download_and_extract "$target" "$asset" "$root"
			append_pkg_config_env "$env_file" "$target" "$root"
			continue
		elif [[ -z "${asset:-}" ]]; then
			log "no official static HiGHS archive configured for $target; using source-build fallback"
		fi

		if host_can_build_source_cache "$target"; then
			local source_cache_name
			local root
			source_cache_name="$(source_cache_dirname "$target")"
			root="$cache_root/$HIGHS_VERSION/$source_cache_name"
			build_source_cache "$target" "$root"
			append_pkg_config_env "$env_file" "$target" "$root"
			continue
		fi

		log "no supported HiGHS prebuilt or source-cache target found for $target"
	done

	if [[ "$configured" -eq 1 ]]; then
		append_env "$env_file" "PKG_CONFIG_ALLOW_CROSS" "1"
	else
		log "no supported HiGHS prebuilt target found in '$raw_targets'"
	fi
}

main "$@"
