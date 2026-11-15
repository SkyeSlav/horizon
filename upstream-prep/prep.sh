#!/usr/bin/env bash
# Build branch upstream-prep: <base> (default origin/main) + the patches in order + one removal commit per category
# (remove.py) + the steps that turn the fork's identity and choices toward upstream's (align.py, reading
# upstream/main), then the static proofs. Re-runnable: it recreates the branch from the base each time.
#
#   bash prep.sh [base]                 # build in $WORK (default: a worktree beside the checkout), then the proofs
#   PUSH=1 bash prep.sh                 # and force-push upstream-prep
#   CARGO_CHECK=1 bash prep.sh          # and a compile-only cargo check of the workspace and the launcher
#
# A patch that no longer applies stops `git am` (3-way): resolve it in $WORK, `git am --continue`, then run
# `python3 remove.py && python3 align.py && bash proofs.sh` there, and refresh the patch with `git format-patch`.
# An align step whose diff no longer applies stops align.py: see align.py's docstring (report.md, "Rerunning").
# A patch already on the base (the naming commit once it merges into main) is skipped.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=${REPO_DIR:-/box/work/arismoko/horizon}
base=${1:-origin/main}
work=${WORK:-$(dirname "$repo")/horizon-upstream-prep}

git -C "$repo" fetch -q origin main
git -C "$repo" fetch -q upstream main
# The upstream commit the base already contains: the align steps take upstream's files from it, never from a newer
# upstream main the base has not synced (that comes with the sync after the pull request opens).
UPSTREAM_REF=$(git -C "$repo" merge-base "$base" upstream/main)
export UPSTREAM_REF
echo "upstream commit in the base: $UPSTREAM_REF"
# Reuse the work tree when there is one, keeping its target/ (a cargo check stays incremental between runs).
if [ -d "$work/.git" ] || [ -f "$work/.git" ]; then
  git -C "$work" am --abort 2>/dev/null || true
  git -C "$work" checkout -q -f -B upstream-prep "$base"
  git -C "$work" clean -qfdx -e target -e .local
else
  git -C "$repo" worktree add -q -B upstream-prep "$work" "$base"
fi
cd "$work"
for patch in "$here"/patches/*.patch; do
  if git apply --check --reverse "$patch" 2>/dev/null; then
    echo "already on the base, skipped: $(basename "$patch")"
    continue
  fi
  git am -q --3way "$patch"
  echo "applied: $(basename "$patch")"
done
python3 "$here/remove.py"
python3 "$here/align.py"
BASE=$base bash "$here/proofs.sh" || { echo "proofs failed: see $here/proofs.log" >&2; exit 1; }
if [ "${PUSH:-}" = 1 ]; then
  git push --force-with-lease origin upstream-prep
fi
git log --oneline "$base"..HEAD
