#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
grammar_path="$repo_root/tools/tree-sitter-arco-kdl"

if ! command -v tree-sitter >/dev/null 2>&1; then
  echo "tree-sitter CLI is required for KDL overlay checks." >&2
  exit 1
fi

if [ ! -f "$grammar_path/grammar.js" ]; then
  echo "Missing overlay grammar at $grammar_path" >&2
  exit 1
fi

failures=()
file_count=0
skipped_count=0
while IFS= read -r file; do
  rel="${file#"$repo_root/"}"
  case "$rel" in
    .worktrees/*|*is_rejected*|crates/arco-kdl/tests/fixtures/rejects_*)
      skipped_count=$((skipped_count + 1))
      continue
      ;;
  esac

  file_count=$((file_count + 1))
  parsed="$(tree-sitter parse -p "$grammar_path" "$file" 2>/dev/null || true)"
  if printf '%s' "$parsed" | rg -q '\(ERROR|\(MISSING'; then
    failures+=("$rel")
  fi
done < <(find "$repo_root" -type f -name '*.kdl' -not -path '*/target/*' -not -path '*/.git/*' | sort)

if [ "$file_count" -eq 0 ]; then
  echo "No .kdl files found."
  exit 0
fi

if [ "${#failures[@]}" -gt 0 ]; then
  echo "KDL overlay parse failed." >&2
  echo "checked=${file_count} skipped=${skipped_count} failures=${#failures[@]}" >&2
  printf '  - %s\n' "${failures[@]}" >&2
  exit 1
fi

echo "KDL overlay parse passed."
echo "checked=${file_count} skipped=${skipped_count} failures=0"
