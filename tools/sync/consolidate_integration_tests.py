#!/usr/bin/env python3
"""Finish the sync-3 integration-test moves on the marker-merge tree.

Run once from the repository root. Upstream's tests/it destinations are
canonical; equal duplicate rename sources are removed, not copied twice.
"""
from pathlib import Path
import subprocess

UPSTREAM = "7785026e881d185ba7810fd0f80b4a39cf052fd5"


def git(*args):
    return subprocess.check_output(["git", *map(str, args)], text=True)


def main():
    crates = sorted({Path(p).parents[2] for p in git("ls-tree", "-r", "--name-only", UPSTREAM).splitlines()
                     if p.endswith("/tests/it/main.rs")})
    crates.append(Path("crates/horizon-game"))  # Owns the fork's executable.
    # The launch contract tests require CARGO_BIN_EXE_horizon-client, owned by horizon-game.
    git("rm", "crates/horizon-client/tests/it/launch_api.rs")
    moved = {}
    for crate in crates:
        tests = crate / "tests"
        dest = tests / "it"
        dest.mkdir(parents=True, exist_ok=True)
        sources = sorted(tests.glob("*.rs")) + sorted((tests / "integration").glob("*.rs"))
        for source in sources:
            if source.name == "main.rs":
                # Upstream's generated module list replaces the old list and its source-text test.
                git("rm", source)
                continue
            target = dest / source.name
            if target.exists():
                assert source.read_bytes() == target.read_bytes(), (source, target)
                git("rm", source)
            else:
                git("mv", source, target)
            moved[(crate.name, source.stem)] = "it"
        modules = sorted(f.stem for f in dest.glob("*.rs") if f.name != "main.rs")
        (dest / "main.rs").write_text(
            "//! Integration tests linked into one binary. Add each new test module here.\n\n"
            + "".join(f"mod {name};\n" for name in modules))
    for path in Path("tools/ci/ignored-tests").glob("*.txt"):
        lines = []
        for line in path.read_text().splitlines():
            fields = line.split()
            if len(fields) >= 3 and not line.startswith("#"):
                package, test, target = fields[:3]
                if target == "test:integration" and Path("crates", package) in crates:
                    fields[2] = "test:it"
                    line = " ".join(fields)
                elif target.startswith("test:") and (package, target[5:]) in moved:
                    fields[1] = target[5:] + "::" + test
                    fields[2] = "test:it"
                    line = " ".join(fields)
            lines.append(line)
        path.write_text("\n".join(sorted(lines) if path.name.startswith("daily-") else lines) + "\n")


if __name__ == "__main__":
    main()
