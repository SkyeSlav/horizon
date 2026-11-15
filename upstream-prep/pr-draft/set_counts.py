#!/usr/bin/env python3
"""Update the contributor and commit counts in README.md in place (the draft repo's README is the source).

Usage: set_counts.py PEOPLE COMMITS   e.g. set_counts.py 49 10848
"""
import re
import sys
from pathlib import Path

people, commits = int(sys.argv[1]), int(sys.argv[2])
p = Path(__file__).with_name("README.md")
t = p.read_text()
t, a = re.subn(r"\b\d+ of you have commits in here", f"{people} of you have commits in here", t)
t, b = re.subn(r'alt="\d+ people and [\d,]+ commits', f'alt="{people} people and {commits:,} commits', t)
assert a == 1 and b == 1, (a, b)
p.write_text(t)
print(f"README.md: {people} people, {commits:,} commits")
