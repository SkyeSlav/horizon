#!/usr/bin/env python3
"""Write the fork's bare issue references as full links: `#N` becomes `arismoko/horizon#N`, so that once this tree
is upstream's a reference never points at upstream's issue of the same number (maintainer's answer in
arismoko/horizon#4498). Run from the repository root.

Only the fork's own lines change: the lines `git diff -M <upstream>` adds (UPSTREAM_REF, the upstream commit the
base contains), so upstream's own `#N` stay as they are. In those lines:
- `#N` with 1 <= N <= the fork's newest number (MAX_ISSUE), not already part of `owner/repo#N`, a URL, an anchor
  or a word (`(?<![\\w/&#.-])`);
- in Rust only comments (`//`, `///`, `//!`), in Python only comments and docstrings, in shell scripts and
  workflows only comments, never string literals, so test data, log patterns and messages keep their text;
- other text files whole; JSON, CSS, SVG, WGSL, HTML and lock files are skipped.
In roadmap/roadmap.toml a link becomes the issue's URL, the form tools/roadmap.py accepts for another repository.
Writes issue-refs-review.tsv beside this script (file, line, number, text) for the hand pass.
"""
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
UPSTREAM = os.environ.get("UPSTREAM_REF", "upstream/main")
MAX_ISSUE = int(os.environ.get("MAX_ISSUE", "4800"))
SKIP = {".json", ".css", ".svg", ".wgsl", ".html", ".lock", ".png", ".jpg", ".ico", ".icns", ".jsonl", ".snap"}
REF = re.compile(r"(?<![\w/&#.\-])#([1-9]\d{0,3})\b")
# Generated from roadmap/roadmap.toml by tools/roadmap.py build (align.py runs it after this script).
GENERATED = {"ROADMAP.md", "roadmap/roadmap.json"}
# In the roadmap's `links`, an issue is "#N" (this repository's) or a URL: a fork issue becomes its URL.
ROADMAP_LINK = re.compile(r'"arismoko/horizon#(\d+)"')
HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@")


def added_lines() -> dict[str, set[int]]:
    diff = subprocess.run(["git", "diff", "-M", "-U0", "--no-color", "--diff-filter=AMR", UPSTREAM, "HEAD", "--",
                           ".", ":!third_party"], capture_output=True, text=True, check=True).stdout
    added: dict[str, set[int]] = {}
    path = None
    for line in diff.splitlines():
        if line.startswith("+++ "):
            path = line[6:] if line.startswith("+++ b/") else None
        elif path and (m := HUNK.match(line)):
            start, count = int(m.group(1)), int(m.group(2) or "1")
            added.setdefault(path, set()).update(range(start, start + count))
    return added


def code_split(line: str, marker: str) -> int | None:
    """Where a comment starting with `marker` begins outside a string literal, or None."""
    quote = None
    i = 0
    while i < len(line):
        c = line[i]
        if quote:
            if c == "\\":
                i += 2
                continue
            if c == quote:
                quote = None
        elif c in "\"'" and not (marker == "//" and c == "'"):  # a Rust ' is a lifetime or char
            quote = c
        elif line.startswith(marker, i):
            return i
        i += 1
    return None


def rewrite(text: str, path: str, lines: set[int], review: list) -> str:
    out = []
    suffix = Path(path).suffix
    in_string = None  # inside a Python triple-quoted string: "doc" (a docstring) or "data"
    for number, line in enumerate(text.splitlines(keepends=True), 1):
        start = 0
        if suffix == ".rs":
            start = code_split(line, "//")
        elif suffix in {".sh", ".yml", ".yaml"}:
            start = code_split(line, "#")
        elif suffix == ".py":
            quotes = line.count('"""') + line.count("'''")
            if in_string:
                start = 0 if in_string == "doc" else None
                if quotes % 2:
                    in_string = None
            elif quotes % 2:
                in_string = "doc" if line.lstrip().startswith(('"""', "'''", 'r"""')) else "data"
                start = 0 if in_string == "doc" else code_split(line, "#")
            elif line.lstrip().startswith(('"""', "'''")):
                start = 0
            else:
                start = code_split(line, "#")
        if number in lines and start is not None:
            head, tail = line[:start], line[start:]
            new = REF.sub(lambda m: f"arismoko/horizon#{m.group(1)}" if int(m.group(1)) <= MAX_ISSUE else m.group(0),
                          tail)
            if new != tail:
                for m in REF.finditer(tail):
                    if int(m.group(1)) <= MAX_ISSUE:
                        review.append((path, number, m.group(1), line.strip()[:160]))
                line = head + new
        out.append(line)
    return "".join(out)


def main() -> int:
    review: list = []
    changed = 0
    for path, lines in sorted(added_lines().items()):
        file = Path(path)
        if file.suffix in SKIP or path in GENERATED or not file.is_file():
            continue
        try:
            text = file.read_text()
        except UnicodeDecodeError:
            continue
        new = rewrite(text, path, lines, review)
        if path == "roadmap/roadmap.toml":
            new = ROADMAP_LINK.sub(r'"https://github.com/arismoko/horizon/issues/\1"', new)
        if new != text:
            file.write_text(new)
            changed += 1
    (HERE / "issue-refs-review.tsv").write_text("".join(f"{p}\t{n}\t#{i}\t{t}\n" for p, n, i, t in review))
    print(f"rewrite_issue_refs: {len(review)} references in {changed} files (review list: issue-refs-review.tsv)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
