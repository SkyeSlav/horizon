#!/usr/bin/env python3
"""Point repository references at horizon-rust/horizon: a mechanical pass over every tracked text file, run from the
repository root.

Rewritten: `arismoko/horizon` in URLs, release and API addresses, the launcher's update source, workflow conditions
and prose; `arismoko/horizon-mixins`; the book's pages address `arismoko.github.io/horizon`.
Left alone, by design:
- `arismoko/horizon-machine-sample`: the example mod's own repository, which has no upstream twin;
- references to one issue or pull request (`arismoko/horizon#N`, `.../arismoko/horizon/issues/N`, `.../pull/N`):
  the same number on horizon-rust/horizon is a different issue, so a rewrite would point at the wrong one;
- a line upstream's own file has word for word (GOVERNANCE.md, CONTRIBUTING.md and docs/layers.md describe the
  fork from upstream's side; the maintainers word those lines);
- third_party/.
"""
import re
import subprocess
import sys
from pathlib import Path

REPOSITORY = re.compile(r"arismoko/horizon(?!-machine-sample)(?!#\d)(?!/(?:issues|pull)/\d)")
PAGES = re.compile(r"arismoko\.github\.io/horizon")


def main() -> int:
    files = subprocess.run(["git", "grep", "-l", "-I", "arismoko", "--", ".", ":!third_party"],
                           capture_output=True, text=True).stdout.split()
    changed = 0
    for name in files:
        path = Path(name)
        original = path.read_text()
        upstream = set(subprocess.run(["git", "show", f"upstream/main:{name}"], capture_output=True,
                                      text=True).stdout.splitlines())
        lines = []
        for line in original.splitlines(keepends=True):
            if line.rstrip("\n") not in upstream:
                line = REPOSITORY.sub("horizon-rust/horizon", PAGES.sub("horizon-rust.github.io/horizon", line))
            lines.append(line)
        text = "".join(lines)
        if text != original:
            path.write_text(text)
            changed += 1
    rust = [f for f in files if f.endswith(".rs") and Path(f).exists()]
    for name in rust:
        edition = "2021" if name.startswith("launcher/") else "2024"
        subprocess.run(["rustfmt", "--edition", edition, name], check=True)
    print(f"rename_repository: {changed} files changed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
