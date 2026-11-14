#!/usr/bin/env python3
"""Build only upstream's launcher identity: a mechanical pass over launcher/, run from the repository root.

- `if cfg!(horizon_fork_launcher) { A } else { B }` becomes B (an expression stays bare; statements keep a block),
  and an `if cfg!(horizon_fork_launcher) { A }` without else is dropped.
- Every `identity::NAME` (launcher/src/identity.rs's table) becomes upstream's literal, `owns_launcher_data_dir(x)`
  becomes upstream's `named(x, "horizon-launcher")`, and identity.rs goes.
- rustfmt (edition 2021) on every file it changed.
Prints the places that kept a block, for the hand pass after it.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path("launcher")
SWITCH = "if cfg!(horizon_fork_launcher)"
# Upstream's literal where the identity table's text needs a raw string, as upstream wrote it.
ABOUT = '"https://playhorizon.dev"'


def skip_literal(text: str, i: int) -> int:
    """The index after the string or char literal starting at i, or i when there is none."""
    m = re.match(r'(b?r)(#*)"', text[i:])
    if m:
        end = text.index('"' + m.group(2), i + m.end())
        return end + 1 + len(m.group(2))
    if text[i] == '"' or text.startswith('b"', i):
        j = i + (2 if text[i] == "b" else 1)
        while text[j] != '"':
            j += 2 if text[j] == "\\" else 1
        return j + 1
    m = re.match(r"'(\\.|[^\\'])'", text[i:])
    if m:
        return i + m.end()
    return i


def matching(text: str, open_at: int) -> int:
    """Index of the brace closing the one at open_at."""
    depth, i = 0, open_at
    while True:
        j = skip_literal(text, i)
        if j != i:
            i = j
            continue
        if text.startswith("//", i):
            i = text.index("\n", i)
            continue
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1


def top_level_statements(body: str) -> bool:
    depth, i = 0, 0
    while i < len(body):
        j = skip_literal(body, i)
        if j != i:
            i = j
            continue
        c = body[i]
        if c in "{([":
            depth += 1
        elif c in "})]":
            depth -= 1
        elif c == ";" and depth == 0:
            return True
        i += 1
    return False


def strip_switches(text: str, path: Path, kept: list) -> str:
    while (at := text.find(SWITCH)) >= 0:
        open_a = text.index("{", at)
        close_a = matching(text, open_a)
        rest = text[close_a + 1:]
        m = re.match(r"\s*else\s*\{", rest)
        if not m:
            # A statement without else: drop it and its line.
            start = text.rfind("\n", 0, at) + 1
            end = close_a + 1
            if text[end:end + 1] == ";":
                end += 1
            if text[start:at].strip() == "" and text[end:end + 1] == "\n":
                end += 1
            else:
                start = at
            text = text[:start] + text[end:]
            continue
        open_b = close_a + 1 + m.end() - 1
        close_b = matching(text, open_b)
        body = text[open_b + 1:close_b].strip()
        if top_level_statements(body):
            kept.append(f"{path}:{text.count(chr(10), 0, at) + 1}")
            replacement = "{\n" + body + "\n}"
        else:
            replacement = body
        text = text[:at] + replacement + text[close_b + 1:]
    return text


def identity_table() -> dict:
    source = (ROOT / "src/identity.rs").read_text()
    table = {}
    for name, upstream in re.findall(r'^\s*(\w+): ("(?:[^"\\]|\\.)*") =>', source, re.M):
        value = upstream[1:-1].encode().decode("unicode_escape")
        table[name] = f'r"{value}"' if "\\" in value else upstream
    table["ABOUT_URL"] = ABOUT
    return table


def main() -> int:
    table = identity_table()
    names = "|".join(sorted(table, key=len, reverse=True))
    reference = re.compile(rf"\b(?:crate::|horizon_launcher::)?identity::({names})\b")
    owns = re.compile(r"(?:crate::)?identity::owns_launcher_data_dir\(([^()]*)\)")
    changed, kept = [], []
    for path in sorted(ROOT.rglob("*.rs")):
        if path == ROOT / "src/identity.rs" or "target" in path.parts:
            continue
        text = original = path.read_text()
        text = strip_switches(text, path, kept)
        text = reference.sub(lambda m: table[m.group(1)], text)
        text = owns.sub(lambda m: f'named({m.group(1)}, "horizon-launcher")', text)
        text = text.replace("use horizon_launcher::{build_import, identity};", "use horizon_launcher::build_import;")
        text = re.sub(r"^pub mod identity;\n", "", text, flags=re.M)
        text = re.sub(r"^use crate::identity;\n", "", text, flags=re.M)
        if text != original:
            path.write_text(text)
            changed.append(str(path))
    (ROOT / "src/identity.rs").unlink()
    rs = [p for p in changed if p.endswith(".rs") and not p.endswith("build.rs")]
    if rs:
        subprocess.run(["rustfmt", "--edition", "2021", *rs], check=True)
    leftover = subprocess.run(["git", "grep", "-n", "-E", r"horizon_fork_launcher|identity::(" + names + ")",
                               "--", "launcher"], capture_output=True, text=True).stdout
    print(f"strip_fork_launcher: {len(changed)} files changed")
    for place in kept:
        print("  kept a block:", place)
    for line in leftover.splitlines():
        print("  left for the hand pass:", line[:160])
    return 0


if __name__ == "__main__":
    sys.exit(main())
