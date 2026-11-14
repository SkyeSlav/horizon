#!/usr/bin/env python3
"""Delete the fork's operations from the current work tree, one commit per category.

Runs closure.py (what nothing kept refers to), sorts every file it would remove into a category below and commits
each category with `git rm`. A file no category claims stops the run before anything is deleted: a sync brought a
new fork-operations file, and someone decides where it goes (add a rule) or that it stays (fix the reference).
"""
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

CATEGORIES = [
    ("Remove the fork's CI gate, runner pool and boat plumbing",
     "The fork's own gate (dyna-ci.yml), its auto-merge guard, the boat and cache workflows, the runner-pool and\n"
     "boat scripts, the gate's step summary and the one-off records of moving tests to the daily run. The checks\n"
     "of the gate that test the product run in ci.yml's verify.",
     [r"^\.github/workflows/(dyna-ci|dyna-automerge-guard|boat-recovery|boat-smoke|ci-cache)\.yml$",
      r"^tools/ci/(boat_\w+|dyna[_-][\w.-]+|pr_gate|gate_summary|check_runner_packages)\.(py|sh|txt)$",
      r"^tools/ci/daily-move-[\w-]+\.tsv$",
      r"^tests/test_(dyna_ci|ci_boat|gate_failures|ci_history_fetch|pr_gate)\.py$"]),
    ("Remove the fork's board, roadmap and Discord automation",
     "The project-board sync, the roadmap progress bars and the rollup and sync posts to the fork's Discord.",
     [r"^\.github/workflows/(board|roadmap-progress|merge-updates)\.yml$",
      r"^\.github/scripts/",
      r"^tools/ci/merge_updates\.py$",
      r"^tests/test_(board_sync|merge_updates)\.py$"]),
    ("Remove the fork's upstream-sync tools and records",
     "The scripts and lists the fork's upstream syncs used, and their decision records. The file-size baseline\n"
     "writer stays: tools/check_file_size.py names it.",
     [r"^tools/sync/", r"^tools/(sync_upstream|convert_sync_fragment)\.py$",
      r"^docs/sync-\d+-decisions\.md$", r"^docs/sync-fragments-r2\.md$",
      r"^tests/test_sync_\w+\.py$"]),
    ("Remove the conflict-measurement scripts",
     "The churn and conflict measurements of the fork's rollups and syncs, and the rollup-aware guard; the\n"
     "export-list, stand-in and wire_loopback checks and carry.py stay.",
     [r"^tools/hotspots/(measure|open_prs|game_rs|check_guards)\.(py|txt)$",
      r"^tests/test_(hotspots_measure|hotspots_open_prs|hotspots_game_rs|hotspot_guards)\.py$"]),
    ("Remove the shm-scene bench scratch crate",
     "A local benchmark crate with absolute paths into one machine; not a workspace member.",
     [r"^tools/shm-scene/"]),
]
# Removed whole whatever refers to them (read and confirmed as the fork's own).
EXTRA = ["tools/shm-scene/"]


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def main() -> int:
    result = json.loads(subprocess.run([sys.executable, HERE / "closure.py"], check=True, capture_output=True,
                                       text=True).stdout)
    files = set(result["remove"])
    tracked = git("ls-files").splitlines()
    files |= {f for f in tracked for p in EXTRA if f.startswith(p)}
    buckets: dict[int, list[str]] = {}
    unclaimed = []
    for f in sorted(files):
        index = next((i for i, (_, _, rules) in enumerate(CATEGORIES) if any(re.search(r, f) for r in rules)), None)
        if index is None:
            unclaimed.append(f)
        else:
            buckets.setdefault(index, []).append(f)
    if unclaimed:
        print("No category claims these files; add a rule or keep them:", *unclaimed, sep="\n  ", file=sys.stderr)
        return 1
    (HERE / "last-removed.json").write_text(json.dumps(
        {CATEGORIES[i][0]: fs for i, fs in sorted(buckets.items())}, indent=1) + "\n")
    (HERE / "last-kept.json").write_text(json.dumps(result["keep"], indent=1) + "\n")
    for i, fs in sorted(buckets.items()):
        title, body, _ = CATEGORIES[i]
        subprocess.run(["git", "rm", "-q", "--", *fs], check=True)
        subprocess.run(["git", "commit", "-q", "-m", f"{title}\n\n{body}"], check=True)
        print(f"{title}: {len(fs)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
