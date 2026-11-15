#!/usr/bin/env python3
"""Drop the named files' sections from a git diff, in place: `drop_from_diff.py DIFF PATH...`."""
import re
import sys
from pathlib import Path

diff, drop = Path(sys.argv[1]), set(sys.argv[2:])
parts = re.split(r"(?m)^(?=diff --git )", diff.read_text())
kept = [p for p in parts if not (m := re.match(r"diff --git a/(\S+) b/", p)) or m.group(1) not in drop]
diff.write_text("".join(kept))
print(f"{diff.name}: dropped {len(parts) - len(kept)} sections")
