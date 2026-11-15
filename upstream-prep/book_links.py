#!/usr/bin/env python3
"""Point the modding book's links at its Markdown in this repository: the book is read in docs/modding, not
published to GitHub Pages (maintainer's answer in arismoko/horizon#4498). Run from the repository root, after
rename_repository.py.

- `horizon-rust.github.io/horizon/<page>.html[#anchor]` becomes the page's source,
  `github.com/horizon-rust/horizon/blob/main/docs/modding/src/<page>.md[#anchor]`;
- the book's root becomes its introduction page;
- the two chapters an mdBook preprocessor renders from the hook catalog (hook notes, NeoForge counterparts) have
  no source page: they point at the catalog, crates/horizon-mod-vocab/src/hooks;
- the rustdoc reference (`/api/mod_api/`) points at the crate; `cargo doc -p horizon-mod-api --open` builds it.
The mod registry's own site (horizon-rust.github.io/horizon-mods) is upstream's and stays.
"""
import re
import subprocess
import sys
from pathlib import Path

BLOB = "https://github.com/horizon-rust/horizon/blob/main/"
TREE = "https://github.com/horizon-rust/horizon/tree/main/"
GENERATED = {"reference/hook-notes", "reference/neoforge-counterparts"}
LINK = re.compile(r"https://horizon-rust\.github\.io/horizon(?![-\w])(/[\w/.-]*)?(#[\w-]+)?")


def target(match: re.Match) -> str:
    path, anchor = (match.group(1) or "/").strip("/"), match.group(2) or ""
    if path == "":
        return BLOB + "docs/modding/src/introduction.md"
    if path.startswith("api/"):
        return TREE + "crates/horizon-mod-api"
    page = path.removesuffix(".html")
    if page in GENERATED:
        return TREE + "crates/horizon-mod-vocab/src/hooks"
    if not Path("docs/modding/src", page + ".md").is_file():
        raise SystemExit(f"book_links: no source page for {match.group(0)}")
    return BLOB + f"docs/modding/src/{page}.md{anchor}"


def main() -> int:
    files = subprocess.run(["git", "grep", "-l", "-I", "horizon-rust.github.io/horizon", "--", ".", ":!third_party"],
                           capture_output=True, text=True).stdout.split()
    changed = 0
    for name in files:
        path = Path(name)
        text = path.read_text()
        new = LINK.sub(target, text)
        if new != text:
            path.write_text(new)
            changed += 1
    print(f"book_links: {changed} files changed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
