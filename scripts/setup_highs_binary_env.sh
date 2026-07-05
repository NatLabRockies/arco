#!/usr/bin/env bash
set -euo pipefail

readonly HIGHS_VERSION="${ARCO_HIGHS_VERSION:-1.15.0}"
readonly HIGHS_LINUX_GLIBC_MIN="${ARCO_HIGHS_LINUX_GLIBC_MIN:-2.38}"

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
	rustc -vV | awk '/^host:/ { print $2; exit }'
}

asset_for_target() {
	local target="$1"
	case "$target" in
		x86_64-unknown-linux-gnu)
			printf 'highs-%s-x86_64-linux-gnu-static-mit.tar.gz\n' "$HIGHS_VERSION"
			;;
		aarch64-unknown-linux-gnu)
			printf 'highs-%s-aarch64-linux-gnu-static-mit.tar.gz\n' "$HIGHS_VERSION"
			;;
		*)
			return 1
			;;
	esac
}

host_glibc_version() {
	if command -v getconf >/dev/null 2>&1; then
		getconf GNU_LIBC_VERSION 2>/dev/null | awk '$1 == "glibc" { print $2; exit }'
	elif command -v ldd >/dev/null 2>&1; then
		ldd --version 2>&1 | awk 'NR == 1 { print $NF; exit }'
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

host_can_link_linux_archive() {
	local target="$1"
	local host
	local glibc_version

	case "$target" in
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
	esac
}

rewrite_pkg_config() {
	local root="$1"
	local pc_file="$root/lib/pkgconfig/highs.pc"
	local python_bin
	python_bin="$(find_python)"

	"$python_bin" - "$pc_file" "$root" <<'PY'
from pathlib import Path
import sys

pc_file = Path(sys.argv[1])
root = sys.argv[2]

rewrites = {
    "prefix": root,
    "libdir": "${prefix}/lib",
    "includedir": "${prefix}/include/highs",
    "Libs": "-L${libdir} -lhighs -lz -lstdc++",
}

lines = []
for line in pc_file.read_text(encoding="utf-8").splitlines():
    key = line.split("=", 1)[0].split(":", 1)[0]
    if key in rewrites:
        separator = ":" if ":" in line.split(" ", 1)[0] else "="
        line = f"{key}{separator} {rewrites[key]}" if separator == ":" else f"{key}={rewrites[key]}"
    lines.append(line)

pc_file.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
}

download_and_extract() {
	local target="$1"
	local asset="$2"
	local root="$3"
	local marker="$root/.arco-highs-complete"
	local url="https://github.com/ERGO-Code/HiGHS/releases/download/v${HIGHS_VERSION}/${asset}"
	local archive="${RUNNER_TEMP:-/tmp}/${asset}"

	if [[ -f "$marker" ]]; then
		log "using cached HiGHS $HIGHS_VERSION for $target at $root"
		return
	fi

	rm -rf "$root"
	mkdir -p "$root"
	log "downloading HiGHS $HIGHS_VERSION for $target"
	curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$archive"
	tar -xzf "$archive" -C "$root"
	rewrite_pkg_config "$root"
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
	local raw_targets="${ARCO_HIGHS_TARGETS:-}"
	if [[ -z "$raw_targets" ]]; then
		raw_targets="$(host_target)"
	fi

	local cache_root="${ARCO_HIGHS_CACHE_DIR:-${RUNNER_TEMP:-/tmp}/arco-highs}"
	local configured=0
	IFS=',' read -r -a targets <<<"$raw_targets"

	for target in "${targets[@]}"; do
		target="${target//[[:space:]]/}"
		[[ -n "$target" ]] || continue

		local asset
		if ! asset="$(asset_for_target "$target")"; then
			log "no official static HiGHS archive configured for $target; using source-build fallback"
			continue
		fi
		if ! host_can_link_linux_archive "$target"; then
			continue
		fi

		local root="$cache_root/$HIGHS_VERSION/$target"
		download_and_extract "$target" "$asset" "$root"

		local pkg_config_dir="$root/lib/pkgconfig"
		local suffix="${target//-/_}"
		append_env "$env_file" "PKG_CONFIG_PATH_${suffix}" "$pkg_config_dir"
		if [[ "$configured" -eq 0 ]]; then
			local current_path="${PKG_CONFIG_PATH:-}"
			if [[ -n "$current_path" ]]; then
				append_env "$env_file" "PKG_CONFIG_PATH" "$pkg_config_dir:$current_path"
			else
				append_env "$env_file" "PKG_CONFIG_PATH" "$pkg_config_dir"
			fi
			append_env "$env_file" "ARCO_HIGHS_ROOT" "$root"
		fi
		configured=1
	done

	if [[ "$configured" -eq 1 ]]; then
		append_env "$env_file" "PKG_CONFIG_ALLOW_CROSS" "1"
	else
		log "no supported HiGHS prebuilt target found in '$raw_targets'"
	fi
}

main "$@"
