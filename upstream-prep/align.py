#!/usr/bin/env python3
"""Turn the fork's own identity and choices toward upstream's, one commit per step, in the current work tree.

Runs after remove.py. Each step is a script (a function below), a hand-written diff (patches/align/<name>.diff,
applied with `git apply --3way`), or both, then one commit. Upstream's files are read from `upstream/main`
(fetched by prep.sh), so a rerun takes upstream's newest text.

  python3 align.py                 # every step
  python3 align.py --until NAME    # stop after step NAME (to author the next step's diff by hand)
  python3 align.py --from NAME     # start at step NAME (after a stop)

Authoring a hand step: run with --until <the step before it>, edit, then
  git add -A && git diff --cached --binary > patches/align/<name>.diff && git reset -q --hard
and run with --from <name>.
"""
import argparse
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
PATCHES = HERE / "patches" / "align"
UPSTREAM = "upstream/main"


def git(*args: str, check: bool = True) -> str:
    return subprocess.run(["git", *args], check=check, capture_output=True, text=True).stdout


def take_upstream(*paths: str) -> None:
    """Upstream's current text of `paths`, replacing ours."""
    git("checkout", UPSTREAM, "--", *paths)


def sub_file(path: str, old: str, new: str, count: int = 1) -> None:
    file = Path(path)
    text = file.read_text()
    found = text.count(old)
    if found != count:
        raise SystemExit(f"{path}: expected {count} of {old!r}, found {found}")
    file.write_text(text.replace(old, new))


# ---- scripted steps -------------------------------------------------------------------------------------------

def readme() -> None:
    # Upstream's README. The one product change kept: the game's executable is built by horizon-game here.
    take_upstream("README.md")
    sub_file("README.md", "cargo run --release -p horizon-client\n", "cargo run --release -p horizon-game\n")


def launcher_identity_script() -> None:
    subprocess.run([sys.executable, HERE / "strip_fork_launcher.py"], check=True)


def telemetry_registry() -> None:
    take_upstream(".github/workflows/telemetry-server.yml", ".github/workflows/mod-registry.yml")
    # The package crate is renamed on this branch (horizon-mods -> horizon-mod-package).
    sub_file(".github/workflows/mod-registry.yml", "'crates/horizon-mods/**'", "'crates/horizon-mod-package/**'", 2)


def claim_limit() -> None:
    take_upstream(".github/workflows/claim-command.yml", "tools/ci/claim_command.py", "tests/test_claim_command.py")


def restore_workflows() -> None:
    take_upstream(".github/workflows/runner-watch.yml")


def changelog() -> None:
    git("rm", "-q", ".github/workflows/changelog.yml", ".github/workflows/changelog-test.yml", "tools/changelog.py",
        "tests/test_changelog.py", "tests/fixtures/changelog.json")


def repository_rename() -> None:
    subprocess.run([sys.executable, HERE / "rename_repository.py"], check=True)


STEPS = [
    ("readme", "README: upstream's text, without the fork notice",
     "Upstream's README as it is on upstream's main. The one product change kept is the build line: the game's\n"
     "executable comes from horizon-game on this branch.", readme),
    ("launcher-identity-script", "Launcher: build only upstream's identity (scripted)",
     "strip_fork_launcher.py evaluates the fork-identity switch (cfg horizon_fork_launcher) as off and writes\n"
     "upstream's names back where the identity table stood.", launcher_identity_script),
    ("launcher-identity", "Launcher: drop the fork's names, icons, keys and packaging",
     "Hand corrections after the script: the build script, the release workflow, the fork's icons, release\n"
     "notes, staging tools, nightly banner and release key, and the tests that pinned them.", None),
    ("telemetry-registry", "CI: upstream's telemetry-server and mod-registry workflows",
     "Upstream's current files. Two product changes are kept: mod-registry builds the package crate under its new\n"
     "path, and telemetry-server installs the pinned Rust (tools/ci/rust-version, tests/test_rust_pin.py) instead\n"
     "of `stable`.", telemetry_registry),
    ("claim-limit", "Claims: upstream's claim command and limit",
     "Upstream's claim workflow, script, tests and CONTRIBUTING text: its own limit of 8 open issues, without the\n"
     "fork's lower limit and exemptions.",
     claim_limit),
    ("restore-workflows", "CI: restore upstream's runner watch",
     "runner-watch.yml as upstream has it; tools/ci/runner_watch.py is already the same as upstream's.",
     restore_workflows),
    ("machine-sample", "CI: stop building the external machine sample",
     "The sample mod lives in its own repository; the daily leg, its workflow, pin and checks are removed, and\n"
     "the docs name it as an example only.", None),
    ("scrub", "Docs and tools: drop the remaining local paths and agent-runner names",
     "The persistent prepared-data cache and the apt retry settings get neutral names, the read-only vanilla copy\n"
     "is HORIZON_SHARED_SOURCE's (as tools/ci/vanilla-data.sh names it) instead of a fixed local path, and two\n"
     "one-off refactor scripts for a workflow this branch removed go.", None),
    ("changelog", "Remove the fork's changelog digest",
     "A generated digest of the fork's merged pull requests, aware of its rollups, published to a changelog branch\n"
     "the fork's README linked; upstream's README does not.", changelog),
    ("repository-rename-script", "Point every repository reference at horizon-rust/horizon (scripted)",
     "rename_repository.py rewrites arismoko/horizon (links, release and API URLs, the launcher's update\n"
     "source, issue references) to horizon-rust/horizon. The machine sample's own repository is left alone.",
     repository_rename),
    ("repository-rename", "Repository references: corrections after the rename",
     "Tests that compared the two repositories now compare this one with another; a test that pinned upstream's\n"
     "older path names another repository; the switch in nightly.py is named for what it selects (an SDK-paired\n"
     "nightly), and a few sentences that called this repository the fork are reworded.", None),
]


def run_step(name: str, title: str, body: str, script) -> None:
    if script:
        script()
    patch = PATCHES / f"{name}.diff"
    if patch.exists():
        git("add", "-A")
        applied = subprocess.run(["git", "apply", "--3way", "--index", "--whitespace=nowarn", str(patch)],
                                 capture_output=True, text=True)
        if applied.returncode:
            raise SystemExit(f"{name}: {patch.name} does not apply; resolve it in the work tree:\n{applied.stderr}")
    git("add", "-A")
    if not git("diff", "--cached", "--name-only").strip():
        print(f"{name}: nothing to change")
        return
    git("commit", "-q", "-m", title + ("\n\n" + body if body else ""))
    stat = git("show", "--shortstat", "--format=", "HEAD").strip()
    print(f"{name}: {title} | {stat}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--until")
    parser.add_argument("--from", dest="start")
    args = parser.parse_args()
    names = [s[0] for s in STEPS]
    start = names.index(args.start) if args.start else 0
    stop = names.index(args.until) if args.until else len(STEPS) - 1
    for step in STEPS[start:stop + 1]:
        run_step(*step)
    return 0


if __name__ == "__main__":
    sys.exit(main())
