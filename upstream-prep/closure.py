#!/usr/bin/env python3
"""Which fork-operations files the upstream branch can drop: everything in the candidate set that nothing kept
refers to, transitively.

Run in the work tree that is about to lose them (after the workflow and doc patches). Prints JSON:
{"remove": [...], "keep": {path: [referrers...]}}.

Candidates are units: a file, or a folder taken whole (fixture crates, the ignored-tests rows, vendored code).
A unit is kept when a kept file names it: its path, its file name, its `tools.ci` module or its folder. Kept files
are every tracked file outside the removal set; a unit that becomes kept is a referrer from then on. A test file
named for a unit (tests/test_<stem>.py, tools/ci/test_<stem>.py) shares that unit's fate and refers to nothing.
"""
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import PurePosixPath

FORCED = [  # removed whatever refers to them (read and confirmed as the fork's own)
    ".github/workflows/dyna-ci.yml",
    ".github/workflows/dyna-automerge-guard.yml",
    ".github/workflows/boat-recovery.yml",
    ".github/workflows/boat-smoke.yml",
    ".github/workflows/ci-cache.yml",
    ".github/workflows/board.yml",
    ".github/workflows/roadmap-progress.yml",
    ".github/workflows/merge-updates.yml",
    ".github/scripts/",
    # Tests of the removed operations (the CI gate's workflow text, boat, board, sync and conflict-measurement tools).
    "tests/test_dyna_ci.py",
    "tests/test_ci_boat.py",
    "tests/test_gate_failures.py",
    "tests/test_ci_history_fetch.py",
    "tests/test_board_sync.py",
    "tests/test_hotspots_measure.py",
    "tests/test_hotspots_open_prs.py",
    "tests/test_hotspots_game_rs.py",
    "tests/test_hotspot_guards.py",
    "tests/test_sync_lang.py",
    "tests/test_sync_policies.py",
]
CANDIDATE_DIRS = ["tools/ci/", "tools/hotspots/", "tools/sync/"]
# Tests whose subject is not in their name.
TIES = {"tools/ci/test_prepared_cache.py": "tools/ci/vanilla-data.sh",
        "tools/ci/test_prepared_extras.py": "tools/ci/seed_prepared_extras.py"}
CANDIDATE_FILES = ["tools/sync_upstream.py", "tools/convert_sync_fragment.py", "docs/sync-fragments-r2.md"]
CANDIDATE_GLOBS = [re.compile(r"docs/sync-\d+-decisions\.md$")]
WHOLE_DIRS = ["tools/ci/fn-replace-check/", "tools/ci/ignored-tests/", "tools/ci/vendor/", "docs/sync/"]
GENERIC = {"README.md", "Cargo.toml", "Cargo.lock", "lib.rs", "main.rs", "mod.rs"}


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def main() -> int:
    files = git("ls-files").splitlines()
    forced = {f for f in files if any(f == p or (p.endswith("/") and f.startswith(p)) for p in FORCED)}
    units: dict[str, list[str]] = {}  # unit -> files
    for f in files:
        if f in forced:
            continue
        whole = next((d for d in WHOLE_DIRS if f.startswith(d)), None)
        if whole:
            units.setdefault(whole, []).append(f)
        elif (any(f.startswith(d) for d in CANDIDATE_DIRS) or f in CANDIDATE_FILES
              or any(g.match(f) for g in CANDIDATE_GLOBS)):
            units.setdefault(f, []).append(f)
    # Tests named for a unit share its fate.
    stems = {PurePosixPath(u).stem: u for u in units if not u.endswith("/") and not PurePosixPath(u).stem.startswith("test_")}
    tied: dict[str, str] = {t: s for t, s in TIES.items() if t in units and s in units}
    for f in files:
        name = PurePosixPath(f)
        if f in forced or f in tied or name.suffix != ".py" or not name.stem.startswith("test_"):
            continue
        if not (f.startswith("tests/") or f.startswith("tools/ci/")):
            continue
        subject = stems.get(name.stem[len("test_"):])
        if subject and subject != f:
            tied[f] = subject
    for f, subject in tied.items():
        units.pop(f, None)

    def patterns(unit: str) -> list[str]:
        p = PurePosixPath(unit.rstrip("/"))
        out = [unit.rstrip("/")]
        if unit.endswith("/"):
            out.append(p.name + "/")
        elif p.name not in GENERIC:
            out.append(p.name)
            if p.suffix == ".py":
                out += [f"tools.ci import {p.stem}", f"import {p.stem}\n"]
        return out

    pat_owner: dict[str, set[str]] = defaultdict(set)
    for u in units:
        for pat in patterns(u):
            pat_owner[pat].add(u)
    # One pass over the tree: which file names which pattern.
    plain = [p for p in pat_owner if not p.endswith("\n")]
    hits: dict[str, set[str]] = defaultdict(set)  # file -> units it names
    cmd = ["git", "grep", "-I", "-o", "-F", "--full-name"]
    for p in plain:
        cmd += ["-e", p]
    out = subprocess.run(cmd, capture_output=True, text=True).stdout
    for line in out.splitlines():
        path, _, match = line.partition(":")
        for u in pat_owner.get(match, ()):
            hits[path].add(u)
    for p in [p for p in pat_owner if p.endswith("\n")]:
        out = subprocess.run(["git", "grep", "-l", "-E", "--full-name", f"^\\s*import {re.escape(p.strip()[7:])}\\b"],
                             capture_output=True, text=True).stdout
        for path in out.splitlines():
            hits[path].update(pat_owner[p])

    unit_of = {f: u for u, fs in units.items() for f in fs}
    removal = set(forced) | set(unit_of) | set(tied)
    kept_units: dict[str, set[str]] = {}
    referrers = [f for f in files if f not in removal]
    queue = list(referrers)
    while queue:
        src = queue.pop()
        src_unit = unit_of.get(src)
        for u in hits.get(src, ()):
            if u == src_unit:
                continue
            if u not in kept_units:
                kept_units[u] = set()
                queue.extend(units[u])
                queue.extend(t for t, s in tied.items() if s == u)
            kept_units[u].add(src_unit or src)
    removed = sorted(set(forced) | {f for u, fs in units.items() if u not in kept_units for f in fs}
                     | {t for t, s in tied.items() if s not in kept_units})
    json.dump({"remove": removed,
               "keep": {u: sorted(r) for u, r in sorted(kept_units.items())},
               "tied_tests_kept": sorted(t for t, s in tied.items() if s in kept_units)},
              sys.stdout, indent=1)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
