#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
artifact_dir="$repo_root/target/editor-artifacts"
tree_sitter_dir="$repo_root/tools/tree-sitter-arco-kdl"
nvim_dir="$repo_root/tools/arco-kdl-nvim"
vscode_dir="$repo_root/tools/vscode-arco-kdl"
nvim_name="arco-kdl-nvim"
nvim_version="0.1.0"
nvim_stage="$artifact_dir/$nvim_name"
nvim_archive="$artifact_dir/$nvim_name-$nvim_version.tar.gz"
vsix_name="arco-kdl-vscode-0.1.0.vsix"

mkdir -p "$artifact_dir"

mkdir -p "$nvim_dir/src/tree_sitter" "$nvim_dir/queries/arco_kdl"
cp "$tree_sitter_dir/src/parser.c" "$nvim_dir/src/parser.c"
cp "$tree_sitter_dir/src/scanner.c" "$nvim_dir/src/scanner.c"
cp "$tree_sitter_dir/src/tree_sitter/parser.h" "$nvim_dir/src/tree_sitter/parser.h"
cp "$tree_sitter_dir/queries/highlights.scm" "$nvim_dir/queries/arco_kdl/highlights.scm"
cp "$tree_sitter_dir/queries/injections.scm" "$nvim_dir/queries/arco_kdl/injections.scm"

rm -rf "$nvim_stage"
mkdir -p "$nvim_stage"
cp -R "$nvim_dir/." "$nvim_stage/"

tar -C "$artifact_dir" -czf "$nvim_archive" "$nvim_name"

(
  cd "$vscode_dir"
  npm run check
  npm run package
  cp "$vsix_name" "$artifact_dir/$vsix_name"
)

echo "Created editor artifacts:"
echo "  $nvim_archive"
echo "  $artifact_dir/$vsix_name"
