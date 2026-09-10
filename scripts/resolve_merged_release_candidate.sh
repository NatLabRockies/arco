#!/usr/bin/env bash
set -euo pipefail

repo=${1:?repository is required}
pr=${2:?pull request number is required}
merge_commit=${3:?merge commit is required}
base_branch=${4:?base branch is required}
output=${5:?output path is required}

pr_file=$(mktemp)
checks_file=$(mktemp)
job_file=$(mktemp)
run_file=$(mktemp)
trap 'rm -f "$pr_file" "$checks_file" "$job_file" "$run_file"' EXIT

gh api "repos/$repo/pulls/$pr" > "$pr_file"
if ! jq -e --arg repo "$repo" --arg branch "$base_branch" --arg merge "$merge_commit" '
  .state == "closed" and .merged == true and .draft == false and
  .merge_commit_sha == $merge and .base.ref == $branch and
  .head.repo.full_name == $repo and .user.login == "github-actions[bot]" and
  (.head.ref | startswith("release-please--branches--")) and
  any(.labels[]; .name == "autorelease: pending")
' "$pr_file" >/dev/null; then
  printf 'release=false\n' >> "$output"
  exit 0
fi

candidate_commit=$(jq -er '.head.sha' "$pr_file")
candidate_base=$(jq -er '.base.sha' "$pr_file")

gh api --method GET "repos/$repo/commits/$candidate_commit/check-runs" \
  -f check_name='Verify and save immutable candidate' \
  -f filter=latest \
  -f per_page=100 > "$checks_file"
# shellcheck disable=SC2016 # jq expands $commit, not Bash.
candidate_filter='map(select(
  .name == "Verify and save immutable candidate" and
  .status == "completed" and .conclusion == "success" and
  .head_sha == $commit and .app.slug == "github-actions"
))'
candidate_count=$(jq -r --arg commit "$candidate_commit" \
  ".check_runs | $candidate_filter | length" "$checks_file")
if [[ $candidate_count -ne 1 ]]; then
  echo "Expected one successful candidate check for release PR #$pr." >&2
  exit 1
fi
candidate_job=$(jq -er --arg commit "$candidate_commit" \
  ".check_runs | $candidate_filter | .[0].id" "$checks_file")

gh api "repos/$repo/actions/jobs/$candidate_job" > "$job_file"
candidate_run=$(jq -er --arg commit "$candidate_commit" '
  select(
    .name == "Verify and save immutable candidate" and
    .status == "completed" and .conclusion == "success" and
    .head_sha == $commit and .workflow_name == "Build candidate for release PR"
  ) | .run_id
' "$job_file")

gh api "repos/$repo/actions/runs/$candidate_run" > "$run_file"
jq -e --arg commit "$candidate_commit" '
  .id > 0 and .event == "pull_request" and
  .status == "completed" and .conclusion == "success" and
  .head_sha == $commit
' "$run_file" >/dev/null
workflow_id=$(jq -er '.workflow_id' "$run_file")
test "$(gh api "repos/$repo/actions/workflows/$workflow_id" --jq .path)" = \
  ".github/workflows/build-candidate.yml"

{
  printf 'release=true\n'
  printf 'candidate_run=%s\n' "$candidate_run"
  printf 'pr=%s\n' "$pr"
  printf 'candidate_commit=%s\n' "$candidate_commit"
  printf 'candidate_base=%s\n' "$candidate_base"
  printf 'base_branch=%s\n' "$base_branch"
  printf 'merge_commit=%s\n' "$merge_commit"
} >> "$output"
