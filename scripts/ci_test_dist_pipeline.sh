#!/usr/bin/env bash
set -euo pipefail

readonly dist_bin="${1:-dist}"
release_tag="${2:-v$(cargo metadata --no-deps --format-version=1 | jq --raw-output '.packages[] | select(.name == "arco-cli") | .version')}"
readonly release_tag
repo_root="$(git rev-parse --show-toplevel)"
readonly repo_root
scratch_dir="$(mktemp -d)"
readonly scratch_dir
readonly plan_manifest="${scratch_dir}/dist-manifest.json"

cleanup() {
    local exit_code=$?
    rm -rf -- "${scratch_dir}"
    exit "${exit_code}"
}
trap cleanup EXIT

cd "${repo_root}"

"${dist_bin}" plan --tag="${release_tag}" --output-format=json > "${plan_manifest}"

jq --exit-status '
    .ci.github.artifacts_matrix.include
    | map(.targets[])
    | sort
    == [
        "aarch64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-gnu"
    ]
' "${plan_manifest}" >/dev/null

jq --exit-status '
    .artifacts
    | to_entries
    | map(select(.value.kind == "executable-zip") | .key)
    | sort
    == [
        "arco-cli-aarch64-apple-darwin.tar.gz",
        "arco-cli-aarch64-unknown-linux-gnu.tar.gz",
        "arco-cli-x86_64-apple-darwin.tar.gz",
        "arco-cli-x86_64-pc-windows-msvc.zip",
        "arco-cli-x86_64-unknown-linux-gnu.tar.gz"
    ]
' "${plan_manifest}" >/dev/null

jq --exit-status '
    (.artifacts["arco-cli-installer.sh"].kind == "installer") and
    (.artifacts["arco-cli-installer.ps1"].kind == "installer") and
    (.artifacts["sha256.sum"].kind == "unified-checksum") and
    ([.artifacts | to_entries[] | select(.value.kind == "checksum")] | length == 6)
' "${plan_manifest}" >/dev/null

grep -Fq 'allow-dirty = ["ci"]' dist-workspace.toml
# GitHub evaluates this expression; the shell must compare it literally.
# shellcheck disable=SC2016
test "$(grep -Fc 'ref: ${{ inputs.source-sha }}' .github/workflows/cargo-dist-build.yaml)" -eq 3
grep -Fq 'name: cargo-dist-local-' .github/workflows/cargo-dist-build.yaml
grep -Fq 'name: release-artifacts-cargo-dist-global' .github/workflows/cargo-dist-build.yaml
grep -Fq 'cp dist-manifest.json target/distrib/dist-manifest.json' .github/workflows/cargo-dist-build.yaml
# The workflow's Bash shell expands this expression at runtime.
# shellcheck disable=SC2016
grep -Fq '"${packaged_binaries[0]}" --version' .github/workflows/cargo-dist-build.yaml

printf 'cargo-dist pipeline contract is valid for %s\n' "${release_tag}"
