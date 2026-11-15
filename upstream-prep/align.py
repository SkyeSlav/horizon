#!/usr/bin/env python3
"""Turn the fork's own identity and choices toward upstream's, one commit per step, in the current work tree.

Runs after remove.py. Each step is a script (a function below), a hand-written diff (patches/align/<name>.diff,
applied with `git apply --3way`), or both, then one commit. Upstream's files are read from UPSTREAM_REF, which
prep.sh sets to the upstream commit the base already contains (its merge base with upstream/main): a file equal to
it there leaves the pull request's merge free to take upstream's newer text.

  python3 align.py                 # every step
  python3 align.py --until NAME    # stop after step NAME (to author the next step's diff by hand)
  python3 align.py --from NAME     # start at step NAME (after a stop)

Authoring a hand step: run with --until <the step before it>, edit, then
  git add -A && git diff --cached --binary > patches/align/<name>.diff && git reset -q --hard
and run with --from <name>.
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
PATCHES = HERE / "patches" / "align"
UPSTREAM = os.environ.get("UPSTREAM_REF", "upstream/main")


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
    take_upstream("README.md")


def launcher_identity_script() -> None:
    subprocess.run([sys.executable, HERE / "strip_fork_launcher.py"], check=True)


def telemetry_registry() -> None:
    take_upstream(".github/workflows/telemetry-server.yml", ".github/workflows/mod-registry.yml")
    # The package crate is renamed on this branch (horizon-mods -> horizon-mod-package).
    sub_file(".github/workflows/mod-registry.yml", "'crates/horizon-mods/**'", "'crates/horizon-mod-package/**'", 2)


def claim_limit() -> None:
    take_upstream(".github/workflows/claim-command.yml", "tools/ci/claim_command.py", "tests/test_claim_command.py")


def restore_workflows() -> None:
    take_upstream(".github/workflows/runner-watch.yml", ".github/workflows/nightly.yml",
                  ".github/workflows/parity-main.yml", ".github/workflows/pack-bench.yml",
                  ".github/workflows/run-parity.yml")


# The wasm32 check of the mod API, the one step added to upstream's ci.yml (maintainer's answer in
# arismoko/horizon#4498): in verify-lint, after the server build's clippy, when the mod API or a crate it builds
# changes. Everything else the fork checks for mods is in mod-checks.yml.
WASM_ANCHOR = "        run: cargo +stable clippy -p horizon-client --lib --bins --locked --no-default-features -- -D warnings\n"
WASM_STEP = """      # Wasm mods link the mod API, so it must build for wasm32: checked when a pull request changes it or a crate it
      # builds, and on every push to main.
      - name: Does this change touch the mod API?
        if: steps.rust.outputs.required != 'false'
        id: mod-api
        env:
          BASE_SHA: ${{ github.event.pull_request.base.sha }}
        run: |
          changed=true
          if [ -n "$BASE_SHA" ] && ! git diff --name-only "$BASE_SHA"...HEAD | grep -qE '^(crates/horizon-(mod-api|mod-api-macros|mod-api-gen|mod-vocab|mod-runtime|core)/|Cargo\\.(toml|lock)$)'; then
            changed=false
          fi
          echo "changed=$changed" >> "$GITHUB_OUTPUT"
      - name: The mod API builds for wasm32
        if: steps.mod-api.outputs.changed == 'true'
        run: |
          rustup target add wasm32-unknown-unknown --toolchain stable
          cargo +stable check -p horizon-mod-api --target wasm32-unknown-unknown --locked
"""


def ci_upstream() -> None:
    take_upstream(".github/workflows/ci.yml")
    sub_file(".github/workflows/ci.yml", WASM_ANCHOR, WASM_ANCHOR + WASM_STEP)


def book_links() -> None:
    subprocess.run([sys.executable, HERE / "book_links.py"], check=True)
    git("rm", "-q", "-f", ".github/workflows/modding-book-pages.yml")
    sub_file("docs/modding-book.md",
             "The modding book (`docs/modding`, published at <https://github.com/horizon-rust/horizon/blob/main/docs/modding/src/introduction.md>) is the manual for mod\n",
             "The modding book (`docs/modding`, read as Markdown in this repository; `modding-book.yml` checks that it builds) is the manual for mod\n")
    sub_file("docs/modding/src/reference/api.md",
             "**[Open the API reference](https://github.com/horizon-rust/horizon/tree/main/crates/horizon-mod-api)**\n\nTo build it locally, from a Horizon checkout:\n",
             "Build it from a Horizon checkout:\n")

def issue_refs() -> None:
    subprocess.run([sys.executable, HERE / "rewrite_issue_refs.py"], check=True)
    subprocess.run([sys.executable, "tools/roadmap.py", "build"], check=True, capture_output=True)

def repository_rename() -> None:
    subprocess.run([sys.executable, HERE / "rename_repository.py"], check=True)


STEPS = [
    ("readme", "README: upstream's text, without the fork notice",
     "Upstream's README, without the fork's notice at the top.", readme),
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
    ("restore-workflows", "CI: upstream's runner watch, nightly audit, parity-main, pack-bench and run-parity",
     "These workflows as upstream has them: the fork's copies differed only by older pins and step order that\n"
     "upstream has since changed. tools/ci/runner_watch.py is already the same as upstream's.",
     restore_workflows),
    ("machine-sample", "CI: stop building the external machine sample",
     "The sample mod lives in its own repository; the daily leg, its workflow, pin and checks are removed, and\n"
     "the docs name it as an example only.", None),
    ("scrub", "Docs and tools: drop the remaining local paths and agent-runner names",
     "The persistent prepared-data cache and the apt retry settings get neutral names, the read-only vanilla copy\n"
     "is HORIZON_SHARED_SOURCE's (as tools/ci/vanilla-data.sh names it) instead of a fixed local path, and two\n"
     "one-off refactor scripts for a workflow this branch removed go.", None),
    ("ci-upstream", "CI: upstream's ci.yml, plus the mod API's wasm32 check",
     "ci.yml is upstream's file as it is, with one step in verify-lint: horizon-mod-api builds for wasm32 when a pull\n"
     "request changes it or a crate it builds, and on every push to main. The fork's other additions to ci.yml (the\n"
     "vanilla-parity job, its tiered verify) go; the mod checks are in mod-checks.yml.", ci_upstream),
    ("nightly-sdk", "Release: the SDK built and published by the Nightly; no snapshot or daily-green flow",
     "nightly-build.yml builds the mod SDK for the Nightly's commit and its publish job releases it as sdk-<sha> with\n"
     "the same keys, before the Nightly, which carries the SDK's index in its signed sums. The fork's snapshot builds,\n"
     "the daily-green tag and the daily run's nightly are removed; release.yml is upstream's.", None),
    ("daily-github", "Daily tests on GitHub's runners, outside the gate",
     "The daily workflow runs on GitHub's runners with a time limit and files one issue per failing test. Its shards\n"
     "run the ignored tests that need no prepared vanilla data (ignored_tests.py --without-data); the rows that need\n"
     "it wait for a runner that holds the data. The legs that need the data become dispatch-only, and the hot-path\n"
     "trend no longer records to a branch.", None),
    ("repository-rename-script", "Point every repository reference at horizon-rust/horizon (scripted)",
     "rename_repository.py rewrites arismoko/horizon (links, release and API URLs, the launcher's update\n"
     "source, issue references) to horizon-rust/horizon. The machine sample's own repository is left alone.",
     repository_rename),
    ("repository-rename", "Repository references: corrections after the rename",
     "Tests that compared the two repositories now compare this one with another; a test that pinned upstream's\n"
     "older path names another repository; the switch in nightly.py is named for what it selects (an SDK-paired\n"
     "nightly), and a few sentences that called this repository the fork are reworded.", None),
    ("book-links", "Modding book: read in the repository, not published to Pages",
     "Links to the book's Pages site point at its Markdown in docs/modding; the two chapters the book build renders\n"
     "from the hook catalog point at the catalog, and the API reference says how to build it. The Pages workflow\n"
     "goes; modding-book.yml still checks that the book builds.", book_links),
    ("issue-refs-script", "Write the fork's issue references as arismoko/horizon#N (scripted)",
     "rewrite_issue_refs.py turns a bare #N in the fork's own lines (comments, docs, notes; never string literals)\n"
     "into arismoko/horizon#N, so a reference keeps pointing at the fork's issue once this tree is upstream's.", issue_refs),
    ("issue-refs", "Issue references: corrections after the rewrite",
     "Numbers the script took for issue references that are not: a CI run number.", None),
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
