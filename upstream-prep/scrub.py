#!/usr/bin/env python3
"""Count the scrub terms in two trees and list what remains in the second.

    python3 scrub.py <before-ref> <after-ref>     # e.g. origin/main HEAD

Third-party code (third_party/) is left out: its DYNAMIC_* constants are not ours.
"""
import re
import subprocess
import sys

TERMS = {
    "portizon": r"[Pp]ortizon|PORTIZON",
    "/box/ paths": r"/box/",
    "ses_ ids": r"ses_[0-9A-Z]{20,}",
    "room_ ids": r"room_[0-9A-Z]{20,}",
    "hermeslab": r"hermeslab",
    "ovh-rise5": r"ovh-rise5",
    "arch as a host": r"\barch(linux)?\b(\.lan|-host| host)|host[ =:]+\"?arch\b",
    "Dyna (word)": r"\bDyna\b",
    "dyna (identifiers: dyna-ci, dyna_ci, DYNA_, dyna-large...)": r"\bdyna\b|dyna-|dyna_|DYNA_",
    "agent swarm disclosure": r"agent swarm",
    "rollup": r"\b[Rr]ollups?\b",
}


def hits(ref: str, pattern: str) -> list[str]:
    out = subprocess.run(["git", "grep", "-I", "-n", "-E", pattern, ref, "--", ".", ":!third_party"],
                         capture_output=True, text=True).stdout
    return [line.split(":", 1)[1] if line.startswith(ref + ":") else line for line in out.splitlines()]


def main() -> int:
    before, after = sys.argv[1], sys.argv[2]
    print("| Term | Before (lines / files) | After (lines / files) |\n| --- | --- | --- |")
    remaining = {}
    for name, pattern in TERMS.items():
        b, a = hits(before, pattern), hits(after, pattern)
        files = lambda xs: len({x.split(":", 1)[0] for x in xs})
        print(f"| {name} | {len(b)} / {files(b)} | {len(a)} / {files(a)} |")
        remaining[name] = a
    for name, lines in remaining.items():
        if not lines or name == "rollup":
            continue
        print(f"\n### Remaining: {name}\n")
        by_file = {}
        for line in lines:
            path = line.split(":", 1)[0]
            by_file[path] = by_file.get(path, 0) + 1
        for path, count in sorted(by_file.items()):
            print(f"- `{path}` ({count})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
