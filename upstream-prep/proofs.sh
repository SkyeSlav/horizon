#!/usr/bin/env bash
# Static proofs on the upstream-prep work tree (run from it): nothing kept names a removed path, actionlint on the
# workflows, the Python tests CI runs, which Rust/Cargo files the branch touches, the scrub counts, the repository
# references left, and with CARGO_CHECK=1 a compile-only `cargo check --workspace --all-targets`. Writes proofs.log
# beside this script and exits non-zero on proofs 1, 2, 3, 6 and 7.
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
python3 - "$base" <<'PY' | tee -a "$log"
import subprocess, sys
from pathlib import PurePosixPath
removed = subprocess.run(["git", "diff", "--name-only", "--diff-filter=D", f"{sys.argv[1]}..HEAD"],
                         capture_output=True, text=True, check=True).stdout.split()
generic = {"README.md", "Cargo.toml", "main.rs", "lib.rs", "launcher.svg", "launcher.ico", "launcher.icns",
           "identity.rs"}
generic |= {f"{n}.png" for n in (16, 24, 32, 48, 64, 128, 256, 512)}
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

say "== 2b. built-in addresses (tools/check_addresses.py, as ci.yml verify runs it)"
if python3 tools/check_addresses.py >> "$log" 2>&1; then say "addresses: OK"; else say "addresses: FAILED"; status=1; fi

say "== 3. Python tests (as ci.yml verify runs them), $(python3 --version)"
# The tests make throwaway repositories and commit in them: a plain identity, without this box's commit signing,
# as on CI's runners.
plain_git=$(mktemp); printf '[user]\n\tname = proofs\n\temail = proofs@example.invalid\n' > "$plain_git"
if GIT_CONFIG_GLOBAL=$plain_git python3 -m unittest discover -s tests >> "$log" 2>&1 && \
   GIT_CONFIG_GLOBAL=$plain_git python3 -m unittest discover -s tools/ci -p 'test_prepared_*.py' >> "$log" 2>&1; then
  say "unittest: OK"
else
  say "unittest: FAILED"; status=1
fi
tail -4 "$log" | head -3

say "== 4. Rust, build.rs and Cargo files the branch changes, by commit"
for c in $(git rev-list --reverse "$base"..HEAD); do
  files=$(git diff-tree --no-commit-id --name-status -r "$c" -- '*.rs' '**/Cargo.toml' 'Cargo.toml' '**/Cargo.lock' 'Cargo.lock')
  [ -n "$files" ] && say "-- $(git log -1 --format=%s "$c"): $(echo "$files" | wc -l) files" && echo "$files" >> "$log"
done
true

say "== 5. scrub"
python3 "$here/scrub.py" "$base" HEAD > "$here/scrub.md"
head -14 "$here/scrub.md" | tee -a "$log"

say "== 6. arismoko references left (allowed: the machine sample's repository, one issue or pull request, upstream's own lines)"
python3 - <<'PY' | tee -a "$log"
import os, re, subprocess, sys
out = subprocess.run(["git", "grep", "-n", "-I", "arismoko", "--", ".", ":!third_party"], capture_output=True,
                     text=True).stdout.splitlines()
allowed = re.compile(r"arismoko/horizon-machine-sample|arismoko/horizon#\d|arismoko/horizon/(issues|pull)/\d"
                     r"|users/arismoko/projects|arismoko/(main|topic)\b|arismoko's|\barismoko:|login.*arismoko|@arismoko\b")
upstream_cache = {}
def upstream_has(path, text):
    if path not in upstream_cache:
        upstream_cache[path] = set(subprocess.run(["git", "show", f"{os.environ.get('UPSTREAM_REF', 'upstream/main')}:{path}"], capture_output=True,
                                                  text=True).stdout.splitlines())
    return text in upstream_cache[path]
bad = []
for line in out:
    path, _, rest = line.split(":", 2)
    stripped = re.sub(r"arismoko/horizon-machine-sample|arismoko/horizon#\d+|arismoko/horizon/(issues|pull)/\d+", "", rest)
    if "arismoko" not in stripped or upstream_has(path, rest):
        continue
    if not allowed.search(rest):
        bad.append(line)
for line in bad:
    print(line[:220])
print(f"references: {len(out)} lines; unexpected: {len(bad)}")
sys.exit(1 if bad else 0)
PY
[ "${PIPESTATUS[0]}" -eq 0 ] || status=1

if [ "${CARGO_CHECK:-}" = 1 ]; then
  say "== 7. cargo check --workspace --all-targets (compile only)"
  if CARGO_INCREMENTAL=0 cargo check --workspace --all-targets --locked >> "$log" 2>&1; then say "cargo check: OK"
  else say "cargo check: FAILED"; status=1; fi
  if (cd launcher && cargo check --all-targets --locked >> "$log" 2>&1); then say "cargo check (launcher): OK"
  else say "cargo check (launcher): FAILED"; status=1; fi
fi
exit $status
