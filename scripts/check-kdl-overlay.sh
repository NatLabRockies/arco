#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
grammar_path="$repo_root/tools/tree-sitter-arco-kdl"

if ! command -v tree-sitter >/dev/null 2>&1; then
  echo "tree-sitter CLI is required for KDL overlay checks." >&2
  exit 1
fi

if [[ ! -f "$grammar_path/grammar.js" ]]; then
  echo "Missing overlay grammar at $grammar_path" >&2
  exit 1
fi

file_count=0
skipped_count=0
while IFS= read -r -d '' rel; do
  case "$rel" in
    .worktrees/*|*is_rejected*|crates/arco-kdl/tests/fixtures/rejects_*)
      skipped_count=$((skipped_count + 1))
      continue
      ;;
  esac

  file_count=$((file_count + 1))
  file="$repo_root/$rel"
  if ! (cd "$grammar_path" && tree-sitter parse --quiet "$file"); then
    printf 'tree-sitter failed for %s\n' "$rel" >&2
    exit 1
  fi
done < <(git -C "$repo_root" ls-files -z -- '*.kdl')

if [[ "$file_count" -eq 0 ]]; then
  echo "No .kdl files found."
  exit 0
fi

echo "KDL overlay parse passed."
echo "checked=${file_count} skipped=${skipped_count} failures=0"
