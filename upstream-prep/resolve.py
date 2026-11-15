#!/usr/bin/env python3
"""Resolve every conflict in FILE to one side: resolve.py FILE ours|theirs (diff3 base sections are dropped)."""
import sys
from pathlib import Path

path, side = Path(sys.argv[1]), sys.argv[2]
out, state, count = [], None, 0
for line in path.read_text().splitlines(keepends=True):
    if line.startswith("<<<<<<< "):
        state, count = "ours", count + 1
    elif line.startswith("||||||| ") and state == "ours":
        state = "base"
    elif line.startswith("=======") and state in ("ours", "base"):
        state = "theirs"
    elif line.startswith(">>>>>>> ") and state == "theirs":
        state = None
    elif state is None or state == side:
        out.append(line)
path.write_text("".join(out))
print(f"{path} {count} conflicts")
