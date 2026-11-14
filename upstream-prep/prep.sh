#!/usr/bin/env bash
# Build branch upstream-prep: <base> (default origin/main) + the patches in order + one removal commit per category,
# then the static proofs. Re-runnable: it recreates the branch from the base each time.
#
#   bash prep.sh [base]                 # build in $WORK (default: a worktree beside the checkout), then the proofs
#   PUSH=1 bash prep.sh                 # and force-push upstream-prep
#
# A patch that no longer applies stops `git am` (3-way): resolve it in $WORK, `git am --continue`, then run
# `python3 remove.py && bash proofs.sh` there, and refresh the patch with `git format-patch` (see report.md).
# A patch already on the base (the naming commit once it merges into main) is skipped.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=${REPO_DIR:-/box/work/arismoko/horizon}
base=${1:-origin/main}
work=${WORK:-$(dirname "$repo")/horizon-upstream-prep}

git -C "$repo" fetch -q origin main
git -C "$repo" worktree remove --force "$work" 2>/dev/null || true
git -C "$repo" worktree add -q -B upstream-prep "$work" "$base"
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
BASE=$base bash "$here/proofs.sh" || { echo "proofs failed: see $here/proofs.log" >&2; exit 1; }
if [ "${PUSH:-}" = 1 ]; then
  git push --force-with-lease origin upstream-prep
fi
git log --oneline "$base"..HEAD
