#!/usr/bin/env bash
# Static proofs on the upstream-prep work tree (run from it): nothing kept names a removed path, actionlint on the
# workflows, the Python tests CI runs, and which Rust/Cargo files the branch touches. Writes proofs.log beside
# this script and exits non-zero on the first three.
set -uo pipefail
here=$(cd "$(dirname "$0")" && pwd)
base=${BASE:-origin/main}
log="$here/proofs.log"
# CI's Python is 3.12 (ci.yml); some tests use 3.12 APIs (tarfile's extraction filter). Prefer one from uv, and
# actionlint from ~/.local/bin.
export PATH="$HOME/.local/bin:$PATH"
if py=$(uv python find 3.12 2>/dev/null); then
  shim=$(mktemp -d); ln -s "$py" "$shim/python3"; ln -s "$py" "$shim/python"; export PATH="$shim:$PATH"
fi
: > "$log"
status=0
say() { echo "$*" | tee -a "$log"; }

say "== branch $(git rev-parse --short HEAD) on base $(git rev-parse --short "$base")"

say "== 1. kept files that name a removed path"
python3 - "$here/last-removed.json" <<'PY' | tee -a "$log"
import json, subprocess, sys
from pathlib import PurePosixPath
removed = [f for fs in json.load(open(sys.argv[1])).values() for f in fs]
generic = {"README.md", "Cargo.toml", "main.rs", "lib.rs"}
patterns = set()
for f in removed:
    patterns.add(f)
    name = PurePosixPath(f).name
    if name not in generic:
        patterns.add(name)
patterns |= {"tools/sync/audit_launcher_restoration", "dyna-ci.yml", "dyna-checks", "dyna_ci_runner", "dyna_ci_deps",
             "pr_gate.py", "boat_health", "merge_updates.py", "tools/shm-scene"}
args = ["git", "grep", "-I", "-n", "-F"]
for p in sorted(patterns):
    args += ["-e", p]
out = subprocess.run(args + ["--", "."], capture_output=True, text=True).stdout.splitlines()
# tools/refactors/ is kept whole: its one-off scripts record the workflows they once edited.
history = [line for line in out if line.startswith("tools/refactors/")]
hits = [line for line in out if not line.startswith("tools/refactors/")]
for line in history:
    print("accepted (tools/refactors history):", line[:200])
for line in hits:
    print(line[:220])
print(f"hits: {len(hits)}")
sys.exit(1 if hits else 0)
PY
[ "${PIPESTATUS[0]}" -eq 0 ] || status=1

say "== 2. actionlint"
if actionlint .github/workflows/*.yml >> "$log" 2>&1; then say "actionlint: clean"; else say "actionlint: FAILED"; status=1; fi

say "== 3. Python tests (as ci.yml verify runs them), $(python3 --version)"
if python3 -m unittest discover -s tests >> "$log" 2>&1 && \
   python3 -m unittest discover -s tools/ci -p 'test_prepared_*.py' >> "$log" 2>&1; then
  say "unittest: OK"
else
  say "unittest: FAILED"; status=1
fi
tail -4 "$log" | head -3

say "== 4. Rust, build.rs and Cargo files the branch changes (expected: only the naming commit and shm-scene)"
git diff --name-status "$base"..HEAD -- '*.rs' '**/Cargo.toml' 'Cargo.toml' '**/Cargo.lock' 'Cargo.lock' | tee -a "$log"

say "== 5. scrub"
python3 "$here/scrub.py" "$base" HEAD > "$here/scrub.md"
head -14 "$here/scrub.md" | tee -a "$log"
exit $status
